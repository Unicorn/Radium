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
use radium_tui::components::{render_dialog, render_title_bar, render_toasts, AppMode, StatusFooter};
use radium_tui::views::{render_orchestrator_view, render_prompt, render_setup_wizard, render_shortcuts, render_splash, render_start_page, render_workflow, GlobalLayout};

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

        // Poll for requirement progress updates (non-blocking)
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
            
            render_title_bar(frame, title_area, version, model_info, orchestration_status, &connected_services);

            // Detect view context changes for transitions
            let context_changed = app.previous_context.as_ref()
                .map(|prev| !crate::effects::view_transitions::contexts_equal(prev, &app.prompt_data.context))
                .unwrap_or(true);

            // Trigger view transition animation if context changed
            if context_changed && app.previous_context.is_some() {
                use crate::effects::view_transitions::create_dissolve_transition;
                use tachyonfx::CellFilter;
                let effect = create_dissolve_transition(400)
                    .with_filter(CellFilter::Area(main_area));
                app.effect_manager.add_effect(effect);
            }

            // Render main content area (context-aware)
            if app.show_shortcuts {
                render_shortcuts(frame, main_area);
            } else if let Some(wizard) = &app.setup_wizard {
                render_setup_wizard(frame, main_area, wizard);
            } else if let Some(ref workflow_state) = app.workflow_state {
                // Workflow mode: split-panel layout
                render_workflow(frame, main_area, workflow_state, app.selected_agent_id.as_deref());
            } else if app.orchestration_running {
                // Orchestrator running: show split view with chat log and active agents
                // Get active agents from orchestration service (simplified for now)
                let active_agents: Vec<(String, String, String)> = vec![]; // TODO: Get from orchestration service
                render_orchestrator_view(frame, main_area, &app.prompt_data, &active_agents);
            } else {
                // Check if we should show start page (Help context) or regular prompt
                match app.prompt_data.context {
                    DisplayContext::Help => {
                        // Start page mode: codemachine-style start page
                        render_start_page(frame, main_area, &app.prompt_data);
                    }
                    _ => {
                        // Prompt mode: unified prompt interface (without input - that's in status bar)
                        render_prompt(frame, main_area, &app.prompt_data);
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
            StatusFooter::render_with_input(
                frame,
                status_area,
                &app.prompt_data.input,
                mode,
                Some(&app.prompt_data.context),
            );

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
                    use crate::effects::dialog_animations::create_dialog_open_animation;
                    let effect = create_dialog_open_animation(backdrop_area, dialog_area, 300);
                    app.effect_manager.add_effect(effect);
                }
            } else if !current_dialog_open && app.previous_dialog_open {
                // Dialog just closed - animate close (if we had the areas, but they're gone now)
                // Note: Close animation would need to be triggered before dialog is removed
                // For now, we'll handle this in the dialog manager if needed
            }

            // Render toasts (on top of everything)
            let toast_areas = render_toasts_with_areas(frame, area, &app.toast_manager);

            // Detect toast changes and trigger animations
            let current_toast_count = app.toast_manager.toasts().len();
            if current_toast_count > app.previous_toast_count {
                // New toasts appeared - animate them
                use crate::effects::toast_animations::create_toast_show_animation;
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
