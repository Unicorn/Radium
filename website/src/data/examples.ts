import React, {type ReactNode} from 'react';

export type DifficultyLevel = 'beginner' | 'intermediate' | 'advanced';
export type ExampleCategory = 'workflow' | 'integration' | 'automation' | 'development';

export interface Example {
  id: string;
  title: string;
  description: ReactNode;
  category: ExampleCategory;
  difficulty: DifficultyLevel;
  timeEstimate: string;
  tags: string[];
  codeSnippet?: string;
  docLink: string;
  githubLink?: string;
}

export const examples: Example[] = [
  {
    id: 'multi-agent-workflow',
    title: 'Multi-Agent Code Review Workflow',
    description: (
      <React.Fragment>
        Orchestrate multiple agents to analyze, review, and improve code. Demonstrates DAG
        workflows, agent coordination, and error handling with automatic retries.
      </React.Fragment>
    ),
    category: 'workflow',
    difficulty: 'intermediate',
    timeEstimate: '15-20 min',
    tags: ['orchestration', 'code-review', 'multi-agent', 'dag'],
    codeSnippet: `# workflow.toml
[[agents]]
id = "analyzer"
model = "claude-3-sonnet"

[[agents]]
id = "reviewer"
model = "gpt-4"

[workflow]
steps = [
  { agent = "analyzer", input = "task.code" },
  { agent = "reviewer", depends_on = ["analyzer"] }
]`,
    docLink: '/docs/examples/orchestration-workflows',
    githubLink: 'https://github.com/clay-curry/RAD/tree/main/examples/code-review',
  },
  {
    id: 'ollama-local',
    title: 'Self-Hosted AI with Ollama',
    description: (
      <React.Fragment>
        Run agents completely locally using Ollama. No API keys required - perfect for privacy-sensitive
        applications or offline development. Supports CodeLlama, Mistral, and more.
      </React.Fragment>
    ),
    category: 'integration',
    difficulty: 'beginner',
    timeEstimate: '10 min',
    tags: ['ollama', 'self-hosted', 'privacy', 'local'],
    codeSnippet: `# agents/local-coder.toml
[agent]
id = "local-coder"
name = "Local Code Assistant"

[model]
provider = "ollama"
model = "codellama:34b"
base_url = "http://localhost:11434"`,
    docLink: '/docs/user-guide/guides/ollama-integration',
  },
  {
    id: 'cost-optimization',
    title: 'Cost-Optimized Agent Selection',
    description: (
      <React.Fragment>
        Automatically select the most cost-effective model for each task. Uses policy-driven
        orchestration to balance cost, speed, and quality based on your requirements.
      </React.Fragment>
    ),
    category: 'workflow',
    difficulty: 'intermediate',
    timeEstimate: '20 min',
    tags: ['cost-optimization', 'policy', 'orchestration'],
    codeSnippet: `[orchestration]
policy = "cost-optimized"
max_cost_per_task = 0.50

[agent_selection]
prefer_local = true
fallback_strategy = "cheapest-cloud"

[models.preference]
fast_tasks = ["gpt-3.5-turbo", "claude-haiku"]
complex_tasks = ["gpt-4", "claude-sonnet"]`,
    docLink: '/docs/user-guide/cost-optimization',
  },
  {
    id: 'autonomous-debugging',
    title: 'Autonomous Debugging Agent',
    description: (
      <React.Fragment>
        Enable agents to autonomously debug failing tests. Demonstrates YOLO mode, bounded
        autonomy, and intelligent error recovery with minimal human intervention.
      </React.Fragment>
    ),
    category: 'automation',
    difficulty: 'advanced',
    timeEstimate: '30 min',
    tags: ['autonomous', 'debugging', 'yolo-mode', 'testing'],
    codeSnippet: `[execution]
mode = "autonomous"
require_approval = false
max_iterations = 10
budget = 5.00

[oversight]
vibe_check_frequency = "every-iteration"
on_anomaly = "pause-and-notify"`,
    docLink: '/docs/user-guide/execution-modes',
  },
  {
    id: 'ci-cd-integration',
    title: 'CI/CD Pipeline Integration',
    description: (
      <React.Fragment>
        Integrate Radium agents into your CI/CD pipeline. Automate code reviews, documentation
        generation, and test creation as part of your build process.
      </React.Fragment>
    ),
    category: 'automation',
    difficulty: 'intermediate',
    timeEstimate: '25 min',
    tags: ['ci-cd', 'github-actions', 'automation'],
    codeSnippet: `# .github/workflows/ai-review.yml
- name: Run AI Code Review
  run: |
    radium-cli chat code-reviewer \\
      "Review PR #${{ github.event.number }}" \\
      --mode autonomous \\
      --max-cost 2.00`,
    docLink: '/docs/examples/ci-cd-integration',
  },
  {
    id: 'custom-extension',
    title: 'Building Custom Extensions',
    description: (
      <React.Fragment>
        Create custom tools and integrations for your agents. This example shows how to build a
        GitHub extension that lets agents search issues, create PRs, and manage repositories.
      </React.Fragment>
    ),
    category: 'development',
    difficulty: 'advanced',
    timeEstimate: '45 min',
    tags: ['extensions', 'rust', 'development', 'github'],
    codeSnippet: `use radium_core::Extension;

#[derive(Extension)]
pub struct GitHubExtension {
    api_key: String,
}

impl GitHubExtension {
    fn search_issues(&self, query: &str) -> Result<Vec<Issue>> {
        // Implementation
    }
}`,
    docLink: '/docs/developer-guide/extensions',
  },
  {
    id: 'persona-customization',
    title: 'Custom Agent Personas',
    description: (
      <React.Fragment>
        Define custom personas to shape agent behavior and expertise. This example creates a
        security-focused reviewer with specific knowledge domains and communication style.
      </React.Fragment>
    ),
    category: 'workflow',
    difficulty: 'beginner',
    timeEstimate: '15 min',
    tags: ['personas', 'customization', 'configuration'],
    codeSnippet: `# personas/security-expert.toml
[persona]
id = "security-expert"
expertise = ["security", "owasp", "cryptography"]
style = "thorough"
tone = "professional"

[prompts]
system = """
You are a cybersecurity expert specializing in
application security and the OWASP Top 10...
"""`,
    docLink: '/docs/user-guide/personas',
  },
  {
    id: 'data-pipeline',
    title: 'ETL Data Pipeline Orchestration',
    description: (
      <React.Fragment>
        Build intelligent ETL pipelines with AI-powered data validation and transformation.
        Agents can detect anomalies, suggest optimizations, and handle errors autonomously.
      </React.Fragment>
    ),
    category: 'workflow',
    difficulty: 'advanced',
    timeEstimate: '40 min',
    tags: ['etl', 'data-engineering', 'orchestration', 'validation'],
    codeSnippet: `[[workflow.steps]]
id = "extract"
agent = "data-extractor"

[[workflow.steps]]
id = "validate"
agent = "data-validator"
depends_on = ["extract"]

[[workflow.steps]]
id = "transform"
agent = "data-transformer"
depends_on = ["validate"]
parallel = ["quality-check"]`,
    docLink: '/docs/examples/data-pipeline',
  },
];

export const categories: Record<ExampleCategory, {label: string; description: string}> = {
  workflow: {
    label: 'Workflows',
    description: 'Multi-agent orchestration and workflow patterns',
  },
  integration: {
    label: 'Integrations',
    description: 'Connect Radium with external services and tools',
  },
  automation: {
    label: 'Automation',
    description: 'Automate development tasks and processes',
  },
  development: {
    label: 'Development',
    description: 'Extend Radium with custom code and tools',
  },
};

export const difficultyLabels: Record<DifficultyLevel, string> = {
  beginner: 'Beginner',
  intermediate: 'Intermediate',
  advanced: 'Advanced',
};
