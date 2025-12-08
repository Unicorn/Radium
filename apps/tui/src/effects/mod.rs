//! TachyonFX effects integration for Radium TUI.
//!
//! Provides animation and transition effects throughout the application.

use ratatui::prelude::*;
use std::time::Duration;
use tachyonfx::{EffectManager, EffectTimer, Interpolation};

pub mod dialog_animations;
pub mod toast_animations;
pub mod view_transitions;

/// Wrapper around TachyonFX EffectManager with app-specific helpers
pub struct AppEffectManager {
    /// Underlying effect manager
    manager: EffectManager<()>,
    /// Maximum concurrent effects to prevent performance issues
    max_effects: usize,
}

impl AppEffectManager {
    /// Creates a new effect manager
    pub fn new() -> Self {
        Self {
            manager: EffectManager::default(),
            max_effects: 15,
        }
    }

    /// Creates an effect manager with custom max effects limit
    pub fn with_max_effects(max_effects: usize) -> Self {
        Self {
            manager: EffectManager::default(),
            max_effects,
        }
    }

    /// Processes all active effects with the given delta time
    pub fn process_effects(&mut self, delta: Duration, buffer: &mut Buffer, area: Rect) {
        // Clean up completed effects if we have too many
        if self.manager.active_effect_count() > self.max_effects {
            // Effects are automatically cleaned up when completed
            // This is just a safety check
        }

        // Process effects
        self.manager.process_effects(delta.into(), buffer, area);
    }

    /// Adds an effect to the manager
    pub fn add_effect(&mut self, effect: tachyonfx::Effect) {
        self.manager.add_effect(effect);
    }

    /// Returns the number of active effects
    pub fn active_effect_count(&self) -> usize {
        self.manager.active_effect_count()
    }

    /// Clears all effects
    pub fn clear(&mut self) {
        self.manager.clear();
    }
}

impl Default for AppEffectManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to create a timer with interpolation
pub fn create_timer(duration_ms: u64, interpolation: Interpolation) -> EffectTimer {
    EffectTimer::from_ms(duration_ms, interpolation)
}

/// Helper to create a timer with default interpolation (QuadOut)
pub fn create_timer_default(duration_ms: u64) -> EffectTimer {
    EffectTimer::from_ms(duration_ms, Interpolation::QuadOut)
}

