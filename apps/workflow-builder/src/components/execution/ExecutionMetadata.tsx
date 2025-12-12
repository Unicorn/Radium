'use client';

import { YStack, XStack, Card, Text, Separator } from 'tamagui';
import { Clock, Hash, Activity, CheckCircle, Calendar, Timer } from 'lucide-react';
import { format, formatDistanceToNow } from 'date-fns';

interface ExecutionMetadataProps {
  execution: {
    id: string;
    workflowId: string;
    status: string;
    startedAt: string | Date;
    completedAt?: string | Date | null;
    durationMs?: number | null;
    temporalWorkflowId?: string | null;
    temporalRunId?: string | null;
    activitiesExecuted?: number | null;
    historySyncStatus?: string | null;
    historySyncedAt?: string | Date | null;
  };
}

export function ExecutionMetadata({ execution }: ExecutionMetadataProps) {
  const startDate = new Date(execution.startedAt);
  const completedDate = execution.completedAt ? new Date(execution.completedAt) : null;

  return (
    <Card p="$4" borderWidth={1} borderColor="$borderColor" h="fit-content">
      <YStack gap="$4">
        <Text fontSize="$5" fontWeight="600">
          Execution Metadata
        </Text>

        <Separator />

        {/* Execution ID */}
        <MetadataRow
          icon={Hash}
          label="Execution ID"
          value={execution.id}
          mono
        />

        {/* Workflow ID */}
        <MetadataRow
          icon={Hash}
          label="Workflow ID"
          value={execution.workflowId}
          mono
        />

        {/* Status */}
        <MetadataRow
          icon={Activity}
          label="Status"
          value={
            <StatusBadge status={execution.status} />
          }
        />

        <Separator />

        {/* Started At */}
        <MetadataRow
          icon={Calendar}
          label="Started"
          value={
            <YStack gap="$1">
              <Text fontSize="$2">{format(startDate, 'PPpp')}</Text>
              <Text fontSize="$1" color="$gray11">
                {formatDistanceToNow(startDate, { addSuffix: true })}
              </Text>
            </YStack>
          }
        />

        {/* Completed At */}
        {completedDate && (
          <MetadataRow
            icon={CheckCircle}
            label="Completed"
            value={
              <YStack gap="$1">
                <Text fontSize="$2">{format(completedDate, 'PPpp')}</Text>
                <Text fontSize="$1" color="$gray11">
                  {formatDistanceToNow(completedDate, { addSuffix: true })}
                </Text>
              </YStack>
            }
          />
        )}

        {/* Duration */}
        {execution.durationMs !== null && execution.durationMs !== undefined && (
          <MetadataRow
            icon={Timer}
            label="Duration"
            value={formatDuration(execution.durationMs)}
          />
        )}

        {/* Activities Executed */}
        {execution.activitiesExecuted !== null && execution.activitiesExecuted !== undefined && (
          <MetadataRow
            icon={Activity}
            label="Activities Executed"
            value={execution.activitiesExecuted.toString()}
          />
        )}

        {/* Temporal IDs */}
        {execution.temporalWorkflowId && (
          <>
            <Separator />
            <MetadataRow
              icon={Hash}
              label="Temporal Workflow ID"
              value={execution.temporalWorkflowId}
              mono
              small
            />
          </>
        )}

        {execution.temporalRunId && (
          <MetadataRow
            icon={Hash}
            label="Temporal Run ID"
            value={execution.temporalRunId}
            mono
            small
          />
        )}

        {/* History Sync Status */}
        {execution.historySyncStatus && (
          <>
            <Separator />
            <MetadataRow
              icon={Clock}
              label="History Sync"
              value={
                <YStack gap="$1">
                  <Text fontSize="$2" textTransform="capitalize">
                    {execution.historySyncStatus}
                  </Text>
                  {execution.historySyncedAt && (
                    <Text fontSize="$1" color="$gray11">
                      {formatDistanceToNow(new Date(execution.historySyncedAt), { addSuffix: true })}
                    </Text>
                  )}
                </YStack>
              }
            />
          </>
        )}
      </YStack>
    </Card>
  );
}

interface MetadataRowProps {
  icon: React.ComponentType<{ size?: number; color?: string }>;
  label: string;
  value: string | React.ReactNode;
  mono?: boolean;
  small?: boolean;
}

function MetadataRow({ icon: Icon, label, value, mono = false, small = false }: MetadataRowProps) {
  return (
    <YStack gap="$2">
      <XStack ai="center" gap="$2">
        <Icon size={14} color="$gray11" />
        <Text fontSize="$2" color="$gray11" fontWeight="500">
          {label}
        </Text>
      </XStack>
      {typeof value === 'string' ? (
        <Text
          fontSize={small ? '$1' : '$2'}
          fontFamily={mono ? '$mono' : '$body'}
          wordWrap="break-word"
          whiteSpace={mono ? 'pre-wrap' : 'normal'}
        >
          {value}
        </Text>
      ) : (
        value
      )}
    </YStack>
  );
}

function StatusBadge({ status }: { status: string }) {
  const statusConfig: Record<string, { color: string; bg: string; border: string }> = {
    completed: { color: '$green11', bg: '$green2', border: '$green6' },
    failed: { color: '$red11', bg: '$red2', border: '$red6' },
    running: { color: '$blue11', bg: '$blue2', border: '$blue6' },
    building: { color: '$yellow11', bg: '$yellow2', border: '$yellow6' },
    cancelled: { color: '$gray11', bg: '$gray2', border: '$gray6' },
    timed_out: { color: '$orange11', bg: '$orange2', border: '$orange6' },
  };

  const config = statusConfig[status] || statusConfig.cancelled;

  return (
    <Card
      px="$3"
      py="$1.5"
      bg={config.bg}
      borderWidth={1}
      borderColor={config.border}
    >
      <Text
        fontSize="$2"
        fontWeight="600"
        color={config.color}
        textTransform="capitalize"
      >
        {status}
      </Text>
    </Card>
  );
}

function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`;
  if (ms < 60000) return `${(ms / 1000).toFixed(2)}s`;
  if (ms < 3600000) return `${(ms / 60000).toFixed(2)}m`;
  return `${(ms / 3600000).toFixed(2)}h`;
}
