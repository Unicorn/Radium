//! Agent execution state tracking.

use super::OutputBuffer;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Agent execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentStatus {
    /// Agent is idle (registered but not started)
    Idle,
    /// Agent is starting up
    Starting,
    /// Agent is running
    Running,
    /// Agent is thinking (processing)
    Thinking,
    /// Agent is executing a tool
    ExecutingTool,
    /// Agent completed successfully
    Completed,
    /// Agent failed
    Failed,
    /// Agent was cancelled
    Cancelled,
}

impl AgentStatus {
    /// Returns a display string for the status.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "Idle",
            Self::Starting => "Starting",
            Self::Running => "Running",
            Self::Thinking => "Thinking",
            Self::ExecutingTool => "Executing Tool",
            Self::Completed => "Completed",
            Self::Failed => "Failed",
            Self::Cancelled => "Cancelled",
        }
    }

    /// Returns an emoji/icon for the status.
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Idle => "⏸",
            Self::Starting => "🔄",
            Self::Running => "▶",
            Self::Thinking => "💭",
            Self::ExecutingTool => "🔧",
            Self::Completed => "✓",
            Self::Failed => "✗",
            Self::Cancelled => "⊗",
        }
    }

    /// Returns whether the agent is in an active state.
    pub fn is_active(&self) -> bool {
        matches!(
            self,
            Self::Starting | Self::Running | Self::Thinking | Self::ExecutingTool
        )
    }

    /// Returns whether the agent is in a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }
}

/// Sub-agent state for tracking child agent execution
#[derive(Debug, Clone)]
pub struct SubAgentState {
    /// Sub-agent ID
    pub agent_id: String,
    /// Sub-agent name
    pub agent_name: String,
    /// Sub-agent status
    pub status: AgentStatus,
    /// Start time
    pub start_time: Option<Instant>,
    /// End time
    pub end_time: Option<Instant>,
    /// Output buffer
    pub output_buffer: OutputBuffer,
}

impl SubAgentState {
    /// Creates a new sub-agent state.
    pub fn new(agent_id: String, agent_name: String) -> Self {
        Self {
            agent_id,
            agent_name,
            status: AgentStatus::Idle,
            start_time: None,
            end_time: None,
            output_buffer: OutputBuffer::new(500),
        }
    }

    /// Starts the sub-agent.
    pub fn start(&mut self) {
        self.status = AgentStatus::Running;
        self.start_time = Some(Instant::now());
    }

    /// Completes the sub-agent.
    pub fn complete(&mut self) {
        self.status = AgentStatus::Completed;
        self.end_time = Some(Instant::now());
    }

    /// Fails the sub-agent.
    pub fn fail(&mut self) {
        self.status = AgentStatus::Failed;
        self.end_time = Some(Instant::now());
    }

    /// Returns the elapsed time.
    pub fn elapsed_time(&self) -> Option<Duration> {
        self.start_time.map(|start| {
            if let Some(end) = self.end_time {
                end.duration_since(start)
            } else {
                Instant::now().duration_since(start)
            }
        })
    }
}

/// Main agent state
#[derive(Debug, Clone)]
pub struct AgentState {
    /// Agent ID
    pub agent_id: String,
    /// Agent name/type
    pub agent_name: String,
    /// Current status
    pub status: AgentStatus,
    /// Start time
    pub start_time: Option<Instant>,
    /// End time
    pub end_time: Option<Instant>,
    /// Current tool being executed
    pub current_tool: Option<String>,
    /// Sub-agents spawned by this agent
    pub sub_agents: HashMap<String, SubAgentState>,
    /// Agent output buffer
    pub output_buffer: OutputBuffer,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Token usage for this agent
    pub tokens_used: u64,
    /// Cost for this agent
    pub cost: f64,
}

impl AgentState {
    /// Creates a new agent state.
    pub fn new(agent_id: String, agent_name: String) -> Self {
        Self {
            agent_id,
            agent_name,
            status: AgentStatus::Idle,
            start_time: None,
            end_time: None,
            current_tool: None,
            sub_agents: HashMap::new(),
            output_buffer: OutputBuffer::new(500),
            error_message: None,
            tokens_used: 0,
            cost: 0.0,
        }
    }

    /// Starts the agent.
    pub fn start(&mut self) {
        self.status = AgentStatus::Running;
        self.start_time = Some(Instant::now());
        self.output_buffer.append_line(format!(
            "[{}] Agent started: {}",
            chrono::Utc::now().format("%H:%M:%S"),
            self.agent_name
        ));
    }

    /// Sets the agent to thinking state.
    pub fn set_thinking(&mut self) {
        self.status = AgentStatus::Thinking;
    }

    /// Sets the agent to executing tool state.
    pub fn set_executing_tool(&mut self, tool_name: String) {
        self.status = AgentStatus::ExecutingTool;
        self.current_tool = Some(tool_name.clone());
        self.output_buffer.append_line(format!(
            "[{}] Executing tool: {}",
            chrono::Utc::now().format("%H:%M:%S"),
            tool_name
        ));
    }

    /// Clears the current tool.
    pub fn clear_tool(&mut self) {
        self.current_tool = None;
        if self.status == AgentStatus::ExecutingTool {
            self.status = AgentStatus::Running;
        }
    }

    /// Completes the agent.
    pub fn complete(&mut self) {
        self.status = AgentStatus::Completed;
        self.end_time = Some(Instant::now());
        self.output_buffer.append_line(format!(
            "[{}] Agent completed",
            chrono::Utc::now().format("%H:%M:%S")
        ));
    }

    /// Fails the agent with an error message.
    pub fn fail(&mut self, error: String) {
        self.status = AgentStatus::Failed;
        self.end_time = Some(Instant::now());
        self.error_message = Some(error.clone());
        self.output_buffer.append_line(format!(
            "[{}] Agent failed: {}",
            chrono::Utc::now().format("%H:%M:%S"),
            error
        ));
    }

    /// Cancels the agent.
    pub fn cancel(&mut self) {
        self.status = AgentStatus::Cancelled;
        self.end_time = Some(Instant::now());
        self.output_buffer.append_line(format!(
            "[{}] Agent cancelled",
            chrono::Utc::now().format("%H:%M:%S")
        ));
    }

    /// Registers a sub-agent.
    pub fn register_sub_agent(&mut self, agent_id: String, agent_name: String) {
        let sub_agent = SubAgentState::new(agent_id.clone(), agent_name);
        self.sub_agents.insert(agent_id, sub_agent);
    }

    /// Gets a mutable reference to a sub-agent.
    pub fn get_sub_agent_mut(&mut self, agent_id: &str) -> Option<&mut SubAgentState> {
        self.sub_agents.get_mut(agent_id)
    }

    /// Returns the elapsed time.
    pub fn elapsed_time(&self) -> Option<Duration> {
        self.start_time.map(|start| {
            if let Some(end) = self.end_time {
                end.duration_since(start)
            } else {
                Instant::now().duration_since(start)
            }
        })
    }

    /// Appends a line to the agent output buffer.
    pub fn log(&mut self, line: String) {
        self.output_buffer.append_line(line);
    }

    /// Updates token usage.
    pub fn update_tokens(&mut self, tokens: u64) {
        self.tokens_used += tokens;
    }

    /// Updates cost.
    pub fn update_cost(&mut self, cost: f64) {
        self.cost += cost;
    }

    /// Returns a summary of the agent state.
    pub fn summary(&self) -> String {
        format!(
            "{} {} | {}",
            self.status.icon(),
            self.agent_name,
            self.status.as_str()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_status() {
        assert_eq!(AgentStatus::Running.as_str(), "Running");
        assert_eq!(AgentStatus::Running.icon(), "▶");
        assert!(AgentStatus::Running.is_active());
        assert!(!AgentStatus::Completed.is_active());
        assert!(AgentStatus::Completed.is_terminal());
        assert!(!AgentStatus::Running.is_terminal());
    }

    #[test]
    fn test_agent_lifecycle() {
        let mut agent = AgentState::new("agent-1".to_string(), "Test Agent".to_string());

        assert_eq!(agent.status, AgentStatus::Idle);

        agent.start();
        assert_eq!(agent.status, AgentStatus::Running);
        assert!(agent.start_time.is_some());

        agent.set_thinking();
        assert_eq!(agent.status, AgentStatus::Thinking);

        agent.set_executing_tool("read_file".to_string());
        assert_eq!(agent.status, AgentStatus::ExecutingTool);
        assert_eq!(agent.current_tool, Some("read_file".to_string()));

        agent.clear_tool();
        assert_eq!(agent.status, AgentStatus::Running);
        assert_eq!(agent.current_tool, None);

        agent.complete();
        assert_eq!(agent.status, AgentStatus::Completed);
        assert!(agent.end_time.is_some());
    }

    #[test]
    fn test_agent_failure() {
        let mut agent = AgentState::new("agent-1".to_string(), "Test Agent".to_string());

        agent.start();
        agent.fail("Test error".to_string());

        assert_eq!(agent.status, AgentStatus::Failed);
        assert_eq!(agent.error_message, Some("Test error".to_string()));
        assert!(agent.end_time.is_some());
    }

    #[test]
    fn test_sub_agent() {
        let mut agent = AgentState::new("agent-1".to_string(), "Main Agent".to_string());

        agent.register_sub_agent("sub-1".to_string(), "Sub Agent".to_string());
        assert!(agent.get_sub_agent_mut("sub-1").is_some());

        if let Some(sub_agent) = agent.get_sub_agent_mut("sub-1") {
            sub_agent.start();
            assert_eq!(sub_agent.status, AgentStatus::Running);

            sub_agent.complete();
            assert_eq!(sub_agent.status, AgentStatus::Completed);
        }
    }

    #[test]
    fn test_agent_metrics() {
        let mut agent = AgentState::new("agent-1".to_string(), "Test Agent".to_string());

        agent.update_tokens(100);
        agent.update_tokens(50);
        assert_eq!(agent.tokens_used, 150);

        agent.update_cost(0.01);
        agent.update_cost(0.005);
        assert!((agent.cost - 0.015).abs() < 0.0001);
    }
}
