'use client';

import { YStack, XStack, Text, Card, ScrollView } from 'tamagui';
import { BarChart3, Clock, AlertCircle, CheckCircle, XCircle, TrendingUp } from 'lucide-react';
import { api } from '@/lib/trpc/client';
import { format } from 'date-fns';

interface ProjectStatisticsPanelProps {
  projectId: string;
}

export function ProjectStatisticsPanel({ projectId }: ProjectStatisticsPanelProps) {
  const { data: statsData, isLoading, error } = api.execution.getProjectStatistics.useQuery({
    projectId,
  });

  // Get workflow count for the project
  const { data: workflowsData } = api.workflows.list.useQuery({
    projectId,
  });

  if (isLoading) {
    return (
      <Card p="$4">
        <Text>Loading statistics...</Text>
      </Card>
    );
  }

  if (error || !statsData) {
    return (
      <Card p="$4" bg="$red2">
        <Text color="$red11">Error loading statistics</Text>
      </Card>
    );
  }

  // Map database fields to camelCase and provide defaults
  const stats = {
    totalWorkflows: workflowsData?.total || 0,
    totalExecutions: statsData.total_executions || 0,
    averageDurationMs: statsData.avg_duration_ms || null,
    totalErrors: statsData.total_failures || 0,
    mostUsedTaskQueue: statsData.most_used_task_queue_id ? {
      queueName: statsData.most_used_task_queue_id,
      executionCount: statsData.most_used_task_queue_count || 0,
    } : null,
    mostUsedWorkflow: statsData.most_used_workflow_id ? {
      workflowName: statsData.most_used_workflow_id,
      executionCount: statsData.most_used_workflow_count || 0,
    } : null,
    mostUsedComponent: statsData.most_used_component_id ? {
      componentName: statsData.most_used_component_id,
      usageCount: statsData.most_used_component_count || 0,
    } : null,
    longestRun: statsData.longest_run_workflow_id ? {
      workflowName: statsData.longest_run_workflow_id,
      durationMs: statsData.longest_run_duration_ms || 0,
      startedAt: statsData.last_execution_at,
    } : null,
  };

  return (
    <ScrollView f={1}>
      <YStack gap="$4" p="$4">
        <XStack ai="center" gap="$2">
          <BarChart3 size={20} />
          <Text fontSize="$5" fontWeight="600">Project Statistics</Text>
        </XStack>

        <YStack gap="$3">
          {/* Overview Cards */}
          <XStack gap="$3" flexWrap="wrap">
            <StatCard
              label="Total Workflows"
              value={stats.totalWorkflows || 0}
              icon={BarChart3}
            />
            <StatCard
              label="Total Executions"
              value={stats.totalExecutions || 0}
              icon={TrendingUp}
            />
            <StatCard
              label="Average Duration"
              value={stats.averageDurationMs ? formatDuration(stats.averageDurationMs) : 'N/A'}
              icon={Clock}
            />
            <StatCard
              label="Total Errors"
              value={stats.totalErrors || 0}
              icon={XCircle}
              color="$red11"
            />
          </XStack>

          {/* Most Used Task Queue */}
          {stats.mostUsedTaskQueue && (
            <Card p="$4" bg="$gray2">
              <YStack gap="$2">
                <Text fontSize="$3" fontWeight="600">Most Used Task Queue</Text>
                <Text fontSize="$2" color="$gray11">
                  {stats.mostUsedTaskQueue.queueName}
                </Text>
                <Text fontSize="$1" color="$gray11">
                  {stats.mostUsedTaskQueue.executionCount} executions
                </Text>
              </YStack>
            </Card>
          )}

          {/* Most Used Workflow */}
          {stats.mostUsedWorkflow && (
            <Card p="$4" bg="$gray2">
              <YStack gap="$2">
                <Text fontSize="$3" fontWeight="600">Most Used Workflow</Text>
                <Text fontSize="$2" color="$gray11">
                  {stats.mostUsedWorkflow.workflowName}
                </Text>
                <Text fontSize="$1" color="$gray11">
                  {stats.mostUsedWorkflow.executionCount} executions
                </Text>
              </YStack>
            </Card>
          )}

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

          {/* Longest Run */}
          {stats.longestRun && (
            <Card p="$4" bg="$gray2">
              <YStack gap="$2">
                <Text fontSize="$3" fontWeight="600">Longest Run</Text>
                <Text fontSize="$2" color="$gray11">
                  {stats.longestRun.workflowName}
                </Text>
                <Text fontSize="$1" color="$gray11">
                  Duration: {formatDuration(stats.longestRun.durationMs)}
                </Text>
                {stats.longestRun.startedAt && (
                  <Text fontSize="$1" color="$gray11">
                    {format(new Date(stats.longestRun.startedAt), 'PPp')}
                  </Text>
                )}
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

