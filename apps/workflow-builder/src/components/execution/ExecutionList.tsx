'use client';

import { YStack, XStack, Card, Text, Button } from 'tamagui';
import { formatDistanceToNow } from 'date-fns';
import { useRouter } from 'next/navigation';
import { Clock, CheckCircle, XCircle, AlertCircle, PlayCircle, Loader } from 'lucide-react';

interface Execution {
  id: string;
  workflowId: string;
  status: string;
  startedAt: string;
  completedAt: string | null;
  durationMs: number | null;
  errorMessage?: string | null;
  workflow?: {
    id: string;
    name?: string;
    displayName?: string;
    display_name?: string;
    description?: string | null;
  };
}

interface ExecutionListProps {
  executions: Execution[];
}

export function ExecutionList({ executions }: ExecutionListProps) {
  return (
    <YStack gap="$3" w="100%">
      {executions.map((execution) => (
        <ExecutionRow key={execution.id} execution={execution} />
      ))}
    </YStack>
  );
}

function ExecutionRow({ execution }: { execution: Execution }) {
  const router = useRouter();

  const handleViewDetails = () => {
    router.push(`/executions/${execution.id}`);
  };

  const handleViewWorkflow = () => {
    router.push(`/workflows/${execution.workflowId}`);
  };

  const workflowName = execution.workflow?.displayName ||
                       execution.workflow?.display_name ||
                       execution.workflow?.name ||
                       'Unknown Workflow';

  return (
    <Card
      p="$4"
      hoverStyle={{
        bg: '$gray2',
        transform: [{ scale: 1.005 }],
      }}
      pressStyle={{ scale: 0.995 }}
      animation="quick"
      cursor="pointer"
      borderWidth={1}
      borderColor="$borderColor"
    >
      <XStack jc="space-between" ai="center" gap="$4">
        <YStack gap="$3" f={1}>
          <XStack gap="$3" ai="center" flexWrap="wrap">
            <StatusBadge status={execution.status} />
            <Text
              fontSize="$5"
              fontWeight="600"
              onPress={handleViewWorkflow}
              cursor="pointer"
              hoverStyle={{ color: '$blue11' }}
            >
              {workflowName}
            </Text>
          </XStack>

          <XStack gap="$4" flexWrap="wrap">
            <XStack gap="$2" ai="center">
              <Clock size={14} color="$gray11" />
              <Text color="$gray11" fontSize="$2">
                Started {formatDistanceToNow(new Date(execution.startedAt), { addSuffix: true })}
              </Text>
            </XStack>

            {execution.completedAt && execution.durationMs !== null && (
              <XStack gap="$2" ai="center">
                <Clock size={14} color="$gray11" />
                <Text color="$gray11" fontSize="$2">
                  Duration: {formatDuration(execution.durationMs)}
                </Text>
              </XStack>
            )}

            {execution.errorMessage && (
              <XStack gap="$2" ai="center">
                <AlertCircle size={14} color="$red11" />
                <Text color="$red11" fontSize="$2" numberOfLines={1}>
                  {execution.errorMessage}
                </Text>
              </XStack>
            )}
          </XStack>
        </YStack>

        <Button
          size="$3"
          onPress={handleViewDetails}
          chromeless
        >
          View Details
        </Button>
      </XStack>
    </Card>
  );
}

function StatusBadge({ status }: { status: string }) {
  const statusConfig: Record<string, { icon: React.ComponentType<{ size?: number; color?: string }>; color: string; bg: string; border: string; text: string }> = {
    completed: {
      icon: CheckCircle,
      color: '$green11',
      bg: '$green2',
      border: '$green6',
      text: 'Completed'
    },
    failed: {
      icon: XCircle,
      color: '$red11',
      bg: '$red2',
      border: '$red6',
      text: 'Failed'
    },
    running: {
      icon: Loader,
      color: '$blue11',
      bg: '$blue2',
      border: '$blue6',
      text: 'Running'
    },
    building: {
      icon: PlayCircle,
      color: '$yellow11',
      bg: '$yellow2',
      border: '$yellow6',
      text: 'Building'
    },
    cancelled: {
      icon: XCircle,
      color: '$gray11',
      bg: '$gray2',
      border: '$gray6',
      text: 'Cancelled'
    },
    timed_out: {
      icon: Clock,
      color: '$orange11',
      bg: '$orange2',
      border: '$orange6',
      text: 'Timed Out'
    },
  };

  const config = statusConfig[status] || {
    icon: AlertCircle,
    color: '$gray11',
    bg: '$gray2',
    border: '$gray6',
    text: status,
  };

  const Icon = config.icon;

  return (
    <Card
      bg={config.bg}
      px="$3"
      py="$1.5"
      borderRadius="$4"
      borderWidth={1}
      borderColor={config.border}
    >
      <XStack gap="$2" ai="center">
        <Icon size={14} color={config.color} />
        <Text color={config.color} fontSize="$2" fontWeight="600" textTransform="capitalize">
          {config.text}
        </Text>
      </XStack>
    </Card>
  );
}

function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`;
  if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
  if (ms < 3600000) return `${(ms / 60000).toFixed(1)}m`;
  return `${(ms / 3600000).toFixed(1)}h`;
}
