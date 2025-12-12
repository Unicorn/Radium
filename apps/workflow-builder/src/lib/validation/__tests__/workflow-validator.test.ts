/**
 * Tests for Enhanced Workflow Validator
 */

import { validateWorkflow } from '../workflow-validator';
import type { WorkflowDefinition } from '../../compiler/types';

describe('Workflow Validator', () => {
  describe('Basic Structure Validation', () => {
    it('should require workflow ID', () => {
      const workflow: any = {
        name: 'Test Workflow',
        nodes: [],
        edges: [],
      };

      const result = validateWorkflow(workflow);

      expect(result.valid).toBe(false);
      const idError = result.errors.find((e) => e.field === 'id');
      expect(idError).toBeDefined();
      expect(idError?.message).toContain('Workflow ID is required');
    });

    it('should require workflow name', () => {
      const workflow: any = {
        id: 'wf-1',
        nodes: [],
        edges: [],
      };

      const result = validateWorkflow(workflow);

      expect(result.valid).toBe(false);
      const nameError = result.errors.find((e) => e.field === 'name');
      expect(nameError).toBeDefined();
      expect(nameError?.message).toContain('Workflow name is required');
    });

    it('should require nodes to be an array', () => {
      const workflow: any = {
        id: 'wf-1',
        name: 'Test',
        nodes: 'not-an-array',
      };

      const result = validateWorkflow(workflow);

      expect(result.valid).toBe(false);
      const nodesError = result.errors.find((e) => e.field === 'nodes');
      expect(nodesError).toBeDefined();
      expect(nodesError?.message).toContain('must be an array');
    });

    it('should accept valid basic structure', () => {
      const workflow: WorkflowDefinition = {
        id: 'wf-1',
        name: 'Test Workflow',
        nodes: [
          {
            id: 'trigger-1',
            type: 'trigger',
            data: { label: 'Start' },
            position: { x: 0, y: 0 },
          },
        ],
        edges: [],
      };

      const result = validateWorkflow(workflow);

      expect(result.valid).toBe(true);
      expect(result.errors).toHaveLength(0);
    });
  });

  describe('Node Validation', () => {
    it('should detect duplicate node IDs', () => {
      const workflow: WorkflowDefinition = {
        id: 'wf-1',
        name: 'Test',
        nodes: [
          {
            id: 'node-1',
            type: 'activity',
            data: { label: 'Activity 1', componentId: 'comp-1' },
            position: { x: 0, y: 0 },
          },
          {
            id: 'node-1',
            type: 'activity',
            data: { label: 'Activity 2', componentId: 'comp-2' },
            position: { x: 100, y: 0 },
          },
        ],
        edges: [],
      };

      const result = validateWorkflow(workflow);

      expect(result.valid).toBe(false);
      const duplicateError = result.errors.find((e) =>
        e.message.includes('Duplicate node ID')
      );
      expect(duplicateError).toBeDefined();
      expect(duplicateError?.invalidValue).toBe('node-1');
    });

    it('should require node type', () => {
      const workflow: WorkflowDefinition = {
        id: 'wf-1',
        name: 'Test',
        nodes: [
          {
            id: 'node-1',
            type: '' as any,
            data: { label: 'Test' },
            position: { x: 0, y: 0 },
          },
        ],
        edges: [],
      };

      const result = validateWorkflow(workflow);

      expect(result.valid).toBe(false);
      const typeError = result.errors.find((e) => e.message.includes('missing a type'));
      expect(typeError).toBeDefined();
    });

    it('should require node data', () => {
      const workflow: WorkflowDefinition = {
        id: 'wf-1',
        name: 'Test',
        nodes: [
          {
            id: 'node-1',
            type: 'activity',
            data: null as any,
            position: { x: 0, y: 0 },
          },
        ],
        edges: [],
      };

      const result = validateWorkflow(workflow);

      expect(result.valid).toBe(false);
      const dataError = result.errors.find((e) =>
        e.message.includes('missing data configuration')
      );
      expect(dataError).toBeDefined();
    });

    it('should warn about missing labels', () => {
      const workflow: WorkflowDefinition = {
        id: 'wf-1',
        name: 'Test',
        nodes: [
          {
            id: 'trigger-1',
            type: 'trigger',
            data: { label: '' },
            position: { x: 0, y: 0 },
          },
        ],
        edges: [],
      };

      const result = validateWorkflow(workflow);

      const labelWarning = result.warnings.find((w) => w.message.includes('no label'));
      expect(labelWarning).toBeDefined();
      expect(labelWarning?.severity).toBe('low');
    });

    it('should warn about duplicate labels', () => {
      const workflow: WorkflowDefinition = {
        id: 'wf-1',
        name: 'Test',
        nodes: [
          {
            id: 'trigger-1',
            type: 'trigger',
            data: { label: 'Start' },
            position: { x: 0, y: 0 },
          },
          {
            id: 'node-1',
            type: 'activity',
            data: { label: 'Start', componentId: 'comp-1' },
            position: { x: 100, y: 0 },
          },
        ],
        edges: [],
      };

      const result = validateWorkflow(workflow);

      const duplicateLabelWarning = result.warnings.find((w) =>
        w.message.includes('same label')
      );
      expect(duplicateLabelWarning).toBeDefined();
    });
  });

  describe('Node Type-Specific Validation', () => {
    it('should require component for activity nodes', () => {
      const workflow: WorkflowDefinition = {
        id: 'wf-1',
        name: 'Test',
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
            data: { label: 'Do Something' },
            position: { x: 100, y: 0 },
          },
        ],
        edges: [],
      };

      const result = validateWorkflow(workflow);

      expect(result.valid).toBe(false);
      const componentError = result.errors.find((e) =>
        e.message.includes('must reference a component')
      );
      expect(componentError).toBeDefined();
      expect(componentError?.details?.type).toBe('activity');
    });

    it('should require signal name for signal nodes', () => {
      const workflow: WorkflowDefinition = {
        id: 'wf-1',
        name: 'Test',
        nodes: [
          {
            id: 'signal-1',
            type: 'signal',
            data: { label: 'Wait for Approval' },
            position: { x: 0, y: 0 },
          },
        ],
        edges: [],
      };

      const result = validateWorkflow(workflow);

      expect(result.valid).toBe(false);
      const signalError = result.errors.find((e) =>
        e.message.includes('must have a signal name')
      );
      expect(signalError).toBeDefined();
    });

    it('should require workflow type for child workflow nodes', () => {
      const workflow: WorkflowDefinition = {
        id: 'wf-1',
        name: 'Test',
        nodes: [
          {
            id: 'child-1',
            type: 'child-workflow',
            data: { label: 'Run Child Workflow' },
            position: { x: 0, y: 0 },
          },
        ],
        edges: [],
      };

      const result = validateWorkflow(workflow);

      expect(result.valid).toBe(false);
      const workflowTypeError = result.errors.find((e) =>
        e.message.includes('must reference a workflow')
      );
      expect(workflowTypeError).toBeDefined();
    });

    it('should require variable name for state variable nodes', () => {
      const workflow: WorkflowDefinition = {
        id: 'wf-1',
        name: 'Test',
        nodes: [
          {
            id: 'state-1',
            type: 'state-variable',
            data: { label: 'Set Variable', config: {} },
            position: { x: 0, y: 0 },
          },
        ],
        edges: [],
      };

      const result = validateWorkflow(workflow);

      expect(result.valid).toBe(false);
      const varNameError = result.errors.find((e) =>
        e.message.includes('must have a variable name')
      );
      expect(varNameError).toBeDefined();
    });

    it('should require endpoint path for API endpoint nodes', () => {
      const workflow: WorkflowDefinition = {
        id: 'wf-1',
        name: 'Test',
        nodes: [
          {
            id: 'api-1',
            type: 'api-endpoint',
            data: { label: 'API Trigger', config: {} },
            position: { x: 0, y: 0 },
          },
        ],
        edges: [],
      };

      const result = validateWorkflow(workflow);

      expect(result.valid).toBe(false);
      const pathError = result.errors.find((e) =>
        e.message.includes('must have an endpoint path')
      );
      expect(pathError).toBeDefined();
    });

    it('should require condition for condition nodes', () => {
      const workflow: WorkflowDefinition = {
        id: 'wf-1',
        name: 'Test',
        nodes: [
          {
            id: 'condition-1',
            type: 'condition',
            data: { label: 'Check Status', config: {} },
            position: { x: 0, y: 0 },
          },
        ],
        edges: [],
      };

      const result = validateWorkflow(workflow);

      expect(result.valid).toBe(false);
      const conditionError = result.errors.find((e) =>
        e.message.includes('must have a condition expression')
      );
      expect(conditionError).toBeDefined();
    });
  });

  describe('Edge Validation', () => {
    it('should validate source node exists', () => {
      const workflow: WorkflowDefinition = {
        id: 'wf-1',
        name: 'Test',
        nodes: [
          {
            id: 'node-1',
            type: 'activity',
            data: { label: 'Activity', componentId: 'comp-1' },
            position: { x: 0, y: 0 },
          },
        ],
        edges: [
          {
            id: 'edge-1',
            source: 'non-existent',
            target: 'node-1',
          },
        ],
      };

      const result = validateWorkflow(workflow);

      expect(result.valid).toBe(false);
      const sourceError = result.errors.find((e) =>
        e.message.includes('non-existent source node')
      );
      expect(sourceError).toBeDefined();
    });

    it('should validate target node exists', () => {
      const workflow: WorkflowDefinition = {
        id: 'wf-1',
        name: 'Test',
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
            target: 'non-existent',
          },
        ],
      };

      const result = validateWorkflow(workflow);

      expect(result.valid).toBe(false);
      const targetError = result.errors.find((e) =>
        e.message.includes('non-existent target node')
      );
      expect(targetError).toBeDefined();
    });

    it('should detect self-loops', () => {
      const workflow: WorkflowDefinition = {
        id: 'wf-1',
        name: 'Test',
        nodes: [
          {
            id: 'node-1',
            type: 'activity',
            data: { label: 'Activity', componentId: 'comp-1' },
            position: { x: 0, y: 0 },
          },
        ],
        edges: [
          {
            id: 'edge-1',
            source: 'node-1',
            target: 'node-1',
          },
        ],
      };

      const result = validateWorkflow(workflow);

      expect(result.valid).toBe(false);
      const selfLoopError = result.errors.find((e) => e.message.includes('self-loop'));
      expect(selfLoopError).toBeDefined();
    });

    it('should detect duplicate edge IDs', () => {
      const workflow: WorkflowDefinition = {
        id: 'wf-1',
        name: 'Test',
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
            data: { label: 'Activity', componentId: 'comp-1' },
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
            id: 'edge-1',
            source: 'node-1',
            target: 'node-2',
          },
        ],
      };

      const result = validateWorkflow(workflow);

      expect(result.valid).toBe(false);
      const duplicateEdgeError = result.errors.find((e) =>
        e.message.includes('Duplicate edge ID')
      );
      expect(duplicateEdgeError).toBeDefined();
    });
  });

  describe('Workflow Flow Validation', () => {
    it('should require a start node', () => {
      const workflow: WorkflowDefinition = {
        id: 'wf-1',
        name: 'Test',
        nodes: [
          {
            id: 'node-1',
            type: 'activity',
            data: { label: 'Activity', componentId: 'comp-1' },
            position: { x: 0, y: 0 },
          },
        ],
        edges: [],
      };

      const result = validateWorkflow(workflow);

      expect(result.valid).toBe(false);
      const startNodeError = result.errors.find((e) =>
        e.message.includes('must have a trigger')
      );
      expect(startNodeError).toBeDefined();
    });

    it('should detect orphaned nodes', () => {
      const workflow: WorkflowDefinition = {
        id: 'wf-1',
        name: 'Test',
        nodes: [
          {
            id: 'trigger-1',
            type: 'trigger',
            data: { label: 'Start' },
            position: { x: 0, y: 0 },
          },
          {
            id: 'orphan-1',
            type: 'activity',
            data: { label: 'Orphaned', componentId: 'comp-1' },
            position: { x: 100, y: 100 },
          },
        ],
        edges: [],
      };

      const result = validateWorkflow(workflow);

      const orphanWarning = result.warnings.find((w) =>
        w.message.includes('disconnected')
      );
      expect(orphanWarning).toBeDefined();
      expect(orphanWarning?.severity).toBe('high');
    });

    it('should detect circular dependencies', () => {
      const workflow: WorkflowDefinition = {
        id: 'wf-1',
        name: 'Test',
        nodes: [
          {
            id: 'trigger-1',
            type: 'trigger',
            data: { label: 'Start' },
            position: { x: 0, y: 0 },
          },
          {
            id: 'node-1',
            type: 'activity',
            data: { label: 'Activity 1', componentId: 'comp-1' },
            position: { x: 100, y: 0 },
          },
          {
            id: 'node-2',
            type: 'activity',
            data: { label: 'Activity 2', componentId: 'comp-2' },
            position: { x: 200, y: 0 },
          },
        ],
        edges: [
          { id: 'edge-1', source: 'trigger-1', target: 'node-1' },
          { id: 'edge-2', source: 'node-1', target: 'node-2' },
          { id: 'edge-3', source: 'node-2', target: 'node-1' }, // Creates cycle
        ],
      };

      const result = validateWorkflow(workflow);

      const circularWarning = result.warnings.find((w) =>
        w.message.includes('circular dependency')
      );
      expect(circularWarning).toBeDefined();
      expect(circularWarning?.severity).toBe('high');
    });
  });

  describe('Metadata', () => {
    it('should provide validation metadata', () => {
      const workflow: WorkflowDefinition = {
        id: 'wf-1',
        name: 'Test',
        nodes: [
          {
            id: 'trigger-1',
            type: 'trigger',
            data: { label: 'Start' },
            position: { x: 0, y: 0 },
          },
          {
            id: 'end-1',
            type: 'end',
            data: { label: 'End' },
            position: { x: 200, y: 0 },
          },
        ],
        edges: [
          { id: 'edge-1', source: 'trigger-1', target: 'end-1' },
        ],
      };

      const result = validateWorkflow(workflow);

      expect(result.metadata).toBeDefined();
      expect(result.metadata?.nodeCount).toBe(2);
      expect(result.metadata?.edgeCount).toBe(1);
      expect(result.metadata?.hasStartNode).toBe(true);
      expect(result.metadata?.hasEndNode).toBe(true);
      expect(result.metadata?.orphanedNodeCount).toBe(0);
    });
  });

  describe('Retry Policy Validation', () => {
    it('should require strategy', () => {
      const workflow: WorkflowDefinition = {
        id: 'wf-1',
        name: 'Test',
        settings: {
          retryPolicy: {} as any,
        },
        nodes: [
          {
            id: 'trigger-1',
            type: 'trigger',
            data: { label: 'Start' },
            position: { x: 0, y: 0 },
          },
        ],
        edges: [],
      };

      const result = validateWorkflow(workflow);

      expect(result.valid).toBe(false);
      const strategyError = result.errors.find((e) =>
        e.message.includes('must have a strategy')
      );
      expect(strategyError).toBeDefined();
    });

    it('should validate fail-after-x requires maxAttempts', () => {
      const workflow: WorkflowDefinition = {
        id: 'wf-1',
        name: 'Test',
        settings: {
          retryPolicy: {
            strategy: 'fail-after-x',
          } as any,
        },
        nodes: [
          {
            id: 'trigger-1',
            type: 'trigger',
            data: { label: 'Start' },
            position: { x: 0, y: 0 },
          },
        ],
        edges: [],
      };

      const result = validateWorkflow(workflow);

      expect(result.valid).toBe(false);
      const maxAttemptsError = result.errors.find((e) =>
        e.message.includes('must specify maxAttempts')
      );
      expect(maxAttemptsError).toBeDefined();
    });

    it('should validate maxAttempts is positive', () => {
      const workflow: WorkflowDefinition = {
        id: 'wf-1',
        name: 'Test',
        settings: {
          retryPolicy: {
            strategy: 'fail-after-x',
            maxAttempts: 0,
          },
        },
        nodes: [
          {
            id: 'trigger-1',
            type: 'trigger',
            data: { label: 'Start' },
            position: { x: 0, y: 0 },
          },
        ],
        edges: [],
      };

      const result = validateWorkflow(workflow);

      expect(result.valid).toBe(false);
      const positiveError = result.errors.find((e) =>
        e.message.includes('must be at least 1')
      );
      expect(positiveError).toBeDefined();
    });
  });

  describe('Complete Valid Workflow', () => {
    it('should validate a complete workflow', () => {
      const workflow: WorkflowDefinition = {
        id: 'wf-1',
        name: 'Complete Workflow',
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
              componentId: 'comp-1',
              config: { param: 'value' },
              timeout: '30s',
              retryPolicy: {
                strategy: 'exponential-backoff',
                maxAttempts: 3,
                initialInterval: '1s',
                maxInterval: '10s',
                backoffCoefficient: 2,
              },
            },
            position: { x: 100, y: 0 },
          },
          {
            id: 'end-1',
            type: 'end',
            data: { label: 'Complete' },
            position: { x: 200, y: 0 },
          },
        ],
        edges: [
          { id: 'edge-1', source: 'trigger-1', target: 'activity-1' },
          { id: 'edge-2', source: 'activity-1', target: 'end-1' },
        ],
        variables: [
          {
            name: 'result',
            type: 'object',
            description: 'Processing result',
          },
        ],
        settings: {
          timeout: '1h',
          description: 'Complete workflow example',
          version: '1.0.0',
        },
      };

      const result = validateWorkflow(workflow);

      expect(result.valid).toBe(true);
      expect(result.errors).toHaveLength(0);
      expect(result.metadata?.nodeCount).toBe(3);
      expect(result.metadata?.edgeCount).toBe(2);
      expect(result.metadata?.hasStartNode).toBe(true);
      expect(result.metadata?.hasEndNode).toBe(true);
    });
  });
});
