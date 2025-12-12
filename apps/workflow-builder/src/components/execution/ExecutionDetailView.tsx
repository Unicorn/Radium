'use client';

import { YStack, XStack, Text, Card, ScrollView, Separator } from 'tamagui';
import { Clock, CheckCircle, XCircle, RefreshCw } from 'lucide-react';
import { api } from '@/lib/trpc/client';
import { format } from 'date-fns';

interface ExecutionDetailViewProps {
  executionId: string;
  workflowId: string;
}

export function ExecutionDetailView({ executionId }: ExecutionDetailViewProps) {
  const { data, isLoading, error } = api.execution.getExecutionDetails.useQuery({
    executionId,
  });

  if (isLoading) {
    return (
      <Card p="$4">
        <Text>Loading execution details...</Text>
      </Card>
    );
  }

  if (error || !data) {
    return (
      <Card p="$4" bg="$red2">
        <Text color="$red11">Error loading execution details</Text>
      </Card>
    );
  }

  const execution = data;
  const components = data.componentExecutions || [];

  return (
    <ScrollView f={1}>
      <YStack gap="$4" p="$4">
        {/* Execution Header */}
        <Card p="$4" bg="$gray2">
          <YStack gap="$3">
            <XStack ai="center" jc="space-between">
              <Text fontSize="$5" fontWeight="600">Execution Details</Text>
              <StatusBadge status={execution.status} />
            </XStack>
            
            <Separator />
            
            <YStack gap="$2">
              <XStack gap="$2" ai="center">
                <Clock size={16} color="$gray11" />
                <Text fontSize="$2" color="$gray11">Started</Text>
                <Text fontSize="$2">
                  {execution.startedAt ? format(new Date(execution.startedAt), 'PPpp') : 'N/A'}
                </Text>
              </XStack>
              
              {execution.completedAt && (
                <XStack gap="$2" ai="center">
                  <Clock size={16} color="$gray11" />
                  <Text fontSize="$2" color="$gray11">Completed</Text>
                  <Text fontSize="$2">
                    {format(new Date(execution.completedAt), 'PPpp')}
                  </Text>
                </XStack>
              )}
              
              {execution.durationMs && (
                <XStack gap="$2" ai="center">
                  <Clock size={16} color="$gray11" />
                  <Text fontSize="$2" color="$gray11">Duration</Text>
                  <Text fontSize="$2">{formatDuration(execution.durationMs)}</Text>
                </XStack>
              )}
            </YStack>

            {execution.errorMessage && (
              <Card p="$3" bg="$red2" borderWidth={1} borderColor="$red6">
                <XStack gap="$2" ai="center">
                  <XCircle size={16} color="$red11" />
                  <Text fontSize="$2" color="$red11" fontWeight="600">Error</Text>
                </XStack>
                <Text fontSize="$2" color="$red11" mt="$2">
                  {execution.errorMessage}
                </Text>
              </Card>
            )}
          </YStack>
        </Card>

        {/* Component Executions */}
        <YStack gap="$3">
          <Text fontSize="$4" fontWeight="600">Component Executions</Text>
          
          {components.length === 0 ? (
            <Card p="$4" bg="$gray2">
              <Text color="$gray11">No component executions recorded yet</Text>
            </Card>
          ) : (
            components.map((comp) => (
              <ComponentExecutionCard key={comp.id} component={comp} />
            ))
          )}
        </YStack>
      </YStack>
    </ScrollView>
  );
}

function StatusBadge({ status }: { status: string }) {
  const statusConfig: Record<string, { icon: React.ComponentType<{ size?: number; color?: string }>; color: string; bg: string }> = {
    completed: { icon: CheckCircle, color: '$green11', bg: '$green2' },
    failed: { icon: XCircle, color: '$red11', bg: '$red2' },
    running: { icon: RefreshCw, color: '$blue11', bg: '$blue2' },
    pending: { icon: Clock, color: '$gray11', bg: '$gray2' },
  };

  const config = statusConfig[status] || statusConfig.pending;
  const Icon = config.icon;

  return (
    <Card p="$2" bg={config.bg} borderWidth={1} borderColor={config.color}>
      <XStack gap="$2" ai="center">
        <Icon size={14} color={config.color} />
        <Text fontSize="$2" color={config.color} textTransform="capitalize">
          {status}
        </Text>
      </XStack>
    </Card>
  );
}

interface ComponentExecution {
  id: string;
  componentName?: string;
  nodeId?: string;
  status: string;
  startedAt?: string;
  completedAt?: string;
  durationMs?: number;
  retryCount?: number;
  isExpectedRetry?: boolean;
  inputData?: unknown;
  outputData?: unknown;
  errorMessage?: string;
  errorType?: string;
  config?: unknown;
}

function ComponentExecutionCard({ component }: { component: ComponentExecution }) {
  return (
    <Card p="$4" bg="$gray2">
      <YStack gap="$3">
        <XStack ai="center" jc="space-between">
          <YStack gap="$1">
            <Text fontSize="$3" fontWeight="600">{component.componentName || component.nodeId}</Text>
            {component.nodeId && (
              <Text fontSize="$1" color="$gray11" fontFamily="$mono">Node: {component.nodeId}</Text>
            )}
          </YStack>
          <StatusBadge status={component.status} />
        </XStack>

        <Separator />

        <YStack gap="$2">
          {component.startedAt && (
            <XStack gap="$2" ai="center">
              <Clock size={14} color="$gray11" />
              <Text fontSize="$1" color="$gray11">Started</Text>
              <Text fontSize="$1">
                {format(new Date(component.startedAt), 'PPpp')}
              </Text>
            </XStack>
          )}

          {component.completedAt && (
            <XStack gap="$2" ai="center">
              <Clock size={14} color="$gray11" />
              <Text fontSize="$1" color="$gray11">Completed</Text>
              <Text fontSize="$1">
                {format(new Date(component.completedAt), 'PPpp')}
              </Text>
            </XStack>
          )}

          {component.durationMs && (
            <XStack gap="$2" ai="center">
              <Clock size={14} color="$gray11" />
              <Text fontSize="$1" color="$gray11">Duration</Text>
              <Text fontSize="$1">{formatDuration(component.durationMs)}</Text>
            </XStack>
          )}

          {component.retryCount > 0 && (
            <XStack gap="$2" ai="center">
              <RefreshCw size={14} color="$orange11" />
              <Text fontSize="$1" color="$orange11">
                Retries: {component.retryCount}
                {component.isExpectedRetry && ' (expected)'}
              </Text>
            </XStack>
          )}
        </YStack>

        {component.inputData && (
          <YStack gap="$2">
            <Text fontSize="$2" fontWeight="600">Input</Text>
            <Card p="$2" bg="$background" borderWidth={1} borderColor="$borderColor">
              <Text fontSize="$1" fontFamily="$mono" numberOfLines={5}>
                {JSON.stringify(component.inputData, null, 2)}
              </Text>
            </Card>
          </YStack>
        )}

        {component.outputData && (
          <YStack gap="$2">
            <Text fontSize="$2" fontWeight="600">Output</Text>
            <Card p="$2" bg="$background" borderWidth={1} borderColor="$borderColor">
              <Text fontSize="$1" fontFamily="$mono" numberOfLines={5}>
                {JSON.stringify(component.outputData, null, 2)}
              </Text>
            </Card>
          </YStack>
        )}

        {component.errorMessage && (
          <Card p="$3" bg="$red2" borderWidth={1} borderColor="$red6">
            <XStack gap="$2" ai="center">
              <XCircle size={14} color="$red11" />
              <Text fontSize="$2" color="$red11" fontWeight="600">Error</Text>
            </XStack>
            <Text fontSize="$1" color="$red11" mt="$2">
              {component.errorMessage}
            </Text>
            {component.errorType && (
              <Text fontSize="$1" color="$red11" mt="$1" fontFamily="$mono">
                Type: {component.errorType}
              </Text>
            )}
          </Card>
        )}
      </YStack>
    </Card>
  );
}

function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`;
  if (ms < 60000) return `${(ms / 1000).toFixed(2)}s`;
  if (ms < 3600000) return `${(ms / 60000).toFixed(2)}m`;
  return `${(ms / 3600000).toFixed(2)}h`;
}

