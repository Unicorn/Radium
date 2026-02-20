/**
 * Agent Detail Page
 */

'use client';

import { YStack, XStack, H1, Button, Text, Card, Spinner } from 'tamagui';
import { useParams, useRouter } from 'next/navigation';
import { AuthGuardWithLoading } from '@/components/shared/AuthGuard';
import { Header } from '@/components/shared/Header';
import { Sidebar } from '@/components/shared/Sidebar';
import { AgentTesterModal } from '@/components/agent/AgentTesterModal';
import { api } from '@/lib/trpc/client';
import { useState } from 'react';
import { Play, Edit, Trash2 } from 'lucide-react';

function AgentDetailContent() {
  const params = useParams();
  const router = useRouter();
  const agentId = params.id as string;
  const [testModalOpen, setTestModalOpen] = useState(false);

  const { data: prompt, isLoading, error } = api.agentPrompts.get.useQuery(
    { id: agentId },
    { enabled: !!agentId }
  );

  const deleteMutation = api.agentPrompts.delete.useMutation({
    onSuccess: () => {
      router.push('/agents');
    },
    onError: (error) => {
      alert(`Failed to delete: ${error.message}`);
    },
  });

  if (isLoading) {
    return (
      <YStack flex={1} alignItems="center" justifyContent="center">
        <Spinner size="large" />
        <Text marginTop="$4">Loading agent...</Text>
      </YStack>
    );
  }

  if (error || !prompt) {
    return (
      <YStack flex={1} alignItems="center" justifyContent="center">
        <Text color="$red11">Failed to load agent</Text>
        <Button marginTop="$4" onPress={() => router.push('/agents')}>
          Back to Agents
        </Button>
      </YStack>
    );
  }

  return (
    <YStack flex={1}>
      <Header />
      <XStack flex={1}>
        <Sidebar />
        <YStack flex={1} padding="$6" gap="$4">
          <XStack justifyContent="space-between" alignItems="center">
            <H1>{prompt.display_name}</H1>
            <XStack gap="$3">
              <Button
                theme="blue"
                onPress={() => setTestModalOpen(true)}
                icon={Play}
              >
                Test Agent
              </Button>
              <Button
                variant="outlined"
                onPress={() => router.push(`/agents/${agentId}/edit`)}
                icon={Edit}
              >
                Edit
              </Button>
              <Button
                variant="outlined"
                theme="red"
                onPress={() => {
                  if (confirm('Are you sure you want to delete this agent?')) {
                    deleteMutation.mutate({ id: agentId });
                  }
                }}
                icon={Trash2}
                disabled={deleteMutation.isLoading}
              >
                Delete
              </Button>
            </XStack>
          </XStack>

          <Card padding="$4">
            <YStack gap="$4">
              {prompt.description && (
                <YStack gap="$2">
                  <Text fontSize="$4" fontWeight="600">
                    Description
                  </Text>
                  <Text fontSize="$3" color="$gray11">
                    {prompt.description}
                  </Text>
                </YStack>
              )}

              <YStack gap="$2">
                <Text fontSize="$4" fontWeight="600">
                  Prompt Content
                </Text>
                <Card padding="$3" backgroundColor="$gray2" maxHeight={400}>
                  <Text fontFamily="monospace" fontSize="$2" whiteSpace="pre-wrap">
                    {prompt.prompt_content}
                  </Text>
                </Card>
              </YStack>

              <XStack gap="$4" flexWrap="wrap">
                <YStack gap="$2">
                  <Text fontSize="$3" fontWeight="600">
                    Version
                  </Text>
                  <Text fontSize="$2" color="$gray11">
                    {prompt.version}
                  </Text>
                </YStack>

                <YStack gap="$2">
                  <Text fontSize="$3" fontWeight="600">
                    Visibility
                  </Text>
                  <Text fontSize="$2" color="$gray11">
                    {prompt.visibility?.name || 'private'}
                  </Text>
                </YStack>

                {prompt.capabilities && prompt.capabilities.length > 0 && (
                  <YStack gap="$2">
                    <Text fontSize="$3" fontWeight="600">
                      Capabilities
                    </Text>
                    <XStack gap="$2" flexWrap="wrap">
                      {prompt.capabilities.map((cap: string) => (
                        <Text key={cap} fontSize="$2" color="$gray11">
                          {cap}
                        </Text>
                      ))}
                    </XStack>
                  </YStack>
                )}
              </XStack>
            </YStack>
          </Card>
        </YStack>
      </XStack>

      <AgentTesterModal
        agentPromptId={agentId}
        agentName={prompt.display_name}
        open={testModalOpen}
        onClose={() => setTestModalOpen(false)}
      />
    </YStack>
  );
}

export default function AgentDetailPage() {
  return (
    <AuthGuardWithLoading>
      <AgentDetailContent />
    </AuthGuardWithLoading>
  );
}

