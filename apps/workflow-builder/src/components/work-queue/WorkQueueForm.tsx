'use client';

import { useState } from 'react';
import { YStack, XStack, Text, Button, Input, TextArea, Label, Switch, Select, Adapt, Sheet } from 'tamagui';
import { Check, ChevronDown } from 'lucide-react';
import { api as trpc } from '@/lib/trpc/client';
import { validateWorkQueueConfig, generateDefaultSignalName, generateDefaultQueryName } from '@/utils/work-queue-utils';

interface WorkQueueFormProps {
  workflowId: string;
  onSuccess?: () => void;
  onCancel?: () => void;
}

export function WorkQueueForm({ workflowId, onSuccess, onCancel }: WorkQueueFormProps) {
  const [queueName, setQueueName] = useState('');
  const [description, setDescription] = useState('');
  const [signalName, setSignalName] = useState('');
  const [queryName, setQueryName] = useState('');
  const [maxSize, setMaxSize] = useState('');
  const [priority, setPriority] = useState<'fifo' | 'lifo' | 'priority'>('fifo');
  const [deduplicate, setDeduplicate] = useState(false);
  const [errors, setErrors] = useState<Record<string, string>>({});

  // Auto-generate signal/query names when queue name changes
  const handleQueueNameChange = (value: string) => {
    setQueueName(value);
    
    // Only auto-generate if user hasn't manually set them
    if (!signalName || signalName === generateDefaultSignalName(queueName)) {
      setSignalName(generateDefaultSignalName(value));
    }
    if (!queryName || queryName === generateDefaultQueryName(queueName)) {
      setQueryName(generateDefaultQueryName(value));
    }
  };

  const createMutation = trpc.workQueues.create.useMutation({
    onSuccess: () => {
      onSuccess?.();
    },
    onError: (error) => {
      setErrors({ submit: error.message });
    },
  });

  const handleSubmit = async () => {
    // Validate
    const validation = validateWorkQueueConfig({
      queue_name: queueName,
      signal_name: signalName || generateDefaultSignalName(queueName),
      query_name: queryName || generateDefaultQueryName(queueName),
      max_size: maxSize ? parseInt(maxSize) : null,
      priority,
    });

    if (!validation.valid) {
      const newErrors: Record<string, string> = {};
      validation.errors.forEach(err => {
        newErrors[err.field] = err.message;
      });
      setErrors(newErrors);
      return;
    }

    // Clear errors
    setErrors({});

    // Submit
    await createMutation.mutateAsync({
      workflowId,
      queueName,
      description: description || undefined,
      signalName: signalName || undefined,
      queryName: queryName || undefined,
      maxSize: maxSize ? parseInt(maxSize) : undefined,
      priority,
      deduplicate,
    });
  };

  return (
    <YStack gap="$4">
      <Text fontSize="$6" fontWeight="600">Create Work Queue</Text>

      {/* Queue Name */}
      <YStack gap="$2">
        <Label htmlFor="queueName">
          Queue Name <Text color="$red10">*</Text>
        </Label>
        <Input
          id="queueName"
          value={queueName}
          onChangeText={handleQueueNameChange}
          placeholder="e.g., plansToWrite"
        />
        {errors.queue_name && (
          <Text fontSize="$2" color="$red10">{errors.queue_name}</Text>
        )}
        <Text fontSize="$2" color="$gray11">
          Use camelCase or snake_case. Must start with a letter.
        </Text>
      </YStack>

      {/* Description */}
      <YStack gap="$2">
        <Label htmlFor="description">Description</Label>
        <TextArea
          id="description"
          value={description}
          onChangeText={setDescription}
          placeholder="What does this queue hold?"
          numberOfLines={2}
        />
      </YStack>

      {/* Auto-generated handlers */}
      <YStack gap="$3" p="$3" bg="$blue2" borderRadius="$4" borderWidth={1} borderColor="$blue6">
        <Text fontSize="$4" fontWeight="600" color="$blue11">
          ⚡ Auto-Generated Handlers
        </Text>

        {/* Signal Name */}
        <YStack gap="$2">
          <Label htmlFor="signalName">Signal Handler Name</Label>
          <Input
            id="signalName"
            value={signalName}
            onChangeText={setSignalName}
            placeholder={generateDefaultSignalName(queueName) || 'addToQueue'}
          />
          {errors.signal_name && (
            <Text fontSize="$2" color="$red10">{errors.signal_name}</Text>
          )}
          <Text fontSize="$2" color="$blue11">
            Child workflows call this to add items
          </Text>
        </YStack>

        {/* Query Name */}
        <YStack gap="$2">
          <Label htmlFor="queryName">Query Handler Name</Label>
          <Input
            id="queryName"
            value={queryName}
            onChangeText={setQueryName}
            placeholder={generateDefaultQueryName(queueName) || 'getQueue'}
          />
          {errors.query_name && (
            <Text fontSize="$2" color="$red10">{errors.query_name}</Text>
          )}
          <Text fontSize="$2" color="$blue11">
            Child workflows call this to check queue status
          </Text>
        </YStack>
      </YStack>

      {/* Queue Settings */}
      <YStack gap="$3" p="$3" bg="$gray2" borderRadius="$4" borderWidth={1} borderColor="$gray6">
        <Text fontSize="$4" fontWeight="600">Queue Settings</Text>

        {/* Priority */}
        <YStack gap="$2">
          <Label htmlFor="priority">Priority</Label>
          <Select value={priority} onValueChange={(value) => setPriority(value as any)}>
            <Select.Trigger width="100%" iconAfter={ChevronDown}>
              <Select.Value placeholder="Select priority" />
            </Select.Trigger>

            <Adapt when="sm" platform="touch">
              <Sheet modal dismissOnSnapToBottom>
                <Sheet.Frame>
                  <Sheet.ScrollView>
                    <Adapt.Contents />
                  </Sheet.ScrollView>
                </Sheet.Frame>
                <Sheet.Overlay />
              </Sheet>
            </Adapt>

            <Select.Content zIndex={200000}>
              <Select.ScrollUpButton />
              <Select.Viewport>
                <Select.Group>
                  <Select.Item index={0} value="fifo">
                    <Select.ItemText>FIFO (First In, First Out)</Select.ItemText>
                    <Select.ItemIndicator>
                      <Check size={16} />
                    </Select.ItemIndicator>
                  </Select.Item>
                  <Select.Item index={1} value="lifo">
                    <Select.ItemText>LIFO (Last In, First Out)</Select.ItemText>
                    <Select.ItemIndicator>
                      <Check size={16} />
                    </Select.ItemIndicator>
                  </Select.Item>
                  <Select.Item index={2} value="priority">
                    <Select.ItemText>Priority (Highest First)</Select.ItemText>
                    <Select.ItemIndicator>
                      <Check size={16} />
                    </Select.ItemIndicator>
                  </Select.Item>
                </Select.Group>
              </Select.Viewport>
              <Select.ScrollDownButton />
            </Select.Content>
          </Select>
          <Text fontSize="$2" color="$gray11">
            How items are processed from the queue
          </Text>
        </YStack>

        {/* Max Size */}
        <YStack gap="$2">
          <Label htmlFor="maxSize">Max Size (optional)</Label>
          <Input
            id="maxSize"
            value={maxSize}
            onChangeText={setMaxSize}
            placeholder="Leave empty for unlimited"
            keyboardType="numeric"
          />
          {errors.max_size && (
            <Text fontSize="$2" color="$red10">{errors.max_size}</Text>
          )}
          <Text fontSize="$2" color="$gray11">
            Maximum number of items allowed in queue
          </Text>
        </YStack>

        {/* Deduplicate */}
        <XStack ai="center" gap="$3">
          <Switch
            id="deduplicate"
            checked={deduplicate}
            onCheckedChange={setDeduplicate}
          >
            <Switch.Thumb animation="quick" />
          </Switch>
          <Label htmlFor="deduplicate" flex={1}>
            <YStack gap="$1">
              <Text>Enable Deduplication</Text>
              <Text fontSize="$2" color="$gray11">
                Prevent duplicate work items from being added
              </Text>
            </YStack>
          </Label>
        </XStack>
      </YStack>

      {/* Submit Error */}
      {errors.submit && (
        <Text fontSize="$3" color="$red10" textAlign="center">
          {errors.submit}
        </Text>
      )}

      {/* Actions */}
      <XStack gap="$3" jc="flex-end">
        <Button onPress={onCancel} disabled={createMutation.isLoading}>
          Cancel
        </Button>
        <Button
          themeInverse
          onPress={handleSubmit}
          disabled={createMutation.isLoading || !queueName}
        >
          {createMutation.isLoading ? 'Creating...' : 'Create Work Queue'}
        </Button>
      </XStack>
    </YStack>
  );
}

