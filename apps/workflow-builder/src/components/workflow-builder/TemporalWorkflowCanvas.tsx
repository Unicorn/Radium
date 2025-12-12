'use client';

import { useState, useCallback } from 'react';
import { YStack, XStack, Text } from 'tamagui';
// import { TemporalWorkflowBuilder } from '@bernierllc/temporal-workflow-ui';
import type { TemporalWorkflow } from '@/types/advanced-patterns';
import { NodeTypesPalette } from './NodeTypesPalette';
import { NodePropertyPanel } from './NodePropertyPanel';

interface TemporalWorkflowCanvasProps {
  workflow: any; // From database
  availableComponents: any[];
  workQueues: any[];
  signals: any[];
  queries: any[];
  onChange?: () => void;
}

export function TemporalWorkflowCanvas({
  workflow,
  availableComponents,
  workQueues,
  signals,
  queries,
  onChange,
}: TemporalWorkflowCanvasProps) {
  const [workflowDefinition, setWorkflowDefinition] = useState<TemporalWorkflow>(() => {
    // Convert database workflow to TemporalWorkflow format
    return convertToTemporalWorkflow(workflow, workQueues, signals, queries);
  });

  const [selectedNode, setSelectedNode] = useState<any>(null);

  const handleWorkflowChange = useCallback((updatedWorkflow: TemporalWorkflow) => {
    setWorkflowDefinition(updatedWorkflow);
    onChange?.();
  }, [onChange]);

  const handleAddNode = useCallback((nodeType: any) => {
    console.log('Adding node:', nodeType);
    // TODO: Add node to workflow definition
    onChange?.();
  }, [onChange]);

  const handleNodeSelect = useCallback((nodeId: string) => {
    // Find node in workflow definition
    const node = workflowDefinition.stages.find(s => s.id === nodeId);
    if (node) {
      setSelectedNode({
        id: node.id,
        type: node.type,
        data: node.metadata,
      });
    }
  }, [workflowDefinition]);

  const handlePropertySave = useCallback((nodeId: string, updates: Record<string, any>) => {
    console.log('Saving properties for node:', nodeId, updates);
    // TODO: Update node in workflow definition
    onChange?.();
    setSelectedNode(null);
  }, [onChange]);

  // TODO: Once @bernierllc/temporal-workflow-ui is integrated, use the actual component
  // For now, show layout with palette and property panel
  return (
    <XStack f={1} h="100%">
      {/* Left Sidebar: Component Palette */}
      <NodeTypesPalette
        availableComponents={availableComponents}
        workQueues={workQueues}
        signals={signals}
        queries={queries}
        onAddNode={handleAddNode}
      />

      {/* Center: Workflow Canvas */}
      <YStack
        f={1}
        ai="center"
        jc="center"
        bg="$gray2"
        position="relative"
        role="main"
        aria-label="Workflow canvas area"
      >
      {/* Placeholder - will be replaced with actual TemporalWorkflowBuilder */}
      <YStack
        gap="$4"
        ai="center"
        p="$6"
        bg="$background"
        borderRadius="$4"
        borderWidth={1}
        borderColor="$borderColor"
        role="region"
        aria-label="Workflow builder placeholder"
      >
        <YStack ai="center" gap="$2">
          <YStack w={60} h={60} bg="$blue5" borderRadius="$4" ai="center" jc="center">
            <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <rect x="3" y="3" width="7" height="7" rx="1" />
              <rect x="14" y="3" width="7" height="7" rx="1" />
              <rect x="3" y="14" width="7" height="7" rx="1" />
              <rect x="14" y="14" width="7" height="7" rx="1" />
            </svg>
          </YStack>
          <YStack ai="center" gap="$1">
            <Text fontSize="$6" fontWeight="600">
              Workflow Builder Canvas
            </Text>
            <Text fontSize="$3" color="$gray11" textAlign="center">
              Integration with @bernierllc/temporal-workflow-ui
            </Text>
          </YStack>
        </YStack>

        {/* Stats */}
        <YStack gap="$2" w="100%">
          <YStack p="$3" bg="$blue2" borderRadius="$4" borderWidth={1} borderColor="$blue6">
            <Text fontSize="$2" color="$blue11" fontWeight="600">Available Components</Text>
            <Text fontSize="$5" fontWeight="600">{availableComponents.length}</Text>
          </YStack>
          
          <YStack p="$3" bg="$yellow2" borderRadius="$4" borderWidth={1} borderColor="$yellow6">
            <Text fontSize="$2" color="$yellow11" fontWeight="600">Work Queues</Text>
            <Text fontSize="$5" fontWeight="600">{workQueues.length}</Text>
          </YStack>
          
          <YStack p="$3" bg="$orange2" borderRadius="$4" borderWidth={1} borderColor="$orange6">
            <Text fontSize="$2" color="$orange11" fontWeight="600">Signals</Text>
            <Text fontSize="$5" fontWeight="600">{signals.length}</Text>
          </YStack>
          
          <YStack p="$3" bg="$teal2" borderRadius="$4" borderWidth={1} borderColor="$teal6">
            <Text fontSize="$2" color="$teal11" fontWeight="600">Queries</Text>
            <Text fontSize="$5" fontWeight="600">{queries.length}</Text>
          </YStack>
        </YStack>

        {/* TODO Note */}
        <YStack p="$3" bg="$purple2" borderRadius="$4" borderWidth={1} borderColor="$purple6" w="100%">
          <Text fontSize="$2" color="$purple11" textAlign="center">
            🚧 Integrating TemporalWorkflowBuilder component
          </Text>
          <Text fontSize="$2" color="$purple11" textAlign="center">
            This will provide visual drag-and-drop workflow building
          </Text>
        </YStack>
      </YStack>

      {/* 
      TODO: Uncomment when ready to integrate
      
      <TemporalWorkflowBuilder
        workflow={workflowDefinition}
        onChange={handleWorkflowChange}
        onNodeSelect={handleNodeSelect}
        componentPalette={{
          activities: availableComponents.filter(c => c.component_type === 'activity'),
          agents: availableComponents.filter(c => c.component_type === 'agent'),
          signals: signals,
          queries: queries,
          workQueues: workQueues,
        }}
        theme={{
          // Use Tamagui theme colors
          colors: {
            activity: '$blue9',
            agent: '$purple9',
            signal: '$orange9',
            query: '$teal9',
            workQueue: '$yellow9',
            scheduledWorkflow: '$pink9',
          }
        }}
      />
      */}
      </YStack>

      {/* Right Sidebar: Property Panel (conditional) */}
      {selectedNode && (
        <NodePropertyPanel
          node={selectedNode}
          onClose={() => setSelectedNode(null)}
          onSave={handlePropertySave}
        />
      )}
    </XStack>
  );
}

/**
 * Convert database workflow format to TemporalWorkflow format
 */
function convertToTemporalWorkflow(
  workflow: any,
  workQueues: any[],
  signals: any[],
  queries: any[]
): TemporalWorkflow {
  // Parse workflow definition from database
  const definition = typeof workflow.definition === 'string' 
    ? JSON.parse(workflow.definition) 
    : workflow.definition;

  return {
    id: workflow.id,
    name: workflow.name,
    description: workflow.description,
    stages: definition.nodes?.map((node: any) => ({
      id: node.id,
      type: node.type,
      position: node.position,
      metadata: node.data,
    })) || [],
    transitions: definition.edges?.map((edge: any) => ({
      from: edge.source,
      to: edge.target,
      condition: edge.label,
    })) || [],
    workQueues: workQueues.map(wq => ({
      id: wq.id,
      workflowId: wq.workflow_id,
      name: wq.queue_name,
      description: wq.description,
      signalName: wq.signal_name,
      queryName: wq.query_name,
      maxSize: wq.max_size,
      priority: wq.priority,
      deduplicate: wq.deduplicate,
      workItemSchema: wq.work_item_schema,
      createdAt: new Date(wq.created_at),
      updatedAt: new Date(wq.updated_at),
    })),
    signals: signals.map(s => ({
      id: s.id,
      workflowId: s.workflow_id,
      name: s.signal_name,
      description: s.description,
      parametersSchema: s.parameters_schema,
      autoGenerated: s.auto_generated,
      createdAt: new Date(s.created_at),
    })),
    queries: queries.map(q => ({
      id: q.id,
      workflowId: q.workflow_id,
      name: q.query_name,
      description: q.description,
      returnTypeSchema: q.return_type_schema,
      autoGenerated: q.auto_generated,
      createdAt: new Date(q.created_at),
    })),
  };
}

