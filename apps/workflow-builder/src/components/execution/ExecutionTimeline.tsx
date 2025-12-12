'use client';

import { YStack, XStack, Card, Text, Circle } from 'tamagui';
import { CheckCircle, XCircle, Loader, PlayCircle, AlertTriangle } from 'lucide-react';
import { format } from 'date-fns';

interface ExecutionTimelineProps {
  execution: {
    id: string;
    status: string;
    startedAt: string | Date;
    completedAt?: string | Date | null;
    error?: string | null;
    temporalWorkflowId?: string | null;
    temporalRunId?: string | null;
  };
}

export function ExecutionTimeline({ execution }: ExecutionTimelineProps) {
  const events = buildTimelineEvents(execution);

  return (
    <Card p="$4" borderWidth={1} borderColor="$borderColor">
      <Text fontSize="$5" fontWeight="600" mb="$4">
        Execution Timeline
      </Text>

      <YStack gap="$4" position="relative">
        {/* Timeline line */}
        <YStack
          position="absolute"
          left={16}
          top={20}
          bottom={20}
          width={2}
          bg="$gray6"
          zIndex={0}
        />

        {events.map((event, i) => (
          <TimelineEvent
            key={i}
            event={event}
            isLast={i === events.length - 1}
          />
        ))}
      </YStack>
    </Card>
  );
}

interface TimelineEvent {
  type: 'start' | 'complete' | 'fail' | 'running' | 'building' | 'cancelled' | 'timed_out';
  time: Date;
  label: string;
  description?: string;
  icon: React.ComponentType<{ size?: number; color?: string }>;
  color: string;
  bg: string;
  border: string;
}

function buildTimelineEvents(execution: ExecutionTimelineProps['execution']): TimelineEvent[] {
  const events: TimelineEvent[] = [];

  // Start event
  events.push({
    type: 'start',
    time: new Date(execution.startedAt),
    label: 'Execution Started',
    description: execution.temporalWorkflowId
      ? `Temporal Workflow ID: ${execution.temporalWorkflowId}`
      : undefined,
    icon: PlayCircle,
    color: '$blue11',
    bg: '$blue2',
    border: '$blue6',
  });

  // End event based on status
  if (execution.completedAt) {
    switch (execution.status) {
      case 'completed':
        events.push({
          type: 'complete',
          time: new Date(execution.completedAt),
          label: 'Execution Completed',
          description: 'Workflow finished successfully',
          icon: CheckCircle,
          color: '$green11',
          bg: '$green2',
          border: '$green6',
        });
        break;

      case 'failed':
        events.push({
          type: 'fail',
          time: new Date(execution.completedAt),
          label: 'Execution Failed',
          description: execution.error || 'Workflow execution failed',
          icon: XCircle,
          color: '$red11',
          bg: '$red2',
          border: '$red6',
        });
        break;

      case 'cancelled':
        events.push({
          type: 'cancelled',
          time: new Date(execution.completedAt),
          label: 'Execution Cancelled',
          description: 'Workflow was cancelled',
          icon: XCircle,
          color: '$gray11',
          bg: '$gray2',
          border: '$gray6',
        });
        break;

      case 'timed_out':
        events.push({
          type: 'timed_out',
          time: new Date(execution.completedAt),
          label: 'Execution Timed Out',
          description: 'Workflow exceeded maximum execution time',
          icon: AlertTriangle,
          color: '$orange11',
          bg: '$orange2',
          border: '$orange6',
        });
        break;
    }
  } else {
    // Still running
    if (execution.status === 'building') {
      events.push({
        type: 'building',
        time: new Date(),
        label: 'Building Workflow...',
        description: 'Compiling and preparing workflow for execution',
        icon: Loader,
        color: '$yellow11',
        bg: '$yellow2',
        border: '$yellow6',
      });
    } else if (execution.status === 'running') {
      events.push({
        type: 'running',
        time: new Date(),
        label: 'Execution In Progress...',
        description: 'Workflow is currently running',
        icon: Loader,
        color: '$blue11',
        bg: '$blue2',
        border: '$blue6',
      });
    }
  }

  return events;
}

interface TimelineEventProps {
  event: TimelineEvent;
  isLast: boolean;
}

function TimelineEvent({ event }: TimelineEventProps) {
  const Icon = event.icon;
  const isOngoing = event.type === 'running' || event.type === 'building';

  return (
    <XStack gap="$3" ai="flex-start" position="relative" zIndex={1}>
      {/* Icon Circle */}
      <Circle
        size={32}
        bg={event.bg}
        borderWidth={2}
        borderColor={event.border}
        ai="center"
        jc="center"
        animation={isOngoing ? 'bouncy' : undefined}
      >
        <Icon
          size={16}
          color={event.color}
          className={isOngoing ? 'animate-spin' : ''}
        />
      </Circle>

      {/* Event Content */}
      <YStack f={1} gap="$2">
        <Text fontSize="$4" fontWeight="600" color={event.color}>
          {event.label}
        </Text>

        <Text fontSize="$2" color="$gray11">
          {format(event.time, 'PPpp')}
        </Text>

        {event.description && (
          <Card p="$3" bg={event.bg} borderWidth={1} borderColor={event.border}>
            <Text fontSize="$2" color={event.color}>
              {event.description}
            </Text>
          </Card>
        )}
      </YStack>
    </XStack>
  );
}
