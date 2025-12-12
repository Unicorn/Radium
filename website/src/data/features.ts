import React, {type ReactNode} from 'react';

export interface Feature {
  id: string;
  title: string;
  description: ReactNode;
  icon?: string;
  codeExample?: {
    language: string;
    code: string;
  };
  links?: Array<{
    label: string;
    href: string;
  }>;
}

export interface FeatureCategory {
  id: string;
  title: string;
  description: string;
  features: Feature[];
}

export const featureCategories: FeatureCategory[] = [
  {
    id: 'orchestration',
    title: 'Orchestration & Workflows',
    description: 'Powerful tools for building and managing complex multi-agent workflows',
    features: [
      {
        id: 'intelligent-orchestration',
        title: 'Intelligent Orchestration',
        description: (
          <React.Fragment>
            Radium automatically selects the best agent for each task based on capabilities,
            cost, and performance. No manual routing required—the orchestrator handles
            agent selection using policy-driven decision making.
          </React.Fragment>
        ),
        codeExample: {
          language: 'toml',
          code: `[orchestration]
policy = "cost-optimized"  # or "speed-first", "quality-first"
fallback_strategy = "graceful-degradation"

[agent_selection]
prefer_local = true
max_cost_per_call = 0.10
timeout = 30`,
        },
        links: [
          {label: 'Orchestration Guide', href: '/docs/user-guide/orchestration'},
          {label: 'Policy Configuration', href: '/docs/user-guide/policies'},
        ],
      },
      {
        id: 'multi-agent-workflows',
        title: 'Multi-Agent DAG Workflows',
        description: (
          <React.Fragment>
            Define complex workflows as directed acyclic graphs (DAGs) with multiple agents
            working in parallel or sequence. Built-in dependency resolution, automatic
            retries, and failure handling ensure reliable execution.
          </React.Fragment>
        ),
        codeExample: {
          language: 'toml',
          code: `[[workflow.steps]]
id = "analyze"
agent = "code-analyzer"
input = "task.description"

[[workflow.steps]]
id = "generate"
agent = "code-generator"
depends_on = ["analyze"]

[[workflow.steps]]
id = "review"
agent = "code-reviewer"
depends_on = ["generate"]
parallel = ["security-scan"]`,
        },
        links: [
          {label: 'Workflow Guide', href: '/docs/user-guide/workflows'},
          {label: 'Examples', href: '/docs/examples/orchestration-workflows'},
        ],
      },
      {
        id: 'autonomous-execution',
        title: 'Autonomous Execution (YOLO Mode)',
        description: (
          <React.Fragment>
            Enable agents to run autonomously with minimal human intervention. Agents can
            make decisions, execute tasks, and handle errors independently. Perfect for
            long-running tasks or trusted environments.
          </React.Fragment>
        ),
        codeExample: {
          language: 'bash',
          code: `# Enable autonomous mode
radium-cli chat code-assistant --mode autonomous

# Set execution policy
radium-cli config set execution.mode autonomous
radium-cli config set execution.require_approval false

# Run with bounded autonomy
radium-cli chat code-assistant \\
  --mode autonomous \\
  --max-iterations 10 \\
  --budget 5.00`,
        },
        links: [
          {label: 'Execution Modes', href: '/docs/user-guide/execution-modes'},
          {label: 'Safety Guide', href: '/docs/user-guide/safety'},
        ],
      },
    ],
  },
  {
    id: 'agent-management',
    title: 'Agent Management',
    description: 'Simple, declarative configuration for creating and managing agents',
    features: [
      {
        id: 'toml-config',
        title: 'TOML-Based Configuration',
        description: (
          <React.Fragment>
            Define agents using simple TOML files. No code required—just declare your
            agent's capabilities, model preferences, and behavior. Version control friendly
            and easy to share.
          </React.Fragment>
        ),
        codeExample: {
          language: 'toml',
          code: `[agent]
id = "code-reviewer"
name = "Code Review Assistant"
description = "Expert code reviewer for Rust projects"

[model]
provider = "anthropic"
model = "claude-3-sonnet"
temperature = 0.1
max_tokens = 8000

[persona]
expertise = ["rust", "security", "performance"]
style = "constructive"
verbosity = "detailed"`,
        },
        links: [
          {label: 'Agent Configuration', href: '/docs/user-guide/agent-configuration'},
          {label: 'Configuration Reference', href: '/docs/api/configuration'},
        ],
      },
      {
        id: 'persona-system',
        title: 'Persona System',
        description: (
          <React.Fragment>
            Personas define agent behavior, expertise, and communication style. Mix and
            match personas with different models to find the perfect combination for your
            use case. Extensible and customizable.
          </React.Fragment>
        ),
        codeExample: {
          language: 'toml',
          code: `# personas/expert-reviewer.toml
[persona]
id = "expert-reviewer"
expertise = ["architecture", "security", "testing"]
style = "socratic"
tone = "professional"
output_format = "structured"

[prompts]
system = """
You are a senior software architect with 15+ years
of experience. Focus on architectural patterns,
security vulnerabilities, and test coverage.
"""`,
        },
        links: [
          {label: 'Persona Guide', href: '/docs/user-guide/personas'},
          {label: 'Built-in Personas', href: '/docs/api/personas'},
        ],
      },
      {
        id: 'self-hosted-models',
        title: 'Self-Hosted Model Support',
        description: (
          <React.Fragment>
            Run models locally using Ollama or connect to your own inference servers. Full
            privacy and control over your data. Supports OpenAI-compatible APIs and custom
            providers.
          </React.Fragment>
        ),
        codeExample: {
          language: 'toml',
          code: `# Use Ollama for local inference
[model]
provider = "ollama"
model = "codellama:34b"
base_url = "http://localhost:11434"

# Or custom OpenAI-compatible endpoint
[model]
provider = "openai-compatible"
model = "custom-model"
base_url = "https://my-inference-server.com/v1"
api_key = "{{ env.CUSTOM_API_KEY }}"`,
        },
        links: [
          {label: 'Model Providers', href: '/docs/user-guide/model-providers'},
          {label: 'Ollama Integration', href: '/docs/user-guide/guides/ollama-integration'},
        ],
      },
    ],
  },
  {
    id: 'security-control',
    title: 'Security & Control',
    description: 'Enterprise-grade security and governance for autonomous agents',
    features: [
      {
        id: 'policy-engine',
        title: 'Policy Engine',
        description: (
          <React.Fragment>
            Define fine-grained policies to control agent behavior, resource usage, and
            decision-making. Policies can enforce approval workflows, cost limits, rate
            limiting, and more.
          </React.Fragment>
        ),
        codeExample: {
          language: 'toml',
          code: `[policy]
name = "production-safe"
require_approval = true
max_cost_per_task = 2.00
max_iterations = 20

[policy.allowed_actions]
file_operations = ["read", "write"]
network = ["https"]
tool_use = ["search", "analyze"]

[policy.restrictions]
deny_patterns = ["rm -rf", "DROP TABLE"]
quarantine_suspicious = true`,
        },
        links: [
          {label: 'Policy Guide', href: '/docs/user-guide/policies'},
          {label: 'Security Best Practices', href: '/docs/user-guide/security'},
        ],
      },
      {
        id: 'workflow-behaviors',
        title: 'Workflow Behaviors',
        description: (
          <React.Fragment>
            Control how workflows handle errors, retries, and edge cases. Built-in behaviors
            include graceful degradation, circuit breakers, exponential backoff, and custom
            error recovery strategies.
          </React.Fragment>
        ),
        codeExample: {
          language: 'toml',
          code: `[workflow.error_handling]
strategy = "retry-with-backoff"
max_retries = 3
backoff_multiplier = 2.0
circuit_breaker_threshold = 5

[workflow.fallback]
enabled = true
fallback_agent = "simple-model"
conditions = ["timeout", "rate_limit"]

[workflow.monitoring]
track_costs = true
alert_on_failure = true`,
        },
        links: [
          {label: 'Error Handling', href: '/docs/user-guide/error-handling'},
          {label: 'Workflow Guide', href: '/docs/user-guide/workflows'},
        ],
      },
      {
        id: 'vibe-check',
        title: 'Vibe Check (Metacognitive Oversight)',
        description: (
          <React.Fragment>
            Continuous monitoring of agent behavior and outputs. The vibe check system
            detects anomalies, hallucinations, and unexpected behavior patterns. Acts as a
            second layer of safety for autonomous agents.
          </React.Fragment>
        ),
        codeExample: {
          language: 'toml',
          code: `[oversight]
enabled = true
vibe_check_frequency = "every-iteration"
anomaly_threshold = 0.8

[oversight.monitors]
hallucination_detector = true
cost_monitor = true
output_quality_check = true

[oversight.actions]
on_anomaly = "pause-and-notify"
on_cost_overrun = "terminate"
on_quality_decline = "switch-model"`,
        },
        links: [
          {label: 'Oversight Guide', href: '/docs/user-guide/oversight'},
          {label: 'Safety Mechanisms', href: '/docs/user-guide/safety'},
        ],
      },
    ],
  },
  {
    id: 'developer-experience',
    title: 'Developer Experience',
    description: 'Tools and interfaces designed for developer productivity',
    features: [
      {
        id: 'multiple-interfaces',
        title: 'Multiple Interfaces (CLI/TUI/Desktop)',
        description: (
          <React.Fragment>
            Choose your preferred interface: CLI for automation and scripting, TUI for
            interactive sessions, or Desktop app for visual workflows. All interfaces share
            the same underlying engine and configuration.
          </React.Fragment>
        ),
        codeExample: {
          language: 'bash',
          code: `# CLI - Great for automation
radium-cli chat code-assistant "Review my PR"

# TUI - Interactive terminal UI
radium-tui

# Desktop - Visual workflow builder
radium-desktop

# Or use as a library in your code
cargo add radium-core`,
        },
        links: [
          {label: 'CLI Reference', href: '/docs/cli/README'},
          {label: 'TUI Guide', href: '/docs/user-guide/tui'},
          {label: 'Desktop App', href: '/docs/user-guide/desktop'},
        ],
      },
      {
        id: 'extension-system',
        title: 'Extension System',
        description: (
          <React.Fragment>
            Extend Radium with custom tools, data sources, and integrations. Extensions are
            first-class citizens with full access to the agent runtime. Share your extensions
            with the community.
          </React.Fragment>
        ),
        codeExample: {
          language: 'rust',
          code: `use radium_core::Extension;

#[derive(Extension)]
pub struct GitHubExtension {
    api_key: String,
}

impl GitHubExtension {
    fn search_issues(&self, query: &str) -> Result<Vec<Issue>> {
        // Custom tool implementation
    }

    fn create_pr(&self, params: PrParams) -> Result<PullRequest> {
        // Another tool
    }
}`,
        },
        links: [
          {label: 'Extension Development', href: '/docs/developer-guide/extensions'},
          {label: 'Extension API', href: '/docs/api/extensions'},
        ],
      },
      {
        id: 'session-analytics',
        title: 'Session Analytics',
        description: (
          <React.Fragment>
            Track costs, performance, and agent behavior across sessions. Built-in analytics
            help you optimize workflows, reduce costs, and improve agent performance over
            time. Export data for custom analysis.
          </React.Fragment>
        ),
        codeExample: {
          language: 'bash',
          code: `# View session statistics
radium-cli stats --session last

# Track costs over time
radium-cli stats --metric cost --period week

# Export analytics data
radium-cli stats export --format json > analytics.json

# Analyze agent performance
radium-cli stats agents --sort-by avg_latency`,
        },
        links: [
          {label: 'Analytics Guide', href: '/docs/user-guide/analytics'},
          {label: 'Cost Optimization', href: '/docs/user-guide/cost-optimization'},
        ],
      },
    ],
  },
];
