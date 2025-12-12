'use client';

import { Card, YStack, XStack, Text, Button, Progress, Separator } from 'tamagui';
import { Badge } from '../shared/Badge';
import { Inbox, Trash2, Edit, Info } from 'lucide-react';
import { useState } from 'react';
import type { WorkflowWorkQueue } from '@/types/advanced-patterns';
import { getPriorityDescription, getQueueCapacityPercent } from '@/utils/work-queue-utils';
import { api as trpc } from '@/lib/trpc/client';

interface WorkQueueCardProps {
  workQueue: WorkflowWorkQueue;
  onUpdate?: () => void;
}

export function WorkQueueCard({ workQueue, onUpdate }: WorkQueueCardProps) {
  const [showDetails, setShowDetails] = useState(false);
  
  const utils = trpc.useContext();
  
  // Delete mutation
  const deleteMutation = trpc.workQueues.delete.useMutation({
    onSuccess: () => {
      utils.workQueues.list.invalidate();
      onUpdate?.();
    },
  });

  // Get queue with handlers
  const { data: queueDetails } = trpc.workQueues.getWithHandlers.useQuery(
    { id: workQueue.id },
    { enabled: showDetails }
  );

  const handleDelete = async () => {
    if (confirm(`Delete work queue "${workQueue.queue_name}"? This cannot be undone.`)) {
      await deleteMutation.mutateAsync({ id: workQueue.id });
    }
  };

  // Calculate capacity
  const capacityPercent = getQueueCapacityPercent(0, workQueue.max_size); // TODO: Get actual count from runtime
  const priorityDesc = getPriorityDescription(workQueue.priority);

  return (
    <Card p="$4" borderWidth={1} borderColor="$borderColor" >
      <YStack gap="$3">
        {/* Header */}
        <XStack ai="center" jc="space-between">
          <XStack ai="center" gap="$3">
            <Inbox size={24} color="$yellow9" />
            <YStack gap="$1">
              <Text fontSize="$6" fontWeight="600">
                {workQueue.queue_name}
              </Text>
              {workQueue.description && (
                <Text fontSize="$3" color="$gray11">
                  {workQueue.description}
                </Text>
              )}
            </YStack>
          </XStack>

          <XStack gap="$2">
            <Button
              size="$2"
              icon={Info}
              onPress={() => setShowDetails(!showDetails)}
              chromeless
            >
              {showDetails ? 'Hide' : 'Details'}
            </Button>
            <Button
              size="$2"
              icon={Edit}
              chromeless
            >
              Edit
            </Button>
            <Button
              size="$2"
              icon={Trash2}
              onPress={handleDelete}
              disabled={deleteMutation.isLoading}
              chromeless
              color="$red10"
            >
              Delete
            </Button>
          </XStack>
        </XStack>

        {/* Quick Stats */}
        <XStack gap="$3" flexWrap="wrap">
          <Badge>
            Priority: {priorityDesc}
          </Badge>
          
          {workQueue.max_size && (
            <Badge>
              Max: {workQueue.max_size} items
            </Badge>
          )}

          {workQueue.deduplicate && (
            <Badge>
              Deduplication: ON
            </Badge>
          )}

          <Badge>
            Current: 0 items {/* TODO: Get from runtime */}
          </Badge>
        </XStack>

        {/* Capacity Progress */}
        {workQueue.max_size && (
          <YStack gap="$2">
            <XStack jc="space-between">
              <Text fontSize="$2" color="$gray11">Capacity</Text>
              <Text fontSize="$2" color="$gray11">
                0 / {workQueue.max_size} ({capacityPercent || 0}%)
              </Text>
            </XStack>
            <Progress value={capacityPercent || 0}>
              <Progress.Indicator animation="bouncy" />
            </Progress>
          </YStack>
        )}

        {/* Details Section */}
        {showDetails && queueDetails && (
          <>
            <Separator />
            <YStack gap="$3">
              <Text fontSize="$4" fontWeight="600">Auto-Generated Handlers</Text>
              
              {/* Signal Handler */}
              {queueDetails.signal && (
                <Card bg="$orange2" p="$3" borderWidth={1} borderColor="$orange6">
                  <YStack gap="$2">
                    <XStack ai="center" gap="$2">
                      <Text fontSize="$3" fontWeight="600" color="$orange11">
                        📤 Signal Handler
                      </Text>
                      <Badge size="$1" bg="$orange5">AUTO</Badge>
                    </XStack>
                    <Text fontSize="$2" color="$orange11">
                      <Text fontWeight="600">Name:</Text> {queueDetails.signal.signal_name}
                    </Text>
                    <Text fontSize="$2" color="$orange11">
                      Child workflows can call this signal to add items to the queue
                    </Text>
                  </YStack>
                </Card>
              )}

              {/* Query Handler */}
              {queueDetails.query && (
                <Card bg="$teal2" p="$3" borderWidth={1} borderColor="$teal6">
                  <YStack gap="$2">
                    <XStack ai="center" gap="$2">
                      <Text fontSize="$3" fontWeight="600" color="$teal11">
                        🔍 Query Handler
                      </Text>
                      <Badge size="$1" bg="$teal5">AUTO</Badge>
                    </XStack>
                    <Text fontSize="$2" color="$teal11">
                      <Text fontWeight="600">Name:</Text> {queueDetails.query.query_name}
                    </Text>
                    <Text fontSize="$2" color="$teal11">
                      Child workflows can query this to check queue status
                    </Text>
                  </YStack>
                </Card>
              )}

              {/* Work Item Schema */}
              {workQueue.work_item_schema && (
                <Card bg="$gray2" p="$3" borderWidth={1} borderColor="$gray6">
                  <YStack gap="$2">
                    <Text fontSize="$3" fontWeight="600" color="$gray12">
                      📋 Work Item Schema
                    </Text>
                    <Text fontSize="$1" fontFamily="$mono" color="$gray11">
                      {JSON.stringify(workQueue.work_item_schema, null, 2)}
                    </Text>
                  </YStack>
                </Card>
              )}
            </YStack>
          </>
        )}
      </YStack>
    </Card>
  );
}

