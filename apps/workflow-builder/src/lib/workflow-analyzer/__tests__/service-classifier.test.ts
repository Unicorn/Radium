/**
 * Service Classifier Tests
 * 
 * Tests for automatic workflow classification (service vs task).
 * Validates detection logic against real workflow patterns.
 */

import { describe, it, expect } from 'vitest';
import { classifyWorkflow } from '../service-classifier';
import type { WorkflowDefinition } from '../../compiler/types';

describe('ServiceClassifier', () => {
  describe('classifyWorkflow', () => {
    it('should classify workflow with signal handlers as service', () => {
      const workflow: WorkflowDefinition = {
        id: 'test-service',
        name: 'TestService',
        nodes: [
          {
            id: 'trigger-1',
            type: 'trigger',
            data: { label: 'Start' },
            position: { x: 0, y: 0 },
          },
          {
            id: 'signal-1',
            type: 'signal',
            data: { label: 'Handle Signal', signalName: 'workAvailable' },
            position: { x: 100, y: 0 },
          },
        ],
        edges: [],
        variables: [],
        settings: {},
      };

      const result = classifyWorkflow(workflow);
      expect(result).toBe('service');
    });

    it('should classify workflow with query handlers as service', () => {
      const workflow: WorkflowDefinition = {
        id: 'test-service',
        name: 'TestService',
        nodes: [
          {
            id: 'trigger-1',
            type: 'trigger',
            data: { label: 'Start' },
            position: { x: 0, y: 0 },
          },
          {
            id: 'query-1',
            type: 'query',
            data: { label: 'Get Status' },
            position: { x: 100, y: 0 },
          },
        ],
        edges: [],
        variables: [],
        settings: {},
      };

      const result = classifyWorkflow(workflow);
      expect(result).toBe('service');
    });

    it('should classify workflow with data-out query interface as service', () => {
      const workflow: WorkflowDefinition = {
        id: 'test-service',
        name: 'TestService',
        nodes: [
          {
            id: 'trigger-1',
            type: 'trigger',
            data: { label: 'Start' },
            position: { x: 0, y: 0 },
          },
          {
            id: 'data-out-1',
            type: 'data-out',
            data: { 
              label: 'Get Status',
              config: { interfaceType: 'query' }
            },
            position: { x: 100, y: 0 },
          },
        ],
        edges: [],
        variables: [],
        settings: {},
      };

      const result = classifyWorkflow(workflow);
      expect(result).toBe('service');
    });

    it('should classify workflow with infinite loop as service', () => {
      const workflow: WorkflowDefinition = {
        id: 'test-service',
        name: 'TestService',
        nodes: [
          {
            id: 'trigger-1',
            type: 'trigger',
            data: { label: 'Start' },
            position: { x: 0, y: 0 },
          },
          {
            id: 'loop-1',
            type: 'loop',
            data: { 
              label: 'Process Loop',
              config: {} // No maxIterations = infinite
            },
            position: { x: 100, y: 0 },
          },
        ],
        edges: [],
        variables: [],
        settings: {},
      };

      const result = classifyWorkflow(workflow);
      expect(result).toBe('service');
    });

    it('should classify workflow with no end node as service', () => {
      const workflow: WorkflowDefinition = {
        id: 'test-service',
        name: 'TestService',
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
            data: { label: 'Do Work' },
            position: { x: 100, y: 0 },
          },
          // No end node
        ],
        edges: [],
        variables: [],
        settings: {},
      };

      const result = classifyWorkflow(workflow);
      expect(result).toBe('service');
    });

    it('should classify workflow with end node and max iterations as task', () => {
      const workflow: WorkflowDefinition = {
        id: 'test-task',
        name: 'TestTask',
        nodes: [
          {
            id: 'trigger-1',
            type: 'trigger',
            data: { label: 'Start' },
            position: { x: 0, y: 0 },
          },
          {
            id: 'loop-1',
            type: 'loop',
            data: { 
              label: 'Process Loop',
              config: { maxIterations: 10 }
            },
            position: { x: 100, y: 0 },
          },
          {
            id: 'end-1',
            type: 'end',
            data: { label: 'End' },
            position: { x: 200, y: 0 },
          },
        ],
        edges: [],
        variables: [],
        settings: {},
      };

      const result = classifyWorkflow(workflow);
      expect(result).toBe('task');
    });

    it('should classify simple linear workflow with end as task', () => {
      const workflow: WorkflowDefinition = {
        id: 'test-task',
        name: 'TestTask',
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
            data: { label: 'Do Work' },
            position: { x: 100, y: 0 },
          },
          {
            id: 'end-1',
            type: 'end',
            data: { label: 'End' },
            position: { x: 200, y: 0 },
          },
        ],
        edges: [],
        variables: [],
        settings: {},
      };

      const result = classifyWorkflow(workflow);
      expect(result).toBe('task');
    });

    it('should default to service if ambiguous (safer for long-running)', () => {
      const workflow: WorkflowDefinition = {
        id: 'test-ambiguous',
        name: 'TestAmbiguous',
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
            data: { label: 'Do Work' },
            position: { x: 100, y: 0 },
          },
          // No end node, but no service indicators either
        ],
        edges: [],
        variables: [],
        settings: {},
      };

      const result = classifyWorkflow(workflow);
      expect(result).toBe('service'); // Defaults to service for safety
    });

    it('should handle workflow with loop having maxIterations of 0 as service', () => {
      const workflow: WorkflowDefinition = {
        id: 'test-service',
        name: 'TestService',
        nodes: [
          {
            id: 'trigger-1',
            type: 'trigger',
            data: { label: 'Start' },
            position: { x: 0, y: 0 },
          },
          {
            id: 'loop-1',
            type: 'loop',
            data: { 
              label: 'Process Loop',
              config: { maxIterations: 0 } // 0 = infinite
            },
            position: { x: 100, y: 0 },
          },
        ],
        edges: [],
        variables: [],
        settings: {},
      };

      const result = classifyWorkflow(workflow);
      expect(result).toBe('service');
    });

    it('should handle workflow with loop having negative maxIterations as service', () => {
      const workflow: WorkflowDefinition = {
        id: 'test-service',
        name: 'TestService',
        nodes: [
          {
            id: 'trigger-1',
            type: 'trigger',
            data: { label: 'Start' },
            position: { x: 0, y: 0 },
          },
          {
            id: 'loop-1',
            type: 'loop',
            data: { 
              label: 'Process Loop',
              config: { maxIterations: -1 } // Negative = invalid/infinite
            },
            position: { x: 100, y: 0 },
          },
        ],
        edges: [],
        variables: [],
        settings: {},
      };

      const result = classifyWorkflow(workflow);
      expect(result).toBe('service');
    });
  });
});

