'use client';

import { useState } from 'react';
import { YStack, XStack, Text, Button, Input, TextArea, Label, Switch, Select, Adapt, Sheet } from 'tamagui';
import { Check, ChevronDown } from 'lucide-react';
import { CronExpressionBuilder } from '../cron/CronExpressionBuilder';

interface ScheduledWorkflowFormProps {
  parentWorkflowId: string;
  onSuccess?: () => void;
  onCancel?: () => void;
}

export function ScheduledWorkflowForm({
  parentWorkflowId,
  onSuccess,
  onCancel,
}: ScheduledWorkflowFormProps) {
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [cronExpression, setCronExpression] = useState('');
  const [isCronValid, setIsCronValid] = useState(false);
  const [signalToParent, setSignalToParent] = useState(true);
  const [signalName, setSignalName] = useState('');
  const [queueName, setQueueName] = useState('');
  const [startImmediately, setStartImmediately] = useState(true);
  const [endWithParent, setEndWithParent] = useState(true);
  const [maxRuns, setMaxRuns] = useState('');
  const [errors, setErrors] = useState<Record<string, string>>({});

  // TODO: Replace with actual tRPC mutation
  const handleSubmit = async () => {
    // Validate
    const newErrors: Record<string, string> = {};

    if (!name.trim()) {
      newErrors.name = 'Name is required';
    }

    if (!cronExpression || !isCronValid) {
      newErrors.cron = 'Valid cron expression is required';
    }

    if (signalToParent && !signalName.trim()) {
      newErrors.signalName = 'Signal name is required when signaling to parent';
    }

    if (maxRuns && (parseInt(maxRuns) < 1 || isNaN(parseInt(maxRuns)))) {
      newErrors.maxRuns = 'Max runs must be a positive number';
    }

    if (Object.keys(newErrors).length > 0) {
      setErrors(newErrors);
      return;
    }

    setErrors({});

    // TODO: Submit to backend
    console.log('Creating scheduled workflow:', {
      name,
      description,
      cronExpression,
      signalToParent,
      signalName,
      queueName,
      startImmediately,
      endWithParent,
      maxRuns: maxRuns ? parseInt(maxRuns) : null,
    });

    onSuccess?.();
  };

  return (
    <YStack gap="$4">
      <Text fontSize="$6" fontWeight="600">Create Scheduled Workflow</Text>

      <Text fontSize="$3" color="$gray11">
        A scheduled workflow runs on a cron schedule and can signal the parent coordinator workflow when it discovers work.
      </Text>

      {/* Basic Info */}
      <YStack gap="$3" p="$3" bg="$gray2" borderRadius="$4" borderWidth={1} borderColor="$gray6">
        <Text fontSize="$4" fontWeight="600">Basic Information</Text>

        {/* Name */}
        <YStack gap="$2">
          <Label htmlFor="name">
            Workflow Name <Text color="$red10">*</Text>
          </Label>
          <Input
            id="name"
            value={name}
            onChangeText={setName}
            placeholder="e.g., Check for New Plans"
          />
          {errors.name && (
            <Text fontSize="$2" color="$red10">{errors.name}</Text>
          )}
        </YStack>

        {/* Description */}
        <YStack gap="$2">
          <Label htmlFor="description">Description</Label>
          <TextArea
            id="description"
            value={description}
            onChangeText={setDescription}
            placeholder="What does this scheduled workflow do?"
            numberOfLines={2}
          />
        </YStack>
      </YStack>

      {/* Schedule Configuration */}
      <YStack gap="$3" p="$3" bg="$purple2" borderRadius="$4" borderWidth={1} borderColor="$purple6">
        <Text fontSize="$4" fontWeight="600" color="$purple11">
          📅 Schedule Configuration
        </Text>

        <CronExpressionBuilder
          value={cronExpression}
          onChange={setCronExpression}
          onValidationChange={setIsCronValid}
        />

        {errors.cron && (
          <Text fontSize="$2" color="$red10">{errors.cron}</Text>
        )}
      </YStack>

      {/* Parent Communication */}
      <YStack gap="$3" p="$3" bg="$blue2" borderRadius="$4" borderWidth={1} borderColor="$blue6">
        <Text fontSize="$4" fontWeight="600" color="$blue11">
          🔗 Parent Communication
        </Text>

        {/* Signal to Parent */}
        <XStack ai="center" gap="$3">
          <Switch
            id="signalToParent"
            checked={signalToParent}
            onCheckedChange={setSignalToParent}
          >
            <Switch.Thumb animation="quick" />
          </Switch>
          <Label htmlFor="signalToParent" flex={1}>
            <YStack gap="$1">
              <Text>Signal to Parent Workflow</Text>
              <Text fontSize="$2" color="$blue11">
                Send discovered work to the parent coordinator
              </Text>
            </YStack>
          </Label>
        </XStack>

        {signalToParent && (
          <YStack gap="$3" pl="$6">
            {/* Signal Name */}
            <YStack gap="$2">
              <Label htmlFor="signalName">
                Signal Name <Text color="$red10">*</Text>
              </Label>
              <Input
                id="signalName"
                value={signalName}
                onChangeText={setSignalName}
                placeholder="e.g., addPlanToQueue"
              />
              {errors.signalName && (
                <Text fontSize="$2" color="$red10">{errors.signalName}</Text>
              )}
              <Text fontSize="$2" color="$blue11">
                The signal handler on the parent workflow to call
              </Text>
            </YStack>

            {/* Queue Name (optional) */}
            <YStack gap="$2">
              <Label htmlFor="queueName">Work Queue Name (optional)</Label>
              <Input
                id="queueName"
                value={queueName}
                onChangeText={setQueueName}
                placeholder="e.g., plansToWrite"
              />
              <Text fontSize="$2" color="$blue11">
                If the signal adds to a work queue, specify the queue name
              </Text>
            </YStack>
          </YStack>
        )}
      </YStack>

      {/* Lifecycle Settings */}
      <YStack gap="$3" p="$3" bg="$gray2" borderRadius="$4" borderWidth={1} borderColor="$gray6">
        <Text fontSize="$4" fontWeight="600">Lifecycle Settings</Text>

        {/* Start Immediately */}
        <XStack ai="center" gap="$3">
          <Switch
            id="startImmediately"
            checked={startImmediately}
            onCheckedChange={setStartImmediately}
          >
            <Switch.Thumb animation="quick" />
          </Switch>
          <Label htmlFor="startImmediately" flex={1}>
            <YStack gap="$1">
              <Text>Start Immediately</Text>
              <Text fontSize="$2" color="$gray11">
                Begin schedule when parent workflow starts
              </Text>
            </YStack>
          </Label>
        </XStack>

        {/* End With Parent */}
        <XStack ai="center" gap="$3">
          <Switch
            id="endWithParent"
            checked={endWithParent}
            onCheckedChange={setEndWithParent}
          >
            <Switch.Thumb animation="quick" />
          </Switch>
          <Label htmlFor="endWithParent" flex={1}>
            <YStack gap="$1">
              <Text>End with Parent</Text>
              <Text fontSize="$2" color="$gray11">
                Stop this workflow when parent completes
              </Text>
            </YStack>
          </Label>
        </XStack>

        {/* Max Runs */}
        <YStack gap="$2">
          <Label htmlFor="maxRuns">Maximum Runs (optional)</Label>
          <Input
            id="maxRuns"
            value={maxRuns}
            onChangeText={setMaxRuns}
            placeholder="Leave empty for unlimited"
            keyboardType="numeric"
          />
          {errors.maxRuns && (
            <Text fontSize="$2" color="$red10">{errors.maxRuns}</Text>
          )}
          <Text fontSize="$2" color="$gray11">
            Stop after this many executions
          </Text>
        </YStack>
      </YStack>

      {/* Submit Error */}
      {errors.submit && (
        <Text fontSize="$3" color="$red10" textAlign="center">
          {errors.submit}
        </Text>
      )}

      {/* Actions */}
      <XStack gap="$3" jc="flex-end">
        <Button onPress={onCancel}>
          Cancel
        </Button>
        <Button
          themeInverse
          onPress={handleSubmit}
          disabled={!name || !cronExpression || !isCronValid}
        >
          Create Scheduled Workflow
        </Button>
      </XStack>
    </YStack>
  );
}

