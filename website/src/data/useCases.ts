import React, {type ReactNode} from 'react';

export type UseCaseCategory = 'development' | 'data' | 'enterprise' | 'devops' | 'security';

export interface UseCase {
  id: string;
  title: string;
  category: UseCaseCategory;
  problem: ReactNode;
  solution: ReactNode;
  techStack: string[];
  results: string[];
  codeSnippet?: string;
  tutorialLink?: string;
  exampleLink?: string;
}

export const categories: Record<UseCaseCategory, {label: string; description: string}> = {
  development: {
    label: 'Development',
    description: 'Software development automation and workflows',
  },
  data: {
    label: 'Data Processing',
    description: 'ETL pipelines and data transformation',
  },
  enterprise: {
    label: 'Enterprise',
    description: 'Enterprise-scale operations and cost optimization',
  },
  devops: {
    label: 'DevOps',
    description: 'CI/CD, infrastructure, and deployment automation',
  },
  security: {
    label: 'Security',
    description: 'Security audits, compliance, and vulnerability scanning',
  },
};

export const useCases: UseCase[] = [
  {
    id: 'cicd-automation',
    title: 'CI/CD Pipeline Automation',
    category: 'devops',
    problem: <React.Fragment>
      Manual testing and deployment processes slow down release cycles and increase the risk of
      human error in production deployments.
    </React.Fragment>,
    solution: <React.Fragment>
      Radium orchestrates multiple specialized agents to handle test execution, code quality checks,
      and deployment strategies. The policy engine ensures compliance with deployment rules while
      the cost tracking features monitor cloud resource usage.
    </React.Fragment>,
    techStack: ['Radium', 'GitHub Actions', 'Kubernetes', 'AWS'],
    results: [
      '60% reduction in deployment time',
      '95% decrease in failed deployments',
      'Zero manual intervention required',
    ],
    codeSnippet: `# agents/cicd-pipeline.toml
[agent.test-runner]
persona = "qa-engineer"
capabilities = ["code-testing", "coverage-analysis"]

[agent.deployer]
persona = "devops-specialist"
capabilities = ["kubernetes", "health-checks"]

[workflow]
type = "dag"
nodes = ["test", "build", "deploy"]`,
    tutorialLink: '/docs/user-guide/guides',
    exampleLink: '/examples',
  },
  {
    id: 'codebase-refactoring',
    title: 'Automated Codebase Refactoring',
    category: 'development',
    problem: <React.Fragment>
      Legacy codebases with technical debt require extensive manual refactoring efforts,
      often taking weeks of developer time while risking introduction of new bugs.
    </React.Fragment>,
    solution: <React.Fragment>
      Multi-agent workflows analyze code patterns, identify refactoring opportunities, and
      systematically apply transformations with built-in testing at each step. The vibe check
      system validates changes before committing.
    </React.Fragment>,
    techStack: ['Radium', 'TypeScript', 'Jest', 'ESLint'],
    results: [
      '80% faster refactoring cycles',
      '100% test coverage maintained',
      '45% reduction in code complexity',
    ],
    codeSnippet: `# refactoring-workflow.toml
[workflow]
type = "dag"

[[workflow.node]]
id = "analyze"
agent = "code-analyzer"
output = "refactoring-plan"

[[workflow.node]]
id = "refactor"
agent = "code-transformer"
dependencies = ["analyze"]`,
    tutorialLink: '/docs/user-guide/guides',
    exampleLink: '/examples',
  },
  {
    id: 'etl-orchestration',
    title: 'ETL Pipeline Orchestration',
    category: 'data',
    problem: <React.Fragment>
      Complex data pipelines with multiple sources require coordinating transformations,
      validations, and load operations across different systems with varying failure modes.
    </React.Fragment>,
    solution: <React.Fragment>
      Radium coordinates data extraction agents, transformation workers, and loading processes
      with automatic retry logic and error recovery. Real-time monitoring tracks pipeline health
      and data quality metrics.
    </React.Fragment>,
    techStack: ['Radium', 'PostgreSQL', 'Apache Kafka', 'Snowflake'],
    results: [
      '99.9% pipeline reliability',
      '70% reduction in data errors',
      'Real-time data quality monitoring',
    ],
    codeSnippet: `# etl-pipeline.toml
[workflow]
type = "dag"

[[workflow.node]]
id = "extract"
agent = "data-extractor"
sources = ["api", "database", "files"]

[[workflow.node]]
id = "transform"
agent = "data-transformer"
dependencies = ["extract"]

[[workflow.node]]
id = "validate"
agent = "data-validator"
dependencies = ["transform"]`,
    tutorialLink: '/docs/user-guide/guides',
    exampleLink: '/examples',
  },
  {
    id: 'cloud-cost-optimization',
    title: 'Multi-Cloud Cost Optimization',
    category: 'enterprise',
    problem: <React.Fragment>
      Organizations running workloads across multiple cloud providers struggle to track costs,
      identify waste, and implement optimization strategies consistently.
    </React.Fragment>,
    solution: <React.Fragment>
      Specialized agents continuously monitor resource usage across AWS, Azure, and GCP,
      identify optimization opportunities, and automatically implement approved cost-saving
      measures while respecting policy constraints.
    </React.Fragment>,
    techStack: ['Radium', 'AWS', 'Azure', 'GCP', 'Prometheus'],
    results: [
      '40% reduction in cloud spending',
      'Real-time cost anomaly detection',
      'Automated rightsizing recommendations',
    ],
    codeSnippet: `# cost-optimizer.toml
[agent.aws-analyzer]
capabilities = ["ec2", "rds", "s3-analysis"]
persona = "cost-optimizer"

[agent.azure-analyzer]
capabilities = ["vm-analysis", "storage-optimization"]
persona = "cost-optimizer"

[policy]
require_approval = ["resource-deletion", "instance-changes"]`,
    tutorialLink: '/docs/user-guide/guides',
    exampleLink: '/examples',
  },
  {
    id: 'documentation-generation',
    title: 'Automated Documentation Generation',
    category: 'development',
    problem: <React.Fragment>
      Keeping documentation in sync with code changes is time-consuming and error-prone,
      leading to outdated docs that confuse users and slow down adoption.
    </React.Fragment>,
    solution: <React.Fragment>
      Agents analyze code structure, extract API signatures, generate usage examples, and
      produce comprehensive documentation in multiple formats. Continuous monitoring ensures
      docs stay synchronized with code changes.
    </React.Fragment>,
    techStack: ['Radium', 'Docusaurus', 'TypeDoc', 'Markdown'],
    results: [
      '90% reduction in doc maintenance time',
      '100% API coverage',
      'Always up-to-date documentation',
    ],
    codeSnippet: `# doc-generator.toml
[agent.code-parser]
capabilities = ["ast-analysis", "type-extraction"]

[agent.doc-writer]
capabilities = ["markdown-generation", "example-creation"]
persona = "technical-writer"

[workflow]
trigger = "on_commit"
output_format = ["markdown", "html"]`,
    tutorialLink: '/docs/user-guide/guides',
    exampleLink: '/examples',
  },
  {
    id: 'security-audit',
    title: 'Continuous Security Auditing',
    category: 'security',
    problem: <React.Fragment>
      Manual security reviews are infrequent, inconsistent, and can't keep pace with rapid
      deployment cycles, leaving vulnerabilities undetected until discovered in production.
    </React.Fragment>,
    solution: <React.Fragment>
      Security-specialized agents continuously scan code, dependencies, and infrastructure for
      vulnerabilities. The policy engine enforces security standards and blocks deployments
      that violate compliance requirements.
    </React.Fragment>,
    techStack: ['Radium', 'SAST tools', 'Snyk', 'OWASP ZAP'],
    results: [
      '85% faster vulnerability detection',
      'Zero critical CVEs in production',
      'Automated compliance reporting',
    ],
    codeSnippet: `# security-audit.toml
[agent.code-scanner]
capabilities = ["static-analysis", "secret-detection"]
persona = "security-specialist"

[agent.dependency-checker]
capabilities = ["cve-scanning", "license-compliance"]

[policy]
block_on = ["critical-vulnerabilities", "license-violations"]`,
    tutorialLink: '/docs/user-guide/guides',
    exampleLink: '/examples',
  },
  {
    id: 'infrastructure-provisioning',
    title: 'Infrastructure as Code Automation',
    category: 'devops',
    problem: <React.Fragment>
      Provisioning and managing cloud infrastructure manually leads to configuration drift,
      inconsistencies across environments, and difficulty scaling operations.
    </React.Fragment>,
    solution: <React.Fragment>
      Radium orchestrates infrastructure provisioning agents that apply Terraform/Pulumi configs,
      validate deployments, and maintain state consistency across multiple environments with
      automatic rollback on failures.
    </React.Fragment>,
    techStack: ['Radium', 'Terraform', 'AWS', 'Kubernetes'],
    results: [
      '75% faster environment setup',
      '100% infrastructure consistency',
      'Zero configuration drift',
    ],
    codeSnippet: `# infrastructure.toml
[agent.terraform-operator]
capabilities = ["plan", "apply", "state-management"]
persona = "infrastructure-engineer"

[workflow]
type = "sequential"
steps = ["validate", "plan", "apply", "verify"]

[policy]
require_approval = ["production-changes"]`,
    tutorialLink: '/docs/user-guide/guides',
    exampleLink: '/examples',
  },
  {
    id: 'data-quality-monitoring',
    title: 'Real-Time Data Quality Monitoring',
    category: 'data',
    problem: <React.Fragment>
      Data quality issues go undetected until they impact business reports or analytics,
      resulting in incorrect decisions and loss of stakeholder trust in data systems.
    </React.Fragment>,
    solution: <React.Fragment>
      Monitoring agents continuously validate data against quality rules, detect anomalies,
      and alert on issues in real-time. Automated remediation workflows fix common problems
      while escalating complex issues to human operators.
    </React.Fragment>,
    techStack: ['Radium', 'Great Expectations', 'dbt', 'Airflow'],
    results: [
      '95% reduction in data quality incidents',
      'Real-time anomaly detection',
      'Automated data remediation',
    ],
    codeSnippet: `# data-quality.toml
[agent.validator]
capabilities = ["schema-validation", "anomaly-detection"]
persona = "data-engineer"

[agent.remediator]
capabilities = ["data-correction", "backfill"]

[workflow]
trigger = "on_data_ingestion"
alert_on = ["validation-failure", "anomaly-detected"]`,
    tutorialLink: '/docs/user-guide/guides',
    exampleLink: '/examples',
  },
];
