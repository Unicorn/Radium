'use client';

import { useParams } from 'next/navigation';
import { api as trpc } from '@/lib/trpc/client';
import { Button, YStack, XStack, Heading, Text, Spinner, Card } from 'tamagui';
import { Plus } from 'lucide-react';
import { useState } from 'react';
import { WorkQueueCard } from '@/components/work-queue/WorkQueueCard';
import { WorkQueueForm } from '@/components/work-queue/WorkQueueForm';

export default function WorkflowWorkQueuesPage() {
  const params = useParams();
  const workflowId = params.id as string;
  const [showCreateForm, setShowCreateForm] = useState(false);

  // Fetch work queues for this workflow
  const { data, isLoading, refetch } = trpc.workQueues.list.useQuery({
    workflowId,
  });

  // Fetch workflow details
  const { data: workflowData } = trpc.workflows.get.useQuery({
    id: workflowId,
  });

  if (isLoading) {
    return (
      <YStack f={1} ai="center" jc="center" gap="$4">
        <Spinner size="large" />
        <Text>Loading work queues...</Text>
      </YStack>
    );
  }

  const workQueues = data?.workQueues || [];

  return (
    <YStack f={1} gap="$4" p="$4">
      {/* Header */}
      <XStack ai="center" jc="space-between">
        <YStack gap="$2">
          <Heading size="$8">Work Queues</Heading>
          <Text color="$gray11">
            {workflowData?.workflow?.name || 'Workflow'}
          </Text>
        </YStack>
        <Button
          icon={Plus}
          onPress={() => setShowCreateForm(true)}
          themeInverse
        >
          Add Work Queue
        </Button>
      </XStack>

      {/* Create Form */}
      {showCreateForm && (
        <Card p="$4" elevate>
          <WorkQueueForm
            workflowId={workflowId}
            onSuccess={() => {
              setShowCreateForm(false);
              refetch();
            }}
            onCancel={() => setShowCreateForm(false)}
          />
        </Card>
      )}

      {/* Work Queues List */}
      {workQueues.length === 0 ? (
        <Card p="$6" ai="center" gap="$4">
          <Text fontSize="$6" color="$gray11">
            No work queues yet
          </Text>
          <Text fontSize="$3" color="$gray10" textAlign="center">
            Work queues hold pending work items that child workflows can add to or query.
          </Text>
          <Button
            icon={Plus}
            onPress={() => setShowCreateForm(true)}
            size="$3"
          >
            Create your first work queue
          </Button>
        </Card>
      ) : (
        <YStack gap="$3">
          {workQueues.map((workQueue) => (
            <WorkQueueCard
              key={workQueue.id}
              workQueue={workQueue}
              onUpdate={refetch}
            />
          ))}
        </YStack>
      )}
    </YStack>
  );
}

