/**
 * Workflow Canvas - Main visual workflow editor
 * Integrates with @bernierllc/workflow-ui when available
 */

'use client';

import { useState, useCallback, useEffect, useRef } from 'react';
import { YStack, XStack } from 'tamagui';
import { api } from '@/lib/trpc/client';
import { ComponentPalette } from './ComponentPalette';
import { PropertyPanel } from './PropertyPanel';
import { WorkflowToolbar } from './WorkflowToolbar';
import { CodeViewerModal } from './CodeViewerModal';
import type { WorkflowDefinition, WorkflowNode } from '@/types/workflow';
import ReactFlow, {
  Background,
  Controls,
  MiniMap,
  addEdge,
  useNodesState,
  useEdgesState,
  type Connection,
  type Edge,
  type Node,
} from 'react-flow-renderer';
import { nodeTypes } from './nodes';

interface WorkflowCanvasProps {
  workflowId: string;
  initialDefinition?: WorkflowDefinition;
  readOnly?: boolean;
}

export function WorkflowCanvas({
  workflowId,
  initialDefinition,
  readOnly = false,
}: WorkflowCanvasProps) {
  const [nodes, setNodes, onNodesChange] = useNodesState(initialDefinition?.nodes || []);
  const [edges, setEdges, onEdgesChange] = useEdgesState(initialDefinition?.edges || []);
  const [selectedNode, setSelectedNode] = useState<WorkflowNode | null>(null);
  const [showCodeModal, setShowCodeModal] = useState(false);
  const [generatedCode, setGeneratedCode] = useState<{
    workflowCode?: string;
    activitiesCode?: string;
    workerCode?: string;
  }>({});

  // History management for undo/redo
  const [past, setPast] = useState<Array<{ nodes: Node[]; edges: Edge[] }>>([]);
  const [future, setFuture] = useState<Array<{ nodes: Node[]; edges: Edge[] }>>([]);
  const isApplyingHistoryRef = useRef(false);
  const lastHistoryRef = useRef<string>(JSON.stringify({ nodes: initialDefinition?.nodes || [], edges: initialDefinition?.edges || [] }));

  // Use refs to track the last saved state and avoid re-renders
  const lastSavedNodesRef = useRef<string>(JSON.stringify(initialDefinition?.nodes || []));
  const lastSavedEdgesRef = useRef<string>(JSON.stringify(initialDefinition?.edges || []));
  const autoSaveTimerRef = useRef<NodeJS.Timeout | null>(null);

  const utils = api.useUtils();
  const notificationMutation = api.notifications.sendSlack.useMutation();
  const markForDeletionMutation = api.kongCache.markForDeletion.useMutation();
  
  const saveMutation = api.workflows.update.useMutation({
    onSuccess: async () => {
      utils.workflows.get.invalidate({ id: workflowId });
      // Update last saved state on successful save
      lastSavedNodesRef.current = JSON.stringify(nodes);
      lastSavedEdgesRef.current = JSON.stringify(edges);
      
      // Send success notification if Slack webhook is configured
      const slackWebhook = process.env.NEXT_PUBLIC_SLACK_WEBHOOK_URL;
      if (slackWebhook) {
        await notificationMutation.mutateAsync({
          webhookUrl: slackWebhook,
          message: `💾 Workflow saved successfully`,
        }).catch(err => console.warn('Failed to send notification:', err));
      }
    },
    onError: async (error) => {
      // Send error notification if Slack webhook is configured
      const slackWebhook = process.env.NEXT_PUBLIC_SLACK_WEBHOOK_URL;
      if (slackWebhook) {
        await notificationMutation.mutateAsync({
          webhookUrl: slackWebhook,
          message: `❌ Failed to save workflow: ${error.message}`,
        }).catch(err => console.warn('Failed to send error notification:', err));
      }
    },
  });
  
  const deployMutation = api.workflows.deploy.useMutation({
    onSuccess: async () => {
      utils.workflows.get.invalidate({ id: workflowId });
      utils.workflows.list.invalidate();
      
      // Send success notification if Slack webhook is configured
      const slackWebhook = process.env.NEXT_PUBLIC_SLACK_WEBHOOK_URL;
      if (slackWebhook) {
        await notificationMutation.mutateAsync({
          webhookUrl: slackWebhook,
          message: `🚀 Workflow deployed successfully!`,
          attachments: [{
            title: 'Deployment Details',
            fields: [
              { title: 'Workflow ID', value: workflowId, short: true },
              { title: 'Status', value: 'Deployed', short: true },
            ],
          }],
        }).catch(err => console.warn('Failed to send notification:', err));
      }
    },
    onError: async (error) => {
      // Send error notification if Slack webhook is configured
      const slackWebhook = process.env.NEXT_PUBLIC_SLACK_WEBHOOK_URL;
      if (slackWebhook) {
        await notificationMutation.mutateAsync({
          webhookUrl: slackWebhook,
          message: `❌ Workflow deployment failed: ${error.message}`,
          attachments: [{
            color: 'danger',
            title: 'Deployment Error',
            text: error.message,
          }],
        }).catch(err => console.warn('Failed to send error notification:', err));
      }
    },
  });

  const pauseMutation = api.workflows.pause.useMutation({
    onSuccess: () => {
      utils.workflows.get.invalidate({ id: workflowId });
      utils.workflows.list.invalidate();
      // Reload the page to re-enter edit mode
      window.location.reload();
    },
  });

  const compileMutation = api.compiler.compile.useMutation({
    onSuccess: async (result) => {
      if (result.success && result.compiled) {
        setGeneratedCode({
          workflowCode: result.compiled.workflowCode,
          activitiesCode: result.compiled.activitiesCode,
          workerCode: result.compiled.workerCode,
        });
        setShowCodeModal(true);
        console.log('✅ Workflow compiled successfully! View the generated TypeScript code');
        
        // Send success notification if Slack webhook is configured
        const slackWebhook = process.env.NEXT_PUBLIC_SLACK_WEBHOOK_URL;
        if (slackWebhook) {
          await notificationMutation.mutateAsync({
            webhookUrl: slackWebhook,
            message: `✅ Workflow compiled successfully!`,
            attachments: [{
              title: 'Compilation Details',
              fields: [
                { title: 'Workflow ID', value: workflowId, short: true },
                { title: 'Status', value: 'Compiled', short: true },
              ],
            }],
          }).catch(err => console.warn('Failed to send notification:', err));
        }
      }
    },
    onError: async (error) => {
      console.error('❌ Compilation failed:', error.message);
      
      // Send error notification if Slack webhook is configured
      const slackWebhook = process.env.NEXT_PUBLIC_SLACK_WEBHOOK_URL;
      if (slackWebhook) {
        await notificationMutation.mutateAsync({
          webhookUrl: slackWebhook,
          message: `❌ Workflow compilation failed: ${error.message}`,
          attachments: [{
            color: 'danger',
            title: 'Compilation Error',
            text: error.message,
          }],
        }).catch(err => console.warn('Failed to send error notification:', err));
      }
    },
  });

  // Load components for palette
  const { data: componentsData } = api.components.list.useQuery({
    includeDeprecated: false,
  });

  // Export workflow in compiler-compatible format
  const exportWorkflow = useCallback(() => {
    return {
      id: workflowId,
      nodes: nodes.map((node) => ({
        id: node.id,
        type: node.type,
        data: node.data,
        position: node.position,
      })),
      edges: edges,
      variables: [], // Will be populated in M2
      settings: {
        aiRemediation: false, // Will be enabled in M3
        maxConcurrent: 1, // Will be configurable in M4
      },
    };
  }, [workflowId, nodes, edges]);

  const handleSave = useCallback(async () => {
    const definition: WorkflowDefinition = {
      nodes: nodes as WorkflowNode[],
      edges: edges as any[], // Fix: edges should be Edge[], not WorkflowNode[]
    };

    await saveMutation.mutateAsync({
      id: workflowId,
      definition,
    });

    // Update last saved state after manual save
    lastSavedNodesRef.current = JSON.stringify(nodes);
    lastSavedEdgesRef.current = JSON.stringify(edges);
  }, [workflowId, nodes, edges, saveMutation]);

  const handleCompile = useCallback(async () => {
    await handleSave();
    await compileMutation.mutateAsync({
      workflowId,
      includeComments: true,
      strictMode: true,
    });
  }, [workflowId, handleSave, compileMutation]);

  const handleDeploy = useCallback(async () => {
    await handleSave();
    await deployMutation.mutateAsync({ id: workflowId });
  }, [workflowId, handleSave, deployMutation]);

  const handleEnterEditMode = useCallback(async () => {
    await pauseMutation.mutateAsync({ id: workflowId });
  }, [workflowId, pauseMutation]);

  // Save current state to history before making changes
  const saveToHistory = useCallback(() => {
    if (readOnly || isApplyingHistoryRef.current) return;

    const currentState = { nodes, edges };
    const currentStateStr = JSON.stringify(currentState);

    // Only save if state actually changed
    if (currentStateStr !== lastHistoryRef.current) {
      setPast((prev) => [...prev, JSON.parse(lastHistoryRef.current)]);
      setFuture([]); // Clear future when new action is performed
      lastHistoryRef.current = currentStateStr;
    }
  }, [nodes, edges, readOnly]);

  // Undo function
  const handleUndo = useCallback(() => {
    if (past.length === 0 || readOnly) return;

    isApplyingHistoryRef.current = true;
    const previous = past[past.length - 1];
    if (!previous) return;
    
    const newPast = past.slice(0, past.length - 1);

    setPast(newPast);
    setFuture((prev) => [{ nodes, edges }, ...prev]);
    setNodes(previous.nodes);
    setEdges(previous.edges);
    lastHistoryRef.current = JSON.stringify(previous);

    setTimeout(() => {
      isApplyingHistoryRef.current = false;
    }, 0);
  }, [past, nodes, edges, readOnly, setNodes, setEdges]);

  // Redo function
  const handleRedo = useCallback(() => {
    if (future.length === 0 || readOnly) return;

    isApplyingHistoryRef.current = true;
    const next = future[0];
    if (!next) return;
    
    const newFuture = future.slice(1);

    setPast((prev) => [...prev, { nodes, edges }]);
    setFuture(newFuture);
    setNodes(next.nodes);
    setEdges(next.edges);
    lastHistoryRef.current = JSON.stringify(next);

    setTimeout(() => {
      isApplyingHistoryRef.current = false;
    }, 0);
  }, [future, nodes, edges, readOnly, setNodes, setEdges]);

  const onConnect = useCallback(
    (connection: Connection) => {
      if (readOnly) return;
      saveToHistory();
      setEdges((eds) => addEdge(connection, eds));
    },
    [setEdges, readOnly, saveToHistory]
  );

  const onNodeClick = useCallback((_event: React.MouseEvent, node: Node) => {
    setSelectedNode(node as any);
  }, []);

  const createServiceInterfaceMutation = api.serviceInterfaces.createFromComponent.useMutation();

  const onDrop = useCallback(
    async (event: React.DragEvent) => {
      if (readOnly) return;

      event.preventDefault();

      const reactFlowBounds = event.currentTarget.getBoundingClientRect();
      const componentData = event.dataTransfer.getData('application/json');

      if (!componentData) return;

      const component = JSON.parse(componentData);
      const position = {
        x: event.clientX - reactFlowBounds.left,
        y: event.clientY - reactFlowBounds.top,
      };

      // Extract endpoint config from component
      const config = component.config_schema || {};
      const componentTypeName = component.component_type?.name || component.component_type_id;
      
      // Generate endpoint path if missing for interface components
      let endpointPath = config.endpointPath || '';
      if ((componentTypeName === 'data-in' || componentTypeName === 'data-out') && !endpointPath) {
        // Import generateEndpointPath dynamically to avoid circular dependencies
        const { generateEndpointPath } = await import('@/lib/interfaces/endpoint-path-generator');
        endpointPath = generateEndpointPath(component.name || component.display_name);
      }
      
      const httpMethod = config.httpMethod || (componentTypeName === 'data-in' ? 'POST' : 'GET');

      const newNode: WorkflowNode = {
        id: `${component.id}-${Date.now()}`,
        type: (componentTypeName || 'activity') as any,
        position,
        data: {
          label: component.display_name,
          displayName: component.display_name,
          componentId: component.id,
          componentName: component.name,
          config: {
            endpointPath,
            httpMethod,
            ...config,
          },
        },
      };

      saveToHistory();
      setNodes((nds) => [...nds, newNode as any]);

      // Auto-create service interface for interface components
      if (componentTypeName === 'data-in' || componentTypeName === 'data-out') {
        if (endpointPath) {
          try {
            await createServiceInterfaceMutation.mutateAsync({
              componentId: component.id,
              workflowId,
              endpointPath,
              httpMethod: httpMethod as 'GET' | 'POST' | 'PATCH' | 'PUT' | 'DELETE',
              isPublic: true, // Default to public for interface components
            });
            // Invalidate service interfaces to refresh UI
            utils.serviceInterfaces.list.invalidate({ serviceId: workflowId });
          } catch (error) {
            console.error('Failed to create service interface:', error);
            // Don't block node creation if interface creation fails
          }
        }
      }
    },
    [setNodes, readOnly, saveToHistory, workflowId, createServiceInterfaceMutation, utils]
  );

  const onDragOver = useCallback((event: React.DragEvent) => {
    event.preventDefault();
    event.dataTransfer.dropEffect = readOnly ? 'none' : 'move';
  }, [readOnly]);

  // Auto-save on changes (debounced) - only saves after changes stop for 1.5 seconds
  useEffect(() => {
    if (readOnly) return;

    // Clear any existing timer
    if (autoSaveTimerRef.current) {
      clearTimeout(autoSaveTimerRef.current);
    }

    // Check if there are actual changes from the last saved state
    const currentNodesStr = JSON.stringify(nodes);
    const currentEdgesStr = JSON.stringify(edges);
    const hasChanges =
      currentNodesStr !== lastSavedNodesRef.current ||
      currentEdgesStr !== lastSavedEdgesRef.current;

    // Only set a timer if there are changes
    if (hasChanges && (nodes.length > 0 || edges.length > 0)) {
      autoSaveTimerRef.current = setTimeout(() => {
    const definition: WorkflowDefinition = {
      nodes: nodes as WorkflowNode[],
      edges: edges as any[], // Fix: edges should be Edge[], not WorkflowNode[]
    };

        saveMutation.mutate({
          id: workflowId,
          definition,
        });
      }, 1500); // 1.5 second delay after changes stop
    }

    return () => {
      if (autoSaveTimerRef.current) {
        clearTimeout(autoSaveTimerRef.current);
      }
    };
  }, [nodes, edges, workflowId, readOnly]);

  // Keyboard shortcuts for delete, undo, redo
  useEffect(() => {
    if (readOnly) return;

    const handleKeyDown = (event: KeyboardEvent) => {
      // Delete key
      if (event.key === 'Delete' || event.key === 'Backspace') {
        const selectedNodes = nodes.filter((node) => (node as any).selected);
        const selectedEdges = edges.filter((edge) => (edge as any).selected);

        if (selectedNodes.length > 0 || selectedEdges.length > 0) {
          saveToHistory();
          
          // Mark kong-cache components for deletion before removing
          selectedNodes.forEach((node) => {
            if (node.type === 'kong-cache' && node.data?.config?.isSaved) {
              const projectId = node.data?.projectId || initialDefinition?.projectId;
              if (projectId && node.id) {
                markForDeletionMutation.mutate({
                  componentId: node.id,
                  projectId,
                }, {
                  onError: (error) => {
                    console.error('Failed to mark cache key for deletion:', error);
                  },
                });
              }
            }
          });
          
          setNodes((nds) => nds.filter((node) => !(node as any).selected));
          setEdges((eds) => eds.filter((edge) => !(edge as any).selected));
          setSelectedNode(null);
        }
      }

      // Undo: Cmd+Z or Ctrl+Z
      if ((event.metaKey || event.ctrlKey) && event.key === 'z' && !event.shiftKey) {
        event.preventDefault();
        handleUndo();
      }

      // Redo: Cmd+Shift+Z or Ctrl+Shift+Z or Ctrl+Y
      if (
        ((event.metaKey || event.ctrlKey) && event.shiftKey && event.key === 'z') ||
        (event.ctrlKey && event.key === 'y')
      ) {
        event.preventDefault();
        handleRedo();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [nodes, edges, readOnly, saveToHistory, setNodes, setEdges, handleUndo, handleRedo]);

  // Track changes to nodes/edges for history
  useEffect(() => {
    if (readOnly || isApplyingHistoryRef.current) return;

    const currentState = JSON.stringify({ nodes, edges });
    if (currentState !== lastHistoryRef.current) {
      // Save to history after a short delay to batch rapid changes
      const timeoutId = setTimeout(() => {
        saveToHistory();
      }, 300);

      return () => clearTimeout(timeoutId);
    }
  }, [nodes, edges, readOnly, saveToHistory]);

  // Reset history when workflow changes
  useEffect(() => {
    setPast([]);
    setFuture([]);
    lastHistoryRef.current = JSON.stringify({ nodes, edges });
  }, [workflowId]);

  return (
    <XStack flex={1} height="100vh">
      {/* Component Palette (Left Sidebar) */}
      <ComponentPalette 
        components={componentsData?.components || []} 
        disabled={readOnly}
      />

      {/* Main Canvas (Center) */}
      <YStack flex={1} position="relative">
        <WorkflowToolbar
          onSave={handleSave}
          onCompile={handleCompile}
          onDeploy={handleDeploy}
          onEnterEditMode={handleEnterEditMode}
          onUndo={handleUndo}
          onRedo={handleRedo}
          canUndo={past.length > 0}
          canRedo={future.length > 0}
          isSaving={saveMutation.isLoading}
          isCompiling={compileMutation.isLoading}
          isDeploying={deployMutation.isLoading}
          readOnly={readOnly}
        />

        <div style={{ flex: 1, position: 'relative' }} onDrop={onDrop} onDragOver={onDragOver}>
          <ReactFlow
            nodes={nodes}
            edges={edges}
            onNodesChange={readOnly ? undefined : onNodesChange}
            onEdgesChange={readOnly ? undefined : onEdgesChange}
            onConnect={onConnect}
            onNodeClick={onNodeClick}
            nodeTypes={nodeTypes}
            fitView
            fitViewOptions={{ maxZoom: 0.75 }}
            nodesDraggable={!readOnly}
            nodesConnectable={!readOnly}
            elementsSelectable={!readOnly}
          >
            <Background />
            <Controls />
            <MiniMap />
          </ReactFlow>
          
          {/* Read-only overlay */}
          {readOnly && (
            <div
              style={{
                position: 'absolute',
                top: 0,
                left: 0,
                right: 0,
                bottom: 0,
                backgroundColor: 'rgba(0, 0, 0, 0.05)',
                backdropFilter: 'blur(1px)',
                pointerEvents: 'all',
                cursor: 'not-allowed',
                zIndex: 5,
              }}
            />
          )}
        </div>
      </YStack>

      {/* Property Panel (Right Sidebar) */}
      {selectedNode && !readOnly && (
        <PropertyPanel
          node={selectedNode}
          onUpdate={(updated) => {
            setNodes((nds) =>
              nds.map((n) =>
                n.id === selectedNode.id
                  ? { ...n, data: { ...n.data, ...updated.data } }
                  : n
              )
            );
          }}
          onClose={() => setSelectedNode(null)}
        />
      )}

      {/* Code Viewer Modal */}
      <CodeViewerModal
        code={generatedCode}
        open={showCodeModal}
        onClose={() => setShowCodeModal(false)}
      />
    </XStack>
  );
}

