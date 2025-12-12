'use client';

import { useParams, useRouter } from 'next/navigation';
import { api as trpc } from '@/lib/trpc/client';
import { YStack, XStack, Heading, Text, Spinner, Button, Dialog, Tabs } from 'tamagui';
import { Save, Play, Settings, Code2, Network, GitBranch, Inbox, Radio, Search } from 'lucide-react';
import { useState, useEffect } from 'react';
import { WorkflowCanvas } from '@/components/workflow/WorkflowCanvas';
import { ServiceBuilderView } from '@/components/service/ServiceBuilderView';
import { CodePreviewDialog } from '@/components/workflow-builder/CodePreviewDialog';
import { WorkflowExecutionPanel } from '@/components/workflow-execution/WorkflowExecutionPanel';
import { WorkerStatus } from '@/components/workflow/WorkerStatus';
import { WorkQueueConnectionVisualizer } from '@/components/workflow-builder/WorkQueueConnectionVisualizer';

export default function WorkflowBuilderPage() {
  const params = useParams();
  const router = useRouter();
  const workflowId = params.id as string;
  const [hasUnsavedChanges, setHasUnsavedChanges] = useState(false);
  const [showCodePreview, setShowCodePreview] = useState(false);
  const [showConnectionVisualizer, setShowConnectionVisualizer] = useState(false);
  const [compiledCode, setCompiledCode] = useState<any>(null);
  const [currentExecutionId, setCurrentExecutionId] = useState<string | null>(null);
  const [executionData, setExecutionData] = useState<any>(null);
  const [viewMode, setViewMode] = useState<'workflow' | 'service'>('workflow');

  // Fetch workflow details
  const { data: workflowData, isLoading } = trpc.workflows.get.useQuery({
    id: workflowId,
  });

  // Fetch available components for palette
  const { data: componentsData } = trpc.components.list.useQuery({
    page: 1,
    pageSize: 100,
  });

  // Optionally discover components from file system (if workspace path is available)
  // This would be used for discovering local component files
  // const { data: discoveredComponents } = trpc.fileOperations.findFiles.useQuery(
  //   {
  //     directory: process.env.NEXT_PUBLIC_WORKSPACE_PATH || './components',
  //     pattern: '**/*.ts',
  //     excludeDirs: ['node_modules', '.git', 'dist'],
  //   },
  //   {
  //     enabled: !!process.env.NEXT_PUBLIC_WORKSPACE_PATH,
  //   }
  // );

  // Fetch work queues
  const { data: workQueuesData } = trpc.workQueues.list.useQuery({
    workflowId,
  });

  // Fetch signals
  const { data: signalsData } = trpc.signals.list.useQuery({
    workflowId,
  });

  // Fetch queries
  const { data: queriesData } = trpc.queries.list.useQuery({
    workflowId,
  });

  // Compile mutation
  const compileMutation = trpc.compiler.compile.useMutation({
    onSuccess: (data) => {
      // Handle both success and error response types
      if (data.success && 'workflowCode' in data) {
        setCompiledCode({
          workflowCode: data.workflowCode,
          activitiesCode: data.activitiesCode,
          workerCode: data.workerCode,
        });
        setShowCodePreview(true);
      }
    },
  });

  // Notification mutation for user feedback
  const notificationMutation = trpc.notifications.sendSlack.useMutation();

  // Build/Execute mutation
  const buildMutation = trpc.execution.build.useMutation({
    onSuccess: async (data) => {
      setCurrentExecutionId(data.executionId);
      // Send success notification if Slack webhook is configured
      const slackWebhook = process.env.NEXT_PUBLIC_SLACK_WEBHOOK_URL;
      if (slackWebhook && workflowData?.workflow) {
        await notificationMutation.mutateAsync({
          webhookUrl: slackWebhook,
          message: `✅ Workflow "${(workflowData.workflow as any).display_name || (workflowData.workflow as any).name}" execution started successfully`,
          attachments: [{
            title: 'Execution Details',
            fields: [
              { title: 'Execution ID', value: data.executionId, short: true },
              { title: 'Workflow ID', value: workflowId, short: true },
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
          message: `❌ Failed to start workflow execution: ${error.message}`,
          attachments: [{
            color: 'danger',
            title: 'Error Details',
            text: error.message,
          }],
        }).catch(err => console.warn('Failed to send error notification:', err));
      }
    },
  });

  // Get execution status
  const { data: executionStatus } = trpc.execution.getStatus.useQuery(
    { executionId: currentExecutionId! },
    { 
      enabled: !!currentExecutionId,
      refetchInterval: (data) => {
        // Poll every second while building/running, stop when complete/failed
        return data?.status === 'building' || data?.status === 'running' ? 1000 : false;
      },
    }
  );

  // Update execution data when status changes
  useEffect(() => {
    if (executionStatus) {
      // Simulate execution steps for UI
      const mockSteps = [
        { id: '1', name: 'Compiling workflow', status: 'completed' as const, duration: 500 },
        { id: '2', name: 'Validating code', status: 'completed' as const, duration: 300 },
        { id: '3', name: 'Registering activities', status: executionStatus.status === 'building' ? 'running' as const : 'completed' as const, duration: 400 },
        { id: '4', name: 'Executing workflow', status: executionStatus.status === 'running' ? 'running' as const : executionStatus.status === 'completed' ? 'completed' as const : 'pending' as const },
      ];

      const progress = executionStatus.status === 'building' ? 50 : 
                      executionStatus.status === 'running' ? 75 : 
                      executionStatus.status === 'completed' ? 100 : 0;

      setExecutionData({
        id: currentExecutionId,
        status: executionStatus.status,
        startedAt: executionStatus.startedAt,
        completedAt: executionStatus.completedAt,
        steps: mockSteps,
        progress,
        result: executionStatus.result,
        error: executionStatus.error,
      });
    }
  }, [executionStatus, currentExecutionId]);

  // Save mutation
  const saveMutation = trpc.workflows.update.useMutation({
    onSuccess: async () => {
      setHasUnsavedChanges(false);
      
      // Send success notification if Slack webhook is configured
      const slackWebhook = process.env.NEXT_PUBLIC_SLACK_WEBHOOK_URL;
      if (slackWebhook && workflowData?.workflow) {
        await notificationMutation.mutateAsync({
          webhookUrl: slackWebhook,
          message: `💾 Workflow "${(workflowData.workflow as any).display_name || (workflowData.workflow as any).name}" saved successfully`,
        }).catch(err => console.warn('Failed to send notification:', err));
      }
    },
    onError: async (error) => {
      console.error('Failed to save workflow:', error);
      // Send error notification
      const slackWebhook = process.env.NEXT_PUBLIC_SLACK_WEBHOOK_URL;
      if (slackWebhook) {
        await notificationMutation.mutateAsync({
          webhookUrl: slackWebhook,
          message: `❌ Failed to save workflow: ${error.message}`,
        }).catch(err => console.warn('Failed to send error notification:', err));
      }
    },
  });

  const handleSave = async () => {
    if (!workflowData?.workflow) return;
    
    try {
      // Save workflow definition via tRPC
      await saveMutation.mutateAsync({
        id: workflowId,
        definition: (workflowData.workflow as any).definition,
      });
    } catch (error) {
      // Error handling is done in mutation callbacks
      console.error('Failed to save workflow:', error);
    }
  };

  const handleGenerateCode = async () => {
    await compileMutation.mutateAsync({
      workflowId,
      includeComments: true,
      strictMode: true,
    });
  };

  const handleBuildWorkflow = async () => {
    await buildMutation.mutateAsync({
      workflowId,
      input: {},
    });
  };

  const handleRetryExecution = async () => {
    setCurrentExecutionId(null);
    setExecutionData(null);
    await handleBuildWorkflow();
  };

  if (isLoading) {
    return (
      <YStack f={1} ai="center" jc="center" gap="$4">
        <Spinner size="large" />
        <Text>Loading workflow builder...</Text>
      </YStack>
    );
  }

  if (!workflowData?.workflow) {
    return (
      <YStack f={1} ai="center" jc="center" gap="$4">
        <Text fontSize="$6" color="$red10">Workflow not found</Text>
      </YStack>
    );
  }

  return (
    <YStack f={1} h="100vh">
      {/* Header */}
      <XStack
        ai="center"
        jc="space-between"
        p="$4"
        borderBottomWidth={1}
        borderBottomColor="$borderColor"
        bg="$background"
      >
        <YStack gap="$2" flex={1}>
          <Heading size="$6">{workflowData.workflow.display_name}</Heading>
          <XStack gap="$3" ai="center">
            <Text fontSize="$3" color="$gray11">
              {workflowData.workflow.description || 'No description'}
            </Text>
            {workflowData.workflow.project_id && (
              <WorkerStatus 
                projectId={workflowData.workflow.project_id}
                projectName={workflowData.workflow.project?.name || 'Project'}
              />
            )}
          </XStack>

          {/* Quick Nav Tabs */}
          <XStack gap="$2" mt="$2">
            <Button
              size="$2"
              chromeless
              pressStyle={{ backgroundColor: '$gray4' }}
              backgroundColor={viewMode === 'workflow' ? '$blue4' : 'transparent'}
              onPress={() => setViewMode('workflow')}
            >
              Workflow
            </Button>
            <Button
              size="$2"
              chromeless
              pressStyle={{ backgroundColor: '$gray4' }}
              backgroundColor={viewMode === 'service' ? '$blue4' : 'transparent'}
              onPress={() => setViewMode('service')}
            >
              Service View
            </Button>
            <Button
              size="$2"
              icon={GitBranch}
              chromeless
              pressStyle={{ backgroundColor: '$gray4' }}
              onPress={() => router.push(`/workflows/${workflowId}/child-workflows`)}
            >
              Child Workflows
            </Button>
            <Button
              size="$2"
              icon={Inbox}
              chromeless
              pressStyle={{ backgroundColor: '$gray4' }}
              onPress={() => router.push(`/workflows/${workflowId}/work-queues`)}
            >
              Work Queues
            </Button>
            <Button
              size="$2"
              icon={Radio}
              chromeless
              pressStyle={{ backgroundColor: '$gray4' }}
              onPress={() => router.push(`/workflows/${workflowId}/signals`)}
            >
              Signals
            </Button>
            <Button
              size="$2"
              icon={Search}
              chromeless
              pressStyle={{ backgroundColor: '$gray4' }}
              onPress={() => router.push(`/workflows/${workflowId}/queries`)}
            >
              Queries
            </Button>
          </XStack>
        </YStack>

        <XStack gap="$2" ai="center">
          {hasUnsavedChanges && (
            <Text fontSize="$2" color="$orange10">
              Unsaved changes
            </Text>
          )}
          
          <Button
            icon={Settings}
            size="$3"
            chromeless
          >
            Settings
          </Button>

          <Button
            icon={Save}
            size="$3"
            onPress={handleSave}
            disabled={!hasUnsavedChanges}
          >
            Save
          </Button>

          <Button
            icon={Play}
            size="$3"
            themeInverse
            onPress={handleBuildWorkflow}
            disabled={buildMutation.isLoading || executionData?.status === 'building' || executionData?.status === 'running'}
          >
            {buildMutation.isLoading ? 'Building...' : 'Build Workflow'}
          </Button>

          <Button
            icon={Code2}
            size="$3"
            onPress={handleGenerateCode}
            chromeless
          >
            View Code
          </Button>

          <Button
            icon={Network}
            size="$3"
            onPress={() => setShowConnectionVisualizer(true)}
            chromeless
          >
            Connections
          </Button>
        </XStack>
      </XStack>

      {/* Canvas + Execution Panel */}
      <XStack f={1} position="relative">
        {/* Workflow Canvas or Service Builder View */}
        <YStack f={1}>
          {viewMode === 'workflow' ? (
            <WorkflowCanvas
              workflowId={workflowId}
              initialDefinition={workflowData.workflow.definition}
              readOnly={false}
            />
          ) : (
            <ServiceBuilderView
              serviceId={workflowId}
              readOnly={false}
            />
          )}
        </YStack>

        {/* Execution Panel */}
        <WorkflowExecutionPanel
          workflowId={workflowId}
          execution={executionData}
          onRun={handleBuildWorkflow}
          onRetry={handleRetryExecution}
        />
      </XStack>

      {/* Code Preview Dialog */}
      {compiledCode && (
        <CodePreviewDialog
          open={showCodePreview}
          onOpenChange={setShowCodePreview}
          workflowCode={compiledCode.workflowCode}
          activitiesCode={compiledCode.activitiesCode}
          workerCode={compiledCode.workerCode}
          packageJson={compiledCode.packageJson}
          tsConfig={compiledCode.tsConfig}
        />
      )}

      {/* Connection Visualizer Dialog */}
      <Dialog
        open={showConnectionVisualizer}
        onOpenChange={setShowConnectionVisualizer}
      >
        <Dialog.Portal>
          <Dialog.Overlay
            key="overlay"
            animation="quick"
            opacity={0.5}
            enterStyle={{ opacity: 0 }}
            exitStyle={{ opacity: 0 }}
          />
          <Dialog.Content
            bordered
            elevate
            key="content"
            animateOnly={['transform', 'opacity']}
            animation={[
              'quick',
              {
                opacity: {
                  overshootClamping: true,
                },
              },
            ]}
            enterStyle={{ x: 0, y: -20, opacity: 0, scale: 0.9 }}
            exitStyle={{ x: 0, y: 10, opacity: 0, scale: 0.95 }}
            gap="$4"
            p={0}
          >
            <WorkQueueConnectionVisualizer
              nodes={[]} // TODO: Get actual nodes from workflow
              workflowId={workflowId}
              signals={signalsData?.signals}
              queries={queriesData?.queries}
              workQueues={workQueuesData?.workQueues}
              onClose={() => setShowConnectionVisualizer(false)}
            />
          </Dialog.Content>
        </Dialog.Portal>
      </Dialog>
    </YStack>
  );
}

