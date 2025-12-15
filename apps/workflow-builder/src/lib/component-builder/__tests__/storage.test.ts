/**
 * Storage Unit Tests
 *
 * Tests both filesystem and database storage backends with a mocked component builder.
 * Verifies that components can be saved, retrieved, and used in workflows.
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import * as path from 'path';
import * as fs from 'fs/promises';
import { fileURLToPath } from 'url';
import {
  ComponentStorage,
  FilesystemStorageBackend,
  DatabaseStorageBackend,
  type StoredComponent,
  type StoredArtifacts,
} from '../storage';
import type { WorkflowDefinition, WorkflowNode, WorkflowEdge } from '@/types/workflow';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// ============================================
// MOCK COMPONENT BUILDER OUTPUT
// ============================================

/**
 * Mock component that simulates what the Component Builder would generate.
 * This is the Timestamp Generator component from our earlier tests.
 */
function createMockTimestampGeneratorComponent(): StoredComponent {
  return {
    id: 'timestamp_generator',
    name: 'Timestamp Generator',
    description: 'Generates the current timestamp in various formats',
    category: 'utilities',
    temporalType: 'activity',
    version: '1.0.0',
    artifacts: {
      rustSchema: `use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TimestampFormat {
    #[serde(rename = "ISO8601")]
    Iso8601,
    #[serde(rename = "unix")]
    Unix,
    #[serde(rename = "rfc2822")]
    Rfc2822,
}

impl Default for TimestampFormat {
    fn default() -> Self {
        TimestampFormat::Iso8601
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct TimestampGeneratorInput {
    #[serde(default)]
    pub format: TimestampFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimestampGeneratorOutput {
    pub timestamp: String,
    pub unix_ms: i64,
}`,
      typescriptCode: `/**
 * Timestamp Generator Activity
 */

export type TimestampFormat = 'ISO8601' | 'unix' | 'rfc2822';

export interface TimestampGeneratorInput {
  format?: TimestampFormat;
}

export interface TimestampGeneratorOutput {
  timestamp: string;
  unixMs: number;
}

export function isTimestampGeneratorInput(value: unknown): value is TimestampGeneratorInput {
  if (typeof value !== 'object' || value === null) return false;
  const obj = value as Record<string, unknown>;
  if ('format' in obj && obj.format !== undefined) {
    return ['ISO8601', 'unix', 'rfc2822'].includes(obj.format as string);
  }
  return true;
}

export async function executeTimestampGenerator(
  input: TimestampGeneratorInput
): Promise<TimestampGeneratorOutput> {
  const format = input.format || 'ISO8601';
  const now = new Date();
  const unixMs = now.getTime();

  let timestamp: string;
  switch (format) {
    case 'unix':
      timestamp = Math.floor(unixMs / 1000).toString();
      break;
    case 'rfc2822':
      timestamp = now.toUTCString();
      break;
    case 'ISO8601':
    default:
      timestamp = now.toISOString();
      break;
  }

  return { timestamp, unixMs };
}`,
      testCases: `#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_format() {
        let input = TimestampGeneratorInput::default();
        assert!(matches!(input.format, TimestampFormat::Iso8601));
    }

    #[test]
    fn test_unix_format() {
        let input = TimestampGeneratorInput {
            format: TimestampFormat::Unix,
        };
        assert!(matches!(input.format, TimestampFormat::Unix));
    }
}`,
      migrationRecord: `component:
  name: timestamp_generator
  category: utilities
  version: 1.0.0
  description: Generates the current timestamp in various formats
  temporal_type: activity

migration:
  migrated_by: Component Builder
  migration_date: 2024-01-01T00:00:00.000Z
  difficulty: generated
`,
    },
    schema: {
      inputs: [
        {
          name: 'format',
          type: 'TimestampFormat',
          required: false,
          default: 'ISO8601',
          description: 'Output format for the timestamp',
        },
      ],
      outputs: [
        {
          name: 'timestamp',
          type: 'String',
          required: true,
          description: 'Formatted timestamp string',
        },
        {
          name: 'unix_ms',
          type: 'i64',
          required: true,
          description: 'Unix timestamp in milliseconds',
        },
      ],
      validationRules: [],
      connectionRules: {
        allowedSources: ['*'],
        allowedTargets: ['*'],
      },
    },
    metadata: {
      tags: ['timestamp', 'time', 'utilities'],
      usageCount: 0,
      isMarketplace: false,
    },
    createdAt: new Date('2024-01-01'),
    updatedAt: new Date('2024-01-01'),
    createdBy: 'test',
    status: 'published',
  };
}

// ============================================
// WORKFLOW HELPERS
// ============================================

/**
 * Create a simple start -> component -> end workflow definition
 */
function createStartComponentEndWorkflow(componentId: string): WorkflowDefinition {
  const nodes: WorkflowNode[] = [
    {
      id: 'start',
      type: 'trigger',
      position: { x: 100, y: 100 },
      data: { label: 'Start' },
    },
    {
      id: 'component-1',
      type: 'activity',
      position: { x: 300, y: 100 },
      data: {
        label: 'Timestamp Generator',
        componentId,
        componentName: 'timestamp_generator',
        config: { format: 'ISO8601' },
      },
    },
    {
      id: 'end',
      type: 'end',
      position: { x: 500, y: 100 },
      data: { label: 'End' },
    },
  ];

  const edges: WorkflowEdge[] = [
    { id: 'e1', source: 'start', target: 'component-1' },
    { id: 'e2', source: 'component-1', target: 'end' },
  ];

  return {
    nodes,
    edges,
    metadata: {
      description: 'Test workflow with custom component',
    },
  };
}

/**
 * Resolve component references in a workflow definition
 */
async function resolveWorkflowComponents(
  workflow: WorkflowDefinition,
  storage: ComponentStorage
): Promise<Map<string, StoredComponent>> {
  const components = new Map<string, StoredComponent>();

  for (const node of workflow.nodes) {
    if (node.data.componentId) {
      const component = await storage.get(node.data.componentId);
      if (component) {
        components.set(node.data.componentId, component);
      }
    }
  }

  return components;
}

// ============================================
// FILESYSTEM STORAGE TESTS
// ============================================

describe('FilesystemStorage', () => {
  const testDir = path.join(__dirname, 'test-fs-storage');
  let storage: ComponentStorage;

  beforeEach(async () => {
    // Clean up any existing test directory
    try {
      await fs.rm(testDir, { recursive: true });
    } catch {
      // Directory doesn't exist
    }

    const backend = new FilesystemStorageBackend({ baseDir: testDir });
    storage = new ComponentStorage({
      type: 'custom',
      customBackend: backend,
    });
    await storage.initialize();
  });

  afterEach(async () => {
    // Clean up test directory
    try {
      await fs.rm(testDir, { recursive: true });
    } catch {
      // Ignore errors
    }
  });

  describe('Component CRUD Operations', () => {
    it('should save a component', async () => {
      const component = createMockTimestampGeneratorComponent();
      const saved = await storage.save(component);

      expect(saved.id).toBe('timestamp_generator');
      expect(saved.name).toBe('Timestamp Generator');
    });

    it('should retrieve a saved component', async () => {
      const component = createMockTimestampGeneratorComponent();
      await storage.save(component);

      const retrieved = await storage.get('timestamp_generator');

      expect(retrieved).not.toBeNull();
      expect(retrieved!.id).toBe('timestamp_generator');
      expect(retrieved!.artifacts.rustSchema).toContain('TimestampFormat');
      expect(retrieved!.artifacts.typescriptCode).toContain('executeTimestampGenerator');
    });

    it('should return null for non-existent component', async () => {
      const retrieved = await storage.get('non_existent');
      expect(retrieved).toBeNull();
    });

    it('should check if a component exists', async () => {
      const component = createMockTimestampGeneratorComponent();
      await storage.save(component);

      expect(await storage.exists('timestamp_generator')).toBe(true);
      expect(await storage.exists('non_existent')).toBe(false);
    });

    it('should update a component', async () => {
      const component = createMockTimestampGeneratorComponent();
      await storage.save(component);

      const updated = await storage.update('timestamp_generator', {
        description: 'Updated description',
        version: '1.1.0',
      });

      expect(updated).not.toBeNull();
      expect(updated!.description).toBe('Updated description');
      expect(updated!.version).toBe('1.1.0');
      expect(updated!.name).toBe('Timestamp Generator'); // Unchanged
    });

    it('should delete a component', async () => {
      const component = createMockTimestampGeneratorComponent();
      await storage.save(component);

      const deleted = await storage.delete('timestamp_generator');
      expect(deleted).toBe(true);

      expect(await storage.exists('timestamp_generator')).toBe(false);
    });

    it('should return false when deleting non-existent component', async () => {
      const deleted = await storage.delete('non_existent');
      expect(deleted).toBe(false);
    });
  });

  describe('Component Listing and Search', () => {
    beforeEach(async () => {
      // Add multiple components for testing list/search
      const component1 = createMockTimestampGeneratorComponent();
      await storage.save(component1);

      const component2 = {
        ...createMockTimestampGeneratorComponent(),
        id: 'date_formatter',
        name: 'Date Formatter',
        description: 'Formats dates in various locales',
        metadata: {
          ...createMockTimestampGeneratorComponent().metadata,
          tags: ['date', 'formatter', 'utilities'],
        },
      };
      await storage.save(component2);
    });

    it('should list all components', async () => {
      const result = await storage.list();

      expect(result.total).toBe(2);
      expect(result.components.map(c => c.id)).toContain('timestamp_generator');
      expect(result.components.map(c => c.id)).toContain('date_formatter');
    });

    it('should search components by name', async () => {
      const results = await storage.search('timestamp');

      expect(results.length).toBe(1);
      expect(results[0].id).toBe('timestamp_generator');
    });

    it('should search components by description', async () => {
      const results = await storage.search('locales');

      expect(results.length).toBe(1);
      expect(results[0].id).toBe('date_formatter');
    });

    it('should increment usage count', async () => {
      await storage.incrementUsage('timestamp_generator');
      await storage.incrementUsage('timestamp_generator');
      await storage.incrementUsage('timestamp_generator');

      const component = await storage.get('timestamp_generator');
      expect(component!.metadata.usageCount).toBe(3);
    });
  });

  describe('Workflow Integration', () => {
    it('should save component and use in workflow definition', async () => {
      // Step 1: Mock component builder output and save
      const component = createMockTimestampGeneratorComponent();
      await storage.save(component);

      // Step 2: Create workflow that uses this component
      const workflow = createStartComponentEndWorkflow('timestamp_generator');

      // Step 3: Verify workflow can resolve its components
      const resolvedComponents = await resolveWorkflowComponents(workflow, storage);

      expect(resolvedComponents.size).toBe(1);
      expect(resolvedComponents.has('timestamp_generator')).toBe(true);

      const resolvedComponent = resolvedComponents.get('timestamp_generator')!;
      expect(resolvedComponent.artifacts.typescriptCode).toContain('executeTimestampGenerator');
    });

    it('should handle workflow with missing component gracefully', async () => {
      // Create workflow referencing non-existent component
      const workflow = createStartComponentEndWorkflow('non_existent_component');

      // Resolve should return empty map (no components found)
      const resolvedComponents = await resolveWorkflowComponents(workflow, storage);

      expect(resolvedComponents.size).toBe(0);
    });

    it('should track component usage when used in workflow', async () => {
      const component = createMockTimestampGeneratorComponent();
      await storage.save(component);

      // Simulate workflow execution incrementing usage
      const workflow = createStartComponentEndWorkflow('timestamp_generator');
      const resolvedComponents = await resolveWorkflowComponents(workflow, storage);

      for (const componentId of resolvedComponents.keys()) {
        await storage.incrementUsage(componentId);
      }

      const updated = await storage.get('timestamp_generator');
      expect(updated!.metadata.usageCount).toBe(1);
    });
  });

  describe('File Structure', () => {
    it('should create proper directory structure', async () => {
      const component = createMockTimestampGeneratorComponent();
      await storage.save(component);

      // Verify directories exist
      const dirs = await fs.readdir(testDir);
      expect(dirs).toContain('metadata');
      expect(dirs).toContain('rust');
      expect(dirs).toContain('typescript');
      expect(dirs).toContain('tests');
      expect(dirs).toContain('records');
    });

    it('should save artifacts to separate files', async () => {
      const component = createMockTimestampGeneratorComponent();
      await storage.save(component);

      // Verify individual files exist
      const metadataContent = await fs.readFile(
        path.join(testDir, 'metadata', 'timestamp_generator.json'),
        'utf-8'
      );
      expect(JSON.parse(metadataContent).id).toBe('timestamp_generator');

      const rustContent = await fs.readFile(
        path.join(testDir, 'rust', 'timestamp_generator.rs'),
        'utf-8'
      );
      expect(rustContent).toContain('TimestampFormat');

      const tsContent = await fs.readFile(
        path.join(testDir, 'typescript', 'timestamp_generator.ts'),
        'utf-8'
      );
      expect(tsContent).toContain('executeTimestampGenerator');
    });
  });
});

// ============================================
// DATABASE STORAGE TESTS
// ============================================

describe('DatabaseStorage', () => {
  let storage: ComponentStorage;

  beforeEach(async () => {
    const backend = new DatabaseStorageBackend();
    storage = new ComponentStorage({
      type: 'custom',
      customBackend: backend,
    });
    await storage.initialize();
  });

  describe('Component CRUD Operations', () => {
    it('should save a component', async () => {
      const component = createMockTimestampGeneratorComponent();
      const saved = await storage.save(component);

      expect(saved.id).toBe('timestamp_generator');
      expect(saved.name).toBe('Timestamp Generator');
    });

    it('should retrieve a saved component', async () => {
      const component = createMockTimestampGeneratorComponent();
      await storage.save(component);

      const retrieved = await storage.get('timestamp_generator');

      expect(retrieved).not.toBeNull();
      expect(retrieved!.id).toBe('timestamp_generator');
      expect(retrieved!.artifacts.rustSchema).toContain('TimestampFormat');
    });

    it('should return null for non-existent component', async () => {
      const retrieved = await storage.get('non_existent');
      expect(retrieved).toBeNull();
    });

    it('should check if a component exists', async () => {
      const component = createMockTimestampGeneratorComponent();
      await storage.save(component);

      expect(await storage.exists('timestamp_generator')).toBe(true);
      expect(await storage.exists('non_existent')).toBe(false);
    });

    it('should update a component', async () => {
      const component = createMockTimestampGeneratorComponent();
      await storage.save(component);

      const updated = await storage.update('timestamp_generator', {
        description: 'Updated via database',
      });

      expect(updated).not.toBeNull();
      expect(updated!.description).toBe('Updated via database');
    });

    it('should delete a component', async () => {
      const component = createMockTimestampGeneratorComponent();
      await storage.save(component);

      const deleted = await storage.delete('timestamp_generator');
      expect(deleted).toBe(true);

      expect(await storage.exists('timestamp_generator')).toBe(false);
    });
  });

  describe('Component Listing and Search', () => {
    beforeEach(async () => {
      const component1 = createMockTimestampGeneratorComponent();
      await storage.save(component1);

      const component2 = {
        ...createMockTimestampGeneratorComponent(),
        id: 'date_formatter',
        name: 'Date Formatter',
        description: 'Formats dates in various locales',
        metadata: {
          ...createMockTimestampGeneratorComponent().metadata,
          tags: ['date', 'formatter', 'utilities'],
        },
      };
      await storage.save(component2);
    });

    it('should list all components', async () => {
      const result = await storage.list();

      expect(result.total).toBe(2);
    });

    it('should search components', async () => {
      const results = await storage.search('timestamp');

      expect(results.length).toBe(1);
      expect(results[0].id).toBe('timestamp_generator');
    });

    it('should increment usage count', async () => {
      await storage.incrementUsage('timestamp_generator');

      const component = await storage.get('timestamp_generator');
      expect(component!.metadata.usageCount).toBe(1);
    });
  });

  describe('Workflow Integration', () => {
    it('should save component and use in workflow definition', async () => {
      // Step 1: Mock component builder output and save
      const component = createMockTimestampGeneratorComponent();
      await storage.save(component);

      // Step 2: Create workflow that uses this component
      const workflow = createStartComponentEndWorkflow('timestamp_generator');

      // Step 3: Verify workflow can resolve its components
      const resolvedComponents = await resolveWorkflowComponents(workflow, storage);

      expect(resolvedComponents.size).toBe(1);
      expect(resolvedComponents.has('timestamp_generator')).toBe(true);
    });

    it('should handle concurrent component access', async () => {
      const component = createMockTimestampGeneratorComponent();
      await storage.save(component);

      // Simulate multiple concurrent reads
      const reads = await Promise.all([
        storage.get('timestamp_generator'),
        storage.get('timestamp_generator'),
        storage.get('timestamp_generator'),
      ]);

      expect(reads.every(r => r !== null)).toBe(true);
      expect(reads.every(r => r!.id === 'timestamp_generator')).toBe(true);
    });

    it('should handle concurrent usage increments', async () => {
      const component = createMockTimestampGeneratorComponent();
      await storage.save(component);

      // Simulate multiple concurrent usage increments
      await Promise.all([
        storage.incrementUsage('timestamp_generator'),
        storage.incrementUsage('timestamp_generator'),
        storage.incrementUsage('timestamp_generator'),
      ]);

      const updated = await storage.get('timestamp_generator');
      expect(updated!.metadata.usageCount).toBe(3);
    });
  });
});

// ============================================
// MOCK COMPONENT BUILDER TESTS
// ============================================

describe('MockComponentBuilder', () => {
  /**
   * Mock implementation of the Component Builder's component generation
   * This simulates what happens when the builder agent generates a component
   */
  class MockComponentBuilder {
    private conversationPhase: 'gathering' | 'designing' | 'generating' | 'complete' = 'gathering';
    private componentDesign: Partial<StoredComponent> = {};

    /**
     * Simulate starting a conversation
     */
    start(): string {
      this.conversationPhase = 'gathering';
      return "What type of component would you like to build?";
    }

    /**
     * Simulate gathering requirements
     */
    gatherRequirements(input: string): string {
      this.conversationPhase = 'designing';
      this.componentDesign = {
        name: 'Timestamp Generator',
        description: input,
        category: 'utilities',
        temporalType: 'activity',
      };
      return "I'll create a Timestamp Generator component. Let me design it.";
    }

    /**
     * Simulate generating the component
     */
    async generate(): Promise<StoredComponent> {
      this.conversationPhase = 'generating';

      // Simulate some processing time
      await new Promise(resolve => setTimeout(resolve, 10));

      const component = createMockTimestampGeneratorComponent();
      this.conversationPhase = 'complete';

      return component;
    }

    getPhase(): string {
      return this.conversationPhase;
    }
  }

  describe('Component Builder Flow with Filesystem Storage', () => {
    const testDir = path.join(__dirname, 'test-builder-fs');
    let storage: ComponentStorage;

    beforeEach(async () => {
      try {
        await fs.rm(testDir, { recursive: true });
      } catch {}

      const backend = new FilesystemStorageBackend({ baseDir: testDir });
      storage = new ComponentStorage({
        type: 'custom',
        customBackend: backend,
      });
      await storage.initialize();
    });

    afterEach(async () => {
      try {
        await fs.rm(testDir, { recursive: true });
      } catch {}
    });

    it('should complete full flow: start -> build component -> save -> use in workflow', async () => {
      // Step 1: Start component builder
      const builder = new MockComponentBuilder();
      const greeting = builder.start();
      expect(greeting).toContain('component');
      expect(builder.getPhase()).toBe('gathering');

      // Step 2: Provide requirements
      const response = builder.gatherRequirements('Generate timestamps in various formats');
      expect(response).toContain('Timestamp Generator');
      expect(builder.getPhase()).toBe('designing');

      // Step 3: Generate component
      const component = await builder.generate();
      expect(component.id).toBe('timestamp_generator');
      expect(builder.getPhase()).toBe('complete');

      // Step 4: Save to storage
      await storage.save(component);
      expect(await storage.exists('timestamp_generator')).toBe(true);

      // Step 5: Create and verify workflow
      const workflow = createStartComponentEndWorkflow('timestamp_generator');
      const resolvedComponents = await resolveWorkflowComponents(workflow, storage);

      expect(resolvedComponents.size).toBe(1);

      // Verify component code can be retrieved for execution
      const savedComponent = resolvedComponents.get('timestamp_generator')!;
      expect(savedComponent.artifacts.typescriptCode).toContain('executeTimestampGenerator');
    });
  });

  describe('Component Builder Flow with Database Storage', () => {
    let storage: ComponentStorage;

    beforeEach(async () => {
      const backend = new DatabaseStorageBackend();
      storage = new ComponentStorage({
        type: 'custom',
        customBackend: backend,
      });
      await storage.initialize();
    });

    it('should complete full flow: start -> build component -> save -> use in workflow', async () => {
      // Step 1: Start component builder
      const builder = new MockComponentBuilder();
      const greeting = builder.start();
      expect(greeting).toContain('component');

      // Step 2: Provide requirements
      const response = builder.gatherRequirements('Generate timestamps in various formats');
      expect(response).toContain('Timestamp Generator');

      // Step 3: Generate component
      const component = await builder.generate();
      expect(component.id).toBe('timestamp_generator');

      // Step 4: Save to storage
      await storage.save(component);
      expect(await storage.exists('timestamp_generator')).toBe(true);

      // Step 5: Create and verify workflow
      const workflow = createStartComponentEndWorkflow('timestamp_generator');
      const resolvedComponents = await resolveWorkflowComponents(workflow, storage);

      expect(resolvedComponents.size).toBe(1);

      const savedComponent = resolvedComponents.get('timestamp_generator')!;
      expect(savedComponent.artifacts.typescriptCode).toContain('executeTimestampGenerator');
    });
  });
});
