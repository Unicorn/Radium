//! Timer component schema
//!
//! The Timer component pauses workflow execution for a specified duration.
//! Supports both fixed durations and dynamic expressions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Timer type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum TimerType {
    /// Fixed duration
    #[default]
    Duration,
    /// Wait until specific time
    UntilTime,
}

/// Duration unit
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum DurationUnit {
    Milliseconds,
    Seconds,
    #[default]
    Minutes,
    Hours,
    Days,
}

impl DurationUnit {
    /// Convert value to milliseconds
    pub fn to_milliseconds(&self, value: u64) -> u64 {
        match self {
            DurationUnit::Milliseconds => value,
            DurationUnit::Seconds => value * 1000,
            DurationUnit::Minutes => value * 60 * 1000,
            DurationUnit::Hours => value * 60 * 60 * 1000,
            DurationUnit::Days => value * 24 * 60 * 60 * 1000,
        }
    }
}

/// Timer component input
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct TimerInput {
    /// Timer type
    #[serde(default)]
    pub timer_type: TimerType,

    /// Duration value (for Duration type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<u64>,

    /// Duration unit
    #[serde(default)]
    pub unit: DurationUnit,

    /// Target time (for UntilTime type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub until_time: Option<DateTime<Utc>>,

    /// Dynamic duration from variable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_variable: Option<String>,

    /// Description/label for the timer
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl TimerInput {
    /// Create a duration-based timer
    pub fn duration(value: u64, unit: DurationUnit) -> Self {
        Self {
            timer_type: TimerType::Duration,
            duration: Some(value),
            unit,
            until_time: None,
            duration_variable: None,
            description: None,
        }
    }

    /// Create a timer in milliseconds
    pub fn milliseconds(ms: u64) -> Self {
        Self::duration(ms, DurationUnit::Milliseconds)
    }

    /// Create a timer in seconds
    pub fn seconds(s: u64) -> Self {
        Self::duration(s, DurationUnit::Seconds)
    }

    /// Create a timer in minutes
    pub fn minutes(m: u64) -> Self {
        Self::duration(m, DurationUnit::Minutes)
    }

    /// Create a timer in hours
    pub fn hours(h: u64) -> Self {
        Self::duration(h, DurationUnit::Hours)
    }

    /// Create a timer until a specific time
    pub fn until(time: DateTime<Utc>) -> Self {
        Self {
            timer_type: TimerType::UntilTime,
            duration: None,
            unit: DurationUnit::default(),
            until_time: Some(time),
            duration_variable: None,
            description: None,
        }
    }

    /// Create a timer from a variable
    pub fn from_variable(variable: impl Into<String>) -> Self {
        Self {
            timer_type: TimerType::Duration,
            duration: None,
            unit: DurationUnit::Milliseconds,
            until_time: None,
            duration_variable: Some(variable.into()),
            description: None,
        }
    }

    /// Add description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Get duration in milliseconds
    pub fn duration_ms(&self) -> Option<u64> {
        self.duration.map(|d| self.unit.to_milliseconds(d))
    }

    /// Validate timer configuration
    pub fn validate_config(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        match self.timer_type {
            TimerType::Duration => {
                if self.duration.is_none() && self.duration_variable.is_none() {
                    errors.push("Duration timer requires duration or duration_variable".to_string());
                }
            }
            TimerType::UntilTime => {
                if self.until_time.is_none() {
                    errors.push("UntilTime timer requires until_time".to_string());
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl Default for TimerInput {
    fn default() -> Self {
        Self::minutes(1)
    }
}

/// Timer component output
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimerOutput {
    /// Whether the timer completed normally
    pub completed: bool,

    /// When the timer started
    pub started_at: DateTime<Utc>,

    /// When the timer ended
    pub ended_at: DateTime<Utc>,

    /// Actual duration waited in milliseconds
    pub duration_ms: u64,

    /// Whether the timer was cancelled
    #[serde(default)]
    pub cancelled: bool,
}

impl TimerOutput {
    /// Create a completed timer output
    pub fn completed(started_at: DateTime<Utc>, ended_at: DateTime<Utc>) -> Self {
        let duration_ms = (ended_at - started_at).num_milliseconds() as u64;
        Self {
            completed: true,
            started_at,
            ended_at,
            duration_ms,
            cancelled: false,
        }
    }

    /// Create a cancelled timer output
    pub fn cancelled(started_at: DateTime<Utc>) -> Self {
        let ended_at = Utc::now();
        let duration_ms = (ended_at - started_at).num_milliseconds() as u64;
        Self {
            completed: false,
            started_at,
            ended_at,
            duration_ms,
            cancelled: true,
        }
    }
}

impl Default for TimerOutput {
    fn default() -> Self {
        let now = Utc::now();
        Self::completed(now, now)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_unit_conversion() {
        assert_eq!(DurationUnit::Milliseconds.to_milliseconds(1000), 1000);
        assert_eq!(DurationUnit::Seconds.to_milliseconds(1), 1000);
        assert_eq!(DurationUnit::Minutes.to_milliseconds(1), 60000);
        assert_eq!(DurationUnit::Hours.to_milliseconds(1), 3600000);
        assert_eq!(DurationUnit::Days.to_milliseconds(1), 86400000);
    }

    #[test]
    fn test_timer_seconds() {
        let timer = TimerInput::seconds(30);
        assert_eq!(timer.timer_type, TimerType::Duration);
        assert_eq!(timer.duration, Some(30));
        assert_eq!(timer.unit, DurationUnit::Seconds);
        assert_eq!(timer.duration_ms(), Some(30000));
    }

    #[test]
    fn test_timer_minutes() {
        let timer = TimerInput::minutes(5);
        assert_eq!(timer.duration_ms(), Some(300000));
    }

    #[test]
    fn test_timer_until() {
        let target_time = Utc::now() + chrono::Duration::hours(1);
        let timer = TimerInput::until(target_time);
        assert_eq!(timer.timer_type, TimerType::UntilTime);
        assert!(timer.until_time.is_some());
    }

    #[test]
    fn test_timer_from_variable() {
        let timer = TimerInput::from_variable("delayMs");
        assert!(timer.duration_variable.is_some());
    }

    #[test]
    fn test_timer_validation() {
        let valid = TimerInput::seconds(10);
        assert!(valid.validate_config().is_ok());

        let invalid = TimerInput {
            timer_type: TimerType::Duration,
            duration: None,
            duration_variable: None,
            ..Default::default()
        };
        assert!(invalid.validate_config().is_err());
    }

    #[test]
    fn test_timer_output_completed() {
        let start = Utc::now();
        let end = start + chrono::Duration::seconds(5);
        let output = TimerOutput::completed(start, end);

        assert!(output.completed);
        assert!(!output.cancelled);
        assert_eq!(output.duration_ms, 5000);
    }

    #[test]
    fn test_timer_output_cancelled() {
        let start = Utc::now();
        let output = TimerOutput::cancelled(start);

        assert!(!output.completed);
        assert!(output.cancelled);
    }

    #[test]
    fn test_serialization() {
        let timer = TimerInput::hours(2).with_description("Wait for processing");

        let json = serde_json::to_string(&timer).unwrap();
        assert!(json.contains("timerType"));
        assert!(json.contains("duration"));
        assert!(json.contains("unit"));
    }
}
