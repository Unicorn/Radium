/**
 * State Collector Tests
 * 
 * Tests for automatic state collection for continue-as-new.
 * Validates that all workflow state is properly collected.
 */

import { describe, it, expect } from 'vitest';
import { collectWorkflowState, generateStateObject } from '../state-collector';
import type { WorkflowDefinition } from '../../compiler/types';

describe('StateCollector', () => {
  describe('collectWorkflowState', () => {
    it('should collect workflow variables', () => {
      const workflow: WorkflowDefinition = {
        id: 'test-workflow',
        name: 'TestWorkflow',
        nodes: [],
        edges: [],
        variables: [
          { name: 'counter', type: 'number', initialValue: 0 },
          { name: 'message', type: 'string', initialValue: 'Hello' },
        ],
        settings: {},
      };

      const stateMap = collectWorkflowState(workflow);

      expect(stateMap.counter).toBeDefined();
      expect(stateMap.counter.type).toBe('number');
      expect(stateMap.counter.source).toBe('variable');
      expect(stateMap.message).toBeDefined();
      expect(stateMap.message.type).toBe('string');
      expect(stateMap.message.source).toBe('variable');
    });

    it('should collect state variables from state-variable nodes', () => {
      const workflow: WorkflowDefinition = {
        id: 'test-workflow',
        name: 'TestWorkflow',
        nodes: [
          {
            id: 'state-var-1',
            type: 'state-variable',
            data: {
              label: 'Queue',
              config: {
                variableName: 'workQueue',
                variableType: 'array',
              },
            },
            position: { x: 0, y: 0 },
          },
        ],
        edges: [],
        variables: [],
        settings: {},
      };

      const stateMap = collectWorkflowState(workflow);

      expect(stateMap.workQueue).toBeDefined();
      expect(stateMap.workQueue.type).toBe('array');
      expect(stateMap.workQueue.source).toBe('state-variable');
    });

    it('should collect loop counters', () => {
      const workflow: WorkflowDefinition = {
        id: 'test-workflow',
        name: 'TestWorkflow',
        nodes: [
          {
            id: 'loop-1',
            type: 'loop',
            data: { label: 'Process Loop' },
            position: { x: 0, y: 0 },
          },
          {
            id: 'loop-2',
            type: 'loop',
            data: { label: 'Another Loop' },
            position: { x: 100, y: 0 },
          },
        ],
        edges: [],
        variables: [],
        settings: {},
      };

      const stateMap = collectWorkflowState(workflow);

      expect(stateMap['_loop-1_iterationCount']).toBeDefined();
      expect(stateMap['_loop-1_iterationCount'].type).toBe('number');
      expect(stateMap['_loop-1_iterationCount'].source).toBe('loop-counter');
      expect(stateMap['_loop-2_iterationCount']).toBeDefined();
      expect(stateMap['_loop-2_iterationCount'].type).toBe('number');
      expect(stateMap['_loop-2_iterationCount'].source).toBe('loop-counter');
    });

    it('should collect signal queue when signal handlers exist', () => {
      const workflow: WorkflowDefinition = {
        id: 'test-workflow',
        name: 'TestWorkflow',
        nodes: [
          {
            id: 'signal-1',
            type: 'signal',
            data: { label: 'Handle Signal' },
            position: { x: 0, y: 0 },
          },
        ],
        edges: [],
        variables: [],
        settings: {},
      };

      const stateMap = collectWorkflowState(workflow);

      expect(stateMap['_signalQueue']).toBeDefined();
      expect(stateMap['_signalQueue'].type).toBe('array');
      expect(stateMap['_signalQueue'].source).toBe('signal-queue');
    });

    it('should not collect signal queue when no signal handlers', () => {
      const workflow: WorkflowDefinition = {
        id: 'test-workflow',
        name: 'TestWorkflow',
        nodes: [
          {
            id: 'activity-1',
            type: 'activity',
            data: { label: 'Do Work' },
            position: { x: 0, y: 0 },
          },
        ],
        edges: [],
        variables: [],
        settings: {},
      };

      const stateMap = collectWorkflowState(workflow);

      expect(stateMap['_signalQueue']).toBeUndefined();
    });

    it('should always include internal tracking variables', () => {
      const workflow: WorkflowDefinition = {
        id: 'test-workflow',
        name: 'TestWorkflow',
        nodes: [],
        edges: [],
        variables: [],
        settings: {},
      };

      const stateMap = collectWorkflowState(workflow);

      expect(stateMap['_workflowStartTime']).toBeDefined();
      expect(stateMap['_workflowStartTime'].type).toBe('number');
      expect(stateMap['_workflowStartTime'].source).toBe('internal');
      expect(stateMap['_historyResetCount']).toBeDefined();
      expect(stateMap['_historyResetCount'].type).toBe('number');
      expect(stateMap['_historyResetCount'].source).toBe('internal');
    });

    it('should collect all state types together', () => {
      const workflow: WorkflowDefinition = {
        id: 'test-workflow',
        name: 'TestWorkflow',
        nodes: [
          {
            id: 'state-var-1',
            type: 'state-variable',
            data: {
              label: 'Queue',
              config: {
                variableName: 'workQueue',
                variableType: 'array',
              },
            },
            position: { x: 0, y: 0 },
          },
          {
            id: 'loop-1',
            type: 'loop',
            data: { label: 'Process Loop' },
            position: { x: 100, y: 0 },
          },
          {
            id: 'signal-1',
            type: 'signal',
            data: { label: 'Handle Signal' },
            position: { x: 200, y: 0 },
          },
        ],
        edges: [],
        variables: [
          { name: 'counter', type: 'number' },
        ],
        settings: {},
      };

      const stateMap = collectWorkflowState(workflow);

      // Check all types are present
      expect(stateMap.counter).toBeDefined(); // variable
      expect(stateMap.workQueue).toBeDefined(); // state-variable
      expect(stateMap['_loop-1_iterationCount']).toBeDefined(); // loop-counter
      expect(stateMap['_signalQueue']).toBeDefined(); // signal-queue
      expect(stateMap['_workflowStartTime']).toBeDefined(); // internal
      expect(stateMap['_historyResetCount']).toBeDefined(); // internal
    });
  });

  describe('generateStateObject', () => {
    it('should generate valid TypeScript object literal', () => {
      const stateMap = {
        counter: { type: 'number', source: 'variable' },
        message: { type: 'string', source: 'variable' },
        workQueue: { type: 'array', source: 'state-variable' },
        '_loop-1_iterationCount': { type: 'number', source: 'loop-counter' },
        '_signalQueue': { type: 'array', source: 'signal-queue' },
        '_workflowStartTime': { type: 'number', source: 'internal' },
        '_historyResetCount': { type: 'number', source: 'internal' },
      };

      const stateObject = generateStateObject(stateMap);

      // Should include all non-internal variables
      expect(stateObject).toContain('counter: counter');
      expect(stateObject).toContain('message: message');
      expect(stateObject).toContain('workQueue: workQueue');
      expect(stateObject).toContain('_loop-1_iterationCount: _loop-1_iterationCount');
      expect(stateObject).toContain('_signalQueue: _signalQueue');
      
      // Should set internal variables
      expect(stateObject).toContain('_workflowStartTime: workflowInfo().runStartTime.getTime()');
      expect(stateObject).toContain('_historyResetCount: (_historyResetCount || 0) + 1');
    });

    it('should handle empty state map', () => {
      const stateMap = {
        '_workflowStartTime': { type: 'number', source: 'internal' },
        '_historyResetCount': { type: 'number', source: 'internal' },
      };

      const stateObject = generateStateObject(stateMap);

      // Should only have internal variables
      expect(stateObject).toContain('_workflowStartTime: workflowInfo().runStartTime.getTime()');
      expect(stateObject).toContain('_historyResetCount: (_historyResetCount || 0) + 1');
    });
  });
});
