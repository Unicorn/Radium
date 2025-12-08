//! TachyonFX effects integration for Radium TUI.
//!
//! Provides animation and transition effects throughout the application.

use ratatui::prelude::*;
use tachyonfx::{Duration, EffectManager, EffectTimer, Interpolation};

pub mod dialog_animations;
pub mod toast_animations;
pub mod view_transitions;

/// Wrapper around TachyonFX EffectManager with app-specific helpers
pub struct AppEffectManager {
    /// Underlying effect manager
    manager: EffectManager<()>,
    /// Maximum concurrent effects to prevent performance issues
    max_effects: usize,
    /// Whether animations are enabled
    enabled: bool,
    /// Duration multiplier for animations
    duration_multiplier: f64,
    /// Whether to use reduced motion
    reduced_motion: bool,
}

impl AppEffectManager {
    /// Creates a new effect manager
    pub fn new() -> Self {
        Self {
            manager: EffectManager::default(),
            max_effects: 15,
            enabled: true,
            duration_multiplier: 1.0,
            reduced_motion: false,
        }
    }

    /// Creates an effect manager with custom max effects limit
    pub fn with_max_effects(max_effects: usize) -> Self {
        Self {
            manager: EffectManager::default(),
            max_effects,
            enabled: true,
            duration_multiplier: 1.0,
            reduced_motion: false,
        }
    }

    /// Creates an effect manager with configuration
    pub fn with_config(enabled: bool, duration_multiplier: f64, reduced_motion: bool) -> Self {
        Self {
            manager: EffectManager::default(),
            max_effects: 15,
            enabled,
            duration_multiplier,
            reduced_motion,
        }
    }

    /// Updates configuration
    pub fn update_config(&mut self, enabled: bool, duration_multiplier: f64, reduced_motion: bool) {
        self.enabled = enabled;
        self.duration_multiplier = duration_multiplier;
        self.reduced_motion = reduced_motion;
    }

    /// Processes all active effects with the given delta time
    pub fn process_effects(&mut self, delta: std::time::Duration, buffer: &mut Buffer, area: Rect) {
        if !self.enabled {
            return;
        }

        // Clean up completed effects if we have too many
        // Effects are automatically cleaned up when completed by process_effects

        // With std-duration feature, Duration is the same as std::time::Duration
        let tachyon_duration = Duration::from_millis(delta.as_millis() as u32);
        self.manager.process_effects(tachyon_duration, buffer, area);
    }

    /// Adds an effect to the manager
    pub fn add_effect(&mut self, effect: tachyonfx::Effect) {
        if !self.enabled {
            return;
        }
        self.manager.add_effect(effect);
    }

    /// Returns whether there are active effects
    pub fn is_running(&self) -> bool {
        self.manager.is_running()
    }
}

impl Default for AppEffectManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to create a timer with interpolation
pub fn create_timer(duration_ms: u64, interpolation: Interpolation) -> EffectTimer {
    EffectTimer::from_ms(duration_ms as u32, interpolation)
}

/// Helper to create a timer with default interpolation (QuadOut)
pub fn create_timer_default(duration_ms: u64) -> EffectTimer {
    EffectTimer::from_ms(duration_ms as u32, Interpolation::QuadOut)
}

