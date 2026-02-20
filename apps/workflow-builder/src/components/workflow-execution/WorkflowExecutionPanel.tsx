'use client';

import { YStack, XStack, Text, Card, Progress, ScrollView, Button, Separator } from 'tamagui';
import { Badge } from '../shared/Badge';
import { CheckCircle, Clock, XCircle, AlertCircle, Play, RefreshCw } from 'lucide-react';
import { format } from 'date-fns';

interface ExecutionStep {
  id: string;
  name: string;
  status: 'pending' | 'running' | 'completed' | 'failed';
  startedAt?: Date;
  completedAt?: Date;
  duration?: number;
  result?: any;
  error?: string;
}

interface WorkflowExecutionPanelProps {
  workflowId: string;
  execution: {
    id: string;
    status: 'building' | 'running' | 'completed' | 'failed';
    startedAt: Date;
    completedAt?: Date;
    currentStep?: string;
    steps: ExecutionStep[];
    progress: number;
    result?: any;
    error?: string;
  } | null;
  onRun?: () => void;
  onRetry?: () => void;
}

export function WorkflowExecutionPanel({
  execution,
  onRun,
  onRetry,
}: WorkflowExecutionPanelProps) {
  if (!execution) {
    return (
      <Card p="$6" ai="center" gap="$4" bg="$gray2" data-testid="execution-panel">
        <YStack ai="center" gap="$2">
          <Play size={40} color="$gray10" />
          <Text fontSize="$5" fontWeight="600" color="$gray12">
            Ready to Run
          </Text>
          <Text fontSize="$3" color="$gray11" textAlign="center">
            Click "Build Workflow" to compile and execute your workflow
          </Text>
        </YStack>
        {onRun && (
          <Button
            icon={Play}
            size="$4"
            themeInverse
            onPress={onRun}
            data-testid="build-run-workflow-button"
          >
            Build & Run Workflow
          </Button>
        )}
      </Card>
    );
  }

  const statusConfig = {
    building: { icon: RefreshCw, color: '$blue9', label: 'Building' },
    running: { icon: Play, color: '$orange9', label: 'Running' },
    completed: { icon: CheckCircle, color: '$green9', label: 'Completed' },
    failed: { icon: XCircle, color: '$red9', label: 'Failed' },
  };

  const config = statusConfig[execution.status];
  const Icon = config.icon;

  return (
    <YStack f={1} gap="$4" p="$4" bg="$background" borderLeftWidth={1} borderLeftColor="$borderColor" minWidth={350} maxWidth={450} data-testid="execution-panel" data-execution-id={execution.id}>
      {/* Header */}
      <YStack gap="$2">
        <XStack ai="center" jc="space-between">
          <Text fontSize="$5" fontWeight="600">Workflow Execution</Text>
          {execution.status === 'completed' && onRetry && (
            <Button
              size="$2"
              icon={RefreshCw}
              onPress={onRetry}
              chromeless
              data-testid="retry-execution-button"
            >
              Run Again
            </Button>
          )}
        </XStack>

        {/* Status Badge */}
        <XStack ai="center" gap="$3">
          <Icon size={20} color={config.color} />
          <Badge bg={`${config.color.replace('9', '3')}`} borderColor={`${config.color.replace('9', '6')}`} data-testid="execution-status">
            {config.label}
          </Badge>
          {execution.status === 'running' && execution.currentStep && (
            <Text fontSize="$2" color="$gray11">
              {execution.currentStep}
            </Text>
          )}
        </XStack>
      </YStack>

      {/* Progress Bar */}
      {(execution.status === 'building' || execution.status === 'running') && (
        <YStack gap="$2">
          <XStack jc="space-between">
            <Text fontSize="$2" color="$gray11">Progress</Text>
            <Text fontSize="$2" color="$gray11">{execution.progress}%</Text>
          </XStack>
          <Progress value={execution.progress}>
            <Progress.Indicator animation="bouncy" />
          </Progress>
        </YStack>
      )}

      {/* Timing Info */}
      <Card p="$3" bg="$gray2">
        <YStack gap="$2">
          <XStack jc="space-between">
            <Text fontSize="$2" color="$gray11">Started</Text>
            <Text fontSize="$2" color="$gray12">
              {format(execution.startedAt, 'MMM d, h:mm:ss a')}
            </Text>
          </XStack>
          {execution.completedAt && (
            <>
              <XStack jc="space-between">
                <Text fontSize="$2" color="$gray11">Completed</Text>
                <Text fontSize="$2" color="$gray12">
                  {format(execution.completedAt, 'MMM d, h:mm:ss a')}
                </Text>
              </XStack>
              <XStack jc="space-between">
                <Text fontSize="$2" color="$gray11">Duration</Text>
                <Text fontSize="$2" color="$gray12" data-testid="execution-duration">
                  {Math.round((execution.completedAt.getTime() - execution.startedAt.getTime()) / 1000)}s
                </Text>
              </XStack>
            </>
          )}
        </YStack>
      </Card>

      <Separator />

      {/* Execution Steps */}
      <YStack gap="$2">
        <Text fontSize="$4" fontWeight="600">Execution Steps</Text>
        <ScrollView f={1} maxHeight={400}>
          <YStack gap="$2">
            {execution.steps.map((step, index) => (
              <ExecutionStepCard key={step.id} step={step} index={index} />
            ))}
          </YStack>
        </ScrollView>
      </YStack>

      {/* Error Display */}
      {execution.error && (
        <Card p="$3" bg="$red2" borderWidth={1} borderColor="$red6" data-testid="execution-error">
          <YStack gap="$2">
            <XStack ai="center" gap="$2">
              <AlertCircle size={16} color="$red11" />
              <Text fontSize="$3" fontWeight="600" color="$red11">
                Error
              </Text>
            </XStack>
            <Text fontSize="$2" color="$red11">
              {execution.error}
            </Text>
          </YStack>
        </Card>
      )}

      {/* Result Display */}
      {execution.result && execution.status === 'completed' && (
        <Card p="$3" bg="$green2" borderWidth={1} borderColor="$green6" data-testid="execution-output">
          <YStack gap="$2">
            <Text fontSize="$3" fontWeight="600" color="$green11">
              Result
            </Text>
            <Text fontSize="$2" color="$green11" fontFamily="$mono">
              {JSON.stringify(execution.result, null, 2)}
            </Text>
          </YStack>
        </Card>
      )}
    </YStack>
  );
}

function ExecutionStepCard({ step, index }: { step: ExecutionStep; index: number }) {
  const statusConfig = {
    pending: { icon: Clock, color: '$gray9', bg: '$gray3' },
    running: { icon: Play, color: '$orange9', bg: '$orange3' },
    completed: { icon: CheckCircle, color: '$green9', bg: '$green3' },
    failed: { icon: XCircle, color: '$red9', bg: '$red3' },
  };

  const config = statusConfig[step.status];
  const Icon = config.icon;

  return (
    <Card p="$3" bg={config.bg} borderWidth={1} borderColor={`${config.color.replace('9', '6')}`}>
      <YStack gap="$2">
        <XStack ai="center" jc="space-between">
          <XStack ai="center" gap="$2">
            <Icon size={16} color={config.color} />
            <Text fontSize="$3" fontWeight="600" color={config.color}>
              {index + 1}. {step.name}
            </Text>
          </XStack>
          {step.duration && (
            <Text fontSize="$2" color={config.color}>
              {step.duration}ms
            </Text>
          )}
        </XStack>

        {step.result && step.status === 'completed' && (
          <Text fontSize="$2" color={config.color} numberOfLines={2}>
            ✓ {typeof step.result === 'string' ? step.result : JSON.stringify(step.result)}
          </Text>
        )}

        {step.error && step.status === 'failed' && (
          <Text fontSize="$2" color={config.color}>
            ✗ {step.error}
          </Text>
        )}
      </YStack>
    </Card>
  );
}

