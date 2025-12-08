//! View transition effects for smooth context switching.

use ratatui::prelude::*;
use tachyonfx::{fx, pattern::RadialPattern, Effect, EffectTimer, Interpolation};

use crate::commands::DisplayContext;

/// Creates a dissolve transition effect for view changes
pub fn create_dissolve_transition(duration_ms: u64) -> Effect {
    fx::dissolve(create_timer(duration_ms, Interpolation::QuadInOut))
        .with_pattern(RadialPattern::center())
}

/// Creates a fade transition effect for view changes
pub fn create_fade_transition(duration_ms: u64) -> Effect {
    fx::fade_from_fg(Color::Black, create_timer(duration_ms, Interpolation::QuadInOut))
}

/// Creates a slide transition effect for view changes
pub fn create_slide_transition(direction: tachyonfx::Motion, duration_ms: u64) -> Effect {
    fx::slide_in(
        direction,
        5,  // gradient_length
        0,  // randomness
        Color::Black,  // color_behind_cells
        create_timer(duration_ms, Interpolation::QuadInOut),
    )
}

/// Detects if a view context has changed
pub fn has_context_changed(previous: Option<&DisplayContext>, current: &DisplayContext) -> bool {
    match previous {
        Some(prev) => !contexts_equal(prev, current),
        None => true, // First render, consider it a change
    }
}

/// Compares two display contexts for equality
pub fn contexts_equal(a: &DisplayContext, b: &DisplayContext) -> bool {
    match (a, b) {
        (
            DisplayContext::Chat {
                agent_id: a_id,
                session_id: a_sess,
            },
            DisplayContext::Chat {
                agent_id: b_id,
                session_id: b_sess,
            },
        ) => a_id == b_id && a_sess == b_sess,
        (DisplayContext::AgentList, DisplayContext::AgentList) => true,
        (DisplayContext::SessionList, DisplayContext::SessionList) => true,
        (DisplayContext::ModelSelector, DisplayContext::ModelSelector) => true,
        (DisplayContext::Dashboard, DisplayContext::Dashboard) => true,
        (DisplayContext::Help, DisplayContext::Help) => true,
        _ => false,
    }
}

fn create_timer(duration_ms: u64, interpolation: Interpolation) -> EffectTimer {
    EffectTimer::from_ms(duration_ms as u32, interpolation)
}

