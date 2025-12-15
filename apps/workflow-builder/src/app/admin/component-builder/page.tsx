/**
 * Component Builder Page
 *
 * AI-powered interface for creating new workflow components
 * through conversational interaction.
 */

'use client';

import { useState, useRef, useEffect, useCallback } from 'react';
import {
  YStack,
  XStack,
  H1,
  H2,
  H3,
  Button,
  Spinner,
  Text,
  Input,
  ScrollView,
  Card,
  Tabs,
  TextArea,
} from 'tamagui';
import { AuthGuardWithLoading } from '@/components/shared/AuthGuard';
import { Header } from '@/components/shared/Header';
import { Sidebar } from '@/components/shared/Sidebar';

/** Message in the conversation */
interface Message {
  role: 'user' | 'assistant';
  content: string;
  timestamp: Date;
}

/** Generated artifacts */
interface Artifacts {
  rustSchema?: string;
  typescript?: string;
  tests?: string;
}

/** API response type */
interface ChatResponse {
  response: string;
  conversationId: string;
  phase: string;
  phaseChanged: boolean;
  suggestedActions?: string[];
  artifacts?: Artifacts;
  error?: string;
}

/** Phase display names */
const PHASE_LABELS: Record<string, string> = {
  gathering: 'Gathering Requirements',
  designing: 'Designing Schema',
  refining: 'Refining Design',
  generating: 'Generating Code',
  reviewing: 'Reviewing Artifacts',
  complete: 'Complete',
  error: 'Error',
};

/** Phase colors */
const PHASE_COLORS: Record<string, string> = {
  gathering: '$blue10',
  designing: '$purple10',
  refining: '$orange10',
  generating: '$green10',
  reviewing: '$cyan10',
  complete: '$green10',
  error: '$red10',
};

function ComponentBuilderContent() {
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState('');
  const [loading, setLoading] = useState(false);
  const [conversationId, setConversationId] = useState<string | null>(null);
  const [phase, setPhase] = useState<string>('gathering');
  const [suggestedActions, setSuggestedActions] = useState<string[]>([]);
  const [artifacts, setArtifacts] = useState<Artifacts | null>(null);
  const [activeTab, setActiveTab] = useState<string>('rust');
  const scrollRef = useRef<ScrollView>(null);

  // Auto-scroll to bottom when new messages arrive
  useEffect(() => {
    if (scrollRef.current) {
      // Small delay to ensure content is rendered
      setTimeout(() => {
        scrollRef.current?.scrollToEnd?.({ animated: true });
      }, 100);
    }
  }, [messages]);

  // Send message to API
  const sendMessage = useCallback(async (messageText?: string) => {
    const text = messageText || input.trim();
    if (!text) return;

    // Add user message immediately
    const userMessage: Message = {
      role: 'user',
      content: text,
      timestamp: new Date(),
    };
    setMessages((prev) => [...prev, userMessage]);
    setInput('');
    setLoading(true);

    try {
      const response = await fetch('/api/component-builder/chat', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          message: text,
          conversationId,
        }),
      });

      const data: ChatResponse = await response.json();

      if (data.error) {
        throw new Error(data.error);
      }

      // Update state with response
      setConversationId(data.conversationId);
      setPhase(data.phase);
      setSuggestedActions(data.suggestedActions || []);

      if (data.artifacts) {
        setArtifacts(data.artifacts);
      }

      // Add assistant message
      const assistantMessage: Message = {
        role: 'assistant',
        content: data.response,
        timestamp: new Date(),
      };
      setMessages((prev) => [...prev, assistantMessage]);
    } catch (error) {
      console.error('Chat error:', error);
      const errorMessage: Message = {
        role: 'assistant',
        content: `Error: ${error instanceof Error ? error.message : 'Unknown error occurred'}`,
        timestamp: new Date(),
      };
      setMessages((prev) => [...prev, errorMessage]);
    } finally {
      setLoading(false);
    }
  }, [input, conversationId]);

  // Handle suggested action click
  const handleSuggestedAction = (action: string) => {
    sendMessage(action);
  };

  // Reset conversation
  const resetConversation = async () => {
    if (conversationId) {
      try {
        await fetch(`/api/component-builder/chat?conversationId=${conversationId}`, {
          method: 'DELETE',
        });
      } catch (error) {
        console.error('Error deleting session:', error);
      }
    }

    setMessages([]);
    setConversationId(null);
    setPhase('gathering');
    setSuggestedActions([]);
    setArtifacts(null);
    setInput('');
  };

  // Handle key press
  const handleKeyPress = (e: { key: string; shiftKey?: boolean }) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      sendMessage();
    }
  };

  return (
    <YStack flex={1}>
      <Header />
      <XStack flex={1}>
        <Sidebar />
        <XStack flex={1} padding="$4" gap="$4">
          {/* Conversation Panel */}
          <YStack flex={1} gap="$3">
            <XStack justifyContent="space-between" alignItems="center">
              <YStack>
                <H1>Component Builder</H1>
                <XStack gap="$2" alignItems="center">
                  <Text color="$gray11">Phase:</Text>
                  <Text
                    color={PHASE_COLORS[phase] || '$gray11'}
                    fontWeight="bold"
                  >
                    {PHASE_LABELS[phase] || phase}
                  </Text>
                </XStack>
              </YStack>
              <Button
                size="$3"
                variant="outlined"
                onPress={resetConversation}
                disabled={messages.length === 0}
              >
                New Component
              </Button>
            </XStack>

            {/* Messages */}
            <Card flex={1} padding="$3" bordered>
              <ScrollView ref={scrollRef} flex={1}>
                <YStack gap="$3" paddingBottom="$4">
                  {messages.length === 0 ? (
                    <YStack padding="$6" alignItems="center">
                      <Text color="$gray11" textAlign="center">
                        Describe the component you want to create.
                      </Text>
                      <Text color="$gray10" fontSize="$2" marginTop="$2" textAlign="center">
                        Example: &quot;I need a component that sends emails via SMTP&quot;
                      </Text>
                    </YStack>
                  ) : (
                    messages.map((msg, i) => (
                      <Card
                        key={i}
                        padding="$3"
                        backgroundColor={msg.role === 'user' ? '$blue2' : '$gray2'}
                        alignSelf={msg.role === 'user' ? 'flex-end' : 'flex-start'}
                        maxWidth="85%"
                        bordered
                      >
                        <Text
                          fontSize="$2"
                          color="$gray11"
                          marginBottom="$1"
                        >
                          {msg.role === 'user' ? 'You' : 'Builder'}
                        </Text>
                        <Text whiteSpace="pre-wrap">{msg.content}</Text>
                      </Card>
                    ))
                  )}
                  {loading && (
                    <Card
                      padding="$3"
                      backgroundColor="$gray2"
                      alignSelf="flex-start"
                      bordered
                    >
                      <XStack gap="$2" alignItems="center">
                        <Spinner size="small" />
                        <Text color="$gray11">Thinking...</Text>
                      </XStack>
                    </Card>
                  )}
                </YStack>
              </ScrollView>
            </Card>

            {/* Suggested Actions */}
            {suggestedActions.length > 0 && (
              <XStack gap="$2" flexWrap="wrap">
                {suggestedActions.map((action, i) => (
                  <Button
                    key={i}
                    size="$2"
                    variant="outlined"
                    onPress={() => handleSuggestedAction(action)}
                    disabled={loading}
                  >
                    {action}
                  </Button>
                ))}
              </XStack>
            )}

            {/* Input */}
            <XStack gap="$2">
              <Input
                flex={1}
                value={input}
                onChangeText={setInput}
                placeholder="Describe your component or respond to questions..."
                onKeyPress={handleKeyPress as (e: unknown) => void}
                disabled={loading || phase === 'complete'}
              />
              <Button
                onPress={() => sendMessage()}
                disabled={loading || !input.trim() || phase === 'complete'}
                theme="blue"
              >
                Send
              </Button>
            </XStack>
          </YStack>

          {/* Preview Panel */}
          <YStack width={500} gap="$3">
            <H2>Generated Code</H2>

            {artifacts ? (
              <Card flex={1} bordered>
                <Tabs
                  value={activeTab}
                  onValueChange={setActiveTab}
                  orientation="horizontal"
                  flexDirection="column"
                  flex={1}
                >
                  <Tabs.List>
                    <Tabs.Tab value="rust">
                      <Text>Rust</Text>
                    </Tabs.Tab>
                    <Tabs.Tab value="typescript">
                      <Text>TypeScript</Text>
                    </Tabs.Tab>
                    <Tabs.Tab value="tests">
                      <Text>Tests</Text>
                    </Tabs.Tab>
                  </Tabs.List>

                  <Tabs.Content value="rust" flex={1}>
                    <ScrollView flex={1} padding="$3">
                      <Text
                        fontFamily="$mono"
                        fontSize="$2"
                        whiteSpace="pre-wrap"
                        color="$gray12"
                      >
                        {artifacts.rustSchema || 'No Rust code generated yet'}
                      </Text>
                    </ScrollView>
                  </Tabs.Content>

                  <Tabs.Content value="typescript" flex={1}>
                    <ScrollView flex={1} padding="$3">
                      <Text
                        fontFamily="$mono"
                        fontSize="$2"
                        whiteSpace="pre-wrap"
                        color="$gray12"
                      >
                        {artifacts.typescript || 'No TypeScript code generated yet'}
                      </Text>
                    </ScrollView>
                  </Tabs.Content>

                  <Tabs.Content value="tests" flex={1}>
                    <ScrollView flex={1} padding="$3">
                      <Text
                        fontFamily="$mono"
                        fontSize="$2"
                        whiteSpace="pre-wrap"
                        color="$gray12"
                      >
                        {artifacts.tests || 'No tests generated yet'}
                      </Text>
                    </ScrollView>
                  </Tabs.Content>
                </Tabs>
              </Card>
            ) : (
              <Card flex={1} padding="$6" bordered alignItems="center" justifyContent="center">
                <YStack alignItems="center" gap="$3">
                  <Text color="$gray10" textAlign="center">
                    Code will appear here once the design is approved
                  </Text>
                  <Text color="$gray9" fontSize="$2" textAlign="center">
                    Complete the conversation to generate code
                  </Text>
                </YStack>
              </Card>
            )}

            {/* Phase Progress */}
            <Card padding="$3" bordered>
              <H3 marginBottom="$2">Progress</H3>
              <YStack gap="$2">
                {Object.entries(PHASE_LABELS).map(([key, label]) => {
                  if (key === 'error') return null;
                  const isActive = key === phase;
                  const isPast = getPhaseOrder(key) < getPhaseOrder(phase);
                  return (
                    <XStack key={key} gap="$2" alignItems="center">
                      <YStack
                        width={20}
                        height={20}
                        borderRadius={10}
                        backgroundColor={
                          isPast ? '$green9' : isActive ? '$blue9' : '$gray5'
                        }
                        alignItems="center"
                        justifyContent="center"
                      >
                        {isPast && (
                          <Text color="white" fontSize={12}>
                            ✓
                          </Text>
                        )}
                      </YStack>
                      <Text
                        color={isActive ? '$blue11' : isPast ? '$gray11' : '$gray9'}
                        fontWeight={isActive ? 'bold' : 'normal'}
                      >
                        {label}
                      </Text>
                    </XStack>
                  );
                })}
              </YStack>
            </Card>
          </YStack>
        </XStack>
      </XStack>
    </YStack>
  );
}

/** Get numeric order of phase for comparison */
function getPhaseOrder(phase: string): number {
  const order = ['gathering', 'designing', 'refining', 'generating', 'reviewing', 'complete'];
  return order.indexOf(phase);
}

export default function ComponentBuilderPage() {
  return (
    <AuthGuardWithLoading>
      <ComponentBuilderContent />
    </AuthGuardWithLoading>
  );
}
