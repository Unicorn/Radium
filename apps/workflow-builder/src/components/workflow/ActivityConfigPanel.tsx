/**
 * Activity Configuration Panel - Configure activity-specific settings
 * Used for configuring timeouts, retry policies, and activity names
 */

'use client';

import { YStack, Text, Input, Label, Select } from 'tamagui';
import { useState, useEffect } from 'react';
import type { WorkflowNode } from '@/types/workflow';

interface ActivityConfigPanelProps {
  node: WorkflowNode;
  onUpdate: (updated: Partial<WorkflowNode>) => void;
}

const RETRY_STRATEGIES = [
  { value: 'none', label: 'No Retry' },
  { value: 'keep-trying', label: 'Keep Trying' },
  { value: 'fail-after-x', label: 'Fail After X Attempts' },
  { value: 'exponential-backoff', label: 'Exponential Backoff' },
] as const;

export function ActivityConfigPanel({ node, onUpdate }: ActivityConfigPanelProps) {
  const [activityName, setActivityName] = useState(node.data.componentName || '');
  const [timeout, setTimeout] = useState('');
  const [retryStrategy, setRetryStrategy] = useState<string>(
    node.data.retryPolicy?.strategy || 'none'
  );
  const [maxAttempts, setMaxAttempts] = useState<string>(
    node.data.retryPolicy?.maxAttempts?.toString() || '3'
  );
  const [initialInterval, setInitialInterval] = useState<string>(
    node.data.retryPolicy?.initialInterval || '1s'
  );
  const [maxInterval, setMaxInterval] = useState<string>(
    node.data.retryPolicy?.maxInterval || '1m'
  );
  const [backoffCoefficient, setBackoffCoefficient] = useState<string>(
    node.data.retryPolicy?.backoffCoefficient?.toString() || '2'
  );

  useEffect(() => {
    setActivityName(node.data.componentName || '');
    setTimeout('');
    setRetryStrategy(node.data.retryPolicy?.strategy || 'none');
    setMaxAttempts(node.data.retryPolicy?.maxAttempts?.toString() || '3');
    setInitialInterval(node.data.retryPolicy?.initialInterval || '1s');
    setMaxInterval(node.data.retryPolicy?.maxInterval || '1m');
    setBackoffCoefficient(node.data.retryPolicy?.backoffCoefficient?.toString() || '2');
  }, [node]);

  const handleSave = () => {
    const retryPolicy =
      retryStrategy === 'none'
        ? undefined
        : {
            strategy: retryStrategy as 'keep-trying' | 'fail-after-x' | 'exponential-backoff',
            ...(retryStrategy === 'fail-after-x' && {
              maxAttempts: parseInt(maxAttempts) || 3,
            }),
            ...(retryStrategy === 'exponential-backoff' && {
              initialInterval,
              maxInterval,
              backoffCoefficient: parseFloat(backoffCoefficient) || 2,
            }),
          };

    onUpdate({
      data: {
        ...node.data,
        componentName: activityName,
        retryPolicy,
      },
    });
  };

  return (
    <YStack gap="$4" paddingTop="$4">
      <YStack gap="$2">
        <Label htmlFor="activity-name" fontSize="$3" fontWeight="600">
          Activity Name
        </Label>
        <Input
          id="activity-name"
          size="$4"
          value={activityName}
          onChangeText={setActivityName}
          onBlur={handleSave}
          placeholder="myActivity"
        />
        <Text fontSize="$2" color="$gray11">
          TypeScript function name for this activity
        </Text>
      </YStack>

      <YStack gap="$2">
        <Label htmlFor="timeout" fontSize="$3" fontWeight="600">
          Timeout
        </Label>
        <Input
          id="timeout"
          size="$4"
          value={timeout}
          onChangeText={setTimeout}
          onBlur={handleSave}
          placeholder="5 minutes"
        />
        <Text fontSize="$2" color="$gray11">
          Maximum execution time (e.g., "5 minutes", "30 seconds")
        </Text>
      </YStack>

      <YStack gap="$2">
        <Label htmlFor="retry-strategy" fontSize="$3" fontWeight="600">
          Retry Policy
        </Label>
        <Select
          id="retry-strategy"
          value={retryStrategy}
          onValueChange={(value) => {
            setRetryStrategy(value);
            // Trigger save after strategy change
            setTimeout(handleSave, 0);
          }}
        >
          <Select.Trigger width="100%">
            <Select.Value placeholder="Select retry strategy" />
          </Select.Trigger>

          <Select.Content>
            <Select.Viewport>
              {RETRY_STRATEGIES.map((strategy) => (
                <Select.Item key={strategy.value} value={strategy.value}>
                  <Select.ItemText>{strategy.label}</Select.ItemText>
                </Select.Item>
              ))}
            </Select.Viewport>
          </Select.Content>
        </Select>
        <Text fontSize="$2" color="$gray11">
          How to handle activity failures
        </Text>
      </YStack>

      {retryStrategy === 'fail-after-x' && (
        <YStack gap="$2">
          <Label htmlFor="max-attempts" fontSize="$3" fontWeight="600">
            Max Attempts
          </Label>
          <Input
            id="max-attempts"
            size="$4"
            value={maxAttempts}
            onChangeText={setMaxAttempts}
            onBlur={handleSave}
            placeholder="3"
            inputMode="numeric"
          />
          <Text fontSize="$2" color="$gray11">
            Maximum number of retry attempts
          </Text>
        </YStack>
      )}

      {retryStrategy === 'exponential-backoff' && (
        <>
          <YStack gap="$2">
            <Label htmlFor="initial-interval" fontSize="$3" fontWeight="600">
              Initial Interval
            </Label>
            <Input
              id="initial-interval"
              size="$4"
              value={initialInterval}
              onChangeText={setInitialInterval}
              onBlur={handleSave}
              placeholder="1s"
            />
            <Text fontSize="$2" color="$gray11">
              Starting delay between retries (e.g., "1s", "500ms")
            </Text>
          </YStack>

          <YStack gap="$2">
            <Label htmlFor="max-interval" fontSize="$3" fontWeight="600">
              Max Interval
            </Label>
            <Input
              id="max-interval"
              size="$4"
              value={maxInterval}
              onChangeText={setMaxInterval}
              onBlur={handleSave}
              placeholder="1m"
            />
            <Text fontSize="$2" color="$gray11">
              Maximum delay between retries
            </Text>
          </YStack>

          <YStack gap="$2">
            <Label htmlFor="backoff-coefficient" fontSize="$3" fontWeight="600">
              Backoff Coefficient
            </Label>
            <Input
              id="backoff-coefficient"
              size="$4"
              value={backoffCoefficient}
              onChangeText={setBackoffCoefficient}
              onBlur={handleSave}
              placeholder="2"
              inputMode="decimal"
            />
            <Text fontSize="$2" color="$gray11">
              Multiplier for exponential backoff (typically 2)
            </Text>
          </YStack>
        </>
      )}
    </YStack>
  );
}
