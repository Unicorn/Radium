//! Extension analytics and usage tracking.
//!
//! Provides privacy-respecting analytics for extension usage, installation,
//! and uninstallation events.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use thiserror::Error;

/// Analytics errors.
#[derive(Debug, Error)]
pub enum ExtensionAnalyticsError {
    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON parsing error.
    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),

    /// Analytics not enabled.
    #[error("analytics not enabled")]
    NotEnabled,
}

/// Result type for analytics operations.
pub type Result<T> = std::result::Result<T, ExtensionAnalyticsError>;

/// Extension event types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExtensionEventType {
    /// Extension installed.
    Install {
        source: String,
        version: String,
    },
    /// Extension uninstalled.
    Uninstall {
        duration_installed_seconds: Option<u64>,
    },
    /// Extension component used.
    Usage {
        component_type: String,
    },
    /// Extension error occurred.
    Error {
        error_type: String,
    },
}

/// Extension analytics event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionEvent {
    /// Extension name.
    pub extension_name: String,
    /// Event type.
    pub event_type: ExtensionEventType,
    /// Timestamp.
    pub timestamp: DateTime<Utc>,
    /// Optional metadata.
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Aggregated extension analytics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionAnalytics {
    /// Extension name.
    pub extension_name: String,
    /// Total installs.
    pub install_count: u64,
    /// Total uninstalls.
    pub uninstall_count: u64,
    /// Total usage events.
    pub usage_count: u64,
    /// Total error events.
    pub error_count: u64,
    /// First installed timestamp.
    pub first_installed: Option<DateTime<Utc>>,
    /// Last used timestamp.
    pub last_used: Option<DateTime<Utc>>,
    /// Component usage breakdown.
    #[serde(default)]
    pub component_usage: HashMap<String, u64>,
    /// Error type breakdown.
    #[serde(default)]
    pub error_types: HashMap<String, u64>,
}

/// Extension analytics service.
pub struct ExtensionAnalyticsService {
    data_dir: PathBuf,
    enabled: bool,
}

impl ExtensionAnalyticsService {
    /// Creates a new analytics service.
    pub fn new(data_dir: PathBuf) -> Self {
        let enabled = Self::load_preference(&data_dir).unwrap_or(false);
        Self { data_dir, enabled }
    }

    /// Loads analytics preference from config.
    fn load_preference(data_dir: &Path) -> std::result::Result<bool, std::io::Error> {
        let config_path = data_dir.join("analytics.json");
        if !config_path.exists() {
            return Ok(false);
        }

        let content = fs::read_to_string(&config_path)?;
        let config: serde_json::Value = serde_json::from_str(&content)?;
        Ok(config.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false))
    }

    /// Saves analytics preference.
    fn save_preference(&self, enabled: bool) -> Result<()> {
        fs::create_dir_all(&self.data_dir)?;
        let config_path = self.data_dir.join("analytics.json");
        let config = serde_json::json!({
            "enabled": enabled,
            "updated_at": Utc::now().to_rfc3339(),
        });
        fs::write(&config_path, serde_json::to_string_pretty(&config)?)?;
        Ok(())
    }

    /// Checks if analytics is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Enables analytics.
    pub fn enable(&mut self) -> Result<()> {
        self.enabled = true;
        self.save_preference(true)?;
        Ok(())
    }

    /// Disables analytics and clears data.
    pub fn disable(&mut self) -> Result<()> {
        self.enabled = false;
        self.save_preference(false)?;
        self.clear_data()?;
        Ok(())
    }

    /// Records an extension event.
    pub fn record_event(&self, event: ExtensionEvent) -> Result<()> {
        if !self.enabled {
            return Err(ExtensionAnalyticsError::NotEnabled);
        }

        fs::create_dir_all(&self.data_dir)?;
        let events_dir = self.data_dir.join("events");
        fs::create_dir_all(&events_dir)?;

        // Store event in file (one file per day for efficiency)
        let date_str = event.timestamp.format("%Y-%m-%d").to_string();
        let events_file = events_dir.join(format!("{}.jsonl", date_str));

        let event_json = serde_json::to_string(&event)?;
        use std::io::Write;
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&events_file)?;
        writeln!(file, "{}", event_json)?;

        Ok(())
    }

    /// Gets aggregated analytics for an extension.
    pub fn get_analytics(&self, extension_name: &str) -> Result<ExtensionAnalytics> {
        let events = self.load_events(Some(extension_name))?;

        let mut analytics = ExtensionAnalytics {
            extension_name: extension_name.to_string(),
            install_count: 0,
            uninstall_count: 0,
            usage_count: 0,
            error_count: 0,
            first_installed: None,
            last_used: None,
            component_usage: HashMap::new(),
            error_types: HashMap::new(),
        };

        for event in events {
            match &event.event_type {
                ExtensionEventType::Install { .. } => {
                    analytics.install_count += 1;
                    if analytics.first_installed.is_none() {
                        analytics.first_installed = Some(event.timestamp);
                    }
                }
                ExtensionEventType::Uninstall { .. } => {
                    analytics.uninstall_count += 1;
                }
                ExtensionEventType::Usage { component_type } => {
                    analytics.usage_count += 1;
                    *analytics.component_usage.entry(component_type.clone()).or_insert(0) += 1;
                    analytics.last_used = Some(event.timestamp);
                }
                ExtensionEventType::Error { error_type } => {
                    analytics.error_count += 1;
                    *analytics.error_types.entry(error_type.clone()).or_insert(0) += 1;
                }
            }
        }

        Ok(analytics)
    }

    /// Gets all analytics.
    pub fn get_all_analytics(&self) -> Result<Vec<ExtensionAnalytics>> {
        let events = self.load_events(None)?;

        let mut analytics_map: HashMap<String, ExtensionAnalytics> = HashMap::new();

        for event in events {
            let analytics = analytics_map.entry(event.extension_name.clone())
                .or_insert_with(|| ExtensionAnalytics {
                    extension_name: event.extension_name.clone(),
                    install_count: 0,
                    uninstall_count: 0,
                    usage_count: 0,
                    error_count: 0,
                    first_installed: None,
                    last_used: None,
                    component_usage: HashMap::new(),
                    error_types: HashMap::new(),
                });

            match &event.event_type {
                ExtensionEventType::Install { .. } => {
                    analytics.install_count += 1;
                    if analytics.first_installed.is_none() {
                        analytics.first_installed = Some(event.timestamp);
                    }
                }
                ExtensionEventType::Uninstall { .. } => {
                    analytics.uninstall_count += 1;
                }
                ExtensionEventType::Usage { component_type } => {
                    analytics.usage_count += 1;
                    *analytics.component_usage.entry(component_type.clone()).or_insert(0) += 1;
                    analytics.last_used = Some(event.timestamp);
                }
                ExtensionEventType::Error { error_type } => {
                    analytics.error_count += 1;
                    *analytics.error_types.entry(error_type.clone()).or_insert(0) += 1;
                }
            }
        }

        Ok(analytics_map.into_values().collect())
    }

    /// Loads events from storage.
    fn load_events(&self, filter_extension: Option<&str>) -> Result<Vec<ExtensionEvent>> {
        let events_dir = self.data_dir.join("events");
        if !events_dir.exists() {
            return Ok(Vec::new());
        }

        let mut events = Vec::new();
        let entries = fs::read_dir(&events_dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.extension() == Some(std::ffi::OsStr::new("jsonl")) {
                let content = fs::read_to_string(&path)?;
                for line in content.lines() {
                    if let Ok(event) = serde_json::from_str::<ExtensionEvent>(line) {
                        if let Some(filter) = filter_extension {
                            if event.extension_name == filter {
                                events.push(event);
                            }
                        } else {
                            events.push(event);
                        }
                    }
                }
            }
        }

        Ok(events)
    }

    /// Clears all analytics data.
    pub fn clear_data(&self) -> Result<()> {
        let events_dir = self.data_dir.join("events");
        if events_dir.exists() {
            fs::remove_dir_all(&events_dir)?;
        }
        Ok(())
    }
}

