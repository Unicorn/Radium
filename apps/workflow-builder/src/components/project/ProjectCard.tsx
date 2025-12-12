'use client';

import { Card, XStack, YStack, Text, Button } from 'tamagui';
import { Badge } from '@/components/shared/Badge';
import { formatDistanceToNow } from 'date-fns';
import { Trash, Edit, Play, Square } from 'lucide-react';

interface ProjectCardProps {
  project: {
    id: string;
    name: string;
    description?: string | null;
    task_queue_name: string;
    workflow_count?: Array<{ count: number }>;
    active_workers?: Array<{ status: string; last_heartbeat: string }>;
    created_at: string;
    updated_at: string;
    is_default?: boolean;
  };
  onClick?: () => void;
  onEdit?: () => void;
  onArchive?: () => void;
  showActions?: boolean;
}

export function ProjectCard({
  project,
  onClick,
  onEdit,
  onArchive,
  showActions = true,
}: ProjectCardProps) {
  const workflowCount = project.workflow_count?.[0]?.count || 0;
  const worker = project.active_workers?.[0];
  const workerStatus = worker?.status || 'stopped';
  
  const getWorkerStatusColor = () => {
    if (workerStatus === 'running') return '$green10';
    if (workerStatus === 'starting') return '$blue10';
    if (workerStatus === 'error') return '$red10';
    return '$gray10';
  };

  const getWorkerStatusText = () => {
    if (workerStatus === 'running') return 'Running';
    if (workerStatus === 'starting') return 'Starting';
    if (workerStatus === 'error') return 'Error';
    return 'Stopped';
  };

  return (
    <Card
      padding="$4"
      elevate
      pressStyle={{ scale: 0.98 }}
      cursor={onClick ? 'pointer' : 'default'}
      onPress={onClick}
    >
      <YStack gap="$3">
        {/* Header */}
        <XStack justifyContent="space-between" alignItems="flex-start">
          <YStack flex={1} gap="$1">
            <XStack gap="$2" alignItems="center">
              <Text fontSize="$6" fontWeight="600">
                {project.name}
              </Text>
              {project.is_default && (
                <Badge backgroundColor="$blue9" paddingHorizontal="$2" paddingVertical="$1">
                  <Text fontSize="$1" color="white">
                    Default
                  </Text>
                </Badge>
              )}
            </XStack>
            {project.description && (
              <Text fontSize="$3" color="$gray11" numberOfLines={2}>
                {project.description}
              </Text>
            )}
          </YStack>

          {/* Worker Status Badge */}
          <Badge backgroundColor={getWorkerStatusColor()} paddingHorizontal="$3" paddingVertical="$1">
            <Text fontSize="$2" color="white">
              {getWorkerStatusText()}
            </Text>
          </Badge>
        </XStack>

        {/* Stats */}
        <XStack gap="$4" flexWrap="wrap">
          <YStack gap="$1">
            <Text fontSize="$2" color="$gray11">
              Workflows
            </Text>
            <Text fontSize="$5" fontWeight="600">
              {workflowCount}
            </Text>
          </YStack>

          <YStack gap="$1">
            <Text fontSize="$2" color="$gray11">
              Task Queue
            </Text>
            <Text fontSize="$3" fontFamily="$mono">
              {project.task_queue_name}
            </Text>
          </YStack>

          {worker?.last_heartbeat && (
            <YStack gap="$1">
              <Text fontSize="$2" color="$gray11">
                Last Heartbeat
              </Text>
              <Text fontSize="$3">
                {formatDistanceToNow(new Date(worker.last_heartbeat), { addSuffix: true })}
              </Text>
            </YStack>
          )}
        </XStack>

        {/* Footer */}
        <XStack justifyContent="space-between" alignItems="center" marginTop="$2">
          <Text fontSize="$2" color="$gray11">
            Updated {formatDistanceToNow(new Date(project.updated_at), { addSuffix: true })}
          </Text>

          {showActions && (
            <XStack gap="$2">
              {onEdit && (
                <Button
                  size="$2"
                  icon={Edit}
                  variant="outlined"
                  onPress={(e) => {
                    e.stopPropagation();
                    onEdit();
                  }}
                >
                  Edit
                </Button>
              )}
              {onArchive && !project.is_default && (
                <Button
                  size="$2"
                  icon={Trash}
                  theme="orange"
                  variant="outlined"
                  onPress={(e) => {
                    e.stopPropagation();
                    onArchive();
                  }}
                >
                  Archive
                </Button>
              )}
            </XStack>
          )}
        </XStack>
      </YStack>
    </Card>
  );
}

