/**
 * Agent Builder Chat - AI-assisted agent prompt creation interface
 */

'use client';

import { YStack, XStack, Text, Button, TextArea, Input, Label, Card, ScrollView, Spinner } from 'tamagui';
import { useState, useEffect, useRef } from 'react';
import { useRouter } from 'next/navigation';
import { api } from '@/lib/trpc/client';
import { Send, X, Check, RefreshCw } from 'lucide-react';
import type { Message } from '@/types/agent-builder';

interface AgentBuilderChatProps {
  onSuccess?: (promptId: string) => void;
}

export function AgentBuilderChat({ onSuccess }: AgentBuilderChatProps) {
  const router = useRouter();
  const utils = api.useUtils();
  const scrollViewRef = useRef<any>(null);
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [generatedPrompt, setGeneratedPrompt] = useState<string | null>(null);
  const [showSaveForm, setShowSaveForm] = useState(false);
  const [saveFormData, setSaveFormData] = useState({
    name: '',
    displayName: '',
    description: '',
    version: '1.0.0',
    visibility: 'private' as 'public' | 'private' | 'organization',
  });

  // Start session on mount
  const startSessionMutation = api.agentBuilder.startSession.useMutation({
    onSuccess: (data) => {
      setSessionId(data.sessionId);
      setMessages(data.messages);
    },
    onError: (error) => {
      console.error('Failed to start session:', error);
    },
  });

  useEffect(() => {
    if (!sessionId) {
      startSessionMutation.mutate();
    }
  }, []);

  // Scroll to bottom when messages change
  useEffect(() => {
    if (scrollViewRef.current) {
      setTimeout(() => {
        scrollViewRef.current?.scrollToEnd?.({ animated: true });
      }, 100);
    }
  }, [messages]);

  const sendMessageMutation = api.agentBuilder.sendMessage.useMutation({
    onSuccess: (data) => {
      setMessages(data.messages);
      setInput('');
      setIsLoading(false);
      
      if (data.generatedPrompt) {
        setGeneratedPrompt(data.generatedPrompt);
        setShowSaveForm(true);
      }
    },
    onError: (error) => {
      console.error('Failed to send message:', error);
      setIsLoading(false);
    },
  });

  const regenerateMutation = api.agentBuilder.regeneratePrompt.useMutation({
    onSuccess: (data) => {
      setMessages(data.messages);
      setGeneratedPrompt(data.prompt);
      setShowSaveForm(true);
    },
    onError: (error) => {
      console.error('Failed to regenerate prompt:', error);
    },
  });

  const savePromptMutation = api.agentBuilder.savePrompt.useMutation({
    onSuccess: (data) => {
      utils.agentPrompts.list.invalidate();
      if (onSuccess) {
        onSuccess(data.id);
      } else {
        router.push(`/agents/${data.id}`);
      }
    },
    onError: (error) => {
      console.error('Failed to save prompt:', error);
    },
  });

  const cancelSessionMutation = api.agentBuilder.cancelSession.useMutation({
    onSuccess: () => {
      router.push('/agents');
    },
  });

  const handleSend = () => {
    if (!input.trim() || !sessionId || isLoading) return;
    
    setIsLoading(true);
    sendMessageMutation.mutate({
      sessionId,
      message: input.trim(),
    });
  };

  const handleRegenerate = () => {
    if (!sessionId) return;
    regenerateMutation.mutate({ sessionId });
  };

  const handleSave = () => {
    if (!sessionId || !generatedPrompt) return;
    
    if (!saveFormData.name || !saveFormData.displayName) {
      alert('Name and Display Name are required');
      return;
    }
    
    savePromptMutation.mutate({
      sessionId,
      ...saveFormData,
    });
  };

  const handleCancel = () => {
    if (sessionId) {
      cancelSessionMutation.mutate({ sessionId });
    } else {
      router.push('/agents');
    }
  };

  return (
    <YStack flex={1} gap="$4" maxWidth={900} width="100%">
      {/* Chat Messages */}
      <Card flex={1} padding="$4" minHeight={400} maxHeight={600}>
        <ScrollView ref={scrollViewRef} flex={1}>
          <YStack gap="$3">
            {messages.map((message, index) => (
              <XStack
                key={index}
                justifyContent={message.role === 'user' ? 'flex-end' : 'flex-start'}
                paddingHorizontal="$2"
              >
                <Card
                  backgroundColor={message.role === 'user' ? '$blue5' : '$gray3'}
                  padding="$3"
                  maxWidth="80%"
                >
                  <Text fontSize="$3" lineHeight="$4">
                    {message.content}
                  </Text>
                </Card>
              </XStack>
            ))}
            {isLoading && (
              <XStack justifyContent="flex-start" paddingHorizontal="$2">
                <Card backgroundColor="$gray3" padding="$3">
                  <XStack gap="$2" alignItems="center">
                    <Spinner size="small" />
                    <Text fontSize="$3" color="$gray11">
                      Thinking...
                    </Text>
                  </XStack>
                </Card>
              </XStack>
            )}
          </YStack>
        </ScrollView>
      </Card>

      {/* Generated Prompt Preview */}
      {generatedPrompt && (
        <Card padding="$4" backgroundColor="$green2" borderColor="$green8">
          <YStack gap="$3">
            <Text fontSize="$5" fontWeight="600">
              Generated Agent Prompt
            </Text>
            <Card padding="$3" backgroundColor="$background" maxHeight={300}>
              <ScrollView>
                <Text fontFamily="monospace" fontSize="$2" whiteSpace="pre-wrap">
                  {generatedPrompt}
                </Text>
              </ScrollView>
            </Card>
            {!showSaveForm && (
              <XStack gap="$2">
                <Button
                  size="$3"
                  theme="green"
                  onPress={() => setShowSaveForm(true)}
                  icon={Check}
                >
                  Save Prompt
                </Button>
                <Button
                  size="$3"
                  variant="outlined"
                  onPress={handleRegenerate}
                  icon={RefreshCw}
                  disabled={regenerateMutation.isLoading}
                >
                  Request Changes
                </Button>
              </XStack>
            )}
          </YStack>
        </Card>
      )}

      {/* Save Form */}
      {showSaveForm && (
        <Card padding="$4" borderColor="$blue8">
          <YStack gap="$3">
            <Text fontSize="$5" fontWeight="600">
              Save Agent Prompt
            </Text>
            
            <XStack gap="$3" flexWrap="wrap">
              <YStack gap="$2" flex={1} minWidth={200}>
                <Label htmlFor="name">Name *</Label>
                <Input
                  id="name"
                  placeholder="my-agent"
                  value={saveFormData.name}
                  onChangeText={(text) => setSaveFormData({ ...saveFormData, name: text })}
                />
              </YStack>
              
              <YStack gap="$2" flex={1} minWidth={200}>
                <Label htmlFor="displayName">Display Name *</Label>
                <Input
                  id="displayName"
                  placeholder="My Agent"
                  value={saveFormData.displayName}
                  onChangeText={(text) => setSaveFormData({ ...saveFormData, displayName: text })}
                />
              </YStack>
            </XStack>
            
            <YStack gap="$2">
              <Label htmlFor="description">Description</Label>
              <TextArea
                id="description"
                placeholder="What does this agent do?"
                value={saveFormData.description}
                onChangeText={(text) => setSaveFormData({ ...saveFormData, description: text })}
                numberOfLines={2}
              />
            </YStack>
            
            <XStack gap="$3">
              <Button
                size="$4"
                theme="blue"
                onPress={handleSave}
                disabled={savePromptMutation.isLoading || !saveFormData.name || !saveFormData.displayName}
                flex={1}
              >
                {savePromptMutation.isLoading ? 'Saving...' : 'Save Prompt'}
              </Button>
              <Button
                size="$4"
                variant="outlined"
                onPress={() => setShowSaveForm(false)}
                disabled={savePromptMutation.isLoading}
              >
                Cancel
              </Button>
            </XStack>
          </YStack>
        </Card>
      )}

      {/* Input Area */}
      {!showSaveForm && (
        <XStack gap="$2" alignItems="flex-end">
          <TextArea
            flex={1}
            placeholder="Type your message..."
            value={input}
            onChangeText={setInput}
            disabled={isLoading || !sessionId}
            numberOfLines={3}
            onKeyPress={(e) => {
              // Note: TextInputKeyPressEvent doesn't have key/shiftKey properties
              // This handler may not work as expected - consider using onSubmitEditing instead
            }}
          />
          <Button
            size="$4"
            theme="blue"
            onPress={handleSend}
            disabled={!input.trim() || isLoading || !sessionId}
            icon={Send}
          >
            Send
          </Button>
        </XStack>
      )}

      {/* Cancel Button */}
      <XStack justifyContent="flex-end">
        <Button
          size="$3"
          variant="outlined"
          onPress={handleCancel}
          disabled={savePromptMutation.isLoading}
          icon={X}
        >
          Cancel
        </Button>
      </XStack>
    </YStack>
  );
}

