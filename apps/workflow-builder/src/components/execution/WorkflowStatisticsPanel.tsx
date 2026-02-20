'use client';

import { YStack, XStack, Text, Card, ScrollView } from 'tamagui';
import { BarChart3, Clock, AlertCircle, CheckCircle, XCircle } from 'lucide-react';
import { api } from '@/lib/trpc/client';
import { format } from 'date-fns';

interface WorkflowStatisticsPanelProps {
  workflowId: string;
}

// Define the expected statistics type
interface WorkflowStatistics {
  total_runs: number;
  successful_runs: number;
  failed_runs: number;
  avg_duration_ms: number | null;
  min_duration_ms: number | null;
  max_duration_ms: number | null;
  most_used_component_id: string | null;
  most_used_component_count: number;
  total_errors: number;
  last_error_at: string | null;
  last_run_at: string | null;
}

export function WorkflowStatisticsPanel({ workflowId }: WorkflowStatisticsPanelProps) {
  const { data, isLoading, error } = api.execution.getWorkflowStatistics.useQuery({
    workflowId,
  });

  if (isLoading) {
    return (
      <Card p="$4">
        <Text>Loading statistics...</Text>
      </Card>
    );
  }

  if (error || !data) {
    return (
      <Card p="$4" bg="$red2">
        <Text color="$red11">Error loading statistics</Text>
      </Card>
    );
  }

  // Type assertion to narrow down the union type from tRPC
  const typedData = data as unknown as WorkflowStatistics;

  // Map database fields to camelCase
  const stats = {
    totalRuns: typedData.total_runs || 0,
    averageDurationMs: typedData.avg_duration_ms,
    successRate: typedData.total_runs
      ? (typedData.successful_runs || 0) / typedData.total_runs
      : null,
    errorCount: typedData.failed_runs || 0,
    mostUsedComponent: typedData.most_used_component_id
      ? {
          componentName: typedData.most_used_component_id,
          usageCount: typedData.most_used_component_count || 0,
        }
      : null,
    recentExecutions: [], // TODO: Add recent executions query
  };

  return (
    <ScrollView f={1}>
      <YStack gap="$4" p="$4">
        <XStack ai="center" gap="$2">
          <BarChart3 size={20} />
          <Text fontSize="$5" fontWeight="600">Workflow Statistics</Text>
        </XStack>

        <YStack gap="$3">
          {/* Overview Cards */}
          <XStack gap="$3" flexWrap="wrap">
            <StatCard
              label="Total Runs"
              value={stats.totalRuns || 0}
              icon={BarChart3}
            />
            <StatCard
              label="Average Duration"
              value={stats.averageDurationMs ? formatDuration(stats.averageDurationMs) : 'N/A'}
              icon={Clock}
            />
            <StatCard
              label="Success Rate"
              value={stats.successRate ? `${(stats.successRate * 100).toFixed(1)}%` : 'N/A'}
              icon={CheckCircle}
            />
            <StatCard
              label="Error Count"
              value={stats.errorCount || 0}
              icon={XCircle}
              color="$red11"
            />
          </XStack>

          {/* Most Used Component */}
          {stats.mostUsedComponent && (
            <Card p="$4" bg="$gray2">
              <YStack gap="$2">
                <Text fontSize="$3" fontWeight="600">Most Used Component</Text>
                <Text fontSize="$2" color="$gray11">
                  {stats.mostUsedComponent.componentName}
                </Text>
                <Text fontSize="$1" color="$gray11">
                  Used {stats.mostUsedComponent.usageCount} times
                </Text>
              </YStack>
            </Card>
          )}

          {/* Recent Executions */}
          {stats.recentExecutions && stats.recentExecutions.length > 0 && (
            <Card p="$4" bg="$gray2">
              <YStack gap="$2">
                <Text fontSize="$3" fontWeight="600">Recent Executions</Text>
                {stats.recentExecutions.slice(0, 5).map((exec: any) => (
                  <XStack key={exec.id} ai="center" jc="space-between" p="$2">
                    <Text fontSize="$1" color="$gray11">
                      {exec.startedAt
                        ? format(new Date(exec.startedAt), 'PPp')
                        : 'N/A'}
                    </Text>
                    <Text fontSize="$1" color={exec.status === 'completed' ? '$green11' : '$red11'}>
                      {exec.status}
                    </Text>
                  </XStack>
                ))}
              </YStack>
            </Card>
          )}
        </YStack>
      </YStack>
    </ScrollView>
  );
}

function StatCard({
  label,
  value,
  icon: Icon,
  color = '$blue11',
}: {
  label: string;
  value: string | number;
  icon: any;
  color?: string;
}) {
  return (
    <Card p="$3" bg="$gray2" f={1} minWidth={150}>
      <YStack gap="$2">
        <XStack ai="center" gap="$2">
          <Icon size={16} color={color} />
          <Text fontSize="$1" color="$gray11">{label}</Text>
        </XStack>
        <Text fontSize="$4" fontWeight="600" color={color}>
          {value}
        </Text>
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

