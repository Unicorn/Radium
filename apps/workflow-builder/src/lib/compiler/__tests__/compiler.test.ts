/**
 * Workflow Compiler Tests
 * Comprehensive unit tests for the pattern-based workflow compiler
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { WorkflowCompiler } from '../index';
import type { WorkflowDefinition } from '../types';

describe('WorkflowCompiler', () => {
  let compiler: WorkflowCompiler;

  beforeEach(() => {
    compiler = new WorkflowCompiler({
      includeComments: true,
      strictMode: true,
    });
  });

  describe('Validation', () => {
    it('should validate workflow structure', () => {
      const invalidWorkflow: WorkflowDefinition = {
        id: '',
        name: '',
        nodes: [],
        edges: [],
        variables: [],
        settings: {},
      };

      const result = compiler.compile(invalidWorkflow);

      expect(result.success).toBe(false);
      expect(result.errors).toBeDefined();
      expect(result.errors!.length).toBeGreaterThan(0);
    });

    it('should detect missing required fields', () => {
      const workflow: WorkflowDefinition = {
        id: 'test-workflow',
        name: '', // Missing name
        nodes: [
          {
            id: 'node-1',
            type: 'trigger',
            data: { label: 'Start' },
            position: { x: 0, y: 0 },
          },
        ],
        edges: [],
        variables: [],
        settings: {},
      };

      const result = compiler.compile(workflow);

      expect(result.success).toBe(false);
      expect(result.errors?.some(e => e.message.includes('name'))).toBe(true);
    });

    it('should require at least one node', () => {
      const workflow: WorkflowDefinition = {
        id: 'test-workflow',
        name: 'Test Workflow',
        nodes: [],
        edges: [],
        variables: [],
        settings: {},
      };

      const result = compiler.compile(workflow);

      expect(result.success).toBe(false);
      expect(result.errors?.some(e => e.message.includes('at least one node'))).toBe(true);
    });

    it('should validate activity nodes have componentName', () => {
      const workflow: WorkflowDefinition = {
        id: 'test-workflow',
        name: 'Test Workflow',
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
            data: { label: 'Activity' }, // Missing componentName
            position: { x: 100, y: 0 },
          },
        ],
        edges: [
          {
            id: 'edge-1',
            source: 'trigger-1',
            target: 'activity-1',
          },
        ],
        variables: [],
        settings: {},
      };

      const result = compiler.compile(workflow);

      expect(result.success).toBe(false);
      expect(result.errors?.some(e => e.nodeId === 'activity-1')).toBe(true);
    });
  });

  describe('Simple Workflow Compilation', () => {
    it('should compile a workflow with one activity', () => {
      const workflow: WorkflowDefinition = {
        id: 'simple-workflow',
        name: 'SimpleWorkflow',
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
              label: 'Process Data',
              componentName: 'processData',
            },
            position: { x: 100, y: 0 },
          },
        ],
        edges: [
          {
            id: 'edge-1',
            source: 'trigger-1',
            target: 'activity-1',
          },
        ],
        variables: [],
        settings: {
          timeout: '10 minutes',
          taskQueue: 'default',
        },
      };

      const result = compiler.compile(workflow);

      expect(result.success).toBe(true);
      expect(result.workflowCode).toBeDefined();
      expect(result.activitiesCode).toBeDefined();
      expect(result.workerCode).toBeDefined();
      expect(result.packageJson).toBeDefined();
      expect(result.tsConfig).toBeDefined();
    });

    it('should generate correct TypeScript code', () => {
      const workflow: WorkflowDefinition = {
        id: 'test-workflow',
        name: 'TestWorkflow',
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
              label: 'Test Activity',
              componentName: 'testActivity',
            },
            position: { x: 100, y: 0 },
          },
        ],
        edges: [
          {
            id: 'edge-1',
            source: 'trigger-1',
            target: 'activity-1',
          },
        ],
        variables: [],
        settings: {},
      };

      const result = compiler.compile(workflow);

      expect(result.success).toBe(true);
      expect(result.workflowCode).toContain('export async function testWorkflowWorkflow');
      expect(result.workflowCode).toContain('proxyActivities');
      expect(result.workflowCode).toContain('testActivity');
    });

    it('should include imports in workflow code', () => {
      const workflow: WorkflowDefinition = {
        id: 'test-workflow',
        name: 'TestWorkflow',
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
              label: 'Activity',
              componentName: 'myActivity',
            },
            position: { x: 100, y: 0 },
          },
        ],
        edges: [
          {
            id: 'edge-1',
            source: 'trigger-1',
            target: 'activity-1',
          },
        ],
        variables: [],
        settings: {},
      };

      const result = compiler.compile(workflow);

      expect(result.success).toBe(true);
      expect(result.workflowCode).toContain("import { proxyActivities } from '@temporalio/workflow'");
      expect(result.workflowCode).toContain("import type * as activities from './activities'");
    });
  });

  describe('State Variables', () => {
    it('should handle state variable nodes', () => {
      const workflow: WorkflowDefinition = {
        id: 'state-workflow',
        name: 'StateWorkflow',
        nodes: [
          {
            id: 'trigger-1',
            type: 'trigger',
            data: { label: 'Start' },
            position: { x: 0, y: 0 },
          },
          {
            id: 'state-1',
            type: 'state-variable',
            data: {
              label: 'Counter',
              config: {
                name: 'counter',
                operation: 'set',
                initialValue: 0,
              },
            },
            position: { x: 100, y: 0 },
          },
        ],
        edges: [
          {
            id: 'edge-1',
            source: 'trigger-1',
            target: 'state-1',
          },
        ],
        variables: [],
        settings: {},
      };

      const result = compiler.compile(workflow);

      expect(result.success).toBe(true);
      expect(result.workflowCode).toContain('let counter = 0');
    });

    it('should generate state variable operations', () => {
      const workflow: WorkflowDefinition = {
        id: 'state-workflow',
        name: 'StateWorkflow',
        nodes: [
          {
            id: 'trigger-1',
            type: 'trigger',
            data: { label: 'Start' },
            position: { x: 0, y: 0 },
          },
          {
            id: 'state-1',
            type: 'state-variable',
            data: {
              label: 'Initialize',
              config: {
                name: 'items',
                operation: 'set',
                initialValue: [],
              },
            },
            position: { x: 100, y: 0 },
          },
          {
            id: 'state-2',
            type: 'state-variable',
            data: {
              label: 'Append',
              config: {
                name: 'items',
                operation: 'append',
                value: 'item1',
              },
            },
            position: { x: 200, y: 0 },
          },
        ],
        edges: [
          {
            id: 'edge-1',
            source: 'trigger-1',
            target: 'state-1',
          },
          {
            id: 'edge-2',
            source: 'state-1',
            target: 'state-2',
          },
        ],
        variables: [],
        settings: {},
      };

      const result = compiler.compile(workflow);

      expect(result.success).toBe(true);
      expect(result.workflowCode).toContain('let items = []');
      expect(result.workflowCode).toContain('items.push');
    });
  });

  describe('Error Handling', () => {
    it('should handle missing fields gracefully', () => {
      const workflow: any = {
        id: 'test',
        name: 'Test',
        // Missing nodes
      };

      const result = compiler.compile(workflow);

      expect(result.success).toBe(false);
      expect(result.errors).toBeDefined();
    });

    it('should detect circular dependencies', () => {
      const workflow: WorkflowDefinition = {
        id: 'circular-workflow',
        name: 'CircularWorkflow',
        nodes: [
          {
            id: 'node-1',
            type: 'activity',
            data: { label: 'A', componentName: 'activityA' },
            position: { x: 0, y: 0 },
          },
          {
            id: 'node-2',
            type: 'activity',
            data: { label: 'B', componentName: 'activityB' },
            position: { x: 100, y: 0 },
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
            target: 'node-1',
          },
        ],
        variables: [],
        settings: {},
      };

      const result = compiler.compile(workflow);

      // Should compile but may have warnings about circular dependencies
      expect(result.warnings).toBeDefined();
    });

    it('should detect orphaned nodes', () => {
      const workflow: WorkflowDefinition = {
        id: 'orphan-workflow',
        name: 'OrphanWorkflow',
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
            data: { label: 'Connected', componentName: 'connectedActivity' },
            position: { x: 100, y: 0 },
          },
          {
            id: 'activity-2',
            type: 'activity',
            data: { label: 'Orphan', componentName: 'orphanActivity' },
            position: { x: 200, y: 200 },
          },
        ],
        edges: [
          {
            id: 'edge-1',
            source: 'trigger-1',
            target: 'activity-1',
          },
        ],
        variables: [],
        settings: {},
      };

      const result = compiler.compile(workflow);

      expect(result.warnings?.some(w => w.message.includes('orphaned'))).toBe(true);
    });
  });

  describe('Pattern Application', () => {
    it('should apply activity proxy pattern', () => {
      const workflow: WorkflowDefinition = {
        id: 'pattern-test',
        name: 'PatternTest',
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
              label: 'Test',
              componentName: 'testActivity',
            },
            position: { x: 100, y: 0 },
          },
        ],
        edges: [
          {
            id: 'edge-1',
            source: 'trigger-1',
            target: 'activity-1',
          },
        ],
        variables: [],
        settings: {},
      };

      const result = compiler.compile(workflow);

      expect(result.success).toBe(true);
      expect(result.metadata?.patternsApplied).toContain('activity-1');
    });

    it('should register custom patterns', () => {
      const customPattern = {
        name: 'custom-pattern',
        priority: 50,
        detect: () => false,
        generate: () => ({ code: '' }),
      };

      compiler.registerPattern(customPattern);

      const workflow: WorkflowDefinition = {
        id: 'test',
        name: 'Test',
        nodes: [
          {
            id: 'trigger-1',
            type: 'trigger',
            data: { label: 'Start' },
            position: { x: 0, y: 0 },
          },
        ],
        edges: [],
        variables: [],
        settings: {},
      };

      const result = compiler.compile(workflow);

      expect(result.success).toBe(true);
    });
  });

  describe('Code Generation', () => {
    it('should generate package.json', () => {
      const workflow: WorkflowDefinition = {
        id: 'test-workflow',
        name: 'TestWorkflow',
        nodes: [
          {
            id: 'trigger-1',
            type: 'trigger',
            data: { label: 'Start' },
            position: { x: 0, y: 0 },
          },
        ],
        edges: [],
        variables: [],
        settings: {
          version: '2.0.0',
          description: 'Test workflow description',
        },
      };

      const result = compiler.compile(workflow);

      expect(result.success).toBe(true);
      expect(result.packageJson).toBeDefined();

      const pkg = JSON.parse(result.packageJson!);
      expect(pkg.name).toBeDefined();
      expect(pkg.version).toBe('2.0.0');
      expect(pkg.description).toContain('Test workflow description');
    });

    it('should generate tsconfig.json', () => {
      const workflow: WorkflowDefinition = {
        id: 'test-workflow',
        name: 'TestWorkflow',
        nodes: [
          {
            id: 'trigger-1',
            type: 'trigger',
            data: { label: 'Start' },
            position: { x: 0, y: 0 },
          },
        ],
        edges: [],
        variables: [],
        settings: {},
      };

      const result = compiler.compile(workflow);

      expect(result.success).toBe(true);
      expect(result.tsConfig).toBeDefined();

      const tsConfig = JSON.parse(result.tsConfig!);
      expect(tsConfig.compilerOptions.strict).toBe(true);
    });

    it('should generate worker code', () => {
      const workflow: WorkflowDefinition = {
        id: 'test-workflow',
        name: 'TestWorkflow',
        nodes: [
          {
            id: 'trigger-1',
            type: 'trigger',
            data: { label: 'Start' },
            position: { x: 0, y: 0 },
          },
        ],
        edges: [],
        variables: [],
        settings: {
          taskQueue: 'custom-queue',
        },
      };

      const result = compiler.compile(workflow);

      expect(result.success).toBe(true);
      expect(result.workerCode).toContain('Worker.create');
      expect(result.workerCode).toContain("taskQueue:");
      expect(result.workerCode).toContain("'custom-queue'");
    });
  });

  describe('Compilation Metadata', () => {
    it('should return compilation metadata', () => {
      const workflow: WorkflowDefinition = {
        id: 'test-workflow',
        name: 'TestWorkflow',
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
            data: { label: 'Activity', componentName: 'myActivity' },
            position: { x: 100, y: 0 },
          },
        ],
        edges: [
          {
            id: 'edge-1',
            source: 'trigger-1',
            target: 'activity-1',
          },
        ],
        variables: [],
        settings: {},
      };

      const result = compiler.compile(workflow);

      expect(result.metadata).toBeDefined();
      expect(result.metadata!.nodeCount).toBe(2);
      expect(result.metadata!.edgeCount).toBe(1);
      expect(result.metadata!.compilationTime).toBeGreaterThanOrEqual(0);
      expect(result.metadata!.version).toBe('1.0.0');
    });
  });
});
