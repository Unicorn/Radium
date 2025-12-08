//! Toast notification animation effects.

use ratatui::prelude::*;
use tachyonfx::{fx, CellFilter, Effect, EffectTimer, Interpolation};

/// Creates a slide-in animation for toast notifications
pub fn create_toast_slide_in(duration_ms: u64) -> Effect {
    fx::slide_in(
        tachyonfx::Motion::RightToLeft,
        5,  // gradient_length
        0,  // randomness
        Color::Black,  // color_behind_cells
        create_timer(duration_ms, Interpolation::QuadOut),
    )
}

/// Creates a fade-in animation for toast notifications
pub fn create_toast_fade_in(duration_ms: u64) -> Effect {
    fx::fade_from_fg(Color::Black, create_timer(duration_ms, Interpolation::QuadOut))
}

/// Creates a combined slide-in and fade-in for toast show
pub fn create_toast_show_animation(duration_ms: u64) -> Effect {
    fx::parallel(&[
        create_toast_slide_in(duration_ms),
        create_toast_fade_in(duration_ms),
    ])
}

/// Creates a fade-out animation for toast dismiss
pub fn create_toast_fade_out(duration_ms: u64) -> Effect {
    fx::fade_to_fg(
        Color::Black,
        create_timer(duration_ms, Interpolation::QuadIn),
    )
}

/// Creates an animation that targets a specific area (for toast)
pub fn create_toast_animation_for_area(
    area: Rect,
    show: bool,
    duration_ms: u64,
) -> Effect {
    let effect = if show {
        create_toast_show_animation(duration_ms)
    } else {
        create_toast_fade_out(duration_ms)
    };

    // Apply cell filter to target only the toast area
    effect.with_filter(CellFilter::Area(area))
}

fn create_timer(duration_ms: u64, interpolation: Interpolation) -> EffectTimer {
    EffectTimer::from_ms(duration_ms as u32, interpolation)
}

