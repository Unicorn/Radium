'use client';

/**
 * Component Performance Panel
 *
 * Activity Performance Analytics dashboard component.
 * Displays execution metrics, latency distribution, and component type breakdown.
 *
 * Part of the Workflow Observability Platform.
 */

import { useState } from 'react';
import { YStack, XStack, Text, Card, ScrollView, Select, Spinner, Progress } from 'tamagui';
import {
  BarChart3,
  Clock,
  AlertCircle,
  CheckCircle,
  TrendingUp,
  Activity,
  Zap,
  Timer,
  type LucideIcon,
} from 'lucide-react';
import { api } from '@/lib/trpc/client';
import { format, subDays } from 'date-fns';

interface ComponentPerformancePanelProps {
  projectId: string;
}

export function ComponentPerformancePanel({ projectId }: ComponentPerformancePanelProps) {
  const [days, setDays] = useState(30);

  // Fetch execution analytics
  const {
    data: analyticsData,
    isLoading: analyticsLoading,
    error: analyticsError,
  } = api.observability.getExecutionAnalytics.useQuery({
    projectId,
    days,
  });

  // Fetch component type distribution
  const { data: distributionData, isLoading: distributionLoading } =
    api.observability.getComponentTypeDistribution.useQuery({
      projectId,
      days,
    });

  // Fetch activity performance
  const { data: performanceData, isLoading: performanceLoading } =
    api.observability.getActivityPerformance.useQuery({
      projectId,
      startDate: subDays(new Date(), days),
      endDate: new Date(),
    });

  if (analyticsLoading || distributionLoading || performanceLoading) {
    return (
      <Card p="$4" ai="center" jc="center" minHeight={200}>
        <Spinner size="large" />
        <Text mt="$2" color="$gray11">
          Loading performance analytics...
        </Text>
      </Card>
    );
  }

  if (analyticsError) {
    return (
      <Card p="$4" bg="$red2">
        <XStack ai="center" gap="$2">
          <AlertCircle size={20} color="var(--red11)" />
          <Text color="$red11">Error loading performance data</Text>
        </XStack>
      </Card>
    );
  }

  const summary = analyticsData?.summary || {
    totalExecutions: 0,
    successRate: 100,
    avgDurationMs: 0,
    totalFailures: 0,
  };

  return (
    <ScrollView f={1}>
      <YStack gap="$4" p="$4">
        {/* Header */}
        <XStack ai="center" jc="space-between">
          <XStack ai="center" gap="$2">
            <Activity size={20} />
            <Text fontSize="$5" fontWeight="600">
              Activity Performance Analytics
            </Text>
          </XStack>

          {/* Time Period Selector */}
          <Select value={days.toString()} onValueChange={(val) => setDays(parseInt(val, 10))}>
            <Select.Trigger w={140}>
              <Select.Value placeholder="Select period" />
            </Select.Trigger>
            <Select.Content>
              <Select.Item value="7" index={0}>
                <Select.ItemText>Last 7 days</Select.ItemText>
              </Select.Item>
              <Select.Item value="14" index={1}>
                <Select.ItemText>Last 14 days</Select.ItemText>
              </Select.Item>
              <Select.Item value="30" index={2}>
                <Select.ItemText>Last 30 days</Select.ItemText>
              </Select.Item>
              <Select.Item value="60" index={3}>
                <Select.ItemText>Last 60 days</Select.ItemText>
              </Select.Item>
              <Select.Item value="90" index={4}>
                <Select.ItemText>Last 90 days</Select.ItemText>
              </Select.Item>
            </Select.Content>
          </Select>
        </XStack>

        {/* Summary Cards */}
        <XStack gap="$3" flexWrap="wrap">
          <StatCard
            label="Total Executions"
            value={formatNumber(summary.totalExecutions)}
            icon={TrendingUp}
            color="$blue11"
          />
          <StatCard
            label="Success Rate"
            value={`${summary.successRate.toFixed(1)}%`}
            icon={CheckCircle}
            color={summary.successRate >= 99 ? '$green11' : summary.successRate >= 95 ? '$yellow11' : '$red11'}
          />
          <StatCard
            label="Avg Duration"
            value={formatDuration(summary.avgDurationMs)}
            icon={Clock}
            color="$purple11"
          />
          <StatCard
            label="Total Failures"
            value={formatNumber(summary.totalFailures)}
            icon={AlertCircle}
            color={summary.totalFailures > 0 ? '$red11' : '$gray11'}
          />
        </XStack>

        {/* Component Type Distribution */}
        {distributionData && distributionData.distribution.length > 0 && (
          <Card p="$4" bg="$gray2">
            <YStack gap="$3">
              <XStack ai="center" gap="$2">
                <BarChart3 size={18} />
                <Text fontWeight="600">Execution by Component Type</Text>
              </XStack>

              <YStack gap="$2">
                {distributionData.distribution.map((item) => (
                  <YStack key={item.type} gap="$1">
                    <XStack jc="space-between">
                      <Text fontSize="$3" textTransform="capitalize">
                        {item.type}
                      </Text>
                      <Text fontSize="$3" color="$gray11">
                        {formatNumber(item.total)} ({item.percentage}%)
                      </Text>
                    </XStack>
                    <Progress
                      value={item.percentage}
                      bg="$gray4"
                      h={8}
                      br="$2"
                    >
                      <Progress.Indicator
                        animation="medium"
                        bg={getTypeColor(item.type)}
                      />
                    </Progress>
                  </YStack>
                ))}
              </YStack>
            </YStack>
          </Card>
        )}

        {/* Top Activities by Execution Count */}
        {performanceData && performanceData.components.length > 0 && (
          <Card p="$4" bg="$gray2">
            <YStack gap="$3">
              <XStack ai="center" gap="$2">
                <Zap size={18} />
                <Text fontWeight="600">Top Activities</Text>
              </XStack>

              <YStack gap="$2">
                {performanceData.components.slice(0, 10).map((component, index) => (
                  <Card key={`${component.component_type}-${component.component_name}`} p="$3" bg="$gray3">
                    <XStack jc="space-between" ai="center">
                      <YStack f={1}>
                        <Text fontWeight="500" numberOfLines={1}>
                          {component.component_name}
                        </Text>
                        <Text fontSize="$2" color="$gray11" textTransform="capitalize">
                          {component.component_type}
                        </Text>
                      </YStack>

                      <XStack gap="$4" ai="center">
                        <YStack ai="flex-end">
                          <Text fontSize="$3" fontWeight="500">
                            {formatNumber(component.total_executions)}
                          </Text>
                          <Text fontSize="$2" color="$gray11">
                            executions
                          </Text>
                        </YStack>

                        <YStack ai="flex-end">
                          <Text
                            fontSize="$3"
                            fontWeight="500"
                            color={
                              component.success_rate >= 99
                                ? '$green11'
                                : component.success_rate >= 95
                                ? '$yellow11'
                                : '$red11'
                            }
                          >
                            {component.success_rate.toFixed(1)}%
                          </Text>
                          <Text fontSize="$2" color="$gray11">
                            success
                          </Text>
                        </YStack>

                        <YStack ai="flex-end">
                          <Text fontSize="$3" fontWeight="500">
                            {formatDuration(component.avg_duration_ms)}
                          </Text>
                          <Text fontSize="$2" color="$gray11">
                            avg
                          </Text>
                        </YStack>
                      </XStack>
                    </XStack>
                  </Card>
                ))}
              </YStack>
            </YStack>
          </Card>
        )}

        {/* Daily Timeline */}
        {analyticsData && analyticsData.timeline.length > 0 && (
          <Card p="$4" bg="$gray2">
            <YStack gap="$3">
              <XStack ai="center" gap="$2">
                <Timer size={18} />
                <Text fontWeight="600">Execution Timeline</Text>
              </XStack>

              <YStack gap="$2">
                {analyticsData.timeline.slice(-14).map((day) => (
                  <XStack key={day.date} ai="center" gap="$3">
                    <Text fontSize="$2" color="$gray11" w={80}>
                      {format(new Date(day.date), 'MMM d')}
                    </Text>
                    <YStack f={1}>
                      <Progress
                        value={
                          analyticsData.timeline.length > 0
                            ? (day.executions /
                                Math.max(...analyticsData.timeline.map((d) => d.executions))) *
                              100
                            : 0
                        }
                        bg="$gray4"
                        h={6}
                        br="$1"
                      >
                        <Progress.Indicator animation="medium" bg="$blue9" />
                      </Progress>
                    </YStack>
                    <Text fontSize="$2" w={60} ta="right">
                      {formatNumber(day.executions)}
                    </Text>
                    {day.failures > 0 && (
                      <Text fontSize="$2" color="$red11" w={40} ta="right">
                        {day.failures} err
                      </Text>
                    )}
                  </XStack>
                ))}
              </YStack>
            </YStack>
          </Card>
        )}

        {/* Empty State */}
        {summary.totalExecutions === 0 && (
          <Card p="$6" ai="center">
            <Activity size={48} color="var(--gray8)" />
            <Text mt="$3" fontSize="$4" fontWeight="500">
              No Activity Data Yet
            </Text>
            <Text mt="$1" color="$gray11" ta="center">
              Execute some workflows to see performance analytics here.
            </Text>
          </Card>
        )}
      </YStack>
    </ScrollView>
  );
}

// Helper Components

interface StatCardProps {
  label: string;
  value: string | number;
  icon: LucideIcon;
  color?: string;
}

function StatCard({ label, value, icon: Icon, color = '$gray12' }: StatCardProps) {
  return (
    <Card p="$4" bg="$gray2" f={1} minWidth={150}>
      <XStack ai="center" gap="$3">
        <Icon size={24} color={`var(--${color.replace('$', '')})`} />
        <YStack>
          <Text fontSize="$5" fontWeight="600" color={color}>
            {value}
          </Text>
          <Text fontSize="$2" color="$gray11">
            {label}
          </Text>
        </YStack>
      </XStack>
    </Card>
  );
}

// Helper Functions

function formatNumber(num: number): string {
  if (num >= 1000000) {
    return `${(num / 1000000).toFixed(1)}M`;
  }
  if (num >= 1000) {
    return `${(num / 1000).toFixed(1)}K`;
  }
  return num.toString();
}

function formatDuration(ms: number): string {
  if (ms < 1000) {
    return `${Math.round(ms)}ms`;
  }
  if (ms < 60000) {
    return `${(ms / 1000).toFixed(1)}s`;
  }
  if (ms < 3600000) {
    return `${(ms / 60000).toFixed(1)}m`;
  }
  return `${(ms / 3600000).toFixed(1)}h`;
}

function getTypeColor(type: string): string {
  const colors: Record<string, string> = {
    agent: '$purple9',
    activity: '$blue9',
    http: '$green9',
    database: '$yellow9',
    notification: '$orange9',
    transform: '$cyan9',
    state: '$pink9',
    custom: '$gray9',
  };
  return colors[type] || '$blue9';
}
