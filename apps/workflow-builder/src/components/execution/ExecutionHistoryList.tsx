'use client';

import { YStack, XStack, Text, Card, Button, ScrollView } from 'tamagui';
import { Clock, ChevronRight } from 'lucide-react';
import { api } from '@/lib/trpc/client';
import { format } from 'date-fns';
import { useState } from 'react';

interface ExecutionHistoryListProps {
  workflowId: string;
  onSelectExecution?: (executionId: string) => void;
}

export function ExecutionHistoryList({ workflowId, onSelectExecution }: ExecutionHistoryListProps) {
  const [page, setPage] = useState(1);
  const { data, isLoading, error } = api.execution.getExecutionHistory.useQuery({
    workflowId,
    page,
    limit: 20,
  });

  if (isLoading) {
    return (
      <Card p="$4">
        <Text>Loading execution history...</Text>
      </Card>
    );
  }

  if (error) {
    return (
      <Card p="$4" bg="$red2">
        <Text color="$red11">Error loading execution history</Text>
      </Card>
    );
  }

  const executions = data?.executions || [];
  const totalPages = data?.totalPages || 1;

  return (
    <YStack gap="$3" f={1}>
      <Text fontSize="$4" fontWeight="600">Execution History</Text>
      
      {executions.length === 0 ? (
        <Card p="$4" bg="$gray2">
          <Text color="$gray11">No executions found</Text>
        </Card>
      ) : (
        <>
          <ScrollView f={1}>
            <YStack gap="$2">
              {executions.map((exec) => (
                <ExecutionListItem
                  key={exec.id}
                  execution={exec}
                  onClick={() => onSelectExecution?.(exec.id)}
                />
              ))}
            </YStack>
          </ScrollView>

          {totalPages > 1 && (
            <XStack gap="$2" jc="center" ai="center">
              <Button
                size="$2"
                onPress={() => setPage(p => Math.max(1, p - 1))}
                disabled={page === 1}
              >
                Previous
              </Button>
              <Text fontSize="$2" color="$gray11">
                Page {page} of {totalPages}
              </Text>
              <Button
                size="$2"
                onPress={() => setPage(p => Math.min(totalPages, p + 1))}
                disabled={page === totalPages}
              >
                Next
              </Button>
            </XStack>
          )}
        </>
      )}
    </YStack>
  );
}

interface ExecutionItem {
  id: string;
  status: string;
  startedAt: Date;
  completedAt: Date | null;
  durationMs: number | null;
  result: unknown;
  error: string | null;
  historySyncStatus: string | null;
}

function ExecutionListItem({ execution, onClick }: { execution: ExecutionItem; onClick?: () => void }) {
  const statusColors: Record<string, string> = {
    completed: '$green11',
    failed: '$red11',
    running: '$blue11',
    pending: '$gray11',
  };

  const statusColor = statusColors[execution.status] || '$gray11';

  return (
    <Card
      p="$3"
      bg="$gray2"
      pressStyle={{ bg: '$gray3' }}
      onPress={onClick}
      cursor="pointer"
    >
      <XStack ai="center" jc="space-between">
        <YStack gap="$1" f={1}>
          <XStack gap="$2" ai="center">
            <Text fontSize="$3" fontWeight="600">
              {execution.workflowName || 'Workflow'}
            </Text>
            <Card p="$1" bg="$background" borderWidth={1} borderColor={statusColor}>
              <Text fontSize="$1" color={statusColor} textTransform="capitalize">
                {execution.status}
              </Text>
            </Card>
          </XStack>
          
          <XStack gap="$2" ai="center">
            <Clock size={12} color="$gray11" />
            <Text fontSize="$1" color="$gray11">
              {execution.startedAt
                ? format(new Date(execution.startedAt), 'PPpp')
                : 'N/A'}
            </Text>
          </XStack>

          {execution.durationMs && (
            <Text fontSize="$1" color="$gray11">
              Duration: {formatDuration(execution.durationMs)}
            </Text>
          )}
        </YStack>

        {onClick && (
          <ChevronRight size={20} color="$gray11" />
        )}
      </XStack>
    </Card>
  );
}

function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`;
  if (ms < 60000) return `${(ms / 1000).toFixed(2)}s`;
  if (ms < 3600000) return `${(ms / 60000).toFixed(2)}m`;
  return `${(ms / 3600000).toFixed(2)}h`;
}

