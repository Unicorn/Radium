/**
 * Tests for Interface Validation
 */

import { describe, it, expect } from 'vitest';
import {
  validateDataInConnection,
  validateDataOutConnection,
  validateInterfaceComponents,
} from '../interface-validation';
import type { Edge, Node } from 'react-flow-renderer';

describe('Interface Validation', () => {
  describe('validateDataInConnection', () => {
    it('should return null for non-data-in nodes', () => {
      const node: Node = {
        id: 'node-1',
        type: 'activity',
        data: { label: 'Activity' },
        position: { x: 0, y: 0 },
      };
      const edges: Edge[] = [];
      const nodes: Node[] = [node];

      const result = validateDataInConnection('node-1', edges, nodes);

      expect(result).toBeNull();
    });

    it('should return error when data-in node has no outgoing edge', () => {
      const node: Node = {
        id: 'data-in-1',
        type: 'data-in',
        data: { label: 'Receive Data' },
        position: { x: 0, y: 0 },
      };
      const edges: Edge[] = [];
      const nodes: Node[] = [node];

      const result = validateDataInConnection('data-in-1', edges, nodes);

      expect(result).not.toBeNull();
      expect(result?.nodeId).toBe('data-in-1');
      expect(result?.message).toContain("isn't connected to another component");
    });

    it('should return error when data-in node connects to non-existent target', () => {
      const node: Node = {
        id: 'data-in-1',
        type: 'data-in',
        data: { label: 'Receive Data' },
        position: { x: 0, y: 0 },
      };
      const edge: Edge = {
        id: 'edge-1',
        source: 'data-in-1',
        target: 'non-existent',
      };
      const edges: Edge[] = [edge];
      const nodes: Node[] = [node];

      const result = validateDataInConnection('data-in-1', edges, nodes);

      expect(result).not.toBeNull();
      expect(result?.message).toContain('non-existent node');
    });

    it('should return error when data-in node connects to invalid target type', () => {
      const dataInNode: Node = {
        id: 'data-in-1',
        type: 'data-in',
        data: { label: 'Receive Data' },
        position: { x: 0, y: 0 },
      };
      const endNode: Node = {
        id: 'end-1',
        type: 'end',
        data: { label: 'End' },
        position: { x: 100, y: 0 },
      };
      const edge: Edge = {
        id: 'edge-1',
        source: 'data-in-1',
        target: 'end-1',
      };
      const edges: Edge[] = [edge];
      const nodes: Node[] = [dataInNode, endNode];

      const result = validateDataInConnection('data-in-1', edges, nodes);

      expect(result).not.toBeNull();
      expect(result?.message).toContain('invalid target type');
    });

    it('should return null for valid data-in connection', () => {
      const dataInNode: Node = {
        id: 'data-in-1',
        type: 'data-in',
        data: { label: 'Receive Data' },
        position: { x: 0, y: 0 },
      };
      const activityNode: Node = {
        id: 'activity-1',
        type: 'activity',
        data: { label: 'Process Data' },
        position: { x: 100, y: 0 },
      };
      const edge: Edge = {
        id: 'edge-1',
        source: 'data-in-1',
        target: 'activity-1',
      };
      const edges: Edge[] = [edge];
      const nodes: Node[] = [dataInNode, activityNode];

      const result = validateDataInConnection('data-in-1', edges, nodes);

      expect(result).toBeNull();
    });
  });

  describe('validateDataOutConnection', () => {
    it('should return null for non-data-out nodes', () => {
      const node: Node = {
        id: 'node-1',
        type: 'activity',
        data: { label: 'Activity' },
        position: { x: 0, y: 0 },
      };
      const edges: Edge[] = [];
      const nodes: Node[] = [node];

      const result = validateDataOutConnection('node-1', edges, nodes);

      expect(result).toBeNull();
    });

    it('should return error when data-out node has no incoming edge', () => {
      const node: Node = {
        id: 'data-out-1',
        type: 'data-out',
        data: { label: 'Provide Data' },
        position: { x: 0, y: 0 },
      };
      const edges: Edge[] = [];
      const nodes: Node[] = [node];

      const result = validateDataOutConnection('data-out-1', edges, nodes);

      expect(result).not.toBeNull();
      expect(result?.nodeId).toBe('data-out-1');
      expect(result?.message).toContain("isn't connected to a data source");
    });

    it('should return error when data-out node connects from non-existent source', () => {
      const node: Node = {
        id: 'data-out-1',
        type: 'data-out',
        data: { label: 'Provide Data' },
        position: { x: 0, y: 0 },
      };
      const edge: Edge = {
        id: 'edge-1',
        source: 'non-existent',
        target: 'data-out-1',
      };
      const edges: Edge[] = [edge];
      const nodes: Node[] = [node];

      const result = validateDataOutConnection('data-out-1', edges, nodes);

      expect(result).not.toBeNull();
      expect(result?.message).toContain('non-existent source node');
    });

    it('should return error when data-out node connects from invalid source type', () => {
      const triggerNode: Node = {
        id: 'trigger-1',
        type: 'trigger',
        data: { label: 'Trigger' },
        position: { x: 0, y: 0 },
      };
      const dataOutNode: Node = {
        id: 'data-out-1',
        type: 'data-out',
        data: { label: 'Provide Data' },
        position: { x: 100, y: 0 },
      };
      const edge: Edge = {
        id: 'edge-1',
        source: 'trigger-1',
        target: 'data-out-1',
      };
      const edges: Edge[] = [edge];
      const nodes: Node[] = [triggerNode, dataOutNode];

      const result = validateDataOutConnection('data-out-1', edges, nodes);

      expect(result).not.toBeNull();
      expect(result?.message).toContain('invalid source type');
    });

    it('should return null for valid data-out connection from state variable', () => {
      const stateVarNode: Node = {
        id: 'state-1',
        type: 'state-variable',
        data: { label: 'State' },
        position: { x: 0, y: 0 },
      };
      const dataOutNode: Node = {
        id: 'data-out-1',
        type: 'data-out',
        data: { label: 'Provide Data' },
        position: { x: 100, y: 0 },
      };
      const edge: Edge = {
        id: 'edge-1',
        source: 'state-1',
        target: 'data-out-1',
      };
      const edges: Edge[] = [edge];
      const nodes: Node[] = [stateVarNode, dataOutNode];

      const result = validateDataOutConnection('data-out-1', edges, nodes);

      expect(result).toBeNull();
    });

    it('should return null for valid data-out connection from activity', () => {
      const activityNode: Node = {
        id: 'activity-1',
        type: 'activity',
        data: { label: 'Get Data' },
        position: { x: 0, y: 0 },
      };
      const dataOutNode: Node = {
        id: 'data-out-1',
        type: 'data-out',
        data: { label: 'Provide Data' },
        position: { x: 100, y: 0 },
      };
      const edge: Edge = {
        id: 'edge-1',
        source: 'activity-1',
        target: 'data-out-1',
      };
      const edges: Edge[] = [edge];
      const nodes: Node[] = [activityNode, dataOutNode];

      const result = validateDataOutConnection('data-out-1', edges, nodes);

      expect(result).toBeNull();
    });
  });

  describe('validateInterfaceComponents', () => {
    it('should validate all interface components in workflow', () => {
      const dataInNode: Node = {
        id: 'data-in-1',
        type: 'data-in',
        data: { label: 'Receive Data' },
        position: { x: 0, y: 0 },
      };
      const dataOutNode: Node = {
        id: 'data-out-1',
        type: 'data-out',
        data: { label: 'Provide Data' },
        position: { x: 200, y: 0 },
      };
      const activityNode: Node = {
        id: 'activity-1',
        type: 'activity',
        data: { label: 'Process' },
        position: { x: 100, y: 0 },
      };
      const stateNode: Node = {
        id: 'state-1',
        type: 'state-variable',
        data: { label: 'State' },
        position: { x: 150, y: 0 },
      };

      // Valid connections
      const edge1: Edge = {
        id: 'edge-1',
        source: 'data-in-1',
        target: 'activity-1',
      };
      const edge2: Edge = {
        id: 'edge-2',
        source: 'state-1',
        target: 'data-out-1',
      };

      const edges: Edge[] = [edge1, edge2];
      const nodes: Node[] = [dataInNode, dataOutNode, activityNode, stateNode];

      const result = validateInterfaceComponents(nodes, edges);

      expect(result).toHaveLength(0);
    });

    it('should return errors for invalid interface components', () => {
      const dataInNode: Node = {
        id: 'data-in-1',
        type: 'data-in',
        data: { label: 'Receive Data' },
        position: { x: 0, y: 0 },
      };
      const dataOutNode: Node = {
        id: 'data-out-1',
        type: 'data-out',
        data: { label: 'Provide Data' },
        position: { x: 200, y: 0 },
      };

      // No connections
      const edges: Edge[] = [];
      const nodes: Node[] = [dataInNode, dataOutNode];

      const result = validateInterfaceComponents(nodes, edges);

      expect(result).toHaveLength(2);
      expect(result[0]?.nodeId).toBe('data-in-1');
      expect(result[1]?.nodeId).toBe('data-out-1');
    });

    it('should skip non-interface components', () => {
      const activityNode: Node = {
        id: 'activity-1',
        type: 'activity',
        data: { label: 'Activity' },
        position: { x: 0, y: 0 },
      };
      const triggerNode: Node = {
        id: 'trigger-1',
        type: 'trigger',
        data: { label: 'Trigger' },
        position: { x: 100, y: 0 },
      };

      const edges: Edge[] = [];
      const nodes: Node[] = [activityNode, triggerNode];

      const result = validateInterfaceComponents(nodes, edges);

      expect(result).toHaveLength(0);
    });
  });
});

