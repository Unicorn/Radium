'use client';

import { api as trpc } from '@/lib/trpc/client';
import { XStack, YStack, Text, Circle, Button, Spinner, Tooltip } from 'tamagui';
import { Play, Square, Activity, AlertCircle } from 'lucide-react';
import { formatDistanceToNow } from 'date-fns';
import { useState } from 'react';

interface WorkerStatusProps {
  projectId: string;
  projectName: string;
}

export function WorkerStatus({ projectId, projectName }: WorkerStatusProps) {
  const [isStarting, setIsStarting] = useState(false);
  const [isStopping, setIsStopping] = useState(false);

  // Poll worker status every 5 seconds
  const { data: health, isLoading } = trpc.projects.workerHealth.useQuery(
    { id: projectId },
    { 
      refetchInterval: 5000,
      retry: 1,
    }
  );

  // Start worker mutation
  const startWorkerMutation = trpc.projects.startWorker.useMutation({
    onSuccess: () => {
      setIsStarting(false);
    },
    onError: (error) => {
      console.error('Failed to start worker:', error);
      setIsStarting(false);
    },
  });

  // Stop worker mutation
  const stopWorkerMutation = trpc.projects.stopWorker.useMutation({
    onSuccess: () => {
      setIsStopping(false);
    },
    onError: (error) => {
      console.error('Failed to stop worker:', error);
      setIsStopping(false);
    },
  });

  const handleStartWorker = async () => {
    setIsStarting(true);
    await startWorkerMutation.mutateAsync({ id: projectId });
  };

  const handleStopWorker = async () => {
    setIsStopping(true);
    await stopWorkerMutation.mutateAsync({ id: projectId });
  };

  // Determine status display
  const getStatusInfo = () => {
    if (isLoading) {
      return {
        color: '$gray10',
        label: 'Loading...',
        icon: Spinner,
        showActions: false,
      };
    }

    if (isStarting) {
      return {
        color: '$blue10',
        label: 'Starting',
        icon: Spinner,
        showActions: false,
      };
    }

    if (isStopping) {
      return {
        color: '$orange10',
        label: 'Stopping',
        icon: Spinner,
        showActions: false,
      };
    }

    if (!health) {
      return {
        color: '$gray10',
        label: 'Unknown',
        icon: AlertCircle,
        showActions: true,
        canStart: true,
      };
    }

    if (health.status === 'running' && health.isHealthy) {
      return {
        color: '$green10',
        label: 'Running',
        icon: Activity,
        showActions: true,
        canStop: true,
      };
    }

    if (health.status === 'running' && !health.isHealthy) {
      return {
        color: '$orange10',
        label: 'Unhealthy',
        icon: AlertCircle,
        showActions: true,
        canStop: true,
      };
    }

    if (health.status === 'failed') {
      return {
        color: '$red10',
        label: 'Failed',
        icon: AlertCircle,
        showActions: true,
        canStart: true,
      };
    }

    // stopped or other status
    return {
      color: '$yellow10',
      label: 'Stopped',
      icon: Square,
      showActions: true,
      canStart: true,
    };
  };

  const statusInfo = getStatusInfo();
  const StatusIcon = statusInfo.icon;

  // Build tooltip content
  const tooltipContent = health ? (
    <YStack gap="$2" p="$2">
      <Text fontSize="$3" fontWeight="600" color="$gray12">Worker Details</Text>
      <YStack gap="$1">
        <Text fontSize="$2" color="$gray11">Project: {projectName}</Text>
        <Text fontSize="$2" color="$gray11">Status: {health.status}</Text>
        {health.workerId && (
          <Text fontSize="$2" color="$gray11">Worker ID: {health.workerId}</Text>
        )}
        {health.lastHeartbeat && (
          <Text fontSize="$2" color="$gray11">
            Last heartbeat: {formatDistanceToNow(new Date(health.lastHeartbeat), { addSuffix: true })}
          </Text>
        )}
        {health.taskQueueName && (
          <Text fontSize="$2" color="$gray11">Queue: {health.taskQueueName}</Text>
        )}
      </YStack>
    </YStack>
  ) : (
    <YStack gap="$2" p="$2">
      <Text fontSize="$3" fontWeight="600" color="$gray12">Worker Status</Text>
      <Text fontSize="$2" color="$gray11">No worker information available</Text>
    </YStack>
  );

  return (
    <Tooltip>
      <Tooltip.Trigger>
        <XStack 
          gap="$2" 
          ai="center" 
          p="$2" 
          pr="$3"
          borderRadius="$4" 
          bg="$background"
          borderWidth={1}
          borderColor="$borderColor"
          hoverStyle={{ bg: '$gray3' }}
          cursor="pointer"
        >
          {/* Status indicator circle */}
          <Circle 
            size={8} 
            backgroundColor={statusInfo.color}
            animation="quick"
          />

          {/* Status icon */}
          {StatusIcon === Spinner ? (
            <Spinner size="small" color={statusInfo.color} />
          ) : (
            // @ts-expect-error - Lucide icons accept number for size, but types are not properly exposed
            <StatusIcon size={16} color={statusInfo.color} />
          )}

          {/* Status label */}
          <Text fontSize="$3" color="$gray12" fontWeight="500">
            Worker: {statusInfo.label}
          </Text>

          {/* Last heartbeat (only if running and healthy) */}
          {health?.lastHeartbeat && health.isHealthy && (
            <Text fontSize="$2" color="$gray11">
              {formatDistanceToNow(new Date(health.lastHeartbeat), { addSuffix: true })}
            </Text>
          )}

          {/* Action buttons */}
          {statusInfo.showActions && (
            <XStack gap="$1" ml="$2">
              {statusInfo.canStart && (
                <Button
                  size="$2"
                  icon={Play}
                  chromeless
                  onPress={handleStartWorker}
                  disabled={isStarting || startWorkerMutation.isLoading}
                >
                  Start
                </Button>
              )}
              {statusInfo.canStop && (
                <Button
                  size="$2"
                  icon={Square}
                  chromeless
                  onPress={handleStopWorker}
                  disabled={isStopping || stopWorkerMutation.isLoading}
                >
                  Stop
                </Button>
              )}
            </XStack>
          )}
        </XStack>
      </Tooltip.Trigger>

      <Tooltip.Content
        enterStyle={{ x: 0, y: -5, opacity: 0, scale: 0.9 }}
        exitStyle={{ x: 0, y: -5, opacity: 0, scale: 0.9 }}
        scale={1}
        x={0}
        y={0}
        opacity={1}
        animation={[
          'quick',
          {
            opacity: {
              overshootClamping: true,
            },
          },
        ]}
      >
        <Tooltip.Arrow />
        {tooltipContent}
      </Tooltip.Content>
    </Tooltip>
  );
}

