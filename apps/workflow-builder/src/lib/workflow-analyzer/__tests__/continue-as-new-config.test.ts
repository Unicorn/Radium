/**
 * Continue-as-New Configuration Tests
 * 
 * Tests for automatic continue-as-new configuration.
 * Validates that services get continue-as-new enabled and tasks don't.
 */

import { describe, it, expect } from 'vitest';
import { configureContinueAsNew, getContinueAsNewConfig } from '../continue-as-new-config';
import type { WorkflowDefinition } from '../../compiler/types';

describe('ContinueAsNewConfig', () => {
  describe('configureContinueAsNew', () => {
    it('should enable continue-as-new for service workflows', () => {
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
            data: { label: 'Handle Signal' },
            position: { x: 100, y: 0 },
          },
        ],
        edges: [],
        variables: [],
        settings: {},
      };

      const result = configureContinueAsNew(workflow, 'service');

      expect(result.settings._workflowType).toBe('service');
      expect(result.settings._longRunning).toBeDefined();
      expect(result.settings._longRunning?.autoContinueAsNew).toBe(true);
      expect(result.settings._longRunning?.maxHistoryEvents).toBe(1000);
      expect(result.settings._longRunning?.maxDurationMs).toBe(24 * 60 * 60 * 1000);
      expect(result.settings._longRunning?.preserveState).toBe(true);
    });

    it('should disable continue-as-new for task workflows', () => {
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
            id: 'end-1',
            type: 'end',
            data: { label: 'End' },
            position: { x: 100, y: 0 },
          },
        ],
        edges: [],
        variables: [],
        settings: {},
      };

      const result = configureContinueAsNew(workflow, 'task');

      expect(result.settings._workflowType).toBe('task');
      expect(result.settings._longRunning).toBeDefined();
      expect(result.settings._longRunning?.autoContinueAsNew).toBe(false);
    });

    it('should auto-classify if workflow type not provided', () => {
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
            data: { label: 'Handle Signal' },
            position: { x: 100, y: 0 },
          },
        ],
        edges: [],
        variables: [],
        settings: {},
      };

      const result = configureContinueAsNew(workflow);

      expect(result.settings._workflowType).toBe('service');
      expect(result.settings._longRunning?.autoContinueAsNew).toBe(true);
    });

    it('should preserve existing settings when configuring', () => {
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
        ],
        edges: [],
        variables: [],
        settings: {
          timeout: '1 hour',
          description: 'Test workflow',
        },
      };

      const result = configureContinueAsNew(workflow, 'service');

      expect(result.settings.timeout).toBe('1 hour');
      expect(result.settings.description).toBe('Test workflow');
      expect(result.settings._longRunning?.autoContinueAsNew).toBe(true);
    });
  });

  describe('getContinueAsNewConfig', () => {
    it('should return config when continue-as-new is enabled', () => {
      const settings = {
        _longRunning: {
          autoContinueAsNew: true,
          maxHistoryEvents: 1000,
          maxDurationMs: 24 * 60 * 60 * 1000,
          preserveState: true,
        },
      };

      const result = getContinueAsNewConfig(settings);

      expect(result).toBeDefined();
      expect(result?.autoContinueAsNew).toBe(true);
    });

    it('should return undefined when continue-as-new is disabled', () => {
      const settings = {
        _longRunning: {
          autoContinueAsNew: false,
          maxHistoryEvents: 1000,
          maxDurationMs: 24 * 60 * 60 * 1000,
          preserveState: true,
        },
      };

      const result = getContinueAsNewConfig(settings);

      expect(result).toBeUndefined();
    });

    it('should return undefined when _longRunning is not set', () => {
      const settings = {};

      const result = getContinueAsNewConfig(settings);

      expect(result).toBeUndefined();
    });
  });
});

