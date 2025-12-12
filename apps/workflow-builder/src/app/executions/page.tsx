'use client';

import { useState } from 'react';
import { api } from '@/lib/trpc/client';
import { YStack, XStack, Text, Card, Button, Separator, Spinner } from 'tamagui';
import { ExecutionStats } from '@/components/execution/ExecutionStats';
import { ExecutionList } from '@/components/execution/ExecutionList';
import { RefreshCw } from 'lucide-react';

export default function ExecutionsPage() {
  const [statusFilter, setStatusFilter] = useState<'all' | 'running' | 'completed' | 'failed'>('all');
  const [page, setPage] = useState(1);
  const pageSize = 20;

  // Fetch all executions for the current user
  const { data: executionsData, isLoading, refetch, isRefetching } = api.execution.listUserExecutions.useQuery({
    page,
    pageSize,
    status: statusFilter === 'all' ? undefined : statusFilter,
  });

  // Fetch global execution statistics
  const { data: globalStats } = api.execution.getGlobalStats.useQuery();

  const handleRefresh = () => {
    void refetch();
  };

  const handlePageChange = (newPage: number) => {
    setPage(newPage);
  };

  const totalPages = executionsData ? Math.ceil(executionsData.total / pageSize) : 1;

  return (
    <YStack f={1} p="$6" gap="$6">
      {/* Header */}
      <XStack ai="center" jc="space-between">
        <YStack gap="$2">
          <Text fontSize="$9" fontWeight="700">Workflow Executions</Text>
          <Text color="$gray11" fontSize="$3">
            Monitor and track all workflow execution history
          </Text>
        </YStack>

        <Button
          icon={RefreshCw}
          onPress={handleRefresh}
          disabled={isRefetching}
          size="$3"
        >
          {isRefetching ? 'Refreshing...' : 'Refresh'}
        </Button>
      </XStack>

      {/* Stats Section */}
      {globalStats && <ExecutionStats stats={globalStats} />}

      <Separator />

      {/* Filters */}
      <XStack gap="$3" flexWrap="wrap">
        <Button
          size="$3"
          variant={statusFilter === 'all' ? 'outlined' : undefined}
          onPress={() => setStatusFilter('all')}
        >
          All
        </Button>
        <Button
          size="$3"
          variant={statusFilter === 'running' ? 'outlined' : undefined}
          onPress={() => setStatusFilter('running')}
        >
          Running
        </Button>
        <Button
          size="$3"
          variant={statusFilter === 'completed' ? 'outlined' : undefined}
          onPress={() => setStatusFilter('completed')}
        >
          Completed
        </Button>
        <Button
          size="$3"
          variant={statusFilter === 'failed' ? 'outlined' : undefined}
          onPress={() => setStatusFilter('failed')}
        >
          Failed
        </Button>
      </XStack>

      {/* Executions List */}
      {isLoading ? (
        <Card p="$6" ai="center" jc="center">
          <Spinner size="large" />
          <Text mt="$3" color="$gray11">Loading executions...</Text>
        </Card>
      ) : executionsData && executionsData.executions.length > 0 ? (
        <>
          <ExecutionList executions={executionsData.executions} />

          {/* Pagination */}
          {totalPages > 1 && (
            <XStack gap="$3" jc="center" ai="center" mt="$4">
              <Button
                size="$3"
                disabled={page === 1}
                onPress={() => handlePageChange(page - 1)}
              >
                Previous
              </Button>
              <Text fontSize="$3">
                Page {page} of {totalPages}
              </Text>
              <Button
                size="$3"
                disabled={page === totalPages}
                onPress={() => handlePageChange(page + 1)}
              >
                Next
              </Button>
            </XStack>
          )}
        </>
      ) : (
        <Card p="$6" ai="center" jc="center" bg="$gray2">
          <Text color="$gray11" fontSize="$4">
            {statusFilter === 'all'
              ? 'No executions found. Start by running a workflow!'
              : `No ${statusFilter} executions found.`
            }
          </Text>
        </Card>
      )}
    </YStack>
  );
}
