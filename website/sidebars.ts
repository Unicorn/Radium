import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

/**
 * Radium Documentation Sidebar Structure (Phase 1 - Minimal)
 *
 * This is a simplified sidebar for testing. The full 11-section structure
 * will be implemented during Phase 2 (Content Migration).
 *
 * Full structure planned:
 * 1. Getting Started - Installation, quick start, core concepts
 * 2. User Guide - Agent configuration, workflows, persona system
 * 3. Features - Orchestration, policy engine, planning, sandboxing
 * 4. CLI Reference - All CLI commands and usage
 * 5. Extensions - Creating and managing extensions
 * 6. Hooks - Development hooks and lifecycle
 * 7. MCP Integration - Model Context Protocol integration
 * 8. Self-Hosted Models - Running local/self-hosted AI models
 * 9. Developer Guide - Architecture, extending Radium
 * 10. API Reference - Rust API documentation
 * 11. Examples - Agent examples, workflows, policies
 */
const sidebars: SidebarsConfig = {
  docsSidebar: [
    {
      type: 'doc',
      id: 'introduction',
      label: 'Introduction',
    },

    // Getting Started (partial - only installation for now)
    {
      type: 'category',
      label: '🚀 Getting Started',
      collapsed: false,
      items: [
        'getting-started/installation',
      ],
    },

    // Placeholder categories for future expansion
    {
      type: 'html',
      value: '<div style="margin-top: 1rem; padding: 0.5rem; opacity: 0.6; font-style: italic;">More sections coming soon...</div>',
      defaultStyle: true,
    },
  ],
};

export default sidebars;
