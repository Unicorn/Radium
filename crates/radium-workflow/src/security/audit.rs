//! Audit Logging Module
//!
//! Provides security audit logging for tracking access, changes,
//! and potential security incidents.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::RwLock;
use std::time::Instant;

/// Audit event types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    /// Workflow compilation requested
    CompilationRequested,
    /// Workflow compilation completed
    CompilationCompleted,
    /// Workflow compilation failed
    CompilationFailed,
    /// Workflow validation requested
    ValidationRequested,
    /// Workflow validation completed
    ValidationCompleted,
    /// Workflow validation failed
    ValidationFailed,
    /// Rate limit exceeded
    RateLimitExceeded,
    /// Input sanitization triggered
    SanitizationTriggered,
    /// Dangerous pattern detected
    DangerousPatternDetected,
    /// Authentication attempt
    AuthenticationAttempt,
    /// Authentication success
    AuthenticationSuccess,
    /// Authentication failure
    AuthenticationFailure,
    /// Authorization denied
    AuthorizationDenied,
    /// Configuration changed
    ConfigurationChanged,
    /// System started
    SystemStarted,
    /// System stopped
    SystemStopped,
    /// Error occurred
    ErrorOccurred,
    /// Security alert
    SecurityAlert,
}

impl std::fmt::Display for AuditEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditEventType::CompilationRequested => write!(f, "compilation_requested"),
            AuditEventType::CompilationCompleted => write!(f, "compilation_completed"),
            AuditEventType::CompilationFailed => write!(f, "compilation_failed"),
            AuditEventType::ValidationRequested => write!(f, "validation_requested"),
            AuditEventType::ValidationCompleted => write!(f, "validation_completed"),
            AuditEventType::ValidationFailed => write!(f, "validation_failed"),
            AuditEventType::RateLimitExceeded => write!(f, "rate_limit_exceeded"),
            AuditEventType::SanitizationTriggered => write!(f, "sanitization_triggered"),
            AuditEventType::DangerousPatternDetected => write!(f, "dangerous_pattern_detected"),
            AuditEventType::AuthenticationAttempt => write!(f, "authentication_attempt"),
            AuditEventType::AuthenticationSuccess => write!(f, "authentication_success"),
            AuditEventType::AuthenticationFailure => write!(f, "authentication_failure"),
            AuditEventType::AuthorizationDenied => write!(f, "authorization_denied"),
            AuditEventType::ConfigurationChanged => write!(f, "configuration_changed"),
            AuditEventType::SystemStarted => write!(f, "system_started"),
            AuditEventType::SystemStopped => write!(f, "system_stopped"),
            AuditEventType::ErrorOccurred => write!(f, "error_occurred"),
            AuditEventType::SecurityAlert => write!(f, "security_alert"),
        }
    }
}

/// Audit event severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditSeverity {
    /// Debug level - detailed information
    Debug,
    /// Info level - normal operations
    Info,
    /// Warning level - potential issues
    Warning,
    /// Error level - operation failures
    Error,
    /// Critical level - security incidents
    Critical,
}

impl std::fmt::Display for AuditSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditSeverity::Debug => write!(f, "DEBUG"),
            AuditSeverity::Info => write!(f, "INFO"),
            AuditSeverity::Warning => write!(f, "WARNING"),
            AuditSeverity::Error => write!(f, "ERROR"),
            AuditSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Audit event actor information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditActor {
    /// Actor type (user, system, api_key, etc.)
    pub actor_type: String,
    /// Actor identifier
    pub id: String,
    /// IP address (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,
    /// User agent (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
}

impl AuditActor {
    /// Create a system actor
    pub fn system() -> Self {
        Self {
            actor_type: "system".to_string(),
            id: "system".to_string(),
            ip_address: None,
            user_agent: None,
        }
    }

    /// Create a user actor
    pub fn user(id: impl Into<String>) -> Self {
        Self {
            actor_type: "user".to_string(),
            id: id.into(),
            ip_address: None,
            user_agent: None,
        }
    }

    /// Create an API key actor
    pub fn api_key(key_id: impl Into<String>) -> Self {
        Self {
            actor_type: "api_key".to_string(),
            id: key_id.into(),
            ip_address: None,
            user_agent: None,
        }
    }

    /// Create an anonymous actor
    pub fn anonymous() -> Self {
        Self {
            actor_type: "anonymous".to_string(),
            id: "anonymous".to_string(),
            ip_address: None,
            user_agent: None,
        }
    }

    /// Add IP address
    pub fn with_ip(mut self, ip: impl Into<String>) -> Self {
        self.ip_address = Some(ip.into());
        self
    }

    /// Add user agent
    pub fn with_user_agent(mut self, ua: impl Into<String>) -> Self {
        self.user_agent = Some(ua.into());
        self
    }
}

/// Audit event resource information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditResource {
    /// Resource type (workflow, component, config, etc.)
    pub resource_type: String,
    /// Resource identifier
    pub id: String,
    /// Resource name (if different from ID)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl AuditResource {
    /// Create a workflow resource
    pub fn workflow(id: impl Into<String>) -> Self {
        Self {
            resource_type: "workflow".to_string(),
            id: id.into(),
            name: None,
        }
    }

    /// Create a component resource
    pub fn component(id: impl Into<String>) -> Self {
        Self {
            resource_type: "component".to_string(),
            id: id.into(),
            name: None,
        }
    }

    /// Create a configuration resource
    pub fn config(id: impl Into<String>) -> Self {
        Self {
            resource_type: "config".to_string(),
            id: id.into(),
            name: None,
        }
    }

    /// Add resource name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

/// A single audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique event ID
    pub id: String,
    /// Event timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Event type
    pub event_type: AuditEventType,
    /// Event severity
    pub severity: AuditSeverity,
    /// Event message
    pub message: String,
    /// Actor who triggered the event
    pub actor: AuditActor,
    /// Resource affected (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource: Option<AuditResource>,
    /// Additional context/metadata
    #[serde(skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub metadata: std::collections::HashMap<String, String>,
    /// Request ID for correlation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// Duration of the operation (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    /// Success/failure status
    pub success: bool,
}

impl AuditEvent {
    /// Create a new audit event builder
    pub fn builder(event_type: AuditEventType) -> AuditEventBuilder {
        AuditEventBuilder::new(event_type)
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }

    /// Serialize to JSON (pretty printed)
    pub fn to_json_pretty(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }

    /// Format as log line
    pub fn to_log_line(&self) -> String {
        let resource_info = self.resource.as_ref()
            .map(|r| format!(" resource={}:{}", r.resource_type, r.id))
            .unwrap_or_default();

        format!(
            "[{}] {} {} actor={}:{}{} message=\"{}\" success={}",
            self.timestamp.format("%Y-%m-%dT%H:%M:%S%.3fZ"),
            self.severity,
            self.event_type,
            self.actor.actor_type,
            self.actor.id,
            resource_info,
            self.message,
            self.success
        )
    }
}

/// Builder for audit events
pub struct AuditEventBuilder {
    event_type: AuditEventType,
    severity: AuditSeverity,
    message: Option<String>,
    actor: Option<AuditActor>,
    resource: Option<AuditResource>,
    metadata: std::collections::HashMap<String, String>,
    request_id: Option<String>,
    duration_ms: Option<u64>,
    success: bool,
}

impl AuditEventBuilder {
    /// Create a new builder
    pub fn new(event_type: AuditEventType) -> Self {
        let severity = match event_type {
            AuditEventType::DangerousPatternDetected | AuditEventType::SecurityAlert => {
                AuditSeverity::Critical
            }
            AuditEventType::AuthenticationFailure
            | AuditEventType::AuthorizationDenied
            | AuditEventType::RateLimitExceeded => AuditSeverity::Warning,
            AuditEventType::CompilationFailed
            | AuditEventType::ValidationFailed
            | AuditEventType::ErrorOccurred => AuditSeverity::Error,
            _ => AuditSeverity::Info,
        };

        Self {
            event_type,
            severity,
            message: None,
            actor: None,
            resource: None,
            metadata: std::collections::HashMap::new(),
            request_id: None,
            duration_ms: None,
            success: true,
        }
    }

    /// Set the severity
    pub fn severity(mut self, severity: AuditSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Set the message
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Set the actor
    pub fn actor(mut self, actor: AuditActor) -> Self {
        self.actor = Some(actor);
        self
    }

    /// Set the resource
    pub fn resource(mut self, resource: AuditResource) -> Self {
        self.resource = Some(resource);
        self
    }

    /// Add metadata
    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Set request ID
    pub fn request_id(mut self, id: impl Into<String>) -> Self {
        self.request_id = Some(id.into());
        self
    }

    /// Set duration
    pub fn duration_ms(mut self, ms: u64) -> Self {
        self.duration_ms = Some(ms);
        self
    }

    /// Set success status
    pub fn success(mut self, success: bool) -> Self {
        self.success = success;
        self
    }

    /// Build the audit event
    pub fn build(self) -> AuditEvent {
        AuditEvent {
            id: Self::generate_id(),
            timestamp: chrono::Utc::now(),
            event_type: self.event_type,
            severity: self.severity,
            message: self.message.unwrap_or_else(|| self.event_type.to_string()),
            actor: self.actor.unwrap_or_else(AuditActor::system),
            resource: self.resource,
            metadata: self.metadata,
            request_id: self.request_id,
            duration_ms: self.duration_ms,
            success: self.success,
        }
    }

    /// Generate a unique event ID
    fn generate_id() -> String {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);

        let timestamp = chrono::Utc::now().timestamp_millis() as u64;
        let counter = COUNTER.fetch_add(1, Ordering::Relaxed);

        format!("audit-{:016x}-{:08x}", timestamp, counter as u32)
    }
}

/// Audit log configuration
#[derive(Debug, Clone)]
pub struct AuditLogConfig {
    /// Maximum events to keep in memory
    pub max_events: usize,
    /// Minimum severity to log
    pub min_severity: AuditSeverity,
    /// Whether to log to stdout
    pub log_to_stdout: bool,
    /// Whether to include metadata in logs
    pub include_metadata: bool,
}

impl Default for AuditLogConfig {
    fn default() -> Self {
        Self {
            max_events: 10_000,
            min_severity: AuditSeverity::Info,
            log_to_stdout: false,
            include_metadata: true,
        }
    }
}

/// Audit log collector
pub struct AuditLog {
    config: AuditLogConfig,
    events: RwLock<VecDeque<AuditEvent>>,
    start_time: Instant,
}

impl AuditLog {
    /// Create a new audit log
    pub fn new(config: AuditLogConfig) -> Self {
        Self {
            config,
            events: RwLock::new(VecDeque::new()),
            start_time: Instant::now(),
        }
    }

    /// Log an audit event
    pub fn log(&self, event: AuditEvent) {
        // Check severity filter
        if event.severity < self.config.min_severity {
            return;
        }

        // Log to stdout if configured
        if self.config.log_to_stdout {
            println!("{}", event.to_log_line());
        }

        // Store in memory
        let mut events = self.events.write().unwrap();
        events.push_back(event);

        // Trim if over capacity
        while events.len() > self.config.max_events {
            events.pop_front();
        }
    }

    /// Log a compilation request
    pub fn log_compilation_requested(&self, actor: AuditActor, workflow_id: &str) {
        let event = AuditEvent::builder(AuditEventType::CompilationRequested)
            .actor(actor)
            .resource(AuditResource::workflow(workflow_id))
            .message(format!("Compilation requested for workflow '{}'", workflow_id))
            .build();
        self.log(event);
    }

    /// Log a compilation completion
    pub fn log_compilation_completed(
        &self,
        actor: AuditActor,
        workflow_id: &str,
        duration_ms: u64,
    ) {
        let event = AuditEvent::builder(AuditEventType::CompilationCompleted)
            .actor(actor)
            .resource(AuditResource::workflow(workflow_id))
            .duration_ms(duration_ms)
            .message(format!(
                "Compilation completed for workflow '{}' in {}ms",
                workflow_id, duration_ms
            ))
            .build();
        self.log(event);
    }

    /// Log a compilation failure
    pub fn log_compilation_failed(&self, actor: AuditActor, workflow_id: &str, error: &str) {
        let event = AuditEvent::builder(AuditEventType::CompilationFailed)
            .actor(actor)
            .resource(AuditResource::workflow(workflow_id))
            .success(false)
            .message(format!(
                "Compilation failed for workflow '{}': {}",
                workflow_id, error
            ))
            .build();
        self.log(event);
    }

    /// Log a rate limit exceeded event
    pub fn log_rate_limit_exceeded(&self, actor: AuditActor, endpoint: &str) {
        let event = AuditEvent::builder(AuditEventType::RateLimitExceeded)
            .actor(actor)
            .metadata("endpoint", endpoint)
            .success(false)
            .message(format!("Rate limit exceeded for endpoint '{}'", endpoint))
            .build();
        self.log(event);
    }

    /// Log a dangerous pattern detection
    pub fn log_dangerous_pattern(&self, actor: AuditActor, pattern: &str, location: &str) {
        let event = AuditEvent::builder(AuditEventType::DangerousPatternDetected)
            .actor(actor)
            .metadata("pattern", pattern)
            .metadata("location", location)
            .success(false)
            .message(format!("Dangerous pattern '{}' detected in {}", pattern, location))
            .build();
        self.log(event);
    }

    /// Log a security alert
    pub fn log_security_alert(&self, actor: AuditActor, alert: &str) {
        let event = AuditEvent::builder(AuditEventType::SecurityAlert)
            .severity(AuditSeverity::Critical)
            .actor(actor)
            .success(false)
            .message(alert.to_string())
            .build();
        self.log(event);
    }

    /// Get all events
    pub fn get_events(&self) -> Vec<AuditEvent> {
        let events = self.events.read().unwrap();
        events.iter().cloned().collect()
    }

    /// Get events by type
    pub fn get_events_by_type(&self, event_type: AuditEventType) -> Vec<AuditEvent> {
        let events = self.events.read().unwrap();
        events
            .iter()
            .filter(|e| e.event_type == event_type)
            .cloned()
            .collect()
    }

    /// Get events by severity
    pub fn get_events_by_severity(&self, min_severity: AuditSeverity) -> Vec<AuditEvent> {
        let events = self.events.read().unwrap();
        events
            .iter()
            .filter(|e| e.severity >= min_severity)
            .cloned()
            .collect()
    }

    /// Get events for a specific actor
    pub fn get_events_by_actor(&self, actor_id: &str) -> Vec<AuditEvent> {
        let events = self.events.read().unwrap();
        events
            .iter()
            .filter(|e| e.actor.id == actor_id)
            .cloned()
            .collect()
    }

    /// Get events for a specific resource
    pub fn get_events_by_resource(&self, resource_type: &str, resource_id: &str) -> Vec<AuditEvent> {
        let events = self.events.read().unwrap();
        events
            .iter()
            .filter(|e| {
                e.resource
                    .as_ref()
                    .map(|r| r.resource_type == resource_type && r.id == resource_id)
                    .unwrap_or(false)
            })
            .cloned()
            .collect()
    }

    /// Get security events (warnings and above)
    pub fn get_security_events(&self) -> Vec<AuditEvent> {
        self.get_events_by_severity(AuditSeverity::Warning)
    }

    /// Get event count
    pub fn event_count(&self) -> usize {
        let events = self.events.read().unwrap();
        events.len()
    }

    /// Get uptime
    pub fn uptime(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    /// Clear all events
    pub fn clear(&self) {
        let mut events = self.events.write().unwrap();
        events.clear();
    }

    /// Export events to JSON
    pub fn export_json(&self) -> String {
        let events = self.get_events();
        serde_json::to_string_pretty(&events).unwrap_or_else(|_| "[]".to_string())
    }
}

impl Default for AuditLog {
    fn default() -> Self {
        Self::new(AuditLogConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_event_types() {
        assert_eq!(
            AuditEventType::CompilationRequested.to_string(),
            "compilation_requested"
        );
        assert_eq!(
            AuditEventType::SecurityAlert.to_string(),
            "security_alert"
        );
    }

    #[test]
    fn test_audit_severity() {
        assert!(AuditSeverity::Debug < AuditSeverity::Info);
        assert!(AuditSeverity::Info < AuditSeverity::Warning);
        assert!(AuditSeverity::Warning < AuditSeverity::Error);
        assert!(AuditSeverity::Error < AuditSeverity::Critical);
    }

    #[test]
    fn test_audit_actor() {
        let system = AuditActor::system();
        assert_eq!(system.actor_type, "system");

        let user = AuditActor::user("user123").with_ip("192.168.1.1");
        assert_eq!(user.actor_type, "user");
        assert_eq!(user.id, "user123");
        assert_eq!(user.ip_address, Some("192.168.1.1".to_string()));
    }

    #[test]
    fn test_audit_resource() {
        let workflow = AuditResource::workflow("wf-123").with_name("MyWorkflow");
        assert_eq!(workflow.resource_type, "workflow");
        assert_eq!(workflow.id, "wf-123");
        assert_eq!(workflow.name, Some("MyWorkflow".to_string()));
    }

    #[test]
    fn test_audit_event_builder() {
        let event = AuditEvent::builder(AuditEventType::CompilationRequested)
            .message("Test compilation")
            .actor(AuditActor::user("test-user"))
            .resource(AuditResource::workflow("wf-1"))
            .metadata("key", "value")
            .build();

        assert_eq!(event.event_type, AuditEventType::CompilationRequested);
        assert_eq!(event.message, "Test compilation");
        assert_eq!(event.actor.id, "test-user");
        assert_eq!(event.resource.unwrap().id, "wf-1");
        assert_eq!(event.metadata.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_audit_event_serialization() {
        let event = AuditEvent::builder(AuditEventType::CompilationCompleted)
            .message("Compilation done")
            .duration_ms(100)
            .build();

        let json = event.to_json();
        assert!(json.contains("compilation_completed"));
        assert!(json.contains("Compilation done"));
    }

    #[test]
    fn test_audit_log() {
        let log = AuditLog::new(AuditLogConfig {
            max_events: 100,
            min_severity: AuditSeverity::Info,
            log_to_stdout: false,
            include_metadata: true,
        });

        log.log_compilation_requested(AuditActor::user("user1"), "wf-1");
        log.log_compilation_completed(AuditActor::user("user1"), "wf-1", 150);

        let events = log.get_events();
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn test_audit_log_filtering() {
        let log = AuditLog::default();

        log.log_compilation_requested(AuditActor::user("user1"), "wf-1");
        log.log_compilation_failed(AuditActor::user("user2"), "wf-2", "syntax error");
        log.log_dangerous_pattern(AuditActor::user("user3"), "eval(", "expression");

        // Filter by type
        let compilation_events = log.get_events_by_type(AuditEventType::CompilationRequested);
        assert_eq!(compilation_events.len(), 1);

        // Filter by severity
        let security_events = log.get_security_events();
        assert!(security_events.len() >= 2); // Failed + dangerous pattern

        // Filter by actor
        let user1_events = log.get_events_by_actor("user1");
        assert_eq!(user1_events.len(), 1);
    }

    #[test]
    fn test_audit_log_capacity() {
        let log = AuditLog::new(AuditLogConfig {
            max_events: 3,
            ..Default::default()
        });

        for i in 0..5 {
            log.log(
                AuditEvent::builder(AuditEventType::CompilationRequested)
                    .message(format!("Event {}", i))
                    .build(),
            );
        }

        // Should only keep last 3 events
        let events = log.get_events();
        assert_eq!(events.len(), 3);
        assert!(events[0].message.contains("2"));
        assert!(events[2].message.contains("4"));
    }

    #[test]
    fn test_audit_log_severity_filter() {
        let log = AuditLog::new(AuditLogConfig {
            min_severity: AuditSeverity::Warning,
            ..Default::default()
        });

        // This should be filtered out (Info severity)
        log.log(
            AuditEvent::builder(AuditEventType::CompilationCompleted)
                .severity(AuditSeverity::Info)
                .build(),
        );

        // This should be kept (Warning severity)
        log.log(
            AuditEvent::builder(AuditEventType::RateLimitExceeded)
                .severity(AuditSeverity::Warning)
                .build(),
        );

        let events = log.get_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, AuditEventType::RateLimitExceeded);
    }

    #[test]
    fn test_audit_event_log_line() {
        let event = AuditEvent::builder(AuditEventType::CompilationCompleted)
            .actor(AuditActor::user("test"))
            .resource(AuditResource::workflow("wf-1"))
            .message("Test message")
            .build();

        let log_line = event.to_log_line();
        assert!(log_line.contains("INFO"));
        assert!(log_line.contains("compilation_completed"));
        assert!(log_line.contains("actor=user:test"));
        assert!(log_line.contains("resource=workflow:wf-1"));
    }

    #[test]
    fn test_export_json() {
        let log = AuditLog::default();
        log.log_compilation_requested(AuditActor::system(), "test-wf");

        let json = log.export_json();
        assert!(json.contains("compilation_requested"));
        assert!(json.contains("test-wf"));
    }
}
