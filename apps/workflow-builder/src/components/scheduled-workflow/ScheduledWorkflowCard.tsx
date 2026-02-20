'use client';

import { Card, YStack, XStack, Text, Button, Separator } from 'tamagui';
import { Badge } from '../shared/Badge';
import { Clock, Trash2, Edit, Play, Pause } from 'lucide-react';
import { useState } from 'react';
import { getHumanReadableDescription } from '@/utils/cron-validation';

interface ScheduledWorkflowCardProps {
  scheduledWorkflow: {
    id: string;
    name: string;
    description?: string;
    schedule_spec: string;
    signal_to_parent_name?: string;
    signal_to_parent_queue_name?: string;
    start_immediately: boolean;
    end_with_parent: boolean;
    max_runs?: number;
    is_paused?: boolean;
  };
  onUpdate?: () => void;
}

export function ScheduledWorkflowCard({ scheduledWorkflow, onUpdate }: ScheduledWorkflowCardProps) {
  const [showDetails, setShowDetails] = useState(false);
  
  const handleDelete = async () => {
    if (confirm(`Delete scheduled workflow "${scheduledWorkflow.name}"? This cannot be undone.`)) {
      // TODO: Implement delete mutation
      console.log('Delete scheduled workflow:', scheduledWorkflow.id);
      onUpdate?.();
    }
  };

  const handleTogglePause = async () => {
    // TODO: Implement pause/resume mutation
    console.log('Toggle pause for:', scheduledWorkflow.id);
    onUpdate?.();
  };

  const cronDescription = getHumanReadableDescription(scheduledWorkflow.schedule_spec);

  return (
    <Card p="$4" borderWidth={1} borderColor="$borderColor">
      <YStack gap="$3">
        {/* Header */}
        <XStack ai="center" jc="space-between">
          <XStack ai="center" gap="$3">
            <Clock size={24} color="$purple9" />
            <YStack gap="$1">
              <Text fontSize="$6" fontWeight="600">
                {scheduledWorkflow.name}
              </Text>
              {scheduledWorkflow.description && (
                <Text fontSize="$3" color="$gray11">
                  {scheduledWorkflow.description}
                </Text>
              )}
            </YStack>
          </XStack>

          <XStack gap="$2">
            <Button
              size="$2"
              icon={scheduledWorkflow.is_paused ? Play : Pause}
              onPress={handleTogglePause}
              chromeless
            >
              {scheduledWorkflow.is_paused ? 'Resume' : 'Pause'}
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
              chromeless
              color="$red10"
            >
              Delete
            </Button>
          </XStack>
        </XStack>

        {/* Status Badges */}
        <XStack gap="$3" flexWrap="wrap">
          <Badge bg={scheduledWorkflow.is_paused ? "$orange5" : "$green5"}>
            {scheduledWorkflow.is_paused ? 'Paused' : 'Active'}
          </Badge>
          
          {scheduledWorkflow.start_immediately && (
            <Badge>Auto-Start</Badge>
          )}

          {scheduledWorkflow.end_with_parent && (
            <Badge>Ends with Parent</Badge>
          )}

          {scheduledWorkflow.max_runs && (
            <Badge>Max Runs: {scheduledWorkflow.max_runs}</Badge>
          )}
        </XStack>

        {/* Schedule Display */}
        <Card bg="$purple2" p="$3" borderWidth={1} borderColor="$purple6">
          <YStack gap="$2">
            <Text fontSize="$3" fontWeight="600" color="$purple11">
              📅 Schedule
            </Text>
            <Text fontSize="$4" color="$purple11" fontWeight="500">
              {cronDescription}
            </Text>
            <Text fontSize="$2" color="$purple10" fontFamily="$mono">
              {scheduledWorkflow.schedule_spec}
            </Text>
          </YStack>
        </Card>

        {/* Parent Communication */}
        {scheduledWorkflow.signal_to_parent_name && (
          <Card bg="$blue2" p="$3" borderWidth={1} borderColor="$blue6">
            <YStack gap="$2">
              <Text fontSize="$3" fontWeight="600" color="$blue11">
                🔗 Signals to Parent
              </Text>
              <XStack ai="center" gap="$2">
                <Text fontSize="$2" color="$blue11">
                  <Text fontWeight="600">Signal:</Text> {scheduledWorkflow.signal_to_parent_name}
                </Text>
              </XStack>
              {scheduledWorkflow.signal_to_parent_queue_name && (
                <XStack ai="center" gap="$2">
                  <Text fontSize="$2" color="$blue11">
                    <Text fontWeight="600">Queue:</Text> {scheduledWorkflow.signal_to_parent_queue_name}
                  </Text>
                </XStack>
              )}
              <Text fontSize="$2" color="$blue11" fontStyle="italic">
                Discovered work will be added to the parent coordinator workflow
              </Text>
            </YStack>
          </Card>
        )}

        {/* Execution Stats (TODO: From runtime) */}
        <XStack gap="$3" jc="space-around" p="$3" bg="$gray2" borderRadius="$4">
          <YStack ai="center" gap="$1">
            <Text fontSize="$2" color="$gray11">Total Runs</Text>
            <Text fontSize="$5" fontWeight="600">-</Text>
          </YStack>
          <YStack ai="center" gap="$1">
            <Text fontSize="$2" color="$gray11">Last Run</Text>
            <Text fontSize="$5" fontWeight="600">-</Text>
          </YStack>
          <YStack ai="center" gap="$1">
            <Text fontSize="$2" color="$gray11">Next Run</Text>
            <Text fontSize="$5" fontWeight="600">-</Text>
          </YStack>
        </XStack>
      </YStack>
    </Card>
  );
}

