/**
 * Project View - Multi-service overview showing how services connect
 * Displays all services in a project with their connections
 */

'use client';

import { useMemo, useEffect, useState } from 'react';
import { useRouter } from 'next/navigation';
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
import * as dagre from 'dagre';

interface ProjectViewProps {
  projectId: string;
  readOnly?: boolean;
}

/**
 * Project View Component
 * 
 * Displays all services in a project with:
 * - Service containers (compact view)
 * - Service-to-service connections via interfaces
 * - External connectors shown
 * - Click to navigate to service builder view
 */
export function ProjectView({ 
  projectId, 
  readOnly = false 
}: ProjectViewProps) {
  const router = useRouter();
  const utils = api.useUtils();

  // Fetch project data
  const { data: projectData, isLoading: isLoadingProject } = api.projects.get.useQuery({ 
    id: projectId 
  });

  // Fetch all workflows (services) in the project
  const { data: workflowsData, isLoading: isLoadingWorkflows } = api.workflows.list.useQuery({
    projectId,
  });

  // Fetch connectors for the project
  const { data: connectors, isLoading: isLoadingConnectors } = api.connectors.list.useQuery(
    { projectId },
    { enabled: !!projectId }
  );

  // Fetch project connectors (service-to-service connections)
  const { data: projectConnectors, isLoading: isLoadingProjectConnectors } = 
    api.projectConnectors.list.useQuery(
      { projectId, source: true },
      { enabled: !!projectId }
    );

  // Get service IDs - we'll fetch interfaces individually but safely
  const serviceIds = useMemo(() => {
    return workflowsData?.workflows?.map(w => w.id) || [];
  }, [workflowsData?.workflows]);

  // Fetch interfaces for each service using useEffect and tRPC utils
  const [interfacesByService, setInterfacesByService] = useState<Map<string, any[]>>(new Map());
  const [isLoadingInterfaces, setIsLoadingInterfaces] = useState(false);

  // Fetch interfaces for all services
  useEffect(() => {
    if (serviceIds.length === 0) {
      setInterfacesByService(new Map());
      setIsLoadingInterfaces(false);
      return;
    }

    setIsLoadingInterfaces(true);
    const fetchInterfaces = async () => {
      const map = new Map<string, any[]>();
      const promises = serviceIds.map(async (serviceId) => {
        try {
          // Use tRPC utils to fetch interfaces for each service
          const interfaces = await utils.serviceInterfaces.list.fetch({ workflowId: serviceId });
          return { serviceId, interfaces: interfaces || [] };
        } catch (error) {
          console.error(`Failed to fetch interfaces for service ${serviceId}:`, error);
          return { serviceId, interfaces: [] };
        }
      });
      
      const results = await Promise.all(promises);
      results.forEach(({ serviceId, interfaces }) => {
        map.set(serviceId, interfaces);
      });
      setInterfacesByService(map);
      setIsLoadingInterfaces(false);
    };

    fetchInterfaces();
  }, [serviceIds, utils.serviceInterfaces.list]);

  const isLoading = isLoadingProject || isLoadingWorkflows || isLoadingConnectors || 
    isLoadingProjectConnectors || isLoadingInterfaces;

  // Transform data into React Flow nodes and edges
  const { initialNodes, initialEdges } = useMemo(() => {
    if (!workflowsData?.workflows) {
      return { initialNodes: [], initialEdges: [] };
    }

    const services = workflowsData.workflows;
    const nodes: Node<ServiceContainerNodeData>[] = [];
    const edges: Edge[] = [];

    // Create service container nodes
    services.forEach((service, index) => {
      const serviceInterfaces = interfacesByService.get(service.id) || [];
      
      const incomingInterfaces = serviceInterfaces
        .filter(i => i.interface_type === 'signal' || i.interface_type === 'query')
        .map((iface) => ({
          id: iface.id,
          name: iface.name,
          displayName: iface.display_name,
          interfaceType: iface.interface_type as 'signal' | 'query' | 'update' | 'start_child',
        }));

      const outgoingInterfaces = serviceInterfaces
        .filter(i => i.interface_type === 'update' || i.interface_type === 'start_child')
        .map((iface) => ({
          id: iface.id,
          name: iface.name,
          displayName: iface.display_name,
          interfaceType: iface.interface_type as 'signal' | 'query' | 'update' | 'start_child',
        }));

      // Get component count and interface components from workflow definition
      const definition = service.definition as any;
      const nodes = definition?.nodes || [];
      const componentCount = nodes.length;
      
      // Extract interface components from nodes
      const interfaceComponents = nodes
        .filter((node: any) => node.type === 'data-in' || node.type === 'data-out')
        .map((node: any) => ({
          id: node.id,
          name: node.data?.name || node.data?.label || node.id,
          displayName: node.data?.displayName || node.data?.label || node.id,
          interfaceType: node.type === 'data-in' ? 'signal' : 'query',
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

      const containerData: ServiceContainerNodeData = {
        serviceId: service.id,
        serviceName: service.display_name || service.name,
        incomingInterfaces: allIncomingInterfaces,
        outgoingInterfaces: allOutgoingInterfaces,
        externalConnectors: [], // Not shown in project view
        internalComponents: Array(componentCount).fill(null).map((_, i) => ({
          id: `component-${i}`,
          type: 'component',
          label: `Component ${i + 1}`,
        })),
        externalConnections: [], // Not shown in project view
        viewMode: 'project',
      };

      const containerNode: Node<ServiceContainerNodeData> = {
        id: `service-${service.id}`,
        type: 'service-container',
        position: { x: index * 400, y: 0 }, // Initial position, will be auto-laid out
        data: containerData,
        style: { width: 300, height: 200 },
      };

      nodes.push(containerNode);
    });

    // Create edges for service-to-service connections
    (projectConnectors || []).forEach((connector) => {
      const sourceServiceId = connector.source_project_id === projectId 
        ? connector.target_service_id 
        : null;
      const targetServiceId = connector.target_service_id;

      if (!sourceServiceId || !targetServiceId) return;

      const sourceNodeId = `service-${sourceServiceId}`;
      const targetNodeId = `service-${targetServiceId}`;

      // Find the interface IDs
      const sourceInterfaces = interfacesByService.get(sourceServiceId) || [];
      const targetInterfaces = interfacesByService.get(targetServiceId) || [];

      const outgoingInterface = sourceInterfaces.find(
        i => i.id === connector.target_interface_id
      );
      const incomingInterface = targetInterfaces.find(
        i => i.id === connector.target_interface_id
      );

      if (outgoingInterface && incomingInterface) {
        const edge: Edge = {
          id: `connector-${connector.id}`,
          source: sourceNodeId,
          target: targetNodeId,
          sourceHandle: `interface-out-${outgoingInterface.id}`,
          targetHandle: `interface-in-${incomingInterface.id}`,
          type: 'smoothstep',
          label: connector.display_name || connector.name,
          style: { 
            stroke: '#14b8a6', 
            strokeWidth: 3,
          },
          labelStyle: {
            fill: '#14b8a6',
            fontWeight: 600,
          },
        };
        edges.push(edge);
      }
    });

    // Auto-layout using dagre
    const g = new dagre.graphlib.Graph();
    g.setDefaultEdgeLabel(() => ({}));
    g.setGraph({ rankdir: 'LR', nodesep: 100, ranksep: 150 });

    nodes.forEach((node) => {
      g.setNode(node.id, { width: 300, height: 200 });
    });

    edges.forEach((edge) => {
      g.setEdge(edge.source, edge.target);
    });

    dagre.layout(g);

    // Update node positions
    nodes.forEach((node) => {
      const nodeWithPosition = g.node(node.id);
      if (nodeWithPosition) {
        node.position = {
          x: nodeWithPosition.x - 150, // Center the node
          y: nodeWithPosition.y - 100,
        };
      }
    });

    return { initialNodes: nodes, initialEdges: edges };
  }, [workflowsData, interfacesByService, projectConnectors, projectId]);

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

  // Handle node click to navigate to service builder
  const onNodeClick = useMemo(
    () => (event: React.MouseEvent, node: Node) => {
      if (node.type === 'service-container' && node.data) {
        const serviceId = node.data.serviceId;
        router.push(`/workflows/${serviceId}`);
      }
    },
    [router]
  );

  if (isLoading) {
    return (
      <YStack flex={1} alignItems="center" justifyContent="center" gap="$3">
        <Spinner size="large" />
        <Text>Loading project view...</Text>
      </YStack>
    );
  }

  return (
    <div style={{ width: '100%', height: '100%', minHeight: '600px' }}>
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onConnect={onConnect}
        onNodeClick={onNodeClick}
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

