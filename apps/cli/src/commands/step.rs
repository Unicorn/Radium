//! Step command implementation.
//!
//! Executes a single agent from configuration.

use anyhow::{Context, bail};
use chrono::Utc;
use colored::Colorize;
use futures::StreamExt;
use radium_abstraction::{ContentBlock, ImageSource, MediaSource, MessageContent, ModelError, ModelParameters, ReasoningEffort, ResponseFormat, StreamItem, StreamingModel};
use radium_core::engines::ExecutionResponse;
use radium_models::gemini::file_api::GeminiFileApi;
use serde_json;
use radium_core::{
    analytics::{ReportFormatter, SessionAnalytics, SessionReport, SessionStorage},
    auth::{CredentialStore, ProviderType},
    context::{ContextFileLoader, ContextManager}, AgentDiscovery, monitoring::MonitoringService, PromptContext,
    PromptTemplate, Workspace,
    engines::{EngineRegistry, ExecutionRequest},
    engines::providers::{ClaudeEngine, GeminiEngine, MockEngine, OpenAIEngine},
    syntax::SyntaxHighlighter,
    code_blocks::{CodeBlockParser, CodeBlockStore},
    terminal::{TerminalCapabilities, ColorSupport},
};
use radium_models::{GeminiModel, MockModel, OpenAIModel, ClaudeModel};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::fs;
use uuid::Uuid;

// Tool execution imports
use radium_orchestrator::orchestration::{
    tool_builder,
    tool::Tool as OrchestrationTool,
};
use radium_abstraction::{Model, Tool as AbstractionTool, ToolCall, ToolConfig, ToolUseMode, ChatMessage};

/// Parse response format arguments and construct ResponseFormat enum.
///
/// Handles:
/// - `--response-format text` → ResponseFormat::Text
/// - `--response-format json` → ResponseFormat::Json
/// - `--response-format json-schema` with `--response-schema` → ResponseFormat::JsonSchema
///
/// Returns error if format is invalid or schema file cannot be read.
fn parse_response_format(
    response_format: Option<String>,
    response_schema: Option<String>,
) -> anyhow::Result<Option<ResponseFormat>> {
    match response_format.as_deref() {
        Some("text") => Ok(Some(ResponseFormat::Text)),
        Some("json") => {
            if response_schema.is_some() {
                return Err(anyhow::anyhow!(
                    "--response-schema cannot be used with --response-format json. Use --response-format json-schema instead."
                ));
            }
            Ok(Some(ResponseFormat::Json))
        }
        Some("json-schema") => {
            let schema = response_schema.ok_or_else(|| {
                anyhow::anyhow!("--response-schema is required when using --response-format json-schema")
            })?;
            
            // Try to detect if it's a file path or inline JSON
            let schema_content = if std::path::Path::new(&schema).exists() {
                // It's a file path
                std::fs::read_to_string(&schema).with_context(|| {
                    format!("Failed to read schema file: {}", schema)
                })?
            } else {
                // Assume it's inline JSON
                schema
            };
            
            // Validate that it's valid JSON
            serde_json::from_str::<serde_json::Value>(&schema_content)
                .with_context(|| format!("Invalid JSON schema: {}", schema_content))?;
            
            Ok(Some(ResponseFormat::JsonSchema(schema_content)))
        }
        Some(format) => Err(anyhow::anyhow!(
            "Invalid response format: '{}'. Must be one of: text, json, json-schema",
            format
        )),
        None => {
            // If no format specified but schema is provided, that's an error
            if response_schema.is_some() {
                return Err(anyhow::anyhow!(
                    "--response-schema requires --response-format json-schema"
                ));
            }
            Ok(None)
        }
    }
}

/// Execute the step command.
///
/// Executes a single workflow step (agent from configuration).
pub async fn execute(
    id: String,
    prompt: Vec<String>,
    model: Option<String>,
    engine: Option<String>,
    reasoning: Option<String>,
    model_tier: Option<String>,
    session_id: Option<String>,
    stream: bool,
    show_metadata: bool,
    json: bool,
    safety_behavior: Option<String>,
    image: Vec<PathBuf>,
    audio: Vec<PathBuf>,
    video: Vec<PathBuf>,
    file: Vec<PathBuf>,
    auto_upload: bool,
    response_format: Option<String>,
    response_schema: Option<String>,
) -> anyhow::Result<()> {
    println!("{}", "rad step".bold().cyan());
    println!();
    
    // Parse response format arguments
    let response_format = parse_response_format(response_format, response_schema)
        .context("Failed to parse response format arguments")?;
    
    // Use provided session ID or generate new one
    let session_id = session_id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let session_start_time = Utc::now();

    // Discover workspace (optional for step command)
    let workspace = Workspace::discover().ok();
    let workspace_root = workspace
        .as_ref()
        .map(|w| w.root().to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")));

    // Load context files if available
    let loader = ContextFileLoader::new(&workspace_root);
    let current_dir = std::env::current_dir().unwrap_or_else(|_| workspace_root.clone());
    let context_files = loader.load_hierarchical(&current_dir).unwrap_or_default();

    // Initialize ContextManager if workspace is available (for memory context gathering)
    let mut context_manager = workspace.as_ref().map(|w| {
        // Create a basic ContextManager (without requirement_id for step command)
        ContextManager::new(w)
    });

    // Discover all available agents
    println!("  {}", "Discovering agents...".dimmed());
    let discovery = AgentDiscovery::new();
    let agents = discovery.discover_all().context("Failed to discover agents")?;

    if agents.is_empty() {
        bail!("No agents found. Place agent configs in ./agents/ or ~/.radium/agents/");
    }

    println!("  {} Found {} agents", "✓".green(), agents.len());
    println!();

    // Find the requested agent
    let agent = agents.get(&id).ok_or_else(|| anyhow::anyhow!("Agent not found: {}", id))?;

    // Display agent information
    println!("{}", "Agent Information:".bold());
    println!("  ID: {}", agent.id.cyan());
    println!("  Name: {}", agent.name);
    println!("  Description: {}", agent.description.dimmed());
    println!("  Prompt: {}", agent.prompt_path.display().to_string().dimmed());

    // Initialize engine registry
    let config_path = workspace
        .as_ref()
        .map(|w| w.radium_dir().join("config.toml"));
    let registry = if let Some(ref path) = config_path {
        EngineRegistry::with_config_path(path)
    } else {
        EngineRegistry::new()
    };
    
    // Register all available engines
    let _ = registry.register(Arc::new(MockEngine::new()));
    let _ = registry.register(Arc::new(ClaudeEngine::new()));
    let _ = registry.register(Arc::new(OpenAIEngine::new()));
    let _ = registry.register(Arc::new(GeminiEngine::new()));
    
    // Load config after engines are registered
    let _ = registry.load_config();

    // Resolve engine using precedence: CLI flag → env var → agent preference → default → first available
    let selected_engine_arc = registry
        .select_engine(engine.as_deref(), agent.engine.as_deref())
        .await
        .with_context(|| {
            let available = registry.list().unwrap_or_default();
            let engine_ids: Vec<String> = available.iter().map(|m| m.id.clone()).collect();
            format!(
                "Failed to select engine. Available engines: {}. Run `rad models list` for more details.",
                engine_ids.join(", ")
            )
        })?;
    
    let selected_engine_id = selected_engine_arc.metadata().id.as_str();
    let selected_engine_name = selected_engine_arc.metadata().name.as_str();

    // Model selection: CLI flag → agent config → engine default
    let default_model = selected_engine_arc.default_model();
    let selected_model = model.as_deref()
        .or_else(|| agent.model.as_deref())
        .unwrap_or_else(|| default_model.as_str());
    // Resolve reasoning effort: CLI flag → agent config → default (Medium)
    let selected_reasoning_effort = reasoning.as_deref()
        .and_then(|r| match r.to_lowercase().as_str() {
            "low" => Some(ReasoningEffort::Low),
            "medium" => Some(ReasoningEffort::Medium),
            "high" => Some(ReasoningEffort::High),
            _ => None,
        })
        .or_else(|| agent.reasoning_effort.map(|e| match e {
            radium_core::ReasoningEffort::Low => ReasoningEffort::Low,
            radium_core::ReasoningEffort::Medium => ReasoningEffort::Medium,
            radium_core::ReasoningEffort::High => ReasoningEffort::High,
        }))
        .unwrap_or(ReasoningEffort::Medium);
    
    let selected_reasoning = selected_reasoning_effort.to_string();

    println!();
    println!("{}", "Execution Configuration:".bold());
    println!("  Engine: {} ({})", selected_engine_id.cyan(), selected_engine_name);
    println!("  Model: {}", selected_model.cyan());
    println!("  Reasoning: {}", selected_reasoning.cyan());

    // Load and render prompt
    println!();
    println!("  {}", "Loading prompt template...".dimmed());

    let prompt_content = load_prompt(&agent.prompt_path)?;
    let user_input = if prompt.is_empty() {
        String::from("No additional input provided")
    } else {
        prompt.join(" ")
    };

    println!("  {} Loaded {} bytes", "✓".green(), prompt_content.len());

    if !user_input.is_empty() && user_input != "No additional input provided" {
        println!();
        println!("{}", "User Input:".bold());
        println!("  {}", user_input.dimmed());
    }

    println!();
    println!("{}", "Rendering prompt template...".bold());

    let mut context = PromptContext::new();
    context.set("user_input", user_input.clone());

    // Use ContextManager to build comprehensive context if available
    let mut additional_context = String::new();
    if let Some(ref mut manager) = context_manager {
        // Build context with agent invocation
        let invocation = format!("{id}");
        match manager.build_context(&invocation, None) {
            Ok(ctx) => {
                additional_context = ctx;
                // Inject as context variable
                context.set("context", additional_context.clone());
            }
            Err(e) => {
                // Log but continue - context gathering is optional
                eprintln!("  {} Warning: Failed to gather context: {}", "⚠".yellow(), e);
            }
        }
    }

    // Inject context files if available (for backward compatibility)
    if !context_files.is_empty() {
        context.set("context_files", context_files.clone());
        let context_file_paths = loader.get_context_file_paths(&current_dir);
        if !context_file_paths.is_empty() {
            println!("  {} Loaded context from {} file(s)", "✓".green(), context_file_paths.len());
        }
    }

    let template = PromptTemplate::from_string(prompt_content);
    let rendered = template.render(&context)?;

    println!("  {} Rendered {} bytes", "✓".green(), rendered.len());

    // Display prompt preview
    println!();
    println!("{}", "Prompt Preview:".bold().dimmed());
    println!("{}", "─".repeat(60).dimmed());
    let preview = if rendered.len() > 500 {
        format!("{}...\n\n[truncated {} bytes]", &rendered[..500], rendered.len() - 500)
    } else {
        rendered.clone()
    };
    println!("{}", preview.dimmed());
    println!("{}", "─".repeat(60).dimmed());

    // Register agent in monitoring for session tracking
    let tracked_agent_id = format!("{}-{}", session_id, agent.id);
    let monitoring_path = workspace
        .as_ref()
        .map(|w| w.radium_dir().join("monitoring.db"))
        .unwrap_or_else(|| std::path::PathBuf::from(".radium/monitoring.db"));
    let monitoring = MonitoringService::open(&monitoring_path).ok();
    
    if let Some(monitoring) = monitoring.as_ref() {
        use radium_core::monitoring::{AgentRecord, AgentStatus};
        let mut agent_record = AgentRecord::new(tracked_agent_id.clone(), agent.id.clone());
        agent_record.plan_id = Some(session_id.clone());
        if monitoring.register_agent(&agent_record).is_ok() {
            let _ = monitoring.update_status(&tracked_agent_id, AgentStatus::Running);
        }
    }

    // Check if multimodal input is provided
    let has_multimodal = !image.is_empty() || !audio.is_empty() || !video.is_empty() || !file.is_empty();

    // Validate engine for multimodal input
    if has_multimodal {
        if !selected_engine_id.contains("gemini") {
            return Err(anyhow::anyhow!(
                "Multimodal input is only supported with Gemini models. Current engine: {}. Please use --engine gemini",
                selected_engine_id
            ));
        }

        // Check for GEMINI_API_KEY
        if std::env::var("GEMINI_API_KEY").is_err() {
            return Err(anyhow::anyhow!(
                "GEMINI_API_KEY environment variable is required for multimodal input. Please set it and try again."
            ));
        }
    }

    // Execute agent
    println!();
    println!("{}", "Executing agent...".bold());
    println!();

    let execution_result = if has_multimodal {
        // Process multimodal files
        println!("  {}", "Processing multimodal inputs...".dimmed());
        let file_blocks = process_all_files(&image, &audio, &video, &file, auto_upload)
            .await
            .with_context(|| "Failed to process multimodal files")?;
        println!("  {} Processed {} content blocks", "✓".green(), file_blocks.len());

        // Combine text prompt with file blocks
        let mut blocks = Vec::new();
        if !rendered.is_empty() && rendered != "No additional input provided" {
            blocks.push(ContentBlock::Text {
                text: rendered.clone(),
            });
        }
        blocks.extend(file_blocks);

        // Create ChatMessage with multimodal content
        let message = radium_abstraction::ChatMessage {
            role: "user".to_string(),
            content: MessageContent::Blocks(blocks),
        };

        // Get Gemini model directly
        let credential_store = Arc::new(
            CredentialStore::new().unwrap_or_else(|_| {
                let temp_path = std::env::temp_dir().join("radium_credentials.json");
                CredentialStore::with_path(temp_path)
            })
        );
        let api_key = credential_store
            .get(ProviderType::Gemini)
            .map_err(|e| anyhow::anyhow!("Failed to get Gemini API key: {}", e))?;
        let gemini_model = GeminiModel::with_api_key(selected_model.to_string(), api_key);

        // Convert parameters
        let parameters = if stream {
            // For streaming, we'll handle it differently
            None
        } else {
            Some(ModelParameters {
                temperature: None,
                top_p: None,
                max_tokens: Some(512),
                top_k: None,
                frequency_penalty: None,
                presence_penalty: None,
                response_format: response_format.clone(),
                stop_sequences: None,
                enable_grounding: None,
                grounding_threshold: None,
                reasoning_effort: Some(selected_reasoning_effort),
            })
        };

        if stream {
            // Streaming with multimodal content is not yet supported
            // Fall back to non-streaming for multimodal
            println!("  {} Streaming is not supported with multimodal content, using non-streaming mode", "⚠".yellow());
        }

        {
            // Use generate_chat_completion for multimodal
            use radium_abstraction::Model;
            let response = gemini_model
                .generate_chat_completion(&[message], parameters)
                .await
                .map_err(|e| anyhow::anyhow!("Model execution failed: {}", e))?;

            Ok(radium_core::engines::ExecutionResponse {
                content: response.content,
                usage: response.usage.map(|u| radium_core::engines::TokenUsage {
                    input_tokens: u.prompt_tokens as u64,
                    output_tokens: u.completion_tokens as u64,
                    total_tokens: u.total_tokens as u64,
                }),
                model: response.model_id.unwrap_or_else(|| selected_model.to_string()),
                raw: None,
                execution_duration: None,
                metadata: response.metadata,
            })
        }
    } else {
        // Use tool-enabled execution path
        execute_agent_with_tools(
            selected_engine_id,
            selected_model,
            &rendered,
            &user_input,
            &workspace_root,
        ).await
    };
    
    // Handle response display based on flags
    if let Ok(ref response) = execution_result {
        if json {
            // Output as JSON - convert ExecutionResponse to a serializable format
            let json_response = serde_json::json!({
                "content": response.content,
                "model": response.model,
                "usage": response.usage,
                "metadata": response.metadata,
            });
            let json_output = serde_json::to_string_pretty(&json_response)?;
            println!("{}", json_output);
            return Ok(());
        }
        
        // Display response content
        println!("{}", "Response:".bold().green());
        println!("{}", "─".repeat(60).dimmed());
        print_highlighted_output(&response.content);
        println!("{}", "─".repeat(60).dimmed());
        
        // Display metadata if requested
        if show_metadata {
            if let Some(ref metadata) = response.metadata {
                println!();
                println!("{}", "Metadata:".bold().dimmed());
                format_metadata_display(response);
            }
        }
    }
    
    // Parse and store code blocks from response
    if let Ok(ref response) = execution_result {
        let blocks = CodeBlockParser::parse(&response.content);
        if !blocks.is_empty() {
            // Store blocks if workspace is available
            if let Some(ref workspace) = workspace {
                match CodeBlockStore::new(workspace.root(), session_id.clone()) {
                    Ok(mut store) => {
                        if let Err(e) = store.store_blocks(&agent.id, blocks.clone()) {
                            eprintln!("  {} Failed to store code blocks: {}", "⚠".yellow(), e);
                        } else {
                            println!();
                            println!("  {} {} code blocks extracted", "✓".green(), blocks.len());
                        }
                    }
                    Err(e) => {
                        eprintln!("  {} Failed to create code block store: {}", "⚠".yellow(), e);
                    }
                }
            }
        }
    }
    
    // Extract token usage from response for telemetry
    let token_usage = execution_result.as_ref().ok().and_then(|r| r.usage.as_ref());

    // Record telemetry if execution was successful
    if let Ok(ref response) = execution_result {
        if let Some(monitoring) = monitoring.as_ref() {
            use radium_core::monitoring::{TelemetryRecord, TelemetryTracking, AttributionMetadata};
            use radium_core::auth::ProviderType;

            let mut telemetry = TelemetryRecord::new(tracked_agent_id.clone())
                .with_engine_id(selected_engine_id.to_string());

            // Set model info
            if let Some(model) = agent.model.as_deref() {
                telemetry = telemetry.with_model(model.to_string(), selected_engine_id.to_string());
            } else {
                telemetry = telemetry.with_model(response.model.clone(), selected_engine_id.to_string());
            }

            // Set token usage from response
            if let Some(ref usage) = response.usage {
                telemetry = telemetry.with_tokens(usage.input_tokens, usage.output_tokens);
            }
            
            // Extract metadata from response
            if let Some(ref metadata) = response.metadata {
                use radium_abstraction::{Citation, SafetyRating};
                
                // Extract finish_reason
                if let Some(finish_reason) = metadata.get("finish_reason").and_then(|v| v.as_str()) {
                    telemetry.finish_reason = Some(finish_reason.to_string());
                }
                
                // Extract safety_blocked
                if let Some(safety_ratings_val) = metadata.get("safety_ratings") {
                    if let Ok(safety_ratings) = serde_json::from_value::<Vec<SafetyRating>>(safety_ratings_val.clone()) {
                        telemetry.safety_blocked = safety_ratings.iter().any(|r| r.blocked);
                    }
                }
                
                // Extract citation_count
                if let Some(citations_val) = metadata.get("citations") {
                    if let Ok(citations) = serde_json::from_value::<Vec<Citation>>(citations_val.clone()) {
                        telemetry.citation_count = Some(citations.len() as u32);
                    }
                }
            }

            // Try to add attribution metadata based on provider type
            if let Some(provider_type) = match selected_engine_id {
                "openai" => Some(ProviderType::OpenAI),
                "claude" | "anthropic" => Some(ProviderType::Claude),
                "gemini" => Some(ProviderType::Gemini),
                _ => None,
            } {
                // Try to load API key and generate attribution
                use radium_core::auth::CredentialStore;
                if let Ok(store) = CredentialStore::new() {
                    if let Ok(api_key) = store.get(provider_type) {
                        if let Some(attribution) = AttributionMetadata::from_api_key(&api_key, provider_type) {
                            telemetry = telemetry.with_attribution(
                                Some(attribution.api_key_id),
                                attribution.team_name,
                                attribution.project_name,
                                attribution.cost_center,
                            );
                        } else {
                            // Fallback: generate api_key_id only
                            let api_key_id = radium_core::monitoring::generate_api_key_id(&api_key);
                            telemetry = telemetry.with_attribution(Some(api_key_id), None, None, None);
                        }
                    }
                }
            }
            
            telemetry.calculate_cost();
            let _ = monitoring.record_telemetry(&telemetry).await;
        }
    }

    // Complete agent in monitoring
    if let Some(monitoring) = monitoring.as_ref() {
        
        match execution_result {
            Ok(_) => {
                let _ = monitoring.complete_agent(&tracked_agent_id, 0);
            }
            Err(ref e) => {
                let _ = monitoring.fail_agent(&tracked_agent_id, &e.to_string());
            }
        }
    }

    // Execution already succeeded (we got here), response was used for telemetry above

    // Generate and display session report
    let session_end_time = Some(Utc::now());
    if let Some(monitoring) = monitoring {
        let analytics = SessionAnalytics::new(monitoring);
        let agent_ids = vec![tracked_agent_id];
        
        if let Ok(metrics) = analytics.generate_session_metrics_with_workspace(
            &session_id,
            &agent_ids,
            session_start_time,
            session_end_time,
            workspace.as_ref().map(|w| w.root()),
        ) {
            let report = SessionReport::new(metrics);
            if let Some(workspace) = workspace.as_ref() {
                let storage = SessionStorage::new(workspace.root()).ok();
                if let Some(ref storage) = storage {
                    let _ = storage.save_report(&report);
                    display_session_summary(&report);
                }
            }
        }
    }

    println!();
    println!("{}", "Agent execution completed!".green().bold());
    println!();

    Ok(())
}

/// Display session summary at end of execution.
fn display_session_summary(report: &SessionReport) {
    println!();
    println!("{}", "─".repeat(60).dimmed());
    println!("{}", "Session Summary".bold().cyan());
    println!("{}", "─".repeat(60).dimmed());
    
    let formatter = ReportFormatter;
    let summary = formatter.format(report);
    
    // Print a condensed version (first few lines)
    for line in summary.lines().take(15) {
        println!("{}", line);
    }
    
    println!();
    println!("  {} Full report: {}", "💡".cyan(), format!("rad stats session {}", report.metrics.session_id).dimmed());
    println!("{}", "─".repeat(60).dimmed());
    println!();
}

/// Load prompt from file.
///
/// Search order (precedence from highest to lowest):
/// 1. Absolute path (if provided)
/// 2. Relative to current directory
/// 3. Relative to workspace root
/// 4. Relative to home directory (.radium/)
/// 5. Extension prompt directories (project-level, then user-level)
/// Process a single file: read, detect MIME type, check size, encode or upload.
async fn process_file(
    path: &PathBuf,
    auto_upload: bool,
    file_api: &GeminiFileApi,
) -> anyhow::Result<ContentBlock> {
    use base64::Engine;
    use std::path::Path;

    // Check file exists
    if !path.exists() {
        return Err(anyhow::anyhow!("File not found: {}", path.display()));
    }

    // Get file metadata to check size
    let metadata = std::fs::metadata(path)
        .with_context(|| format!("Failed to read file metadata: {}", path.display()))?;
    let file_size = metadata.len() as usize;

    // Check 2GB limit
    const MAX_FILE_SIZE: usize = 2_147_483_648; // 2GB
    if file_size > MAX_FILE_SIZE {
        return Err(anyhow::anyhow!(
            "File exceeds 2GB limit: {} (size: {} bytes)",
            path.display(),
            file_size
        ));
    }

    // Detect MIME type
    let mime_type = mime_guess::from_path(path)
        .first_or_octet_stream()
        .to_string();

    // Determine if we should use File API or inline base64
    const INLINE_THRESHOLD: usize = 20 * 1024 * 1024; // 20MB
    let use_file_api = auto_upload || file_size >= INLINE_THRESHOLD;

    if use_file_api {
        // Upload via File API
        let gemini_file = file_api
            .upload_file(path.as_path(), Some(mime_type.clone()), None)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to upload file {}: {}", path.display(), e))?;

        // Determine content block type based on file extension/MIME type
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_default();

        if mime_type.starts_with("image/") {
            // For images uploaded via File API, use Url with the File API URI
            // Note: This requires Gemini model to support ImageSource::Url with File API URIs
            Ok(ContentBlock::Image {
                source: ImageSource::Url {
                    url: gemini_file.uri,
                },
                media_type: mime_type,
            })
        } else if mime_type.starts_with("audio/") {
            Ok(ContentBlock::Audio {
                source: MediaSource::FileApi {
                    file_id: gemini_file.uri,
                },
                media_type: mime_type,
            })
        } else if mime_type.starts_with("video/") {
            Ok(ContentBlock::Video {
                source: MediaSource::FileApi {
                    file_id: gemini_file.uri,
                },
                media_type: mime_type,
            })
        } else {
            // Documents and other files
            Ok(ContentBlock::Document {
                source: MediaSource::FileApi {
                    file_id: gemini_file.uri,
                },
                media_type: mime_type,
                filename: path.file_name().and_then(|n| n.to_str().map(String::from)),
            })
        }
    } else {
        // Read file and encode to base64
        let file_bytes = std::fs::read(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;

        let base64_data = base64::engine::general_purpose::STANDARD.encode(&file_bytes);

        // Determine content block type
        if mime_type.starts_with("image/") {
            Ok(ContentBlock::Image {
                source: ImageSource::Base64 {
                    data: base64_data,
                },
                media_type: mime_type,
            })
        } else if mime_type.starts_with("audio/") {
            Ok(ContentBlock::Audio {
                source: MediaSource::Base64 {
                    data: base64_data,
                },
                media_type: mime_type,
            })
        } else if mime_type.starts_with("video/") {
            Ok(ContentBlock::Video {
                source: MediaSource::Base64 {
                    data: base64_data,
                },
                media_type: mime_type,
            })
        } else {
            // Documents and other files
            Ok(ContentBlock::Document {
                source: MediaSource::Base64 {
                    data: base64_data,
                },
                media_type: mime_type,
                filename: path.file_name().and_then(|n| n.to_str().map(String::from)),
            })
        }
    }
}

/// Process all files from the command arguments.
async fn process_all_files(
    image: &[PathBuf],
    audio: &[PathBuf],
    video: &[PathBuf],
    file: &[PathBuf],
    auto_upload: bool,
) -> anyhow::Result<Vec<ContentBlock>> {
    // Get Gemini API key
    let api_key = std::env::var("GEMINI_API_KEY")
        .map_err(|_| anyhow::anyhow!("GEMINI_API_KEY environment variable not set"))?;

    let file_api = GeminiFileApi::with_api_key(api_key);
    let mut blocks = Vec::new();

    // Process each file type
    for path in image {
        blocks.push(process_file(path, auto_upload, &file_api).await?);
    }
    for path in audio {
        blocks.push(process_file(path, auto_upload, &file_api).await?);
    }
    for path in video {
        blocks.push(process_file(path, auto_upload, &file_api).await?);
    }
    for path in file {
        blocks.push(process_file(path, auto_upload, &file_api).await?);
    }

    Ok(blocks)
}

fn load_prompt(prompt_path: &std::path::Path) -> anyhow::Result<String> {
    use radium_core::extensions::integration::get_extension_prompt_dirs;

    // Try as absolute path first
    if prompt_path.is_absolute() && prompt_path.exists() {
        return Ok(fs::read_to_string(prompt_path)?);
    }

    // Try relative to current directory
    if prompt_path.exists() {
        return Ok(fs::read_to_string(prompt_path)?);
    }

    // Try relative to workspace
    if let Ok(workspace) = Workspace::discover() {
        let workspace_path = workspace.root().join(prompt_path);
        if workspace_path.exists() {
            return Ok(fs::read_to_string(workspace_path)?);
        }
    }

    // Try relative to home directory
    if let Ok(home) = std::env::var("HOME") {
        let home_path = std::path::PathBuf::from(home).join(".radium").join(prompt_path);
        if home_path.exists() {
            return Ok(fs::read_to_string(home_path)?);
        }
    }

    // Try extension prompt directories (lowest precedence)
    // Extract just the filename from the path to search in extension directories
    if let Some(file_name) = prompt_path.file_name() {
        if let Ok(extension_dirs) = get_extension_prompt_dirs() {
            for ext_dir in extension_dirs {
                let ext_prompt_path = ext_dir.join(file_name);
                if ext_prompt_path.exists() {
                    return Ok(fs::read_to_string(ext_prompt_path)?);
                }
            }
        }
    }

    bail!("Prompt file not found: {}", prompt_path.display())
}

/// Print output with syntax highlighting for code blocks and annotations.
fn print_highlighted_output(text: &str) {
    use radium_core::syntax::StyledLine;
    
    // Parse code blocks to get indices and languages
    let blocks = CodeBlockParser::parse(text);
    let mut block_map: std::collections::HashMap<usize, (String, usize)> = blocks
        .iter()
        .map(|b| (b.start_line, (b.language.clone().unwrap_or_else(|| "text".to_string()), b.index)))
        .collect();
    
    let highlighter = SyntaxHighlighter::new();
    let capabilities = TerminalCapabilities::color_support();
    
    let mut in_code_block = false;
    let mut code_block_lang = String::new();
    let mut code_block_content = String::new();
    let mut current_line = 1;
    let mut current_block_index = 0;
    
    for line in text.lines() {
        if line.trim().starts_with("```") {
            if in_code_block {
                // End of code block
                in_code_block = false;
                let lang = code_block_lang.clone();
                code_block_lang.clear();
                
                // Apply syntax highlighting
                let highlighted_lines = highlighter.highlight_code(&code_block_content, &lang);
                for styled_line in highlighted_lines {
                    print_styled_line(&styled_line, capabilities);
                }
                code_block_content.clear();
            } else {
                // Start of code block
                in_code_block = true;
                let lang = line.trim().strip_prefix("```").unwrap_or("");
                code_block_lang = lang.trim().to_string();
                
                // Find block index for this line
                if let Some((_, index)) = block_map.get(&current_line) {
                    current_block_index = *index;
                    let lang_display = if code_block_lang.is_empty() {
                        "text".to_string()
                    } else {
                        code_block_lang.clone()
                    };
                    println!("{}", format!("[Block {}: {}]", index, lang_display).cyan().bold());
                }
            }
            current_line += 1;
            continue;
        }
        
        if in_code_block {
            if !code_block_content.is_empty() {
                code_block_content.push('\n');
            }
            code_block_content.push_str(line);
        } else {
            // Regular text - print as-is
            println!("{}", line);
        }
        current_line += 1;
    }
    
    // Handle unclosed code block
    if in_code_block && !code_block_content.is_empty() {
        let highlighted_lines = highlighter.highlight_code(&code_block_content, &code_block_lang);
        for styled_line in highlighted_lines {
            print_styled_line(&styled_line, capabilities);
        }
    }
}

/// Print a styled line with ANSI color codes.
fn print_styled_line(styled_line: &radium_core::syntax::StyledLine, capabilities: ColorSupport) {
    use radium_core::syntax::StyledSpan;

    for span in &styled_line.spans {
        let (r, g, b) = span.foreground;
        let color_code = match capabilities {
            ColorSupport::Truecolor => format!("\x1b[38;2;{};{};{}m", r, g, b),
            ColorSupport::Color256 => {
                let index = radium_core::terminal::rgb_to_256(r, g, b);
                format!("\x1b[38;5;{}m", index)
            }
            ColorSupport::Color16 => {
                let index = radium_core::terminal::rgb_to_16(r, g, b);
                format!("\x1b[{}m", if index < 8 { 30 + index } else { 90 + (index - 8) })
            }
        };
        print!("{}", color_code);
        
        if let Some((br, bg, bb)) = span.background {
            let bg_code = match capabilities {
                ColorSupport::Truecolor => format!("\x1b[48;2;{};{};{}m", br, bg, bb),
                ColorSupport::Color256 => {
                    let index = radium_core::terminal::rgb_to_256(br, bg, bb);
                    format!("\x1b[48;5;{}m", index)
                }
                ColorSupport::Color16 => {
                    let index = radium_core::terminal::rgb_to_16(br, bg, bb);
                    format!("\x1b[{}m", if index < 8 { 40 + index } else { 100 + (index - 8) })
                }
            };
            print!("{}", bg_code);
        }
        
        if span.bold {
            print!("\x1b[1m");
        }
        if span.italic {
            print!("\x1b[3m");
        }
        if span.underline {
            print!("\x1b[4m");
        }
        
        print!("{}", span.text);
        
        // Reset all attributes
        print!("\x1b[0m");
    }
    
    println!();
}

/// Execute the agent with the engine registry.
/// Returns the execution response for telemetry tracking.
async fn execute_agent_with_engine(
    registry: &EngineRegistry,
    agent_id: &str,
    rendered_prompt: &str,
    engine_id: &str,
    model: &str,
    stream: bool,
    response_format: Option<&ResponseFormat>,
    reasoning_effort: Option<ReasoningEffort>,
) -> anyhow::Result<radium_core::engines::ExecutionResponse> {
    println!("  {} Executing agent with {}...", "•".cyan(), engine_id);
    println!("  {} Agent: {}", "•".dimmed(), agent_id.cyan());
    println!("  {} Engine: {}", "•".dimmed(), engine_id.cyan());
    println!("  {} Model: {}", "•".dimmed(), model.cyan());
    println!();

    // Get engine from registry
    let engine = match registry.get(engine_id) {
        Ok(e) => {
            println!("  {} Engine initialized successfully", "✓".green());
            println!("  {} Sending prompt to engine...", "•".cyan());
            println!();
            e
        }
        Err(e) => {
            println!(
                "  {} {}",
                "!".yellow(),
                format!("Could not find engine: {}", e).yellow()
            );
            println!();
            println!("  {} Falling back to mock engine...", "→".dimmed());
            println!();
            
            // Fall back to mock engine
            registry.get("mock")
                .with_context(|| "Mock engine not available")?
        }
    };

    // Create execution request
    let mut request = ExecutionRequest::new(model.to_string(), rendered_prompt.to_string());
    
    // Add reasoning effort to params if specified
    if let Some(effort) = reasoning_effort {
        request.params.insert(
            "reasoning_effort".to_string(),
            serde_json::Value::String(effort.to_string()),
        );
    }

    // Handle streaming if requested
    if stream {
        // Try to create model directly for streaming
        let model_result: anyhow::Result<radium_core::engines::ExecutionResponse> = match engine_id {
            "gemini" => {
                let credential_store = Arc::new(
                    CredentialStore::new().unwrap_or_else(|_| {
                        let temp_path = std::env::temp_dir().join("radium_credentials.json");
                        CredentialStore::with_path(temp_path)
                    })
                );
                let api_key = credential_store
                    .get(ProviderType::Gemini)
                    .map_err(|e| anyhow::anyhow!("Failed to get API key: {}", e))?;
                let gemini_model = GeminiModel::with_api_key(model.to_string(), api_key);
                
                // Check if model implements StreamingModel (it does)
                let parameters = if request.temperature.is_some() || request.max_tokens.is_some() || response_format.is_some() || reasoning_effort.is_some() {
                    Some(ModelParameters {
                        temperature: request.temperature,
                        top_p: None,
                        max_tokens: request.max_tokens.map(|t| t as u32),
                        top_k: None,
                        frequency_penalty: None,
                        presence_penalty: None,
                        response_format: response_format.cloned(),
                        stop_sequences: None,
                        enable_grounding: None,
                        grounding_threshold: None,
                        reasoning_effort,
                    })
                } else {
                    None
                };

                let mut stream = gemini_model.generate_stream(&rendered_prompt, parameters)
                    .await
                    .map_err(|e| anyhow::anyhow!("Streaming failed: {}", e))?;

                println!("{}", "Response:".bold().green());
                println!("{}", "─".repeat(60).dimmed());
                
                let mut accumulated = String::new();
                let mut last_content = String::new();
                let mut thinking_accumulated = String::new();
                let mut in_thinking_phase = false;
                
                while let Some(result) = stream.next().await {
                    match result {
                        Ok(radium_abstraction::StreamItem::ThinkingToken(token)) => {
                            if !in_thinking_phase {
                                // Start of thinking phase
                                println!();
                                println!("{}", "Thinking Process:".bold().cyan().dimmed());
                                println!("{}", "─".repeat(60).dimmed());
                                in_thinking_phase = true;
                            }
                            thinking_accumulated.push_str(&token);
                            print!("{}", token.dimmed());
                            std::io::stdout().flush().map_err(|e| anyhow::anyhow!("Failed to flush stdout: {}", e))?;
                        }
                        Ok(radium_abstraction::StreamItem::AnswerToken(content)) => {
                            if in_thinking_phase {
                                // Transition from thinking to answer
                                println!();
                                println!("{}", "─".repeat(60).dimmed());
                                println!();
                                println!("{}", "Answer:".bold().green());
                                println!("{}", "─".repeat(60).dimmed());
                                in_thinking_phase = false;
                            }
                            // Only print the new part (delta)
                            if content.len() > last_content.len() {
                                let delta = &content[last_content.len()..];
                                print!("{}", delta);
                                std::io::stdout().flush().map_err(|e| anyhow::anyhow!("Failed to flush stdout: {}", e))?;
                                accumulated = content.clone();
                                last_content = content;
                            }
                        }
                        Ok(radium_abstraction::StreamItem::Metadata(_)) => {
                            // Metadata updates - can be handled if needed
                        }
                        Err(e) => {
                            println!();
                            println!("  {} {}", "✗".red(), format!("Streaming error: {}", e).red());
                            return Err(anyhow::anyhow!("Streaming error: {}", e));
                        }
                    }
                }
                
                println!();
                println!("{}", "─".repeat(60).dimmed());
                
                // Return response with accumulated content
                Ok(radium_core::engines::ExecutionResponse {
                    content: accumulated,
                    usage: None, // Token usage not available during streaming
                    model: model.to_string(),
                    raw: None,
                    execution_duration: None,
                    metadata: None,
                })
            }
            "openai" => {
                let credential_store = Arc::new(
                    CredentialStore::new().unwrap_or_else(|_| {
                        let temp_path = std::env::temp_dir().join("radium_credentials.json");
                        CredentialStore::with_path(temp_path)
                    })
                );
                let api_key = credential_store
                    .get(ProviderType::OpenAI)
                    .map_err(|e| anyhow::anyhow!("Failed to get API key: {}", e))?;
                let openai_model = OpenAIModel::with_api_key(model.to_string(), api_key);
                
                let parameters = if request.temperature.is_some() || request.max_tokens.is_some() || response_format.is_some() || reasoning_effort.is_some() {
                    Some(ModelParameters {
                        temperature: request.temperature,
                        top_p: None,
                        max_tokens: request.max_tokens.map(|t| t as u32),
                        top_k: None,
                        frequency_penalty: None,
                        presence_penalty: None,
                        response_format: response_format.cloned(),
                        stop_sequences: None,
                        enable_grounding: None,
                        grounding_threshold: None,
                        reasoning_effort,
                    })
                } else {
                    None
                };

                let mut stream = openai_model.generate_stream(&rendered_prompt, parameters)
                    .await
                    .map_err(|e| anyhow::anyhow!("Streaming failed: {}", e))?;

                println!("{}", "Response:".bold().green());
                println!("{}", "─".repeat(60).dimmed());
                
                let mut accumulated = String::new();
                let mut last_content = String::new();
                let mut in_thinking_phase = false;
                
                while let Some(result) = stream.next().await {
                    match result {
                        Ok(radium_abstraction::StreamItem::ThinkingToken(token)) => {
                            if !in_thinking_phase {
                                println!();
                                println!("{}", "Thinking Process:".bold().cyan().dimmed());
                                println!("{}", "─".repeat(60).dimmed());
                                in_thinking_phase = true;
                            }
                            print!("{}", token.dimmed());
                            std::io::stdout().flush().map_err(|e| anyhow::anyhow!("Failed to flush stdout: {}", e))?;
                        }
                        Ok(radium_abstraction::StreamItem::AnswerToken(content)) => {
                            if in_thinking_phase {
                                println!();
                                println!("{}", "─".repeat(60).dimmed());
                                println!();
                                println!("{}", "Answer:".bold().green());
                                println!("{}", "─".repeat(60).dimmed());
                                in_thinking_phase = false;
                            }
                            if content.len() > last_content.len() {
                                let delta = &content[last_content.len()..];
                                print!("{}", delta);
                                std::io::stdout().flush().map_err(|e| anyhow::anyhow!("Failed to flush stdout: {}", e))?;
                                accumulated = content.clone();
                                last_content = content;
                            }
                        }
                        Ok(radium_abstraction::StreamItem::Metadata(_)) => {
                            // Metadata updates
                        }
                        Err(e) => {
                            println!();
                            println!("  {} {}", "✗".red(), format!("Streaming error: {}", e).red());
                            return Err(anyhow::anyhow!("Streaming error: {}", e));
                        }
                    }
                }
                
                println!();
                println!("{}", "─".repeat(60).dimmed());
                
                Ok(radium_core::engines::ExecutionResponse {
                    content: accumulated,
                    usage: None,
                    model: model.to_string(),
                    raw: None,
                    execution_duration: None,
                    metadata: None,
                })
            }
            "mock" => {
                let mock_model = MockModel::new(model.to_string());
                
                let parameters = if request.temperature.is_some() || request.max_tokens.is_some() || response_format.is_some() || reasoning_effort.is_some() {
                    Some(ModelParameters {
                        temperature: request.temperature,
                        top_p: None,
                        max_tokens: request.max_tokens.map(|t| t as u32),
                        top_k: None,
                        frequency_penalty: None,
                        presence_penalty: None,
                        response_format: response_format.cloned(),
                        stop_sequences: None,
                        enable_grounding: None,
                        grounding_threshold: None,
                        reasoning_effort,
                    })
                } else {
                    None
                };

                let mut stream = mock_model.generate_stream(&rendered_prompt, parameters)
                    .await
                    .map_err(|e| anyhow::anyhow!("Streaming failed: {}", e))?;

                println!("{}", "Response:".bold().green());
                println!("{}", "─".repeat(60).dimmed());
                
                let mut accumulated = String::new();
                let mut last_content = String::new();
                
                while let Some(result) = stream.next().await {
                    match result {
                        Ok(stream_item) => {
                            // Extract string from StreamItem
                            let content = match stream_item {
                                radium_abstraction::StreamItem::ThinkingToken(s) => s,
                                radium_abstraction::StreamItem::AnswerToken(s) => s,
                                radium_abstraction::StreamItem::Metadata(_) => continue, // Skip metadata
                            };

                            if content.len() > last_content.len() {
                                let delta = &content[last_content.len()..];
                                print!("{}", delta);
                                std::io::stdout().flush().map_err(|e| anyhow::anyhow!("Failed to flush stdout: {}", e))?;
                                accumulated = content.clone();
                                last_content = content;
                            }
                        }
                        Err(e) => {
                            println!();
                            println!("  {} {}", "✗".red(), format!("Streaming error: {}", e).red());
                            return Err(anyhow::anyhow!("Streaming error: {}", e));
                        }
                    }
                }
                
                println!();
                println!("{}", "─".repeat(60).dimmed());
                
                Ok(radium_core::engines::ExecutionResponse {
                    content: accumulated,
                    usage: None,
                    model: model.to_string(),
                    raw: None,
                    execution_duration: None,
                    metadata: None,
                })
            }
            _ => {
                // Engine doesn't support streaming, fall back to non-streaming
                println!("  {} Streaming not supported for engine '{}', using standard mode", "⚠".yellow(), engine_id);
                // Fall through to normal execution
                return execute_normal(&engine, request).await;
            }
        };

        match model_result {
            Ok(response) => {
                if let Some(usage) = &response.usage {
                    println!();
                    println!("{}", "Token Usage:".bold().dimmed());
                    println!("  Input: {} tokens", usage.input_tokens.to_string().dimmed());
                    println!(
                        "  Output: {} tokens",
                        usage.output_tokens.to_string().dimmed()
                    );
                    println!("  Total: {} tokens", usage.total_tokens.to_string().cyan());
                }
                return Ok(response);
            }
            Err(e) => {
                println!();
                println!("  {} {}", "⚠".yellow(), format!("Streaming failed: {}, falling back to standard mode", e).yellow());
                // Fall through to normal execution
            }
        }
    }

    // Normal (non-streaming) execution
    execute_normal(&engine, request).await
}

/// Execute the engine normally (non-streaming).
async fn execute_normal(
    engine: &Arc<dyn radium_core::engines::Engine>,
    request: ExecutionRequest,
) -> anyhow::Result<radium_core::engines::ExecutionResponse> {
    // Execute the engine
    match engine.execute(request).await {
        Ok(response) => {
            if let Some(usage) = &response.usage {
                println!();
                println!("{}", "Token Usage:".bold().dimmed());
                println!("  Input: {} tokens", usage.input_tokens.to_string().dimmed());
                println!(
                    "  Output: {} tokens",
                    usage.output_tokens.to_string().dimmed()
                );
                println!("  Total: {} tokens", usage.total_tokens.to_string().cyan());
            }

            Ok(response)
        }
        Err(e) => {
            println!();
            println!("  {} {}", "✗".red(), format!("Engine execution failed: {}", e).red());
            println!();
            println!("  {} Check your API key and engine configuration", "i".yellow());
            Err(anyhow::anyhow!("Engine execution failed: {}", e))
        }
    }
}

/// Format metadata for human-readable display.
fn format_metadata_display(response: &ExecutionResponse) {
    use radium_abstraction::{Citation, LogProb, SafetyRating};
    
    if let Some(ref metadata) = response.metadata {
        // Thinking process (display first if present)
        if let Some(thinking_val) = metadata.get("thinking_process") {
            println!();
            println!("{}", "─".repeat(60).dimmed());
            println!("{}", "Thinking Process:".bold().cyan());
            println!("{}", "─".repeat(60).dimmed());
            
            // Handle different thinking process formats
            match thinking_val {
                serde_json::Value::String(s) => {
                    // Simple string format - display with indentation
                    for line in s.lines() {
                        println!("  {}", line.dimmed());
                    }
                }
                serde_json::Value::Object(obj) => {
                    // Structured format - display as JSON
                    if let Ok(json_str) = serde_json::to_string_pretty(obj) {
                        for line in json_str.lines() {
                            println!("  {}", line.dimmed());
                        }
                    }
                }
                serde_json::Value::Array(arr) => {
                    // Array format - display each item
                    for (i, item) in arr.iter().enumerate() {
                        println!("  [{}] {}", i + 1, serde_json::to_string(item).unwrap_or_default().dimmed());
                    }
                }
                _ => {
                    // Fallback: display as string representation
                    println!("  {}", thinking_val.to_string().dimmed());
                }
            }
            
            println!("{}", "─".repeat(60).dimmed());
        }
        
        // Finish reason
        if let Some(finish_reason) = metadata.get("finish_reason").and_then(|v| v.as_str()) {
            println!("  {} Finish Reason: {}", "•".dimmed(), finish_reason.cyan());
        }
        
        // Safety ratings
        if let Some(safety_ratings_val) = metadata.get("safety_ratings") {
            if let Ok(safety_ratings) = serde_json::from_value::<Vec<SafetyRating>>(safety_ratings_val.clone()) {
                println!("  {} Safety Ratings:", "•".dimmed());
                for rating in &safety_ratings {
                    let blocked_indicator = if rating.blocked { "BLOCKED".red() } else { "OK".green() };
                    println!("    - {}: {} ({})", rating.category.dimmed(), rating.probability.cyan(), blocked_indicator);
                }
            }
        }
        
        // Citations
        if let Some(citations_val) = metadata.get("citations") {
            if let Ok(citations) = serde_json::from_value::<Vec<Citation>>(citations_val.clone()) {
                println!("  {} Citations: {}", "•".dimmed(), citations.len().to_string().cyan());
                for (i, citation) in citations.iter().enumerate().take(3) {
                    if let Some(ref uri) = citation.uri {
                        println!("    {}. {}", i + 1, uri.dimmed());
                    }
                }
                if citations.len() > 3 {
                    println!("    ... and {} more", (citations.len() - 3).to_string().dimmed());
                }
            }
        }
        
        // Log probabilities
        if let Some(logprobs_val) = metadata.get("logprobs") {
            if let Ok(logprobs) = serde_json::from_value::<Vec<LogProb>>(logprobs_val.clone()) {
                println!("  {} Log Probabilities: {} tokens", "•".dimmed(), logprobs.len().to_string().cyan());
            }
        }
        
        // Model version
        if let Some(model_version) = metadata.get("model_version").and_then(|v| v.as_str()) {
            println!("  {} Model Version: {}", "•".dimmed(), model_version.cyan());
        }
    }
}

// ============================================================================
// Tool Execution Support
// ============================================================================

/// Create a Model instance based on engine ID and model name
fn create_model(
    engine_id: &str,
    model: &str,
    api_key: String,
) -> anyhow::Result<Box<dyn Model>> {
    match engine_id {
        "claude" | "anthropic" => {
            Ok(Box::new(ClaudeModel::with_api_key(model.to_string(), api_key)))
        }
        "gemini" => {
            Ok(Box::new(GeminiModel::with_api_key(model.to_string(), api_key)))
        }
        "openai" => {
            Ok(Box::new(OpenAIModel::with_api_key(model.to_string(), api_key)))
        }
        "mock" => {
            Ok(Box::new(MockModel::new(model.to_string())))
        }
        _ => Err(anyhow::anyhow!("Unsupported engine for tool execution: {}", engine_id))
    }
}

/// Convert orchestration Tool to abstraction Tool
fn convert_tools(tools: &[OrchestrationTool]) -> Vec<AbstractionTool> {
    tools.iter().map(|tool| {
        // Convert ToolParameters to JSON Value
        let parameters_json = serde_json::to_value(&tool.parameters)
            .unwrap_or_else(|_| serde_json::json!({}));

        AbstractionTool {
            name: tool.name.clone(),
            description: tool.description.clone(),
            parameters: parameters_json,
        }
    }).collect()
}

/// Convert ModelResponse to ExecutionResponse
fn convert_to_execution_response(
    response: radium_abstraction::ModelResponse,
    model: &str,
) -> radium_core::engines::ExecutionResponse {
    radium_core::engines::ExecutionResponse {
        content: response.content,
        usage: response.usage.map(|u| radium_core::engines::TokenUsage {
            input_tokens: u.prompt_tokens as u64,
            output_tokens: u.completion_tokens as u64,
            total_tokens: u.total_tokens as u64,
        }),
        model: response.model_id.unwrap_or_else(|| model.to_string()),
        raw: None,
        execution_duration: None,
        metadata: response.metadata,
    }
}

/// Execute a single tool call
async fn execute_tool_call(
    tool_call: &ToolCall,
    tools: &[OrchestrationTool],
    workspace_root: &PathBuf,
) -> anyhow::Result<String> {
    // Find the tool by name
    let tool = tools.iter()
        .find(|t| t.name == tool_call.name)
        .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", tool_call.name))?;

    println!("  {} Calling tool: {}", "•".cyan(), tool_call.name.cyan());

    // Execute the tool using the execute method
    use radium_orchestrator::orchestration::tool::ToolArguments;
    let args = ToolArguments::new(tool_call.arguments.clone());
    let result = tool.execute(&args).await
        .map_err(|e| anyhow::anyhow!("Tool execution failed: {}", e))?;

    println!("  {} Tool result: {} bytes", "✓".green(), result.output.len());

    Ok(result.output)
}

/// Multi-turn tool execution loop
async fn execute_with_tools_loop(
    model: &dyn Model,
    mut messages: Vec<ChatMessage>,
    tools: &[AbstractionTool],
    tool_config: &ToolConfig,
    orchestration_tools: &[OrchestrationTool],
    workspace_root: &PathBuf,
) -> anyhow::Result<radium_abstraction::ModelResponse> {
    const MAX_ITERATIONS: usize = 10;

    for iteration in 0..MAX_ITERATIONS {
        println!("  {} Tool execution iteration {}/{}", "•".dimmed(), iteration + 1, MAX_ITERATIONS);

        // Call model with tools
        println!("  {} DEBUG: Calling model.generate_with_tools() with {} messages, {} tools", "•".yellow(), messages.len(), tools.len());
        let response = match model.generate_with_tools(
            &messages,
            tools,
            Some(tool_config),
        ).await {
            Ok(resp) => {
                println!("  {} DEBUG: generate_with_tools() succeeded", "✓".green());
                resp
            }
            Err(e) => {
                eprintln!("  {} ERROR: Model execution failed: {}", "✗".red(), e);
                eprintln!("  {} ERROR: Error details: {:?}", "•".red(), e);
                return Err(anyhow::anyhow!("Model execution failed: {}", e));
            }
        };

        // Check if model wants to call tools
        if let Some(ref tool_calls) = response.tool_calls {
            if tool_calls.is_empty() {
                // No tool calls, return final response
                println!("  {} Model returned final answer", "✓".green());
                return Ok(response);
            }

            println!("  {} Model requested {} tool call(s)", "•".cyan(), tool_calls.len());

            // Add assistant's message to conversation
            messages.push(ChatMessage {
                role: "assistant".to_string(),
                content: MessageContent::Text(response.content.clone()),
            });

            // Execute each tool call and add results
            for tool_call in tool_calls {
                let result = execute_tool_call(tool_call, orchestration_tools, workspace_root).await?;

                // Add tool result as a user message (like TUI does)
                messages.push(ChatMessage {
                    role: "user".to_string(),
                    content: MessageContent::Text(format!(
                        "[Tool result for {}]\n{}",
                        tool_call.name, result
                    )),
                });
            }

            // Continue loop to get next response
            continue;
        }

        // No tool calls - return final response
        println!("  {} Model returned final answer", "✓".green());
        return Ok(response);
    }

    Err(anyhow::anyhow!("Tool execution loop exceeded maximum iterations ({})", MAX_ITERATIONS))
}

/// Execute agent with tool support using Model trait directly
async fn execute_agent_with_tools(
    engine_id: &str,
    model: &str,
    rendered_prompt: &str,
    user_input: &str,
    workspace_root: &PathBuf,
) -> anyhow::Result<radium_core::engines::ExecutionResponse> {
    println!("  {} Executing agent with tools...", "•".cyan());
    println!("  {} Engine: {}", "•".dimmed(), engine_id.cyan());
    println!("  {} Model: {}", "•".dimmed(), model.cyan());
    println!();

    // Get API key from credential store
    let credential_store = Arc::new(
        CredentialStore::new().unwrap_or_else(|_| {
            let temp_path = std::env::temp_dir().join("radium_credentials.json");
            CredentialStore::with_path(temp_path)
        })
    );

    let provider_type = match engine_id {
        "claude" | "anthropic" => Some(ProviderType::Claude),
        "gemini" => Some(ProviderType::Gemini),
        "openai" => Some(ProviderType::OpenAI),
        "mock" => None, // Mock doesn't need API key
        _ => return Err(anyhow::anyhow!("Unsupported engine: {}", engine_id)),
    };

    let api_key = if let Some(provider) = provider_type {
        credential_store
            .get(provider)
            .map_err(|e| anyhow::anyhow!("Failed to get API key for {}: {}", engine_id, e))?
    } else {
        // Mock engine doesn't need API key
        "".to_string()
    };

    // Create Model instance
    let model_instance = create_model(engine_id, model, api_key)?;
    println!("  {} Model instance created", "✓".green());

    // Build tools using tool_builder
    println!("  {} Building tools...", "•".dimmed());
    let orchestration_tools = tool_builder::build_standard_tools(workspace_root.clone(), None);
    println!("  {} Built {} tools", "✓".green(), orchestration_tools.len());

    // Convert to abstraction tools
    let abstraction_tools = convert_tools(&orchestration_tools);

    // DEBUG: Log tool schemas
    println!("  {} Tool schemas:", "•".cyan());
    for (i, tool) in abstraction_tools.iter().enumerate() {
        println!("    {}. {} - {} bytes parameters", i+1, tool.name,
            serde_json::to_string(&tool.parameters).unwrap_or_default().len());
    }

    // Print first tool's full schema for inspection
    if let Some(first_tool) = abstraction_tools.first() {
        println!("  {} Sample schema ({}):", "•".cyan(), first_tool.name);
        println!("{}", serde_json::to_string_pretty(&first_tool.parameters).unwrap_or_default());
    }

    // Build tool config
    let tool_config = ToolConfig {
        mode: ToolUseMode::Auto,  // Let model decide (try this first)
        allowed_function_names: None,
    };

    // Build initial messages (match TUI structure: system + user)
    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: radium_abstraction::MessageContent::Text(rendered_prompt.to_string()),
        },
        ChatMessage {
            role: "user".to_string(),
            content: radium_abstraction::MessageContent::Text(user_input.to_string()),
        },
    ];

    println!("  {} Starting tool execution loop...", "•".cyan());
    println!();

    // Execute with tools loop
    let response = execute_with_tools_loop(
        model_instance.as_ref(),
        messages,
        &abstraction_tools,
        &tool_config,
        &orchestration_tools,
        workspace_root,
    ).await?;

    println!();
    println!("  {} Tool execution completed", "✓".green());

    // Convert to ExecutionResponse
    Ok(convert_to_execution_response(response, model))
}
