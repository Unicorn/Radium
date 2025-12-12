/**
 * Agent Tester Modal - Test agent prompts in a conversational interface
 */

'use client';

import { YStack, XStack, Text, Button, TextArea, Card, ScrollView, Spinner, Dialog, Sheet } from 'tamagui';
import { useState, useEffect, useRef } from 'react';
import { api } from '@/lib/trpc/client';
import { Send, X, Square } from 'lucide-react';
import type { Message } from '@/types/agent-builder';

interface AgentTesterModalProps {
  agentPromptId: string;
  agentName: string;
  open: boolean;
  onClose: () => void;
}

export function AgentTesterModal({
  agentPromptId,
  agentName,
  open,
  onClose,
}: AgentTesterModalProps) {
  const scrollViewRef = useRef<any>(null);
  const [workflowId, setWorkflowId] = useState<string | null>(null);
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [isActive, setIsActive] = useState(false);
  const [isStarting, setIsStarting] = useState(false);

  // Check for active test on mount
  const { data: activeTest } = api.agentTester.getActiveTest.useQuery(
    { agentPromptId },
    { enabled: open && !workflowId }
  );

  // Start test mutation
  const startTestMutation = api.agentTester.startTest.useMutation({
    onSuccess: (data) => {
      setWorkflowId(data.workflowId);
      setIsActive(true);
      setIsStarting(false);
    },
    onError: (error) => {
      console.error('Failed to start test:', error);
      setIsStarting(false);
      alert(`Failed to start test: ${error.message}`);
    },
  });

  // Send message mutation
  const sendMessageMutation = api.agentTester.sendMessage.useMutation({
    onSuccess: () => {
      setInput('');
      setIsLoading(true);
      // Polling will pick up the new message
    },
    onError: (error) => {
      console.error('Failed to send message:', error);
      setIsLoading(false);
      alert(`Failed to send message: ${error.message}`);
    },
  });

  // End test mutation
  const endTestMutation = api.agentTester.endTest.useMutation({
    onSuccess: () => {
      setIsActive(false);
      setWorkflowId(null);
      setMessages([]);
      onClose();
    },
    onError: (error) => {
      console.error('Failed to end test:', error);
      alert(`Failed to end test: ${error.message}`);
    },
  });

  // Poll for conversation updates
  const { data: conversationState, refetch: refetchConversation } = api.agentTester.getConversation.useQuery(
    { workflowId: workflowId! },
    {
      enabled: open && !!workflowId,
      refetchInterval: isActive ? 2000 : false, // Poll every 2 seconds when active
    }
  );

  // Update messages when conversation state changes
  useEffect(() => {
    if (conversationState) {
      setMessages(conversationState.messages);
      setIsActive(conversationState.isActive);
      setIsLoading(false);
      
      // If no longer active, stop polling
      if (!conversationState.isActive && workflowId) {
        setWorkflowId(null);
      }
    }
  }, [conversationState, workflowId]);

  // Scroll to bottom when messages change
  useEffect(() => {
    if (scrollViewRef.current) {
      setTimeout(() => {
        scrollViewRef.current?.scrollToEnd?.({ animated: true });
      }, 100);
    }
  }, [messages]);

  // Initialize workflow if active test exists
  useEffect(() => {
    if (open && activeTest && !workflowId) {
      setWorkflowId(activeTest.temporal_workflow_id);
      setIsActive(true);
    }
  }, [open, activeTest, workflowId]);

  // Start new test when modal opens
  const handleStartTest = () => {
    setIsStarting(true);
    startTestMutation.mutate({
      agentPromptId,
      initialMessage: input.trim() || undefined,
    });
  };

  const handleSend = () => {
    if (!input.trim() || !workflowId || isLoading) return;
    
    sendMessageMutation.mutate({
      workflowId,
      message: input.trim(),
    });
  };

  const handleEndTest = () => {
    if (workflowId) {
      endTestMutation.mutate({ workflowId });
    } else {
      onClose();
    }
  };

  const handleClose = () => {
    if (isActive && workflowId) {
      if (confirm('End the test session? This will close the conversation.')) {
        handleEndTest();
      }
    } else {
      onClose();
    }
  };

  return (
    <Dialog modal open={open} onOpenChange={(open) => !open && handleClose()}>
      <Dialog.Portal>
        <Dialog.Overlay
          key="overlay"
          animation="quick"
          opacity={0.5}
          enterStyle={{ opacity: 0 }}
          exitStyle={{ opacity: 0 }}
        />
        <Dialog.Content
          bordered
          elevate
          key="content"
          animateOnly={['transform', 'opacity']}
          animation={[
            'quick',
            {
              opacity: {
                overshootClamping: true,
              },
            },
          ]}
          enterStyle={{ x: 0, y: -20, opacity: 0, scale: 0.9 }}
          exitStyle={{ x: 0, y: 10, opacity: 0, scale: 0.95 }}
          gap="$4"
          maxWidth={800}
          width="90%"
          maxHeight="90vh"
        >
          <Dialog.Title>Test Agent: {agentName}</Dialog.Title>
          
          {!workflowId && !isStarting && (
            <YStack gap="$4" padding="$4">
              <Text>Start a conversation with this agent to test its behavior.</Text>
              <TextArea
                placeholder="Type your first message..."
                value={input}
                onChangeText={setInput}
                numberOfLines={3}
              />
              <XStack gap="$3" justifyContent="flex-end">
                <Button variant="outlined" onPress={onClose}>
                  Cancel
                </Button>
                <Button theme="blue" onPress={handleStartTest}>
                  Start Test
                </Button>
              </XStack>
            </YStack>
          )}

          {isStarting && (
            <YStack padding="$4" alignItems="center" gap="$3">
              <Spinner size="large" />
              <Text>Starting test session...</Text>
            </YStack>
          )}

          {workflowId && (
            <>
              {/* Chat Messages */}
              <Card flex={1} padding="$4" minHeight={400} maxHeight={500}>
                <ScrollView ref={scrollViewRef} flex={1}>
                  <YStack gap="$3">
                    {messages.length === 0 && (
                      <Text color="$gray11" textAlign="center" padding="$4">
                        No messages yet. Start the conversation!
                      </Text>
                    )}
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
                              Agent is thinking...
                            </Text>
                          </XStack>
                        </Card>
                      </XStack>
                    )}
                  </YStack>
                </ScrollView>
              </Card>

              {/* Input Area */}
              {isActive && (
                <XStack gap="$2" alignItems="flex-end">
                  <TextArea
                    flex={1}
                    placeholder="Type your message..."
                    value={input}
                    onChangeText={setInput}
                    disabled={isLoading}
                    numberOfLines={2}
                    onKeyPress={(e) => {
                      // Note: TextInputKeyPressEvent doesn't have key/shiftKey properties
                      // This handler may not work as expected - consider using onSubmitEditing instead
                    }}
                  />
                  <Button
                    size="$4"
                    theme="blue"
                    onPress={handleSend}
                    disabled={!input.trim() || isLoading}
                    icon={Send}
                  >
                    Send
                  </Button>
                </XStack>
              )}

              {!isActive && (
                <Card padding="$3" backgroundColor="$gray2">
                  <Text textAlign="center" color="$gray11">
                    Test session ended
                  </Text>
                </Card>
              )}

              {/* Actions */}
              <XStack gap="$3" justifyContent="flex-end">
                <Button
                  variant="outlined"
                  onPress={handleEndTest}
                  disabled={!isActive}
                  icon={Square}
                >
                  End Test
                </Button>
                <Button variant="outlined" onPress={onClose}>
                  Close
                </Button>
              </XStack>
            </>
          )}

          <Dialog.Close asChild>
            <Button
              position="absolute"
              top="$3"
              right="$3"
              size="$2"
              circular
              icon={X}
              onPress={handleClose}
            />
          </Dialog.Close>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog>
  );
}

