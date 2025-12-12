/**
 * Agent Prompt Editor - Create/Edit agent prompts
 */

'use client';

import { YStack, XStack, Text, Button, Input, TextArea, Label, Select, Adapt, Sheet } from 'tamagui';
import { useState } from 'react';
import { useRouter } from 'next/navigation';
import { api } from '@/lib/trpc/client';
import { Check, ChevronDown } from 'lucide-react';

interface AgentPromptEditorProps {
  promptId?: string;
  onSuccess?: () => void;
}

export function AgentPromptEditor({ promptId, onSuccess }: AgentPromptEditorProps) {
  const router = useRouter();
  const utils = api.useUtils();

  // Form state
  const [name, setName] = useState('');
  const [displayName, setDisplayName] = useState('');
  const [description, setDescription] = useState('');
  const [version, setVersion] = useState('1.0.0');
  const [promptContent, setPromptContent] = useState('');
  const [visibility, setVisibility] = useState<string>('public');
  const [capabilities, setCapabilities] = useState('');
  const [tags, setTags] = useState('');
  const [error, setError] = useState('');

  const createMutation = api.agentPrompts.create.useMutation({
    onSuccess: () => {
      utils.agentPrompts.list.invalidate();
      if (onSuccess) {
        onSuccess();
      } else {
        router.push('/agents');
      }
    },
    onError: (err) => {
      setError(err.message);
    },
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setError('');

    if (!name || !displayName || !promptContent) {
      setError('Name, display name, and prompt content are required');
      return;
    }

    createMutation.mutate({
      name: name.trim(),
      displayName: displayName.trim(),
      description: description.trim() || undefined,
      version,
      promptContent,
      visibility: visibility as any,
      capabilities: capabilities
        ? capabilities.split(',').map(c => c.trim()).filter(Boolean)
        : undefined,
      tags: tags
        ? tags.split(',').map(t => t.trim()).filter(Boolean)
        : undefined,
    });
  };

  return (
    <YStack gap="$4" maxWidth={800}>
        {error && (
          <YStack padding="$3" backgroundColor="$red3" borderRadius="$4">
            <Text color="$red11">{error}</Text>
          </YStack>
        )}

        <XStack gap="$4" flexWrap="wrap">
          {/* Name */}
          <YStack gap="$2" flex={1} minWidth={250}>
            <Label htmlFor="name" fontSize="$3" fontWeight="600">
              Name *
            </Label>
            <Input
              id="name"
              size="$4"
              placeholder="fix-agent"
              value={name}
              onChangeText={setName}
              disabled={createMutation.isLoading}
            />
            <Text fontSize="$2" color="$gray11">
              Kebab-case identifier
            </Text>
          </YStack>

          {/* Display Name */}
          <YStack gap="$2" flex={1} minWidth={250}>
            <Label htmlFor="displayName" fontSize="$3" fontWeight="600">
              Display Name *
            </Label>
            <Input
              id="displayName"
              size="$4"
              placeholder="Fix Agent"
              value={displayName}
              onChangeText={setDisplayName}
              disabled={createMutation.isLoading}
            />
          </YStack>
        </XStack>

        {/* Description */}
        <YStack gap="$2">
          <Label htmlFor="description" fontSize="$3" fontWeight="600">
            Description
          </Label>
          <TextArea
            id="description"
            size="$4"
            placeholder="What does this agent do?"
            value={description}
            onChangeText={setDescription}
            disabled={createMutation.isLoading}
            numberOfLines={2}
          />
        </YStack>

        <XStack gap="$4" flexWrap="wrap">
          {/* Version */}
          <YStack gap="$2" flex={1} minWidth={150}>
            <Label htmlFor="version" fontSize="$3" fontWeight="600">
              Version
            </Label>
            <Input
              id="version"
              size="$4"
              placeholder="1.0.0"
              value={version}
              onChangeText={setVersion}
              disabled={createMutation.isLoading}
            />
          </YStack>

          {/* Visibility */}
          <YStack gap="$2" flex={1} minWidth={150}>
            <Label htmlFor="visibility" fontSize="$3" fontWeight="600">
              Visibility
            </Label>
            <Select
              value={visibility}
              onValueChange={setVisibility}
            >
              <Select.Trigger width="100%" iconAfter={ChevronDown}>
                <Select.Value />
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
                <Select.Viewport>
                  <Select.Item index={0} value="public">
                    <Select.ItemText>Public</Select.ItemText>
                    <Select.ItemIndicator marginLeft="auto">
                      <Check size={16} />
                    </Select.ItemIndicator>
                  </Select.Item>
                  <Select.Item index={1} value="private">
                    <Select.ItemText>Private</Select.ItemText>
                    <Select.ItemIndicator marginLeft="auto">
                      <Check size={16} />
                    </Select.ItemIndicator>
                  </Select.Item>
                  <Select.Item index={2} value="organization">
                    <Select.ItemText>Organization</Select.ItemText>
                    <Select.ItemIndicator marginLeft="auto">
                      <Check size={16} />
                    </Select.ItemIndicator>
                  </Select.Item>
                </Select.Viewport>
              </Select.Content>
            </Select>
          </YStack>
        </XStack>

        {/* Prompt Content */}
        <YStack gap="$2">
          <Label htmlFor="promptContent" fontSize="$3" fontWeight="600">
            Prompt Content *
          </Label>
          <TextArea
            id="promptContent"
            size="$4"
            placeholder="You are an AI agent that..."
            value={promptContent}
            onChangeText={setPromptContent}
            disabled={createMutation.isLoading}
            numberOfLines={15}
            fontFamily="monospace"
          />
          <Text fontSize="$2" color="$gray11">
            The complete agent prompt in markdown format
          </Text>
        </YStack>

        {/* Capabilities */}
        <YStack gap="$2">
          <Label htmlFor="capabilities" fontSize="$3" fontWeight="600">
            Capabilities
          </Label>
          <Input
            id="capabilities"
            size="$4"
            placeholder="fix-failing-tests, analyze-errors, suggest-improvements"
            value={capabilities}
            onChangeText={setCapabilities}
            disabled={createMutation.isLoading}
          />
          <Text fontSize="$2" color="$gray11">
            Comma-separated list of what this agent can do
          </Text>
        </YStack>

        {/* Tags */}
        <YStack gap="$2">
          <Label htmlFor="tags" fontSize="$3" fontWeight="600">
            Tags
          </Label>
          <Input
            id="tags"
            size="$4"
            placeholder="fix, testing, quality"
            value={tags}
            onChangeText={setTags}
            disabled={createMutation.isLoading}
          />
          <Text fontSize="$2" color="$gray11">
            Comma-separated list for categorization
          </Text>
        </YStack>

        {/* Actions */}
        <XStack gap="$3" marginTop="$4">
          <Button
            size="$4"
            theme="blue"
            onPress={handleSubmit}
            disabled={createMutation.isLoading || !promptContent}
            flex={1}
          >
            {createMutation.isLoading ? 'Creating...' : 'Create Agent Prompt'}
          </Button>
          <Button
            size="$4"
            variant="outlined"
            onPress={() => router.back()}
            disabled={createMutation.isLoading}
          >
            Cancel
          </Button>
        </XStack>
      </YStack>
  );
}

