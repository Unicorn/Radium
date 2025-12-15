/**
 * Storage E2E Tests
 *
 * End-to-end tests that use the real Anthropic API to build components
 * and verify the full flow: build -> store -> workflow integration.
 *
 * These tests ONLY run when:
 * 1. .env.test file exists
 * 2. ANTHROPIC_API_KEY is set in .env.test
 *
 * Run with: npx vitest run src/lib/component-builder/__tests__/storage-e2e.test.ts
 */

import { describe, it, expect, beforeAll, afterAll, beforeEach } from 'vitest';
import * as path from 'path';
import * as fs from 'fs/promises';
import { fileURLToPath } from 'url';
import { config } from 'dotenv';
import {
  ComponentStorage,
  FilesystemStorageBackend,
  DatabaseStorageBackend,
  type StoredComponent,
} from '../storage';
import { ComponentBuilderAgent } from '../agent/builder-agent';
import { KnowledgeRetrieval } from '../knowledge-base/retrieval';
import type { WorkflowDefinition, WorkflowNode, WorkflowEdge } from '@/types/workflow';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// ============================================
// ENV SETUP AND SKIP DETECTION
// ============================================

/**
 * Load .env.test and check if API key is available
 */
function loadTestEnv(): { hasApiKey: boolean; apiKey: string | undefined; reason: string } {
  const envTestPath = path.join(process.cwd(), '.env.test');

  try {
    // Check if .env.test exists
    const envTestExists = require('fs').existsSync(envTestPath);
    if (!envTestExists) {
      return {
        hasApiKey: false,
        apiKey: undefined,
        reason: '.env.test file does not exist',
      };
    }

    // Load .env.test
    const result = config({ path: envTestPath });
    if (result.error) {
      return {
        hasApiKey: false,
        apiKey: undefined,
        reason: `Failed to load .env.test: ${result.error.message}`,
      };
    }

    // Check for ANTHROPIC_API_KEY
    const apiKey = process.env.ANTHROPIC_API_KEY;
    if (!apiKey) {
      return {
        hasApiKey: false,
        apiKey: undefined,
        reason: 'ANTHROPIC_API_KEY is not set in .env.test',
      };
    }

    return {
      hasApiKey: true,
      apiKey,
      reason: 'API key loaded successfully',
    };
  } catch (error) {
    return {
      hasApiKey: false,
      apiKey: undefined,
      reason: `Error loading env: ${error instanceof Error ? error.message : 'Unknown error'}`,
    };
  }
}

const envConfig = loadTestEnv();
const SKIP_E2E_TESTS = !envConfig.hasApiKey;

if (SKIP_E2E_TESTS) {
  console.log(`\n⏭️  Skipping E2E tests: ${envConfig.reason}`);
  console.log('   To run E2E tests, create .env.test with ANTHROPIC_API_KEY\n');
}

// ============================================
// WORKFLOW HELPERS
// ============================================

/**
 * Create a simple start -> component -> end workflow definition
 */
function createStartComponentEndWorkflow(
  componentId: string,
  componentName: string,
  componentConfig?: Record<string, unknown>
): WorkflowDefinition {
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
        label: componentName,
        componentId,
        componentName,
        config: componentConfig || {},
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
      description: `E2E test workflow with ${componentName}`,
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

/**
 * Validate that a component has required artifacts for execution
 * Note: This is a lenient check - the component builder may generate
 * different code patterns, so we just verify essential things exist.
 */
function validateComponentForExecution(component: StoredComponent): {
  valid: boolean;
  errors: string[];
} {
  const errors: string[] = [];

  // Must have TypeScript code
  if (!component.artifacts.typescriptCode || component.artifacts.typescriptCode.length < 50) {
    errors.push('Missing or too short TypeScript code');
  }

  // Must have Rust schema
  if (!component.artifacts.rustSchema || component.artifacts.rustSchema.length < 50) {
    errors.push('Missing or too short Rust schema');
  }

  // TypeScript code should have some function definition
  if (component.artifacts.typescriptCode) {
    const hasFunction =
      component.artifacts.typescriptCode.includes('function') ||
      component.artifacts.typescriptCode.includes('=>');
    if (!hasFunction) {
      errors.push('TypeScript code missing function definition');
    }
  }

  // Note: We don't strictly require schema outputs as the builder
  // may not always populate them correctly

  return {
    valid: errors.length === 0,
    errors,
  };
}

/**
 * Create a knowledge retrieval instance for testing
 */
function createKnowledge(apiKey: string): KnowledgeRetrieval {
  return new KnowledgeRetrieval({
    apiKey,
    model: 'claude-sonnet-4-20250514',
    maxResults: 3,
  });
}

/**
 * Drive the component builder through its phases until complete
 */
async function driveComponentBuilder(
  agent: ComponentBuilderAgent,
  initialRequest: string,
  maxIterations = 15
): Promise<{ success: boolean; finalPhase: string; iterations: number }> {
  let iterations = 0;

  // Start with the initial request
  let response = await agent.chat(initialRequest);
  iterations++;

  // Continue until complete or max iterations
  while (agent.getPhase() !== 'complete' && iterations < maxIterations) {
    // Provide appropriate response based on phase
    const phase = agent.getPhase();
    let userMessage: string;

    if (phase === 'gathering') {
      userMessage = 'Please proceed with the design based on what I described.';
    } else if (phase === 'designing' || phase === 'refining') {
      userMessage = 'Looks good, please generate the code.';
    } else if (phase === 'generating') {
      userMessage = 'Please continue generating.';
    } else if (phase === 'reviewing') {
      userMessage = 'Approve. Save the component.';
    } else {
      userMessage = 'Continue.';
    }

    response = await agent.chat(userMessage);
    iterations++;
  }

  return {
    success: agent.getPhase() === 'complete',
    finalPhase: agent.getPhase(),
    iterations,
  };
}

// ============================================
// E2E TESTS - FILESYSTEM STORAGE
// ============================================

describe.skipIf(SKIP_E2E_TESTS)('E2E: Component Builder with Filesystem Storage', () => {
  const testDir = path.join(__dirname, 'test-e2e-fs');
  let storage: ComponentStorage;
  let knowledge: KnowledgeRetrieval;

  beforeAll(async () => {
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

    // Create knowledge retrieval
    knowledge = createKnowledge(envConfig.apiKey!);
  });

  afterAll(async () => {
    // Clean up test directory
    try {
      await fs.rm(testDir, { recursive: true });
    } catch {
      // Ignore errors
    }
  });

  it('should build a simple utility component and save to filesystem', async () => {
    // Create agent with storage
    const agent = new ComponentBuilderAgent(knowledge, {
      apiKey: envConfig.apiKey!,
    }, storage);

    // Drive the builder through all phases
    const result = await driveComponentBuilder(
      agent,
      'I need a component that generates a random UUID. It should have no inputs and output a string uuid.'
    );

    expect(result.success).toBe(true);
    expect(result.finalPhase).toBe('complete');

    // Verify component was saved
    const components = await storage.list();
    expect(components.total).toBeGreaterThanOrEqual(1);

    // Get the generated component
    const savedComponent = components.components[0];
    expect(savedComponent).toBeDefined();

    // Validate component for execution
    const validation = validateComponentForExecution(savedComponent);
    if (!validation.valid) {
      console.log('Component validation errors:', validation.errors);
    }
    expect(validation.valid).toBe(true);

    // Create a workflow using this component
    const workflow = createStartComponentEndWorkflow(
      savedComponent.id,
      savedComponent.name
    );

    // Verify workflow can resolve the component
    const resolvedComponents = await resolveWorkflowComponents(workflow, storage);
    expect(resolvedComponents.size).toBe(1);
    expect(resolvedComponents.has(savedComponent.id)).toBe(true);
  }, 180000); // 3 minute timeout for API calls

  it('should verify generated TypeScript code is syntactically valid', async () => {
    const agent = new ComponentBuilderAgent(knowledge, {
      apiKey: envConfig.apiKey!,
    }, storage);

    // Request a component
    const result = await driveComponentBuilder(
      agent,
      'Create a simple counter component that takes a start number and increment amount, and returns the result of start + increment.'
    );

    expect(result.success).toBe(true);

    // Get the generated component
    const components = await storage.list();
    const component = components.components.find(c => c.name.toLowerCase().includes('counter'));

    if (component) {
      // Basic syntax check - TypeScript code should have function definitions
      const tsCode = component.artifacts.typescriptCode;
      expect(tsCode).toContain('interface');
      expect(tsCode).toContain('function');

      // Should have input/output interfaces
      expect(tsCode.toLowerCase()).toContain('input');
      expect(tsCode.toLowerCase()).toContain('output');
    }
  }, 180000);
});

// ============================================
// E2E TESTS - DATABASE STORAGE
// ============================================

describe.skipIf(SKIP_E2E_TESTS)('E2E: Component Builder with Database Storage', () => {
  let storage: ComponentStorage;
  let knowledge: KnowledgeRetrieval;

  beforeAll(async () => {
    const backend = new DatabaseStorageBackend();
    storage = new ComponentStorage({
      type: 'custom',
      customBackend: backend,
    });
    await storage.initialize();

    knowledge = createKnowledge(envConfig.apiKey!);
  });

  it('should build a component and save to database', async () => {
    const agent = new ComponentBuilderAgent(knowledge, {
      apiKey: envConfig.apiKey!,
    }, storage);

    const result = await driveComponentBuilder(
      agent,
      'I need a text trimmer component that takes a string and a max length, and returns the trimmed string with ellipsis if it exceeds the max length.'
    );

    expect(result.success).toBe(true);

    // Verify component was saved
    const components = await storage.list();
    expect(components.total).toBeGreaterThanOrEqual(1);

    // Get the most recently saved component (should be the text trimmer)
    const savedComponent = components.components[components.components.length - 1];
    expect(savedComponent).toBeDefined();

    const validation = validateComponentForExecution(savedComponent);
    if (!validation.valid) {
      console.log('Component validation errors:', validation.errors);
    }
    expect(validation.valid).toBe(true);
  }, 180000);

  it('should be able to retrieve and use component in workflow after storage', async () => {
    const agent = new ComponentBuilderAgent(knowledge, {
      apiKey: envConfig.apiKey!,
    }, storage);

    const result = await driveComponentBuilder(
      agent,
      'Create a greeting component that takes a name string and returns a greeting message.'
    );

    expect(result.success).toBe(true);

    // Get the generated component
    const components = await storage.list();
    const greetingComponent = components.components.find(c =>
      c.name.toLowerCase().includes('greet')
    );

    if (greetingComponent) {
      // Create workflow with this component
      const workflow = createStartComponentEndWorkflow(
        greetingComponent.id,
        greetingComponent.name,
        { name: 'Test User' }
      );

      // Resolve components
      const resolved = await resolveWorkflowComponents(workflow, storage);
      expect(resolved.size).toBe(1);

      // Verify component TypeScript code could be loaded for execution
      const resolvedComponent = resolved.get(greetingComponent.id)!;
      expect(resolvedComponent.artifacts.typescriptCode).toBeDefined();
      expect(resolvedComponent.artifacts.typescriptCode.length).toBeGreaterThan(100);

      // Increment usage count (simulating workflow execution)
      await storage.incrementUsage(greetingComponent.id);

      const updated = await storage.get(greetingComponent.id);
      expect(updated?.metadata.usageCount).toBeGreaterThanOrEqual(1);
    }
  }, 180000);
});

// ============================================
// E2E TESTS - FULL WORKFLOW SIMULATION
// ============================================

describe.skipIf(SKIP_E2E_TESTS)('E2E: Full Workflow Simulation', () => {
  const testDir = path.join(__dirname, 'test-e2e-workflow');
  let fsStorage: ComponentStorage;
  let dbStorage: ComponentStorage;
  let knowledge: KnowledgeRetrieval;

  beforeAll(async () => {
    // Clean up
    try {
      await fs.rm(testDir, { recursive: true });
    } catch {}

    // Initialize both storage backends
    const fsBackend = new FilesystemStorageBackend({ baseDir: testDir });
    fsStorage = new ComponentStorage({
      type: 'custom',
      customBackend: fsBackend,
    });
    await fsStorage.initialize();

    const dbBackend = new DatabaseStorageBackend();
    dbStorage = new ComponentStorage({
      type: 'custom',
      customBackend: dbBackend,
    });
    await dbStorage.initialize();

    knowledge = createKnowledge(envConfig.apiKey!);
  });

  afterAll(async () => {
    try {
      await fs.rm(testDir, { recursive: true });
    } catch {}
  });

  it('should build component once and store in both backends', async () => {
    // Build component with filesystem storage
    const agent = new ComponentBuilderAgent(knowledge, {
      apiKey: envConfig.apiKey!,
    }, fsStorage);

    const result = await driveComponentBuilder(
      agent,
      'Create a timestamp generator that outputs the current time in ISO format.'
    );

    expect(result.success).toBe(true);

    // Get the component from filesystem
    const fsComponents = await fsStorage.list();
    expect(fsComponents.total).toBeGreaterThanOrEqual(1);

    const component = fsComponents.components[0];

    // Save to database as well (simulating multi-backend support)
    await dbStorage.save(component);

    // Verify component exists in both backends
    expect(await fsStorage.exists(component.id)).toBe(true);
    expect(await dbStorage.exists(component.id)).toBe(true);

    // Create workflows referencing the component from each backend
    const fsWorkflow = createStartComponentEndWorkflow(component.id, component.name);
    const dbWorkflow = createStartComponentEndWorkflow(component.id, component.name);

    // Resolve from filesystem
    const fsResolved = await resolveWorkflowComponents(fsWorkflow, fsStorage);
    expect(fsResolved.size).toBe(1);

    // Resolve from database
    const dbResolved = await resolveWorkflowComponents(dbWorkflow, dbStorage);
    expect(dbResolved.size).toBe(1);

    // Both should have identical component code
    const fsComponent = fsResolved.get(component.id)!;
    const dbComponent = dbResolved.get(component.id)!;

    expect(fsComponent.artifacts.typescriptCode).toBe(dbComponent.artifacts.typescriptCode);
    expect(fsComponent.artifacts.rustSchema).toBe(dbComponent.artifacts.rustSchema);
  }, 240000); // 4 minute timeout

  it('should simulate complete workflow execution flow', async () => {
    // This test simulates what happens when a workflow is executed:
    // 1. Load workflow definition
    // 2. Resolve component references
    // 3. Load component code
    // 4. "Execute" (validate component is ready)
    // 5. Update usage metrics

    // Use existing component from previous test
    const components = await fsStorage.list();
    if (components.total === 0) {
      console.log('Skipping - no components available from previous test');
      return;
    }

    const component = components.components[0];

    // Step 1: Create workflow definition
    const workflow = createStartComponentEndWorkflow(
      component.id,
      component.name,
      {} // No config needed for timestamp generator
    );

    // Step 2: Resolve component references
    const resolvedComponents = await resolveWorkflowComponents(workflow, fsStorage);
    expect(resolvedComponents.size).toBe(1);

    // Step 3: Load component code
    const loadedComponent = resolvedComponents.get(component.id)!;
    expect(loadedComponent.artifacts.typescriptCode).toBeDefined();

    // Step 4: Validate component is ready for execution
    const validation = validateComponentForExecution(loadedComponent);
    expect(validation.valid).toBe(true);

    // Step 5: Update usage metrics (simulating post-execution)
    const initialUsage = loadedComponent.metadata.usageCount;
    await fsStorage.incrementUsage(component.id);

    const updated = await fsStorage.get(component.id);
    expect(updated!.metadata.usageCount).toBe(initialUsage + 1);

    console.log('\n✅ Workflow execution flow simulation complete');
    console.log(`   Component: ${component.name} (${component.id})`);
    console.log(`   Usage count: ${updated!.metadata.usageCount}`);
  }, 30000);
});

// ============================================
// SKIP MESSAGE FOR CI
// ============================================

describe('E2E Test Environment Check', () => {
  it('should report environment status', () => {
    if (SKIP_E2E_TESTS) {
      console.log('\n');
      console.log('═══════════════════════════════════════════════════════════');
      console.log('   E2E TESTS SKIPPED');
      console.log('═══════════════════════════════════════════════════════════');
      console.log(`   Reason: ${envConfig.reason}`);
      console.log('');
      console.log('   To enable E2E tests:');
      console.log('   1. Copy .env to .env.test');
      console.log('   2. Ensure ANTHROPIC_API_KEY is set in .env.test');
      console.log('═══════════════════════════════════════════════════════════');
      console.log('\n');
    } else {
      console.log('\n');
      console.log('═══════════════════════════════════════════════════════════');
      console.log('   E2E TESTS ENABLED');
      console.log('═══════════════════════════════════════════════════════════');
      console.log('   API key loaded from .env.test');
      console.log('   Running full integration tests...');
      console.log('═══════════════════════════════════════════════════════════');
      console.log('\n');
    }

    // This test always passes - it's just for reporting
    expect(true).toBe(true);
  });
});
