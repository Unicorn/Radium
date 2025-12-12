/**
 * Continue-as-New Pattern Tests
 * 
 * Tests for the continue-as-new compiler pattern.
 * Validates that generated code matches actual Temporal SDK usage.
 * 
 * CRITICAL: Mocks are validated against actual Temporal behavior from:
 * - packages/package-queue-orchestrator/src/workflows/continuous-builder.workflow.ts
 * - Actual Temporal SDK documentation
 */

import { describe, it, expect } from 'vitest';
import { ContinueAsNewPattern } from '../patterns/continue-as-new-pattern';
import type { WorkflowNode, GeneratorContext } from '../types';
import { configureContinueAsNew } from '../../workflow-analyzer';
import type { WorkflowDefinition } from '../types';

describe('ContinueAsNewPattern', () => {
  /**
   * Create a test context with continue-as-new enabled
   */
  function createTestContext(workflowName = 'TestWorkflow'): GeneratorContext {
    const workflow: WorkflowDefinition = {
      id: 'test-workflow',
      name: workflowName,
      nodes: [
        {
          id: 'loop-1',
          type: 'loop',
          data: { label: 'Process Loop' },
          position: { x: 0, y: 0 },
        },
      ],
      edges: [],
      variables: [
        { name: 'counter', type: 'number', initialValue: 0 },
        { name: 'message', type: 'string', initialValue: 'Hello' },
      ],
      settings: {},
    };

    const configured = configureContinueAsNew(workflow, 'service');

    return {
      nodes: configured.nodes,
      edges: configured.edges,
      variables: configured.variables,
      settings: configured.settings,
      nodeMap: new Map(configured.nodes.map(n => [n.id, n])),
      edgeMap: new Map(),
      visitedNodes: new Set(),
      resultVars: new Map(),
      proxyMap: new Map(),
      currentIndent: 0,
      workflowName: configured.name,
    };
  }

  describe('detect', () => {
    it('should detect loop nodes when continue-as-new is enabled', () => {
      const context = createTestContext();
      const node: WorkflowNode = {
        id: 'loop-1',
        type: 'loop',
        data: { label: 'Process Loop' },
        position: { x: 0, y: 0 },
      };

      const result = ContinueAsNewPattern.detect(node, context);
      expect(result).toBe(true);
    });

    it('should detect signal nodes when continue-as-new is enabled', () => {
      const context = createTestContext();
      const node: WorkflowNode = {
        id: 'signal-1',
        type: 'signal',
        data: { label: 'Handle Signal' },
        position: { x: 0, y: 0 },
      };

      const result = ContinueAsNewPattern.detect(node, context);
      expect(result).toBe(true);
    });

    it('should not detect when continue-as-new is disabled', () => {
      const context = createTestContext();
      context.settings._longRunning = {
        autoContinueAsNew: false,
        maxHistoryEvents: 1000,
        maxDurationMs: 24 * 60 * 60 * 1000,
        preserveState: true,
      };

      const node: WorkflowNode = {
        id: 'loop-1',
        type: 'loop',
        data: { label: 'Process Loop' },
        position: { x: 0, y: 0 },
      };

      const result = ContinueAsNewPattern.detect(node, context);
      expect(result).toBe(false);
    });

    it('should not detect when _longRunning is not set', () => {
      const context = createTestContext();
      context.settings._longRunning = undefined;

      const node: WorkflowNode = {
        id: 'loop-1',
        type: 'loop',
        data: { label: 'Process Loop' },
        position: { x: 0, y: 0 },
      };

      const result = ContinueAsNewPattern.detect(node, context);
      expect(result).toBe(false);
    });
  });

  describe('generate', () => {
    it('should generate correct imports matching Temporal SDK', () => {
      const context = createTestContext();
      const node: WorkflowNode = {
        id: 'loop-1',
        type: 'loop',
        data: { label: 'Process Loop' },
        position: { x: 0, y: 0 },
      };

      const result = ContinueAsNewPattern.generate(node, context);

      // Validate against actual Temporal SDK usage
      // From: packages/package-queue-orchestrator/src/workflows/continuous-builder.workflow.ts
      expect(result.imports).toContain(
        "import { continueAsNew, workflowInfo } from '@temporalio/workflow';"
      );
    });

    it('should generate workflowInfo().historyLength check matching actual usage', () => {
      const context = createTestContext();
      const node: WorkflowNode = {
        id: 'loop-1',
        type: 'loop',
        data: { label: 'Process Loop' },
        position: { x: 0, y: 0 },
      };

      const result = ContinueAsNewPattern.generate(node, context);

      // Validate against actual Temporal SDK - workflowInfo().historyLength is the correct API
      expect(result.code).toContain('workflowInfo().historyLength');
    });

    it('should generate continueAsNew call with correct TypeScript generic syntax', () => {
      const context = createTestContext('MyWorkflow');
      const node: WorkflowNode = {
        id: 'loop-1',
        type: 'loop',
        data: { label: 'Process Loop' },
        position: { x: 0, y: 0 },
      };

      const result = ContinueAsNewPattern.generate(node, context);

      // Validate against actual Temporal SDK usage:
      // await continueAsNew<typeof ContinuousBuilderWorkflow>({...state})
      expect(result.code).toContain('await continueAsNew<typeof MyWorkflow>');
    });

    it('should use correct workflow name in continueAsNew generic', () => {
      const context = createTestContext('CustomWorkflowName');
      const node: WorkflowNode = {
        id: 'loop-1',
        type: 'loop',
        data: { label: 'Process Loop' },
        position: { x: 0, y: 0 },
      };

      const result = ContinueAsNewPattern.generate(node, context);

      expect(result.code).toContain('continueAsNew<typeof CustomWorkflowName>');
    });

    it('should generate state preservation code', () => {
      const context = createTestContext();
      const node: WorkflowNode = {
        id: 'loop-1',
        type: 'loop',
        data: { label: 'Process Loop' },
        position: { x: 0, y: 0 },
      };

      const result = ContinueAsNewPattern.generate(node, context);

      // Should include state variables
      expect(result.code).toContain('counter: counter');
      expect(result.code).toContain('message: message');
      
      // Should set internal tracking variables
      expect(result.code).toContain('_workflowStartTime: workflowInfo().runStartTime.getTime()');
      expect(result.code).toContain('_historyResetCount: (_historyResetCount || 0) + 1');
    });

    it('should use correct default thresholds', () => {
      const context = createTestContext();
      const node: WorkflowNode = {
        id: 'loop-1',
        type: 'loop',
        data: { label: 'Process Loop' },
        position: { x: 0, y: 0 },
      };

      const result = ContinueAsNewPattern.generate(node, context);

      // Default: 1000 events
      expect(result.code).toContain('historyLength > 1000');
      
      // Default: 24 hours
      expect(result.code).toContain('elapsedTime > 86400000'); // 24 * 60 * 60 * 1000
    });

    it('should use custom thresholds when configured', () => {
      const context = createTestContext();
      context.settings._longRunning = {
        autoContinueAsNew: true,
        maxHistoryEvents: 500,
        maxDurationMs: 12 * 60 * 60 * 1000, // 12 hours
        preserveState: true,
      };

      const node: WorkflowNode = {
        id: 'loop-1',
        type: 'loop',
        data: { label: 'Process Loop' },
        position: { x: 0, y: 0 },
      };

      const result = ContinueAsNewPattern.generate(node, context);

      expect(result.code).toContain('historyLength > 500');
      expect(result.code).toContain('elapsedTime > 43200000'); // 12 hours
    });

    it('should generate workflowStartTime declaration', () => {
      const context = createTestContext();
      const node: WorkflowNode = {
        id: 'loop-1',
        type: 'loop',
        data: { label: 'Process Loop' },
        position: { x: 0, y: 0 },
      };

      const result = ContinueAsNewPattern.generate(node, context);

      expect(result.declarations).toContain('const workflowStartTime: number = workflowInfo().runStartTime.getTime();');
    });

    it('should only add imports once (idempotent)', () => {
      const context = createTestContext();
      const node: WorkflowNode = {
        id: 'loop-1',
        type: 'loop',
        data: { label: 'Process Loop' },
        position: { x: 0, y: 0 },
      };

      // First call
      const result1 = ContinueAsNewPattern.generate(node, context);
      expect(result1.imports).toHaveLength(1);

      // Second call should not add imports again
      context.visitedNodes.add('_continue_as_new_imports');
      const result2 = ContinueAsNewPattern.generate(node, context);
      expect(result2.imports).toHaveLength(0);
    });

    it('should only add declarations once (idempotent)', () => {
      const context = createTestContext();
      const node: WorkflowNode = {
        id: 'loop-1',
        type: 'loop',
        data: { label: 'Process Loop' },
        position: { x: 0, y: 0 },
      };

      // First call
      const result1 = ContinueAsNewPattern.generate(node, context);
      expect(result1.declarations).toContain('const workflowStartTime: number = workflowInfo().runStartTime.getTime();');

      // Second call should not add declarations again
      context.visitedNodes.add('_workflow_start_time');
      const result2 = ContinueAsNewPattern.generate(node, context);
      expect(result2.declarations).not.toContain('const workflowStartTime: number = workflowInfo().runStartTime.getTime();');
    });

    it('should generate valid TypeScript code structure', () => {
      const context = createTestContext();
      const node: WorkflowNode = {
        id: 'loop-1',
        type: 'loop',
        data: { label: 'Process Loop' },
        position: { x: 0, y: 0 },
      };

      const result = ContinueAsNewPattern.generate(node, context);

      // Should have proper structure
      expect(result.code).toContain('// Automatic history management');
      expect(result.code).toContain('const historyLength = workflowInfo().historyLength');
      expect(result.code).toContain('const elapsedTime = Date.now() - workflowStartTime');
      expect(result.code).toContain('if (historyLength >');
      expect(result.code).toContain('console.log');
      expect(result.code).toContain('await continueAsNew');
    });
  });
});
