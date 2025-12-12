import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

/**
 * Radium Documentation Sidebar Structure
 *
 * Complete 12-section structure serving both end users and developers:
 * 1. Getting Started - Installation, quick start, core concepts
 * 2. Roadmap - Open-source roadmap and vision
 * 3. User Guide - Agent configuration, workflows, persona system
 * 4. Features - Orchestration, policy engine, planning, sandboxing
 * 5. CLI Reference - All CLI commands and usage
 * 6. Extensions - Creating and managing extensions
 * 7. Hooks - Development hooks and lifecycle
 * 8. MCP Integration - Model Context Protocol integration
 * 9. Self-Hosted Models - Running local/self-hosted AI models
 * 10. Developer Guide - Architecture, extending Radium
 * 11. API Reference - Rust API documentation
 * 12. Examples - Agent examples, workflows, policies
 */
const sidebars: SidebarsConfig = {
  docsSidebar: [
    {
      type: 'doc',
      id: 'introduction',
      label: 'Introduction',
    },

    // 1. Getting Started
    {
      type: 'category',
      label: 'Getting Started',
      collapsed: false,
      items: [
        'getting-started/installation',
      ],
    },

    // 2. Roadmap
    {
      type: 'category',
      label: 'Roadmap',
      collapsed: false,
      items: [
        'roadmap/roadmap',
        'roadmap/roadmap-vision',
        'roadmap/roadmap-technical-architecture',
        'roadmap/roadmap-protocol-specifications',
        'roadmap/roadmap-governance-operations',
      ],
    },

    // 3. User Guide
    {
      type: 'category',
      label: 'User Guide',
      collapsed: true,
      items: [
        'user-guide/agent-configuration',
        'user-guide/orchestration',
        'user-guide/orchestration-configuration',
        'user-guide/orchestration-testing',
        'user-guide/orchestration-troubleshooting',
        'user-guide/persona-system',
        'user-guide/vibe-check',
        'user-guide/constitution-rules',
        'user-guide/memory-and-context',
        'user-guide/context-sources',
        'user-guide/learning-system',
        'user-guide/custom-commands',
        {
          type: 'category',
          label: 'Advanced Topics',
          items: [
            'user-guide/guides/agent-creation-guide',
            'user-guide/guides/reasoning-configuration',
            'user-guide/guides/sandbox-setup',
            'user-guide/guides/optimizing-costs',
            'user-guide/guides/extension-best-practices',
            'user-guide/guides/extension-distribution',
            'user-guide/guides/extension-versioning',
            'user-guide/guides/extension-testing',
            'user-guide/guides/adding-new-engine-provider',
          ],
        },
      ],
    },

    // 4. Features
    {
      type: 'category',
      label: 'Features',
      collapsed: true,
      items: [
        'features/policy-engine',
        'features/constitution-system',
        'features/autonomous-planning',
        'features/plan-execution',
        'features/workflow-behaviors',
        'features/sandboxing',
        'features/checkpointing',
        'features/dag-dependencies',
        'features/thinking-mode',
        'features/context-files',
        'features/context-caching',
        'features/session-analytics',
        {
          type: 'category',
          label: 'Planning',
          items: [
            'features/planning/autonomous-planning',
            'features/planning/execution-modes',
            'features/planning/best-practices',
            'features/planning/monitoring-integration',
          ],
        },
        {
          type: 'category',
          label: 'Security',
          items: [
            'features/security/configuration',
            'features/security/secret-management',
            'features/security/policy-best-practices',
            'features/security/migration-guide',
          ],
        },
        {
          type: 'category',
          label: 'Requirements',
          items: [
            'features/requirements/async-requirement-execution',
            'features/requirements/requirement-execution-ux',
          ],
        },
        {
          type: 'category',
          label: 'Monitoring',
          items: [
            'features/monitoring/architecture',
            'features/monitoring/usage-guide',
            'features/monitoring/api-reference',
          ],
        },
        {
          type: 'category',
          label: 'Editor Integration',
          items: [
            'features/editor-integration/overview',
            'features/editor-integration/architecture',
            'features/editor-integration/vscode',
            'features/editor-integration/neovim',
            'features/editor-integration/clipboard',
            'features/editor-integration/troubleshooting',
          ],
        },
        {
          type: 'category',
          label: 'YOLO Mode',
          items: [
            'features/yolo-mode/integration-map',
          ],
        },
      ],
    },

    // 5. CLI Reference
    {
      type: 'category',
      label: 'CLI Reference',
      collapsed: true,
      items: [
        'cli/README',
        'cli/configuration',
        'cli/workflows',
        'cli/shell-completion',
        'cli/command-patterns',
        'cli/security',
        'cli/performance',
        'cli/testing',
        'cli/architecture',
        'cli/troubleshooting',
        {
          type: 'category',
          label: 'Commands',
          items: [
            'cli/commands/agents',
            'cli/commands/execution',
            'cli/commands/plan-execution',
            'cli/commands/workspace',
            'cli/commands/mcp',
            'cli/commands/extensions',
            'cli/commands/monitoring',
            'cli/commands/advanced',
          ],
        },
      ],
    },

    // 6. Extensions
    {
      type: 'category',
      label: 'Extensions',
      collapsed: true,
      items: [
        'extensions/README',
        'extensions/quickstart',
        'extensions/creating-extensions',
        'extensions/user-guide',
        'extensions/architecture',
        'extensions/api-reference',
        'extensions/integration-guide',
        'extensions/publishing-guide',
        'extensions/marketplace',
        'extensions/performance',
        {
          type: 'category',
          label: 'Examples',
          items: [
            'extensions/examples/simple-prompts/README',
            'extensions/examples/simple-prompts/prompts/code-reviewer',
            'extensions/examples/complete-extension/README',
            'extensions/examples/complete-extension/prompts/example-agent',
            'extensions/examples/mcp-tools/README',
          ],
        },
      ],
    },

    // 7. Hooks
    {
      type: 'category',
      label: 'Hooks',
      collapsed: true,
      items: [
        'hooks/README',
        'hooks/getting-started',
        'hooks/creating-hooks',
        'hooks/hook-types',
        'hooks/hook-development',
        'hooks/configuration',
        'hooks/api-reference',
        'hooks/testing-hooks',
        'hooks/best-practices',
        'hooks/tutorial',
        'hooks/examples',
        'hooks/architecture',
        'hooks/troubleshooting',
      ],
    },

    // 8. MCP Integration
    {
      type: 'category',
      label: 'MCP Integration',
      collapsed: true,
      items: [
        'mcp/README',
        'mcp/user-guide',
        'mcp/configuration',
        'mcp/architecture',
        'mcp/authentication',
        'mcp/oauth-setup',
        'mcp/tools',
        'mcp/prompts',
        'mcp/mcp-proxy',
        'mcp/mcp-proxy-config',
        'mcp/troubleshooting',
        {
          type: 'category',
          label: 'Examples',
          items: [
            'mcp/examples/stdio-server',
            'mcp/examples/remote-server',
            'mcp/examples/oauth-server',
          ],
        },
      ],
    },

    // 9. Self-Hosted Models
    {
      type: 'category',
      label: 'Self-Hosted Models',
      collapsed: true,
      items: [
        'self-hosted/README',
        'self-hosted/migration',
        'self-hosted/VALIDATION',
        'self-hosted/api-reference',
        'self-hosted/troubleshooting',
        {
          type: 'category',
          label: 'Setup',
          items: [
            'self-hosted/setup/ollama',
            'self-hosted/setup/vllm',
            'self-hosted/setup/localai',
          ],
        },
        {
          type: 'category',
          label: 'Configuration',
          items: [
            'self-hosted/configuration/agent-config',
            'self-hosted/configuration/advanced',
            'self-hosted/configuration/examples',
          ],
        },
      ],
    },

    // 10. Developer Guide
    {
      type: 'category',
      label: 'Developer',
      collapsed: true,
      items: [
        'developer-guide/agent-system-architecture',
        'developer-guide/extending-sources',
        'developer-guide/extension-mcp-integration',
        {
          type: 'category',
          label: 'Architecture',
          items: [
            'developer-guide/architecture/agent-configuration-system',
            'developer-guide/architecture/checkpoint-system',
            'developer-guide/architecture/engine-abstraction',
            'developer-guide/architecture/tui-architecture',
          ],
        },
        {
          type: 'category',
          label: 'Design',
          items: [
            'developer-guide/design/persona-system-architecture',
          ],
        },
        {
          type: 'category',
          label: 'Development',
          items: [
            'developer-guide/development/agent-instructions',
            'developer-guide/development/colors',
            'developer-guide/development/deep-analysis-improvements',
          ],
        },
        {
          type: 'category',
          label: 'Testing',
          items: [
            'developer-guide/testing/coverage-analysis-REQ-172',
            'developer-guide/testing/coverage-backlog',
          ],
        },
        {
          type: 'category',
          label: 'ADR',
          items: [
            'developer-guide/adr/001-yolo-mode-architecture',
          ],
        },
        {
          type: 'category',
          label: 'Guides',
          items: [
            'developer-guide/guides/json-schema-guide',
          ],
        },
      ],
    },

    // 11. API Reference
    {
      type: 'category',
      label: 'API Reference',
      collapsed: true,
      items: [
        'api/context-cache-api',
        {
          type: 'link',
          label: '→ Rust API Docs',
          href: '/RAD/api/radium_core',
        },
      ],
    },

    // 12. Examples
    {
      type: 'category',
      label: 'Examples',
      collapsed: true,
      items: [
        'examples/orchestration-workflows',
        'examples/policy-profiles',
        'examples/vibe-check-workflow',
      ],
    },
  ],
};

export default sidebars;
