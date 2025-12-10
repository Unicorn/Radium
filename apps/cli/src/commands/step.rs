//! Step command implementation.
//!
//! Executes a single agent from configuration.

use anyhow::{Context, bail};
use chrono::Utc;
use colored::Colorize;
use futures::StreamExt;
use radium_abstraction::{ModelParameters, StreamingModel};
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
use radium_models::{GeminiModel, MockModel, OpenAIModel};
use std::io::Write;
use std::sync::Arc;
use std::fs;
use uuid::Uuid;

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
) -> anyhow::Result<()> {
    println!("{}", "rad step".bold().cyan());
    println!();
    
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
    let selected_reasoning =
        reasoning.as_deref().unwrap_or_else(|| match agent.reasoning_effort.unwrap_or_default() {
            radium_core::ReasoningEffort::Low => "low",
            radium_core::ReasoningEffort::Medium => "medium",
            radium_core::ReasoningEffort::High => "high",
        });

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

    // Execute agent (simulated)
    println!();
    println!("{}", "Executing agent...".bold());
    println!();

    let execution_result = execute_agent_with_engine(&registry, &agent.id, &rendered, selected_engine_id, selected_model, stream).await;
    
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
    let request = ExecutionRequest::new(model.to_string(), rendered_prompt.to_string());

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
                let parameters = if request.temperature.is_some() || request.max_tokens.is_some() {
                    Some(ModelParameters {
                        temperature: request.temperature,
                        top_p: None,
                        max_tokens: request.max_tokens.map(|t| t as u32),
                        top_k: None,
                        frequency_penalty: None,
                        presence_penalty: None,
                        response_format: None,
                        stop_sequences: None,
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
                
                while let Some(result) = stream.next().await {
                    match result {
                        Ok(content) => {
                            // Only print the new part (delta)
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
                
                // Return response with accumulated content
                Ok(radium_core::engines::ExecutionResponse {
                    content: accumulated,
                    usage: None, // Token usage not available during streaming
                    model: model.to_string(),
                    raw: None,
                    execution_duration: None,
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
                
                let parameters = if request.temperature.is_some() || request.max_tokens.is_some() {
                    Some(ModelParameters {
                        temperature: request.temperature,
                        top_p: None,
                        max_tokens: request.max_tokens.map(|t| t as u32),
                        top_k: None,
                        frequency_penalty: None,
                        presence_penalty: None,
                        response_format: None,
                        stop_sequences: None,
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
                
                while let Some(result) = stream.next().await {
                    match result {
                        Ok(content) => {
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
                })
            }
            "mock" => {
                let mock_model = MockModel::new(model.to_string());
                
                let parameters = if request.temperature.is_some() || request.max_tokens.is_some() {
                    Some(ModelParameters {
                        temperature: request.temperature,
                        top_p: None,
                        max_tokens: request.max_tokens.map(|t| t as u32),
                        top_k: None,
                        frequency_penalty: None,
                        presence_penalty: None,
                        response_format: None,
                        stop_sequences: None,
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
                        Ok(content) => {
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
                })
            }
            _ => {
                // Engine doesn't support streaming, fall back to non-streaming
                println!("  {} Streaming not supported for engine '{}', using standard mode", "⚠".yellow(), engine_id);
                // Fall through to normal execution
                return execute_normal(engine, request).await;
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
    execute_normal(engine, request).await
}

/// Execute the engine normally (non-streaming).
async fn execute_normal(
    engine: &Arc<dyn radium_core::engines::Engine>,
    request: ExecutionRequest,
) -> anyhow::Result<radium_core::engines::ExecutionResponse> {
    // Execute the engine
    match engine.execute(request).await {
        Ok(response) => {
            println!("{}", "Response:".bold().green());
            println!("{}", "─".repeat(60).dimmed());
            print_highlighted_output(&response.content);
            println!("{}", "─".repeat(60).dimmed());

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
