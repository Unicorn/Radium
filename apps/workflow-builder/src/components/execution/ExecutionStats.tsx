'use client';

import { YStack, XStack, Card, Text } from 'tamagui';
import { TrendingUp, Activity, CheckCircle, XCircle, Clock, Zap } from 'lucide-react';

interface ExecutionStatsProps {
  stats: {
    total: number;
    running: number;
    completed: number;
    failed: number;
    cancelled?: number;
    timedOut?: number;
    avgDuration: number;
    successRate: number;
  };
}

export function ExecutionStats({ stats }: ExecutionStatsProps) {
  return (
    <XStack gap="$4" flexWrap="wrap" w="100%">
      <StatCard
        label="Total Executions"
        value={stats.total.toString()}
        icon={Activity}
        color="blue"
      />
      <StatCard
        label="Running"
        value={stats.running.toString()}
        icon={Clock}
        color="yellow"
        pulse={stats.running > 0}
      />
      <StatCard
        label="Completed"
        value={stats.completed.toString()}
        icon={CheckCircle}
        color="green"
      />
      <StatCard
        label="Failed"
        value={stats.failed.toString()}
        icon={XCircle}
        color="red"
      />
      <StatCard
        label="Success Rate"
        value={`${stats.successRate.toFixed(1)}%`}
        icon={TrendingUp}
        color="purple"
      />
      <StatCard
        label="Avg Duration"
        value={formatDuration(stats.avgDuration)}
        icon={Zap}
        color="gray"
      />
    </XStack>
  );
}

interface StatCardProps {
  label: string;
  value: string;
  icon: React.ComponentType<{ size?: number; color?: string }>;
  color: 'blue' | 'yellow' | 'green' | 'red' | 'purple' | 'gray';
  pulse?: boolean;
}

function StatCard({ label, value, icon: Icon, color, pulse = false }: StatCardProps) {
  const colorMap = {
    blue: { bg: '$blue2', text: '$blue11', border: '$blue6' },
    yellow: { bg: '$yellow2', text: '$yellow11', border: '$yellow6' },
    green: { bg: '$green2', text: '$green11', border: '$green6' },
    red: { bg: '$red2', text: '$red11', border: '$red6' },
    purple: { bg: '$purple2', text: '$purple11', border: '$purple6' },
    gray: { bg: '$gray2', text: '$gray11', border: '$gray6' },
  };

  const colors = colorMap[color];

  return (
    <Card
      f={1}
      minWidth={180}
      p="$4"
      bg={pulse ? colors.bg : '$background'}
      borderWidth={1}
      borderColor={colors.border}
      hoverStyle={{
        borderColor: colors.text,
        transform: [{ scale: 1.02 }],
      }}
      animation="quick"
    >
      <YStack gap="$3">
        <XStack ai="center" jc="space-between">
          <Text color="$gray11" fontSize="$3" fontWeight="500">
            {label}
          </Text>
          <Icon size={20} color={colors.text} />
        </XStack>
        <Text fontSize="$8" fontWeight="700" color={colors.text}>
          {value}
        </Text>
      </YStack>
    </Card>
  );
}

function formatDuration(ms: number): string {
  if (ms === 0) return '0ms';
  if (ms < 1000) return `${ms}ms`;
  if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
  if (ms < 3600000) return `${(ms / 60000).toFixed(1)}m`;
  return `${(ms / 3600000).toFixed(1)}h`;
}
