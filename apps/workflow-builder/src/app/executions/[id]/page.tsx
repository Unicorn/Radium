'use client';

import { useParams, useRouter } from 'next/navigation';
import { api } from '@/lib/trpc/client';
import { YStack, XStack, Text, Card, Button, Separator, Spinner } from 'tamagui';
import { ExecutionTimeline } from '@/components/execution/ExecutionTimeline';
import { ExecutionOutput } from '@/components/execution/ExecutionOutput';
import { ExecutionError } from '@/components/execution/ExecutionError';
import { ExecutionMetadata } from '@/components/execution/ExecutionMetadata';
import { ArrowLeft, RefreshCw } from 'lucide-react';

export default function ExecutionDetailPage() {
  const params = useParams();
  const router = useRouter();
  const executionId = params.id as string;

  // Initial fetch
  const { data: execution, isLoading, refetch } = api.execution.getExecutionDetails.useQuery({
    executionId,
  });

  // Poll for updates if execution is running
  const { data: liveExecution } = api.execution.getExecutionDetails.useQuery(
    { executionId },
    {
      enabled: execution?.status === 'running' || execution?.status === 'building',
      refetchInterval: 2000, // Poll every 2 seconds
    }
  );

  const currentExecution = liveExecution || execution;

  const handleBack = () => {
    router.push('/executions');
  };

  const handleViewWorkflow = () => {
    if (currentExecution?.workflowId) {
      router.push(`/workflows/${currentExecution.workflowId}`);
    }
  };

  const handleRefresh = () => {
    void refetch();
  };

  if (isLoading) {
    return (
      <YStack f={1} ai="center" jc="center" p="$6">
        <Spinner size="large" />
        <Text mt="$3" color="$gray11">Loading execution details...</Text>
      </YStack>
    );
  }

  if (!currentExecution) {
    return (
      <YStack f={1} ai="center" jc="center" p="$6">
        <Card p="$6" bg="$red2" borderWidth={1} borderColor="$red6">
          <Text color="$red11" fontSize="$5" fontWeight="600">
            Execution not found
          </Text>
          <Button mt="$4" onPress={handleBack}>
            Back to Executions
          </Button>
        </Card>
      </YStack>
    );
  }

  const isRunning = currentExecution.status === 'running' || currentExecution.status === 'building';

  return (
    <YStack f={1} p="$6" gap="$6">
      {/* Header */}
      <XStack ai="center" jc="space-between" gap="$4">
        <XStack ai="center" gap="$3">
          <Button
            icon={ArrowLeft}
            size="$3"
            chromeless
            onPress={handleBack}
          >
            Back
          </Button>
          <YStack gap="$2">
            <Text fontSize="$8" fontWeight="700">
              Execution Details
            </Text>
            <Text color="$gray11" fontSize="$2" fontFamily="$mono">
              ID: {currentExecution.id}
            </Text>
          </YStack>
        </XStack>

        <XStack gap="$3">
          {isRunning && (
            <Card px="$3" py="$2" bg="$blue2" borderWidth={1} borderColor="$blue6">
              <XStack gap="$2" ai="center">
                <Spinner size="small" color="$blue11" />
                <Text color="$blue11" fontSize="$3" fontWeight="600">
                  Live Updates Active
                </Text>
              </XStack>
            </Card>
          )}
          <Button
            icon={RefreshCw}
            size="$3"
            onPress={handleRefresh}
          >
            Refresh
          </Button>
          <Button
            size="$3"
            onPress={handleViewWorkflow}
          >
            View Workflow
          </Button>
        </XStack>
      </XStack>

      <Separator />

      {/* Main Content Grid */}
      <XStack gap="$6" w="100%" $gtSm={{ flexDirection: 'row' }} $sm={{ flexDirection: 'column' }}>
        {/* Left Column - Timeline and Output */}
        <YStack f={2} gap="$6">
          <ExecutionTimeline execution={currentExecution} />

          {currentExecution.status === 'completed' && currentExecution.output && (
            <ExecutionOutput output={currentExecution.output} />
          )}

          {currentExecution.status === 'failed' && currentExecution.error && (
            <ExecutionError error={currentExecution.error} />
          )}

          {/* Component Executions */}
          {currentExecution.componentExecutions && currentExecution.componentExecutions.length > 0 && (
            <Card p="$4">
              <Text fontSize="$5" fontWeight="600" mb="$4">
                Component Executions
              </Text>
              <YStack gap="$3">
                {currentExecution.componentExecutions.map((comp: ComponentExecution) => (
                  <ComponentExecutionCard key={comp.id} component={comp} />
                ))}
              </YStack>
            </Card>
          )}
        </YStack>

        {/* Right Column - Metadata */}
        <YStack f={1} minWidth={300}>
          <ExecutionMetadata execution={currentExecution} />
        </YStack>
      </XStack>
    </YStack>
  );
}

interface ComponentExecution {
  id: string;
  componentName?: string;
  nodeId?: string;
  status: string;
  durationMs?: number | null;
  errorMessage?: string;
}

function ComponentExecutionCard({ component }: { component: ComponentExecution }) {
  const getStatusColor = (status: string) => {
    const colors: Record<string, { bg: string; text: string; border: string }> = {
      completed: { bg: '$green2', text: '$green11', border: '$green6' },
      failed: { bg: '$red2', text: '$red11', border: '$red6' },
      running: { bg: '$blue2', text: '$blue11', border: '$blue6' },
    };
    return colors[status] || { bg: '$gray2', text: '$gray11', border: '$gray6' };
  };

  const statusColors = getStatusColor(component.status);

  return (
    <Card p="$3" bg="$gray2" borderWidth={1} borderColor="$borderColor">
      <YStack gap="$2">
        <XStack ai="center" jc="space-between">
          <Text fontSize="$3" fontWeight="600">
            {component.componentName || component.nodeId || 'Unknown Component'}
          </Text>
          <Card px="$2" py="$1" bg={statusColors.bg} borderWidth={1} borderColor={statusColors.border}>
            <Text color={statusColors.text} fontSize="$1" fontWeight="600" textTransform="capitalize">
              {component.status}
            </Text>
          </Card>
        </XStack>

        {component.durationMs !== null && component.durationMs !== undefined && (
          <Text fontSize="$2" color="$gray11">
            Duration: {formatDuration(component.durationMs)}
          </Text>
        )}

        {component.errorMessage && (
          <Card p="$2" bg="$red2" borderWidth={1} borderColor="$red6">
            <Text fontSize="$2" color="$red11">
              {component.errorMessage}
            </Text>
          </Card>
        )}
      </YStack>
    </Card>
  );
}

function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`;
  if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
  if (ms < 3600000) return `${(ms / 60000).toFixed(1)}m`;
  return `${(ms / 3600000).toFixed(1)}h`;
}
