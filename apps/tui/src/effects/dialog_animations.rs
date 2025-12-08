//! Dialog animation effects.

use ratatui::prelude::*;
use tachyonfx::{fx, CellFilter, Effect, EffectTimer, Interpolation};

/// Creates a backdrop fade-in effect
pub fn create_backdrop_fade_in(duration_ms: u64) -> Effect {
    fx::fade_from(
        Color::Black,
        Color::Rgb(0, 0, 0), // Fully opaque black
        create_timer(duration_ms, Interpolation::QuadOut),
    )
}

/// Creates a dialog fade-in effect
pub fn create_dialog_fade_in(duration_ms: u64) -> Effect {
    fx::fade_from_fg(Color::Black, create_timer(duration_ms, Interpolation::QuadOut))
}

/// Creates a combined backdrop and dialog animation for opening
pub fn create_dialog_open_animation(
    backdrop_area: Rect,
    dialog_area: Rect,
    duration_ms: u64,
) -> Effect {
    let backdrop_effect = create_backdrop_fade_in(duration_ms)
        .with_filter(CellFilter::Area(backdrop_area));
    
    let dialog_effect = create_dialog_fade_in(duration_ms + 100) // Slightly longer for dialog
        .with_filter(CellFilter::Area(dialog_area));

    fx::parallel(&[backdrop_effect, dialog_effect])
}

/// Creates a dialog close animation
pub fn create_dialog_close_animation(
    backdrop_area: Rect,
    dialog_area: Rect,
    duration_ms: u64,
) -> Effect {
    let backdrop_effect = fx::fade_to(
        Color::Rgb(0, 0, 0),
        Color::Black,
        create_timer(duration_ms, Interpolation::QuadIn),
    )
    .with_filter(CellFilter::Area(backdrop_area));

    let dialog_effect = fx::fade_to_fg(
        Color::Black,
        create_timer(duration_ms, Interpolation::QuadIn),
    )
    .with_filter(CellFilter::Area(dialog_area));

    fx::parallel(&[backdrop_effect, dialog_effect])
}

fn create_timer(duration_ms: u64, interpolation: Interpolation) -> EffectTimer {
    EffectTimer::from_ms(duration_ms as u32, interpolation)
}

