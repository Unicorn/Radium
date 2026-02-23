//! Delay component schema
//!
//! The Delay component pauses workflow execution for a specified duration.
//! It is a thin wrapper around a Temporal timer and is classified as a
//! **Stateful** tier component — it carries a timeout behavior only; rate
//! limiting and retry are intentionally omitted because the operation is
//! purely time-based and idempotent by nature.

use serde::{Deserialize, Serialize};
use validator::Validate;

use super::behaviors::ComponentBehaviors;

// ---------------------------------------------------------------------------
// DelayUnit enum
// ---------------------------------------------------------------------------

/// Unit of time used to express the delay value.
///
/// The runtime converts `value` + `unit` into milliseconds before scheduling
/// the Temporal timer. `duration_ms` always holds the pre-computed canonical
/// value in milliseconds.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum DelayUnit {
    /// Milliseconds — finest-grained unit; safe for sub-second delays.
    #[default]
    Milliseconds,
    /// Seconds
    Seconds,
    /// Minutes
    Minutes,
    /// Hours
    Hours,
}

impl DelayUnit {
    /// Convert `value` expressed in this unit to milliseconds.
    pub fn to_milliseconds(&self, value: f64) -> u64 {
        let ms = match self {
            DelayUnit::Milliseconds => value,
            DelayUnit::Seconds => value * 1_000.0,
            DelayUnit::Minutes => value * 60_000.0,
            DelayUnit::Hours => value * 3_600_000.0,
        };
        ms.round() as u64
    }
}

// ---------------------------------------------------------------------------
// DelayInput
// ---------------------------------------------------------------------------

/// Input configuration for the Delay component.
///
/// Two ways to specify the delay are provided for convenience:
/// - **`duration_ms`**: canonical millisecond value used directly by the runtime.
/// - **`unit` + `value`**: human-friendly alternative (e.g. `minutes` / `5.0`).
///
/// When both are supplied the runtime uses `duration_ms` as the authoritative
/// value. Use `jitter_ms` to add a random spread and avoid thundering-herd
/// patterns when many workflow instances sleep at the same time.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct DelayInput {
    /// Canonical delay in milliseconds.
    ///
    /// This is the value passed directly to the Temporal `sleep` call.
    /// Defaults to `1000` (one second).
    #[serde(default = "default_duration_ms")]
    pub duration_ms: u64,

    /// Human-friendly time unit for `value`.
    ///
    /// Defaults to `Milliseconds`. When the unit is not `Milliseconds` the
    /// runtime converts `value` to milliseconds before sleeping.
    #[serde(default)]
    pub unit: DelayUnit,

    /// Amount of time expressed in `unit`.
    ///
    /// Defaults to `1.0`. Fractional values are supported (e.g. `0.5` minutes).
    #[serde(default = "default_value")]
    pub value: f64,

    /// Maximum random jitter in milliseconds added to the scheduled delay.
    ///
    /// When set, the runtime selects a uniform random value in `[0, jitter_ms)`
    /// and adds it to the computed delay. Useful for spreading out concurrent
    /// wake-ups across a fleet of workflows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jitter_ms: Option<u64>,

    /// Shared component behaviors.
    ///
    /// For delay the only meaningful behavior is `timeout_ms` — set to
    /// 1 hour by default so that long delays do not trip the activity timeout.
    /// Rate limiting and retry are not applicable.
    #[serde(default = "delay_default_behaviors")]
    #[validate(nested)]
    pub behaviors: ComponentBehaviors,
}

fn default_duration_ms() -> u64 {
    1_000
}

fn default_value() -> f64 {
    1.0
}

/// Default behaviors for the Delay component.
///
/// - `timeout_ms`: 3_600_000 ms (1 hour) — delays can be long, so the timeout
///   must accommodate the maximum expected sleep duration.
/// - `heartbeat_interval_ms`: `None` — not required for a simple timer.
/// - All other fields inherit `ComponentBehaviors` defaults (rate limit, retry,
///   circuit breaker, etc.) but they are effectively no-ops for this component.
pub fn delay_default_behaviors() -> ComponentBehaviors {
    ComponentBehaviors {
        timeout_ms: 3_600_000,
        heartbeat_interval_ms: None,
        ..Default::default()
    }
}

impl Default for DelayInput {
    fn default() -> Self {
        Self {
            duration_ms: default_duration_ms(),
            unit: DelayUnit::default(),
            value: default_value(),
            jitter_ms: None,
            behaviors: delay_default_behaviors(),
        }
    }
}

// ---------------------------------------------------------------------------
// DelayOutput
// ---------------------------------------------------------------------------

/// Output produced by the Delay component after the sleep completes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DelayOutput {
    /// Actual delay applied in milliseconds, including any jitter.
    pub delayed_ms: u64,

    /// Originally requested delay in milliseconds (before jitter was added).
    pub requested_ms: u64,
}

impl Default for DelayOutput {
    fn default() -> Self {
        Self {
            delayed_ms: default_duration_ms(),
            requested_ms: default_duration_ms(),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_with_defaults() {
        let input = DelayInput::default();

        assert_eq!(input.duration_ms, 1_000);
        assert_eq!(input.unit, DelayUnit::Milliseconds);
        assert_eq!(input.value, 1.0);
        assert!(input.jitter_ms.is_none());

        // Stateful tier: 1-hour timeout, no heartbeat.
        assert_eq!(input.behaviors.timeout_ms, 3_600_000);
        assert!(input.behaviors.heartbeat_interval_ms.is_none());
    }

    #[test]
    fn test_full_config_deserialization() {
        let yaml = r#"
duration_ms: 30000
unit: minutes
value: 0.5
jitter_ms: 500
behaviors:
  timeout_ms: 7200000
"#;
        let input: DelayInput = serde_yaml::from_str(yaml).expect("deserialize");

        assert_eq!(input.duration_ms, 30_000);
        assert_eq!(input.unit, DelayUnit::Minutes);
        assert_eq!(input.value, 0.5);
        assert_eq!(input.jitter_ms, Some(500));
        assert_eq!(input.behaviors.timeout_ms, 7_200_000);

        // Fields not specified in YAML fall back to ComponentBehaviors defaults.
        assert_eq!(input.behaviors.rate_limit.requests_per_second, 10);
    }

    #[test]
    fn test_output_serialize_deserialize() {
        let output = DelayOutput {
            delayed_ms: 5_500,
            requested_ms: 5_000,
        };

        let json = serde_json::to_string(&output).expect("serialize to JSON");
        let restored: DelayOutput = serde_json::from_str(&json).expect("deserialize from JSON");

        assert_eq!(restored.delayed_ms, output.delayed_ms);
        assert_eq!(restored.requested_ms, output.requested_ms);

        // Verify snake_case wire format.
        assert!(json.contains("delayed_ms"));
        assert!(json.contains("requested_ms"));

        // Round-trip through YAML as well.
        let yaml = serde_yaml::to_string(&output).expect("serialize to YAML");
        let restored_yaml: DelayOutput = serde_yaml::from_str(&yaml).expect("deserialize from YAML");
        assert_eq!(restored_yaml.delayed_ms, output.delayed_ms);
        assert_eq!(restored_yaml.requested_ms, output.requested_ms);
    }

    #[test]
    fn test_delay_unit_default() {
        let unit = DelayUnit::default();
        assert_eq!(unit, DelayUnit::Milliseconds);

        // Verify all variants convert correctly to milliseconds.
        assert_eq!(DelayUnit::Milliseconds.to_milliseconds(1_000.0), 1_000);
        assert_eq!(DelayUnit::Seconds.to_milliseconds(1.0), 1_000);
        assert_eq!(DelayUnit::Minutes.to_milliseconds(1.0), 60_000);
        assert_eq!(DelayUnit::Hours.to_milliseconds(1.0), 3_600_000);

        // Fractional values round correctly.
        assert_eq!(DelayUnit::Minutes.to_milliseconds(0.5), 30_000);
        assert_eq!(DelayUnit::Hours.to_milliseconds(0.25), 900_000);

        // All variants round-trip through JSON.
        let variants = [
            DelayUnit::Milliseconds,
            DelayUnit::Seconds,
            DelayUnit::Minutes,
            DelayUnit::Hours,
        ];
        for variant in &variants {
            let json = serde_json::to_string(variant).expect("serialize");
            let restored: DelayUnit = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(&restored, variant);
        }

        // Spot-check wire format.
        assert_eq!(
            serde_json::to_string(&DelayUnit::Milliseconds).unwrap(),
            "\"milliseconds\""
        );
        assert_eq!(
            serde_json::to_string(&DelayUnit::Hours).unwrap(),
            "\"hours\""
        );
    }
}
