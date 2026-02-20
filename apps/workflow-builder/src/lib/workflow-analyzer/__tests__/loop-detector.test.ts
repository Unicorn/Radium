/**
 * Loop Detector Tests
 * 
 * Tests for detecting long-running loops that need continue-as-new.
 */

import { describe, it, expect } from 'vitest';
import { detectLongRunningLoops } from '../loop-detector';
import type { WorkflowDefinition } from '../../compiler/types';

describe('LoopDetector', () => {
  describe('detectLongRunningLoops', () => {
    it('should detect infinite loops (no maxIterations)', () => {
      const workflow: WorkflowDefinition = {
        id: 'test-workflow',
        name: 'TestWorkflow',
        nodes: [
          {
            id: 'loop-1',
            type: 'loop',
            data: {
              label: 'Infinite Loop',
              config: {}, // No maxIterations
            },
            position: { x: 0, y: 0 },
          },
        ],
        edges: [],
        variables: [],
        settings: {},
      };

      const result = detectLongRunningLoops(workflow);
      expect(result).toContain('loop-1');
    });

    it('should detect loops with maxIterations > 100', () => {
      const workflow: WorkflowDefinition = {
        id: 'test-workflow',
        name: 'TestWorkflow',
        nodes: [
          {
            id: 'loop-1',
            type: 'loop',
            data: {
              label: 'Long Loop',
              config: { maxIterations: 1000 },
            },
            position: { x: 0, y: 0 },
          },
        ],
        edges: [],
        variables: [],
        settings: {},
      };

      const result = detectLongRunningLoops(workflow);
      expect(result).toContain('loop-1');
    });

    it('should not detect short loops (< 100 iterations)', () => {
      const workflow: WorkflowDefinition = {
        id: 'test-workflow',
        name: 'TestWorkflow',
        nodes: [
          {
            id: 'loop-1',
            type: 'loop',
            data: {
              label: 'Short Loop',
              config: { maxIterations: 10 },
            },
            position: { x: 0, y: 0 },
          },
        ],
        edges: [],
        variables: [],
        settings: {},
      };

      const result = detectLongRunningLoops(workflow);
      expect(result).not.toContain('loop-1');
    });

    it('should detect loops with signal handlers inside', () => {
      const workflow: WorkflowDefinition = {
        id: 'test-workflow',
        name: 'TestWorkflow',
        nodes: [
          {
            id: 'loop-1',
            type: 'loop',
            data: {
              label: 'Loop with Signal',
              config: { maxIterations: 50 },
            },
            position: { x: 0, y: 0 },
          },
          {
            id: 'signal-1',
            type: 'signal',
            data: { label: 'Handle Signal' },
            position: { x: 100, y: 0 },
          },
        ],
        edges: [
          {
            id: 'edge-1',
            source: 'loop-1',
            target: 'signal-1',
          },
        ],
        variables: [],
        settings: {},
      };

      const result = detectLongRunningLoops(workflow);
      expect(result).toContain('loop-1');
    });

    it('should detect multiple long-running loops', () => {
      const workflow: WorkflowDefinition = {
        id: 'test-workflow',
        name: 'TestWorkflow',
        nodes: [
          {
            id: 'loop-1',
            type: 'loop',
            data: {
              label: 'Loop 1',
              config: {}, // Infinite
            },
            position: { x: 0, y: 0 },
          },
          {
            id: 'loop-2',
            type: 'loop',
            data: {
              label: 'Loop 2',
              config: { maxIterations: 500 },
            },
            position: { x: 100, y: 0 },
          },
          {
            id: 'loop-3',
            type: 'loop',
            data: {
              label: 'Loop 3',
              config: { maxIterations: 5 },
            },
            position: { x: 200, y: 0 },
          },
        ],
        edges: [],
        variables: [],
        settings: {},
      };

      const result = detectLongRunningLoops(workflow);
      expect(result).toContain('loop-1');
      expect(result).toContain('loop-2');
      expect(result).not.toContain('loop-3');
    });

    it('should handle loops with maxIterations of 0 as infinite', () => {
      const workflow: WorkflowDefinition = {
        id: 'test-workflow',
        name: 'TestWorkflow',
        nodes: [
          {
            id: 'loop-1',
            type: 'loop',
            data: {
              label: 'Zero Iterations',
              config: { maxIterations: 0 },
            },
            position: { x: 0, y: 0 },
          },
        ],
        edges: [],
        variables: [],
        settings: {},
      };

      const result = detectLongRunningLoops(workflow);
      expect(result).toContain('loop-1');
    });

    it('should handle loops with negative maxIterations as infinite', () => {
      const workflow: WorkflowDefinition = {
        id: 'test-workflow',
        name: 'TestWorkflow',
        nodes: [
          {
            id: 'loop-1',
            type: 'loop',
            data: {
              label: 'Negative Iterations',
              config: { maxIterations: -1 },
            },
            position: { x: 0, y: 0 },
          },
        ],
        edges: [],
        variables: [],
        settings: {},
      };

      const result = detectLongRunningLoops(workflow);
      expect(result).toContain('loop-1');
    });

    it('should return empty array when no loops exist', () => {
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

      const result = detectLongRunningLoops(workflow);
      expect(result).toEqual([]);
    });
  });
});

