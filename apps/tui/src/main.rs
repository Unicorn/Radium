//! Radium TUI - Unified Prompt Interface
//!
//! A CLI-like terminal interface with slash commands and chat functionality.

use std::io::{self, stdout};
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyEventKind},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::prelude::*;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use radium_tui::app::App;
use radium_tui::commands::DisplayContext;
use radium_tui::components::{render_dialog, render_title_bar, render_toasts_with_areas, AppMode, StatusFooter};
use radium_tui::views::{render_checkpoint_browser, render_orchestrator_view, render_prompt, render_setup_wizard, render_shortcuts, render_splash, render_start_page, render_workflow, GlobalLayout};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize minimal logging
    let log_enabled = std::env::var("RUST_LOG_TUI").is_ok();
    if log_enabled {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "radium_tui=warn,error".into()),
            )
            .with(tracing_subscriber::fmt::layer().with_writer(io::stderr))
            .init();
    } else {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "off".into()),
            )
            .with(tracing_subscriber::fmt::layer().with_writer(|| std::io::sink()))
            .init();
    }

    // Setup terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // Show splash screen
    let start_time = std::time::Instant::now();
    let splash_duration = Duration::from_millis(800);

    while start_time.elapsed() < splash_duration {
        terminal.draw(|frame| {
            render_splash(frame, frame.area(), "Loading workspace...");
        })?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    // Skip splash on any key press
                    break;
                }
            }
        }
    }

    // Create app
    let mut app = App::new();

    // Track frame timing for animations
    let mut last_frame_time = std::time::Instant::now();

    // Main loop
    loop {
        // Calculate delta time for animations
        let current_time = std::time::Instant::now();
        let delta_time = last_frame_time.elapsed();
        last_frame_time = current_time;
        // Update toast manager (remove expired toasts)
        app.toast_manager.update();

        // Poll for requirement progress updates (non-blocking) - old system
        if let Some(active_req) = &mut app.active_requirement {
            match active_req.progress_rx.try_recv() {
                Ok(progress) => {
                    // Update active requirement state
                    active_req.update(progress.clone());

                    // Use toast notifications for key events
                    match &progress {
                        radium_core::workflow::RequirementProgress::Started { total_tasks, .. } => {
                            app.toast_manager.info(format!("Starting execution ({} tasks)", total_tasks));
                        }
                        radium_core::workflow::RequirementProgress::TaskCompleted { task_title, .. } => {
                            app.toast_manager.success(format!("Completed: {}", task_title));
                        }
                        radium_core::workflow::RequirementProgress::TaskFailed { task_title, error, .. } => {
                            app.toast_manager.error(format!("Failed: {} - {}", task_title, error));
                        }
                        radium_core::workflow::RequirementProgress::Completed { result } => {
                            if result.success {
                                app.toast_manager.success(format!(
                                    "Requirement {} completed! ({} tasks)",
                                    result.requirement_id, result.tasks_completed
                                ));
                            } else {
                                app.toast_manager.warning(format!(
                                    "Requirement {} completed with {} failures",
                                    result.requirement_id, result.tasks_failed
                                ));
                            }

                            // Show final summary in output
                            app.prompt_data.add_output("".to_string());
                            app.prompt_data.add_output("─".repeat(60));
                            app.prompt_data.add_output("📊 Execution Summary".to_string());
                            app.prompt_data.add_output("─".repeat(60));
                            app.prompt_data.add_output("".to_string());
                            app.prompt_data.add_output(format!("  Requirement: {}", result.requirement_id));
                            app.prompt_data.add_output(format!("  Tasks Completed: {}", result.tasks_completed));
                            app.prompt_data.add_output(format!("  Tasks Failed: {}", result.tasks_failed));
                            app.prompt_data.add_output(format!("  Execution Time: {}s", result.execution_time_secs));
                            app.prompt_data.add_output(format!("  Final Status: {:?}", result.final_status));
                            app.prompt_data.add_output("".to_string());
                            app.prompt_data.add_output("─".repeat(60));

                            // Remove active requirement when done
                            app.active_requirement = None;
                        }
                        radium_core::workflow::RequirementProgress::Failed { error } => {
                            app.toast_manager.error(format!("Execution failed: {}", error));
                            app.prompt_data.add_output(format!("❌ Execution failed: {}", error));
                            app.active_requirement = None;
                        }
                        _ => {
                            // For TaskStarted, just update the UI silently
                        }
                    }
                }
                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                    // No updates available, continue
                }
                Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                    app.toast_manager.warning("Requirement execution channel closed unexpectedly".to_string());
                    app.active_requirement = None;
                }
            }
        }

        // Poll task list and orchestrator logs will be called from orchestrator view rendering
        // The polling methods check elapsed time internally to avoid excessive calls

        // Poll for streaming tokens (non-blocking)
        if let Some(stream_ctx) = &mut app.streaming_context {
            use crate::state::StreamingState;
            
            // Update state to Streaming if it was Connecting
            if stream_ctx.state == StreamingState::Connecting {
                stream_ctx.state = StreamingState::Streaming;
            }
            
            // Poll for tokens
            loop {
                match stream_ctx.token_receiver.try_recv() {
                    Ok(token) => {
                        // Check if token is an error message
                        if token.starts_with("\n[Stream error:") {
                            // Extract error message
                            if let Some(error_end) = token.find("]") {
                                let error_msg = token[16..error_end].to_string();
                                stream_ctx.state = StreamingState::Error(error_msg);
                            } else {
                                stream_ctx.state = StreamingState::Error("Unknown stream error".to_string());
                            }
                            // Still add the error token to output
                            app.prompt_data.add_output(token);
                        } else {
                            // Record timestamp for rate calculation
                            let now = std::time::Instant::now();
                            if stream_ctx.token_timestamps.len() >= 10 {
                                stream_ctx.token_timestamps.pop_front();
                            }
                            stream_ctx.token_timestamps.push_back(now);
                            stream_ctx.token_count += 1;
                            
                            // Add token to buffer
                            stream_ctx.add_token(token);
                            
                            // Flush buffer if it reaches 5-10 tokens
                            if stream_ctx.should_flush() {
                                let flushed = stream_ctx.flush_buffer();
                                if !flushed.is_empty() {
                                    app.prompt_data.add_output(flushed);
                                }
                            }
                        }
                    }
                    Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                        // No tokens available, continue
                        break;
                    }
                    Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                        // Stream ended - flush remaining tokens
                        let remaining = stream_ctx.flush_buffer();
                        if !remaining.is_empty() {
                            app.prompt_data.add_output(remaining);
                        }
                        
                        // Update state based on current state
                        match stream_ctx.state {
                            StreamingState::Cancelled => {
                                // Already cancelled, keep state
                            }
                            StreamingState::Error(_) => {
                                // Already in error state, keep it
                            }
                            _ => {
                                // Mark as completed if not already in error/cancelled state
                                stream_ctx.state = StreamingState::Completed;
                            }
                        }
                        
                        // Get full response for history saving
                        let full_response = stream_ctx.get_full_response();
                        
                        // Save to history (we'll need to get session info from context)
                        // For now, just clear the streaming context after showing completion
                        // TODO: Save to history when streaming completes
                        // Clear after a brief delay to show completion message
                        app.streaming_context = None;
                        break;
                    }
                }
            }
        }

        // Poll for requirement progress updates (non-blocking) - new ProgressMessage system
        if let Some(active_req_progress) = &mut app.active_requirement_progress {
            match active_req_progress.progress_rx.try_recv() {
                Ok(message) => {
                    // Update active requirement progress state
                    active_req_progress.update(message.clone());

                    // Track execution history
                    let req_id = active_req_progress.req_id.clone();
                    

                    // Use toast notifications for key events
                    match &message {
                        radium_tui::progress_channel::ProgressMessage::StatusChange { task_id, task_title, status } => {
                            let status_symbol = status.symbol();
                            app.toast_manager.info(format!("{} {}", status_symbol, task_title));

                            // Create or update execution record
                            let engine = "unknown".to_string(); // TODO: Get from app state/config
                            let model = "unknown".to_string(); // TODO: Get from app state/config
                            
                            let record = app.execution_history.get_or_create_active_record(
                                task_id.clone(),
                                task_title.clone(),
                                req_id.clone(),
                                engine,
                                model,
                                0, // retry_attempt - TODO: Track this
                                1, // cycle_number - TODO: Track this
                            );

                            match status {
                                radium_tui::progress_channel::TaskStatus::Running => {
                                    record.mark_running();
                                }
                                _ => {}
                            }
                        }
                        radium_tui::progress_channel::ProgressMessage::TokenUpdate { task_id, tokens_in, tokens_out } => {
                            // Update tokens for active record
                            if let Some(record) = app.execution_history.get_active_record_mut(task_id) {
                                record.update_tokens(*tokens_in, *tokens_out, 0); // cached tokens not available
                            }
                        }
                        radium_tui::progress_channel::ProgressMessage::DurationUpdate {  .. } => {
                            // Update silently, duration is shown in status message
                            // Duration is calculated automatically when record is finalized
                        }
                        radium_tui::progress_channel::ProgressMessage::TaskComplete { task_id, result: _ } => {
                            app.toast_manager.success(format!("{} Task completed: {}", radium_tui::progress_channel::TaskStatus::Completed.symbol(), task_id));
                            
                            // Mark record as completed and finalize
                            if let Some(record) = app.execution_history.get_active_record_mut(task_id) {
                                record.mark_completed();
                                let record_clone = record.clone();
                                app.execution_history.finalize_active_record(task_id);
                                
                                // Save to disk
                                if let Some(ref ws) = app.workspace_status {
                                    if let Some(ref root) = ws.root {
                                        let history_path = radium_tui::state::ExecutionHistory::default_history_path(root);
                                        let _ = app.execution_history.append_to_file(&history_path, &record_clone);
                                    }
                                }
                            } else {
                                app.execution_history.finalize_active_record(task_id);
                            }
                        }
                        radium_tui::progress_channel::ProgressMessage::TaskFailed { task_id, error } => {
                            app.toast_manager.error(format!("{} Task failed: {}", radium_tui::progress_channel::TaskStatus::Failed.symbol(), error));
                            
                            // Mark record as failed and finalize
                            if let Some(record) = app.execution_history.get_active_record_mut(task_id) {
                                record.mark_failed(error.clone());
                                let record_clone = record.clone();
                                app.execution_history.finalize_active_record(task_id);
                                
                                // Save to disk
                                if let Some(ref ws) = app.workspace_status {
                                    if let Some(ref root) = ws.root {
                                        let history_path = radium_tui::state::ExecutionHistory::default_history_path(root);
                                        let _ = app.execution_history.append_to_file(&history_path, &record_clone);
                                    }
                                }
                            } else {
                                app.execution_history.finalize_active_record(task_id);
                            }
                        }
                        radium_tui::progress_channel::ProgressMessage::RequirementComplete { requirement_id, result } => {
                            if result.tasks_failed == 0 {
                                app.toast_manager.success(format!(
                                    "{} Requirement {} completed! ({} tasks, {}s)",
                                    radium_tui::progress_channel::TaskStatus::Completed.symbol(),
                                    requirement_id,
                                    result.tasks_completed,
                                    result.execution_time_secs
                                ));
                            } else {
                                app.toast_manager.warning(format!(
                                    "{} Requirement {} completed with {} failures ({} tasks, {}s)",
                                    radium_tui::progress_channel::TaskStatus::Failed.symbol(),
                                    requirement_id,
                                    result.tasks_failed,
                                    result.tasks_completed,
                                    result.execution_time_secs
                                ));
                            }

                            // Show final summary in output
                            app.prompt_data.add_output("".to_string());
                            app.prompt_data.add_output("─".repeat(60));
                            app.prompt_data.add_output("📊 Execution Summary".to_string());
                            app.prompt_data.add_output("─".repeat(60));
                            app.prompt_data.add_output("".to_string());
                            app.prompt_data.add_output(format!("  Requirement: {}", result.requirement_id));
                            app.prompt_data.add_output(format!("  Tasks Completed: {}", result.tasks_completed));
                            app.prompt_data.add_output(format!("  Tasks Failed: {}", result.tasks_failed));
                            app.prompt_data.add_output(format!("  Execution Time: {}s", result.execution_time_secs));
                            app.prompt_data.add_output(format!("  Tokens: {} in, {} out", active_req_progress.tokens_in, active_req_progress.tokens_out));
                            app.prompt_data.add_output("".to_string());
                            app.prompt_data.add_output("─".repeat(60));

                            // Check if task handle is complete and clean up
                            if let Some(ref handle) = app.active_requirement_handle {
                                if handle.is_finished() {
                                    // Task completed, clean up
                                    app.active_requirement_progress = None;
                                    app.active_requirement_handle = None;
                                }
                            } else {
                                // No handle to check, just clean up progress
                                app.active_requirement_progress = None;
                            }
                        }
                    }
                }
                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                    // No updates available, continue
                }
                Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                    app.toast_manager.warning("Requirement execution channel closed unexpectedly".to_string());
                    app.active_requirement_progress = None;
                    app.active_requirement_handle = None;
                }
            }
        }

        // Draw UI
        terminal.draw(|frame| {
            let area = frame.area();

            // Add padding around the edges
            let padding = 1;
            let padded_area = Rect {
                x: area.x + padding,
                y: area.y + padding,
                width: area.width.saturating_sub(padding * 2),
                height: area.height.saturating_sub(padding * 2),
            };

            // Create global layout structure
            let [title_area, main_area, status_area] = GlobalLayout::create(padded_area);

            // Render title bar (always visible)
            let version = env!("CARGO_PKG_VERSION");
            let model_info = None; // TODO: Get from app state
            let orchestration_status = if app.orchestration_enabled {
                Some("enabled")
            } else {
                Some("disabled")
            };
            
            // Get connected services
            let connected_services = {
                use radium_core::auth::{CredentialStore, ProviderType};
                if let Ok(store) = CredentialStore::new() {
                    let mut providers = Vec::new();
                    if store.is_configured(ProviderType::Gemini) {
                        providers.push("Gemini".to_string());
                    }
                    if store.is_configured(ProviderType::OpenAI) {
                        providers.push("OpenAI".to_string());
                    }
                    providers
                } else {
                    Vec::new()
                }
            };
            
            render_title_bar(frame, title_area, version, model_info, orchestration_status, &connected_services, Some(&app.privacy_state));

            // Detect view context changes for transitions
            let context_changed = app.previous_context.as_ref()
                .map(|prev| !radium_tui::effects::view_transitions::contexts_equal(prev, &app.prompt_data.context))
                .unwrap_or(true);

            // Trigger view transition animation if context changed
            if context_changed && app.previous_context.is_some() {
                use radium_tui::effects::view_transitions::create_dissolve_transition;
                use tachyonfx::CellFilter;
                let effect = create_dissolve_transition(400)
                    .with_filter(CellFilter::Area(main_area));
                app.effect_manager.add_effect(effect);
            }

            // Detect table selection changes for animations
            // Note: Table animations are handled at render time, but we track state here
            // The actual animation will be applied when the table is rendered

            // Render main content area (context-aware)
            if app.show_checkpoint_browser {
                if let Some(ref browser_state) = app.checkpoint_browser_state {
                    render_checkpoint_browser(frame, main_area, browser_state);
                }
            } else if app.show_shortcuts {
                render_shortcuts(frame, main_area);
            } else if let Some(wizard) = &app.setup_wizard {
                render_setup_wizard(frame, main_area, wizard);
            } else if let Some(ref workflow_state) = app.workflow_state {
                // Workflow mode: split-panel layout
                render_workflow(
                    frame,
                    main_area,
                    workflow_state,
                    app.selected_agent_id.as_deref(),
                    app.spinner_frame,
                    app.config.animations.enabled,
                    app.config.animations.reduced_motion,
                );
            } else if app.orchestration_running {
                // Orchestrator running: show split view with chat log, task list, and orchestrator thinking
                // Get active agents from orchestration service (simplified for now)
                let active_agents: Vec<(String, String, String)> = vec![]; // TODO: Get from orchestration service
                render_orchestrator_view(
                    frame,
                    main_area,
                    &app.prompt_data,
                    &active_agents,
                    app.task_list_state.as_ref(),
                    &mut app.orchestrator_panel,
                    (app.task_panel_visible, app.orchestrator_panel_visible),
                    app.panel_focus,
                );
            } else {
                // Check if we should show start page (Help context) or regular prompt
                match app.prompt_data.context {
                    DisplayContext::CostDashboard => {
                        // Cost dashboard mode
                        if let Some(ref mut state) = app.cost_dashboard_state {
                            // Get workspace and monitoring service
                            if let Ok(workspace) = radium_core::Workspace::discover() {
                                let monitoring_path = workspace.radium_dir().join("monitoring.db");
                                if let Ok(monitoring) = radium_core::monitoring::MonitoringService::open(monitoring_path) {
                                    let analytics = radium_core::analytics::CostAnalytics::new(&monitoring);
                                    radium_tui::views::render_cost_dashboard(frame, main_area, state, &analytics);
                                } else {
                                    // Error: show message
                                    let error_text = "Error: Failed to open monitoring database";
                                    let widget = ratatui::widgets::Paragraph::new(error_text)
                                        .style(ratatui::style::Style::default().fg(ratatui::style::Color::Red))
                                        .alignment(ratatui::prelude::Alignment::Center)
                                        .block(ratatui::widgets::Block::default()
                                            .borders(ratatui::widgets::Borders::ALL)
                                            .title(" Error "));
                                    frame.render_widget(widget, main_area);
                                }
                            } else {
                                // Error: show message
                                let error_text = "Error: No Radium workspace found";
                                let widget = ratatui::widgets::Paragraph::new(error_text)
                                    .style(ratatui::style::Style::default().fg(ratatui::style::Color::Red))
                                    .alignment(ratatui::prelude::Alignment::Center)
                                    .block(ratatui::widgets::Block::default()
                                        .borders(ratatui::widgets::Borders::ALL)
                                        .title(" Error "));
                                frame.render_widget(widget, main_area);
                            }
                        } else {
                            // No state: show loading or error
                            let widget = ratatui::widgets::Paragraph::new("Loading cost dashboard...")
                                .alignment(ratatui::prelude::Alignment::Center)
                                .block(ratatui::widgets::Block::default()
                                    .borders(ratatui::widgets::Borders::ALL)
                                    .title(" Loading "));
                            frame.render_widget(widget, main_area);
                        }
                    }
                    DisplayContext::Help => {
                        // Start page mode: codemachine-style start page
                        render_start_page(frame, main_area, &app.prompt_data);
                    }
                    _ => {
                        // Prompt mode: unified prompt interface (without input - that's in status bar)
                        render_prompt(frame, main_area, &app.prompt_data, Some(&app.model_filter));
                    }
                }
            }

            // Render status bar with input prompt (always visible)
            let mode = if app.orchestration_running {
                AppMode::Chat
            } else if app.workflow_state.is_some() {
                AppMode::Workflow
            } else {
                AppMode::Prompt
            };
            // Check if streaming is active - show streaming footer if so
            if let Some(ref stream_ctx) = app.streaming_context {
                StatusFooter::render_streaming_footer(
                    frame,
                    status_area,
                    stream_ctx,
                    app.spinner_frame,
                    app.config.animations.enabled,
                    app.config.animations.reduced_motion,
                );
            } else {
                StatusFooter::render_with_input(
                    frame,
                    status_area,
                    &app.prompt_data.input,
                    mode,
                    Some(&app.prompt_data.context),
                    app.current_model_id.as_deref(),
                    Some(&app.privacy_state),
                );
            }

            // Render dialogs (on top of everything except shortcuts)
            let dialog_areas = if !app.show_shortcuts {
                if let Some(dialog) = app.dialog_manager.current() {
                    let (backdrop_area, dialog_area) = render_dialog(frame, area, dialog);
                    Some((backdrop_area, dialog_area))
                } else {
                    None
                }
            } else {
                None
            };

            // Detect dialog state changes and trigger animations
            let current_dialog_open = app.dialog_manager.is_open();
            if current_dialog_open && !app.previous_dialog_open {
                // Dialog just opened - animate it
                if let Some((backdrop_area, dialog_area)) = dialog_areas {
                    use radium_tui::effects::dialog_animations::create_dialog_open_animation;
                    let effect = create_dialog_open_animation(backdrop_area, dialog_area, 300);
                    app.effect_manager.add_effect(effect);
                }
            } else if !current_dialog_open && app.previous_dialog_open {
                // Dialog just closed - animate close (if we had the areas, but they're gone now)
                // Note: Close animation would need to be triggered before dialog is removed
                // For now, we'll handle this in the dialog manager if needed
            }

            // Render execution views (on top of main content, below dialogs)
            match &mut app.active_execution_view {
                radium_tui::app::ExecutionView::History(view) => {
                    view.render(frame, area);
                }
                radium_tui::app::ExecutionView::Detail(view) => {
                    view.render(frame, area);
                }
                radium_tui::app::ExecutionView::Summary(view) => {
                    view.render(frame, area);
                }
                radium_tui::app::ExecutionView::None => {}
            }

            // Render checkpoint interrupt modal (on top of execution views, below toasts)
            if let Some(ref interrupt_state) = app.checkpoint_interrupt_state {
                if interrupt_state.is_active() {
                    use radium_tui::components::CheckpointInterruptModal;
                    CheckpointInterruptModal::render(frame, area, interrupt_state, &app.theme);
                }
            }

            // Render toasts (on top of everything)
            let toast_areas = render_toasts_with_areas(frame, area, &app.toast_manager);

            // Detect toast changes and trigger animations
            let current_toast_count = app.toast_manager.toasts().len();
            if current_toast_count > app.previous_toast_count {
                // New toasts appeared - animate them
                use radium_tui::effects::toast_animations::create_toast_show_animation;
                use tachyonfx::CellFilter;
                for (idx, toast_area) in toast_areas.iter().enumerate() {
                    if idx < current_toast_count && idx >= app.previous_toast_count {
                        // This is a new toast - animate it
                        let effect = create_toast_show_animation(300)
                            .with_filter(CellFilter::Area(*toast_area));
                        app.effect_manager.add_effect(effect);
                    }
                }
            }

            // Process and apply effects after all rendering is complete
            app.effect_manager.process_effects(delta_time, frame.buffer_mut(), area);
            
            // Increment spinner frame counter for animations (target 60fps)
            app.spinner_frame = app.spinner_frame.wrapping_add(1);
        })?;

        // Update previous state for transition detection (after rendering)
        app.previous_context = Some(app.prompt_data.context.clone());
        app.previous_dialog_open = app.dialog_manager.is_open();
        app.previous_toast_count = app.toast_manager.toasts().len();

        // Handle events with timeout
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.handle_key(key.code, key.modifiers).await?;
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    let _ = disable_raw_mode();
    let _ = stdout().execute(LeaveAlternateScreen);

    Ok(())
}
