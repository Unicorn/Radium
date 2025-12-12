/**
 * Service Builder View - Full service visualization with zone-based layout
 * Shows a single service with all its interfaces, connectors, and internal flow
 */

'use client';

import { useMemo, useEffect } from 'react';
import ReactFlow, {
  Background,
  Controls,
  MiniMap,
  useNodesState,
  useEdgesState,
  type Node,
  type Edge,
  type Connection,
  addEdge,
} from 'react-flow-renderer';
import { nodeTypes } from '@/components/workflow/nodes';
import type { ServiceContainerNodeData } from '@/components/workflow/nodes/ServiceContainerNode';
import { api } from '@/lib/trpc/client';
import { YStack, Spinner, Text } from 'tamagui';

interface ServiceBuilderViewProps {
  serviceId: string;
  readOnly?: boolean;
}

/**
 * Service Builder View Component
 * 
 * Displays a single service with:
 * - External connectors (top zone)
 * - Incoming interfaces (left zone)
 * - Internal flow/components (center zone)
 * - Outgoing interfaces (right zone)
 * - External service connections (bottom zone)
 */
export function ServiceBuilderView({ 
  serviceId, 
  readOnly = false 
}: ServiceBuilderViewProps) {
  // Fetch service data
  const { data: workflowData, isLoading: isLoadingWorkflow } = api.workflows.get.useQuery({ 
    id: serviceId 
  });

  // Fetch service interfaces
  const { data: interfaces, isLoading: isLoadingInterfaces } = api.serviceInterfaces.list.useQuery({ 
    serviceId 
  });

  // Fetch connectors for the project
  const projectId = workflowData?.workflow?.project_id;
  const { data: connectors, isLoading: isLoadingConnectors } = api.connectors.list.useQuery(
    { projectId: projectId || '' },
    { enabled: !!projectId }
  );

  // Fetch project connectors (service-to-service connections)
  // Get both source (from this project) and target (to this project) connectors
  const { data: sourceConnectors, isLoading: isLoadingSourceConnectors } = 
    api.projectConnectors.list.useQuery(
      { projectId: projectId || '', source: true },
      { enabled: !!projectId }
    );
  const { data: targetConnectors, isLoading: isLoadingTargetConnectors } = 
    api.projectConnectors.list.useQuery(
      { projectId: projectId || '', source: false },
      { enabled: !!projectId }
    );

  const isLoadingProjectConnectors = isLoadingSourceConnectors || isLoadingTargetConnectors;
  const projectConnectors = [...(sourceConnectors || []), ...(targetConnectors || [])];

  const isLoading = isLoadingWorkflow || isLoadingInterfaces || isLoadingConnectors || isLoadingProjectConnectors;

  // Transform data into React Flow nodes and edges
  const { initialNodes, initialEdges } = useMemo(() => {
    if (!workflowData?.workflow || !interfaces) {
      return { initialNodes: [], initialEdges: [] };
    }

    const workflow = workflowData.workflow;
    const definition = workflow.definition as any;
    const nodes = definition?.nodes || [];
    const edges = definition?.edges || [];

    // Separate interfaces by type
    const incomingInterfaces = (interfaces || [])
      .filter(i => i.interface_type === 'signal' || i.interface_type === 'query')
      .map((iface, index) => ({
        id: iface.id,
        name: iface.name,
        displayName: iface.display_name,
        interfaceType: iface.interface_type as 'signal' | 'query' | 'update' | 'start_child',
        position: (index + 1) * (100 / (interfaces?.length || 1)),
      }));

    const outgoingInterfaces = (interfaces || [])
      .filter(i => i.interface_type === 'update' || i.interface_type === 'start_child')
      .map((iface, index) => ({
        id: iface.id,
        name: iface.name,
        displayName: iface.display_name,
        interfaceType: iface.interface_type as 'signal' | 'query' | 'update' | 'start_child',
        position: (index + 1) * (100 / (interfaces?.length || 1)),
      }));

    // Transform connectors
    const externalConnectors = (connectors || [])
      .filter(c => c.is_active)
      .map((connector, index) => ({
        id: connector.id,
        name: connector.name,
        displayName: connector.display_name,
        connectorType: connector.connector_type,
        position: (index + 1) * (100 / ((connectors?.length || 0) + 1)),
      }));

    // Transform project connectors (external service connections)
    // These are connections FROM this service TO other services
    const externalConnections = (sourceConnectors || [])
      .filter(pc => pc.target_service?.id === serviceId || pc.source_project_id === projectId)
      .map((pc, index) => ({
        id: pc.id,
        targetServiceId: pc.target_service_id,
        targetServiceName: pc.target_service?.service_display_name || pc.target_service?.name || 'Unknown',
        interfaceName: pc.target_interface?.display_name || pc.target_interface?.name || '',
        position: (index + 1) * (100 / ((sourceConnectors?.length || 0) + 1)),
      }));

    // Transform internal components
    // Include interface components (data-in, data-out) in the internal flow
    const internalComponents = nodes.map((node: any) => ({
      id: node.id,
      type: node.type,
      label: node.data?.label || node.data?.displayName || node.data?.name || node.type,
      isInterfaceComponent: node.type === 'data-in' || node.type === 'data-out',
      endpointPath: node.data?.config?.endpointPath || node.data?.endpointPath,
      httpMethod: node.data?.config?.httpMethod || node.data?.httpMethod,
    }));

    // Also extract interface components from nodes to show as interfaces
    const interfaceComponents = nodes
      .filter((node: any) => node.type === 'data-in' || node.type === 'data-out')
      .map((node: any) => ({
        id: node.id,
        name: node.data?.name || node.data?.label || node.id,
        displayName: node.data?.displayName || node.data?.label || node.id,
        interfaceType: node.type === 'data-in' ? 'signal' : 'query',
        endpointPath: node.data?.config?.endpointPath || node.data?.endpointPath,
        httpMethod: node.data?.config?.httpMethod || node.data?.httpMethod || (node.type === 'data-in' ? 'POST' : 'GET'),
      }));

    // Merge service interfaces with interface components
    const allIncomingInterfaces = [
      ...incomingInterfaces,
      ...interfaceComponents.filter(ic => ic.interfaceType === 'signal'),
    ];
    const allOutgoingInterfaces = [
      ...outgoingInterfaces,
      ...interfaceComponents.filter(ic => ic.interfaceType === 'query'),
    ];

    // Create service container node data
    const containerData: ServiceContainerNodeData = {
      serviceId: workflow.id,
      serviceName: workflow.display_name || workflow.name,
      incomingInterfaces: allIncomingInterfaces,
      outgoingInterfaces: allOutgoingInterfaces,
      externalConnectors,
      internalComponents,
      externalConnections,
      viewMode: 'builder',
    };

    // Create the service container node
    const containerNode: Node<ServiceContainerNodeData> = {
      id: `service-container-${serviceId}`,
      type: 'service-container',
      position: { x: 0, y: 0 },
      data: containerData,
      style: { width: 1200, height: 800 },
    };

    // Transform internal edges (within the service)
    const internalEdges: Edge[] = (edges || []).map((edge: any) => ({
      id: edge.id,
      source: edge.source,
      target: edge.target,
      sourceHandle: edge.sourceHandle,
      targetHandle: edge.targetHandle,
      type: 'default',
      style: { stroke: '#6366f1', strokeWidth: 2 },
    }));

    return {
      initialNodes: [containerNode],
      initialEdges: internalEdges,
    };
  }, [workflowData, interfaces, connectors, sourceConnectors, projectId, serviceId]);

  const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes);
  const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges);

  // Update nodes when data changes
  useEffect(() => {
    setNodes(initialNodes);
    setEdges(initialEdges);
  }, [initialNodes, initialEdges, setNodes, setEdges]);

  const onConnect = useMemo(
    () => (params: Connection) => {
      if (readOnly) return;
      setEdges((eds) => addEdge(params, eds));
    },
    [readOnly, setEdges]
  );

  if (isLoading) {
    return (
      <YStack flex={1} alignItems="center" justifyContent="center" gap="$3">
        <Spinner size="large" />
        <Text>Loading service...</Text>
      </YStack>
    );
  }

  return (
    <div style={{ width: '100%', height: '100%', minHeight: '800px' }}>
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onConnect={onConnect}
        nodeTypes={nodeTypes}
        connectionMode="loose"
        fitView
        minZoom={0.1}
        maxZoom={2}
        defaultViewport={{ x: 0, y: 0, zoom: 0.8 }}
      >
        <Background />
        <Controls />
        <MiniMap />
      </ReactFlow>
    </div>
  );
}

