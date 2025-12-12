'use client';

import { Card, YStack, XStack, Text, Button, Separator } from 'tamagui';
import { Badge } from '../shared/Badge';
import { GitBranch, Trash2, Edit, ArrowUp, Search, PauseCircle } from 'lucide-react';
import { useState } from 'react';
import type { EnhancedWorkflowNode } from '@/types/advanced-patterns';
import { hasParentCommunication, hasBlockingDependencies } from '@/types/advanced-patterns';

interface ChildWorkflowCardProps {
  childWorkflow: EnhancedWorkflowNode & {
    workflow_name?: string;
    task_queue?: string;
  };
  onUpdate?: () => void;
  onEdit?: () => void;
}

export function ChildWorkflowCard({ childWorkflow, onUpdate, onEdit }: ChildWorkflowCardProps) {
  const [showDetails, setShowDetails] = useState(false);
  
  const handleDelete = async () => {
    if (confirm(`Delete child workflow "${childWorkflow.label}"? This cannot be undone.`)) {
      // TODO: Implement delete mutation
      console.log('Delete child workflow:', childWorkflow.id);
      onUpdate?.();
    }
  };

  const hasParentComm = hasParentCommunication(childWorkflow);
  const hasBlocking = hasBlockingDependencies(childWorkflow);

  return (
    <Card p="$4" borderWidth={1} borderColor="$borderColor">
      <YStack gap="$3">
        {/* Header */}
        <XStack ai="center" jc="space-between">
          <XStack ai="center" gap="$3">
            <GitBranch size={24} color="$blue9" />
            <YStack gap="$1">
              <Text fontSize="$6" fontWeight="600">
                {childWorkflow.label}
              </Text>
              {childWorkflow.workflow_name && (
                <Text fontSize="$2" color="$gray11" fontFamily="$mono">
                  {childWorkflow.workflow_name}
                </Text>
              )}
            </YStack>
          </XStack>

          <XStack gap="$2">
            <Button
              size="$2"
              icon={Edit}
              onPress={onEdit}
              chromeless
            >
              Edit
            </Button>
            <Button
              size="$2"
              icon={Trash2}
              onPress={handleDelete}
              chromeless
              color="$red10"
            >
              Delete
            </Button>
          </XStack>
        </XStack>

        {/* Status Badges */}
        <XStack gap="$3" flexWrap="wrap">
          <Badge>
            Child Workflow
          </Badge>
          
          {childWorkflow.task_queue && (
            <Badge>
              Queue: {childWorkflow.task_queue}
            </Badge>
          )}

          {hasParentComm && (
            <Badge bg="$blue5">
              📡 Parent Communication
            </Badge>
          )}

          {hasBlocking && (
            <Badge bg="$orange5">
              🚫 Has Dependencies
            </Badge>
          )}
        </XStack>

        {/* Parent Communication Indicators */}
        {hasParentComm && (
          <Card bg="$blue2" p="$3" borderWidth={1} borderColor="$blue6">
            <YStack gap="$2">
              <Text fontSize="$3" fontWeight="600" color="$blue11">
                📡 Parent Communication
              </Text>

              {childWorkflow.signal_to_parent && (
                <XStack ai="center" gap="$2">
                  <ArrowUp size={16} color="$blue11" />
                  <YStack gap="$1">
                    <Text fontSize="$2" color="$blue11" fontWeight="500">
                      Signals to Parent
                    </Text>
                    <Text fontSize="$2" color="$blue11" fontFamily="$mono">
                      {childWorkflow.signal_to_parent}
                    </Text>
                    {childWorkflow.work_queue_target && (
                      <Text fontSize="$2" color="$blue10" fontStyle="italic">
                        → Queue: {childWorkflow.work_queue_target}
                      </Text>
                    )}
                  </YStack>
                </XStack>
              )}

              {childWorkflow.query_parent && (
                <XStack ai="center" gap="$2">
                  <Search size={16} color="$teal11" />
                  <YStack gap="$1">
                    <Text fontSize="$2" color="$teal11" fontWeight="500">
                      Queries Parent
                    </Text>
                    <Text fontSize="$2" color="$teal11" fontFamily="$mono">
                      {childWorkflow.query_parent}
                    </Text>
                  </YStack>
                </XStack>
              )}

              <Text fontSize="$2" color="$blue11" fontStyle="italic" mt="$2">
                This child workflow can communicate with its parent coordinator
              </Text>
            </YStack>
          </Card>
        )}

        {/* Blocking Dependencies */}
        {hasBlocking && (
          <Card bg="$orange2" p="$3" borderWidth={1} borderColor="$orange6">
            <YStack gap="$2">
              <XStack ai="center" gap="$2">
                <PauseCircle size={16} color="$orange11" />
                <Text fontSize="$3" fontWeight="600" color="$orange11">
                  Blocking Dependencies
                </Text>
              </XStack>

              {childWorkflow.block_until_queue && (
                <YStack gap="$1">
                  <Text fontSize="$2" color="$orange11" fontWeight="500">
                    Waits for Queue:
                  </Text>
                  <Text fontSize="$2" color="$orange11" fontFamily="$mono">
                    {childWorkflow.block_until_queue}
                  </Text>
                </YStack>
              )}

              {childWorkflow.block_until_work_items && childWorkflow.block_until_work_items.length > 0 && (
                <YStack gap="$1">
                  <Text fontSize="$2" color="$orange11" fontWeight="500">
                    Waits for Work Items:
                  </Text>
                  {childWorkflow.block_until_work_items.map((item, idx) => (
                    <Text key={idx} fontSize="$2" color="$orange11" fontFamily="$mono">
                      • {item}
                    </Text>
                  ))}
                </YStack>
              )}

              <Text fontSize="$2" color="$orange11" fontStyle="italic" mt="$2">
                This workflow will not start until its dependencies complete
              </Text>
            </YStack>
          </Card>
        )}

        {/* Configuration Details */}
        {showDetails && childWorkflow.config && (
          <>
            <Separator />
            <Card bg="$gray2" p="$3" borderWidth={1} borderColor="$gray6">
              <YStack gap="$2">
                <Text fontSize="$3" fontWeight="600" color="$gray12">
                  Configuration
                </Text>
                <Text fontSize="$1" fontFamily="$mono" color="$gray11">
                  {JSON.stringify(childWorkflow.config, null, 2)}
                </Text>
              </YStack>
            </Card>
          </>
        )}

        {/* Execution Stats (TODO: From runtime) */}
        <XStack gap="$3" jc="space-around" p="$3" bg="$gray2" borderRadius="$4">
          <YStack ai="center" gap="$1">
            <Text fontSize="$2" color="$gray11">Total Executions</Text>
            <Text fontSize="$5" fontWeight="600">-</Text>
          </YStack>
          <YStack ai="center" gap="$1">
            <Text fontSize="$2" color="$gray11">Success Rate</Text>
            <Text fontSize="$5" fontWeight="600">-</Text>
          </YStack>
          <YStack ai="center" gap="$1">
            <Text fontSize="$2" color="$gray11">Avg Duration</Text>
            <Text fontSize="$5" fontWeight="600">-</Text>
          </YStack>
        </XStack>
      </YStack>
    </Card>
  );
}

