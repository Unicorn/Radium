/**
 * Compiler Router Integration Tests
 * Tests for the workflow compiler tRPC endpoints
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import type { WorkflowDefinition } from '@/lib/compiler/types';

/**
 * Test Helper: Create a mock tRPC context
 */
function createMockContext() {
  const mockUser = {
    id: 'user-123',
    email: 'test@example.com',
    display_name: 'Test User',
    auth_user_id: 'auth-123',
    role_id: 'role-123',
    organization_id: 'org-123',
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
    last_seen_at: new Date().toISOString(),
    default_visibility_id: 'vis-123',
    archived_at: null,
  };

  const mockWorkflow = {
    id: 'workflow-123',
    name: 'Test Workflow',
    display_name: 'Test Workflow',
    description: 'A test workflow',
    created_by: 'user-123',
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
    status_id: 'status-active',
    visibility_id: 'vis-public',
    task_queue_id: 'queue-123',
    definition: {
      nodes: [
        {
          id: 'node-1',
          type: 'trigger',
          data: { label: 'Start' },
          position: { x: 0, y: 0 },
        },
        {
          id: 'node-2',
          type: 'activity',
          data: {
            label: 'Process Data',
            componentName: 'processData',
            activityName: 'processData',
          },
          position: { x: 100, y: 0 },
        },
      ],
      edges: [
        {
          id: 'edge-1',
          source: 'node-1',
          target: 'node-2',
        },
      ],
      variables: [],
      settings: {},
    },
    version: '1.0.0',
    compiled_typescript: null,
    temporal_workflow_id: null,
    temporal_workflow_type: null,
    deployed_at: null,
    is_scheduled: false,
    schedule_spec: null,
    last_run_at: null,
    next_run_at: null,
    run_count: 0,
    max_runs: null,
    max_concurrent_executions: null,
    execution_timeout_seconds: null,
    parent_workflow_id: null,
    signal_to_parent_name: null,
    query_parent_name: null,
    start_immediately: false,
    end_with_parent: false,
  };

  const supabaseMock = {
    from: vi.fn((table: string) => ({
      select: vi.fn(() => ({
        eq: vi.fn(() => ({
          eq: vi.fn(() => ({
            single: vi.fn(async () => {
              if (table === 'workflows') {
                return { data: mockWorkflow, error: null };
              }
              return { data: null, error: { message: 'Not found' } };
            }),
          })),
        })),
      })),
      update: vi.fn(() => ({
        eq: vi.fn(() => ({
          eq: vi.fn(async () => ({ data: null, error: null })),
        })),
      })),
    })),
  };

  return {
    supabase: supabaseMock as any,
    user: mockUser,
    authUser: { id: 'auth-123', email: 'test@example.com' } as any,
    getUserRecord: vi.fn(async () => mockUser),
  };
}

/**
 * Test Helper: Create a simple workflow definition
 */
function createTestWorkflowDefinition(): WorkflowDefinition {
  return {
    id: 'test-workflow-1',
    name: 'Test Workflow',
    nodes: [
      {
        id: 'node-1',
        type: 'trigger',
        data: { label: 'Start' },
        position: { x: 0, y: 0 },
      },
      {
        id: 'node-2',
        type: 'activity',
        data: {
          label: 'Process Data',
          componentName: 'processData',
          activityName: 'processData',
        },
        position: { x: 100, y: 0 },
      },
      {
        id: 'node-3',
        type: 'end',
        data: { label: 'End' },
        position: { x: 200, y: 0 },
      },
    ],
    edges: [
      {
        id: 'edge-1',
        source: 'node-1',
        target: 'node-2',
      },
      {
        id: 'edge-2',
        source: 'node-2',
        target: 'node-3',
      },
    ],
    variables: [],
    settings: {
      description: 'A simple test workflow',
      version: '1.0.0',
    },
  };
}

describe('Compiler Router', () => {
  describe('compileDefinition', () => {
    it('should compile a simple workflow definition', async () => {
      const { compilerRouter } = await import('../compiler');
      const { createTRPCContext } = await import('../../trpc');

      const workflowDef = createTestWorkflowDefinition();

      // Mock the context creation
      const ctx = createMockContext();

      // Create a caller with mocked context
      const caller = compilerRouter.createCaller(ctx as any);

      const result = await caller.compileDefinition({
        workflow: workflowDef,
        includeComments: true,
        strictMode: true,
        optimizationLevel: 'basic',
      });

      expect(result.success).toBe(true);
      expect(result.workflowCode).toBeDefined();
      expect(result.workflowCode).toContain('proxyActivities');
      expect(result.activitiesCode).toBeDefined();
      expect(result.workerCode).toBeDefined();
      expect(result.errors).toHaveLength(0);
    });

    it('should handle validation errors in workflow definition', async () => {
      const { compilerRouter } = await import('../compiler');

      const invalidWorkflowDef: WorkflowDefinition = {
        id: 'test-invalid',
        name: '', // Invalid: empty name
        nodes: [],
        edges: [],
        variables: [],
        settings: {},
      };

      const ctx = createMockContext();
      const caller = compilerRouter.createCaller(ctx as any);

      const result = await caller.compileDefinition({
        workflow: invalidWorkflowDef,
      });

      expect(result.success).toBe(false);
      expect(result.errors!.length).toBeGreaterThan(0);
    });

    it('should generate TypeScript code with correct structure', async () => {
      const { compilerRouter } = await import('../compiler');

      const workflowDef = createTestWorkflowDefinition();
      const ctx = createMockContext();
      const caller = compilerRouter.createCaller(ctx as any);

      const result = await caller.compileDefinition({
        workflow: workflowDef,
      });

      expect(result.success).toBe(true);
      expect(result.workflowCode).toBeDefined();

      // Check for required TypeScript elements
      const code = result.workflowCode!;
      expect(code).toContain('import');
      expect(code).toContain('export async function');
      expect(code).toContain('proxyActivities');
    });

    it('should respect compiler options', async () => {
      const { compilerRouter } = await import('../compiler');

      const workflowDef = createTestWorkflowDefinition();
      const ctx = createMockContext();
      const caller = compilerRouter.createCaller(ctx as any);

      // Compile with comments
      const resultWithComments = await caller.compileDefinition({
        workflow: workflowDef,
        includeComments: true,
      });

      // Compile without comments
      const resultWithoutComments = await caller.compileDefinition({
        workflow: workflowDef,
        includeComments: false,
      });

      expect(resultWithComments.success).toBe(true);
      expect(resultWithoutComments.success).toBe(true);

      // Version with comments should have more lines
      const linesWithComments = resultWithComments.workflowCode!.split('\n').length;
      const linesWithoutComments = resultWithoutComments.workflowCode!.split('\n').length;

      expect(linesWithComments).toBeGreaterThanOrEqual(linesWithoutComments);
    });
  });

  describe('validate', () => {
    it('should validate a workflow definition', async () => {
      const { compilerRouter } = await import('../compiler');

      const workflowDef = createTestWorkflowDefinition();
      const ctx = createMockContext();
      const caller = compilerRouter.createCaller(ctx as any);

      const result = await caller.validate({
        workflow: workflowDef,
      });

      expect(result.valid).toBe(true);
      expect(result.errors).toHaveLength(0);
    });

    it('should detect validation errors', async () => {
      const { compilerRouter } = await import('../compiler');

      const invalidWorkflowDef: WorkflowDefinition = {
        id: 'test-invalid',
        name: 'Invalid Workflow',
        nodes: [
          {
            id: 'node-1',
            type: 'activity',
            data: {
              label: 'Activity',
              // Missing required componentName
            },
            position: { x: 0, y: 0 },
          },
        ],
        edges: [],
        variables: [],
        settings: {},
      };

      const ctx = createMockContext();
      const caller = compilerRouter.createCaller(ctx as any);

      const result = await caller.validate({
        workflow: invalidWorkflowDef,
      });

      expect(result.valid).toBe(false);
      expect(result.errors.length).toBeGreaterThan(0);
    });

    it('should provide helpful error messages', async () => {
      const { compilerRouter } = await import('../compiler');

      const workflowDef: WorkflowDefinition = {
        id: 'test-workflow',
        name: 'Test Workflow',
        nodes: [
          {
            id: 'node-1',
            type: 'trigger',
            data: { label: 'Start' },
            position: { x: 0, y: 0 },
          },
        ],
        edges: [
          {
            id: 'edge-1',
            source: 'node-1',
            target: 'node-999', // Non-existent target
          },
        ],
        variables: [],
        settings: {},
      };

      const ctx = createMockContext();
      const caller = compilerRouter.createCaller(ctx as any);

      const result = await caller.validate({
        workflow: workflowDef,
      });

      expect(result.valid).toBe(false);
      expect(result.errors.length).toBeGreaterThan(0);
      expect(result.errors[0].message).toContain('node-999');
    });
  });

  describe('getMetadata', () => {
    it('should return compiler metadata', async () => {
      const { compilerRouter } = await import('../compiler');

      const ctx = createMockContext();
      const caller = compilerRouter.createCaller(ctx as any);

      const metadata = await caller.getMetadata();

      expect(metadata.version).toBeDefined();
      expect(metadata.supportedNodeTypes).toBeInstanceOf(Array);
      expect(metadata.supportedNodeTypes).toContain('trigger');
      expect(metadata.supportedNodeTypes).toContain('activity');
      expect(metadata.supportedNodeTypes).toContain('agent');
      expect(metadata.optimizationLevels).toContain('basic');
      expect(metadata.features).toContain('pattern-based-compilation');
    });
  });

  describe('Error Handling', () => {
    it('should handle missing workflow gracefully', async () => {
      const { compilerRouter } = await import('../compiler');

      const ctx = createMockContext();

      // Override the mock to return null
      ctx.supabase.from = vi.fn(() => ({
        select: vi.fn(() => ({
          eq: vi.fn(() => ({
            eq: vi.fn(() => ({
              single: vi.fn(async () => ({ data: null, error: { message: 'Not found' } })),
            })),
          })),
        })),
      })) as any;

      const caller = compilerRouter.createCaller(ctx as any);

      await expect(
        caller.compile({ workflowId: 'non-existent' })
      ).rejects.toThrow('Workflow not found');
    });

    it('should handle compilation errors gracefully', async () => {
      const { compilerRouter } = await import('../compiler');

      // Create a workflow that will fail compilation
      const invalidWorkflowDef: WorkflowDefinition = {
        id: 'invalid',
        name: '',
        nodes: [],
        edges: [],
        variables: [],
        settings: {},
      };

      const ctx = createMockContext();
      const caller = compilerRouter.createCaller(ctx as any);

      const result = await caller.compileDefinition({
        workflow: invalidWorkflowDef,
      });

      expect(result.success).toBe(false);
      expect(result.errors).toBeDefined();
      expect(result.errors!.length).toBeGreaterThan(0);
    });
  });

  describe('Workflow Patterns', () => {
    it('should compile workflow with multiple node types', async () => {
      const { compilerRouter } = await import('../compiler');

      const complexWorkflow: WorkflowDefinition = {
        id: 'complex-workflow',
        name: 'Complex Workflow',
        nodes: [
          {
            id: 'trigger-1',
            type: 'trigger',
            data: { label: 'Start' },
            position: { x: 0, y: 0 },
          },
          {
            id: 'activity-1',
            type: 'activity',
            data: {
              label: 'Process',
              componentName: 'processActivity',
            },
            position: { x: 100, y: 0 },
          },
          {
            id: 'condition-1',
            type: 'condition',
            data: {
              label: 'Check Result',
              config: { condition: 'result.success' },
            },
            position: { x: 200, y: 0 },
          },
          {
            id: 'end-1',
            type: 'end',
            data: { label: 'End' },
            position: { x: 300, y: 0 },
          },
        ],
        edges: [
          { id: 'e1', source: 'trigger-1', target: 'activity-1' },
          { id: 'e2', source: 'activity-1', target: 'condition-1' },
          { id: 'e3', source: 'condition-1', target: 'end-1' },
        ],
        variables: [],
        settings: {},
      };

      const ctx = createMockContext();
      const caller = compilerRouter.createCaller(ctx as any);

      const result = await caller.compileDefinition({
        workflow: complexWorkflow,
      });

      expect(result.success).toBe(true);
      expect(result.workflowCode).toBeDefined();
    });

    it('should compile workflow with retry policies', async () => {
      const { compilerRouter } = await import('../compiler');

      const workflowWithRetry: WorkflowDefinition = {
        id: 'retry-workflow',
        name: 'Retry Workflow',
        nodes: [
          {
            id: 'trigger-1',
            type: 'trigger',
            data: { label: 'Start' },
            position: { x: 0, y: 0 },
          },
          {
            id: 'activity-1',
            type: 'activity',
            data: {
              label: 'Flaky Activity',
              componentName: 'flakyActivity',
              retryPolicy: {
                strategy: 'exponential-backoff',
                maxAttempts: 3,
                initialInterval: '1s',
                maxInterval: '60s',
                backoffCoefficient: 2,
              },
            },
            position: { x: 100, y: 0 },
          },
        ],
        edges: [
          { id: 'e1', source: 'trigger-1', target: 'activity-1' },
        ],
        variables: [],
        settings: {},
      };

      const ctx = createMockContext();
      const caller = compilerRouter.createCaller(ctx as any);

      const result = await caller.compileDefinition({
        workflow: workflowWithRetry,
      });

      expect(result.success).toBe(true);
      expect(result.workflowCode).toBeDefined();
      expect(result.workflowCode).toContain('retry');
    });
  });

  describe('Performance', () => {
    it('should compile large workflows efficiently', async () => {
      const { compilerRouter } = await import('../compiler');

      // Create a workflow with many nodes
      const nodes: any[] = [
        {
          id: 'trigger-1',
          type: 'trigger',
          data: { label: 'Start' },
          position: { x: 0, y: 0 },
        },
      ];

      const edges: any[] = [];

      // Add 50 activity nodes
      for (let i = 1; i <= 50; i++) {
        nodes.push({
          id: `activity-${i}`,
          type: 'activity',
          data: {
            label: `Activity ${i}`,
            componentName: `activity${i}`,
          },
          position: { x: i * 100, y: 0 },
        });

        edges.push({
          id: `edge-${i}`,
          source: i === 1 ? 'trigger-1' : `activity-${i - 1}`,
          target: `activity-${i}`,
        });
      }

      const largeWorkflow: WorkflowDefinition = {
        id: 'large-workflow',
        name: 'Large Workflow',
        nodes,
        edges,
        variables: [],
        settings: {},
      };

      const ctx = createMockContext();
      const caller = compilerRouter.createCaller(ctx as any);

      const startTime = Date.now();
      const result = await caller.compileDefinition({
        workflow: largeWorkflow,
      });
      const endTime = Date.now();

      expect(result.success).toBe(true);
      expect(result.metadata?.nodeCount).toBe(51); // 1 trigger + 50 activities

      // Compilation should complete in reasonable time (< 5 seconds)
      expect(endTime - startTime).toBeLessThan(5000);
    });
  });
});
