/**
 * Agent Prompts Page
 */

'use client';

import { YStack, XStack, H1, Button, Spinner, Text } from 'tamagui';
import { useRouter } from 'next/navigation';
import { AuthGuardWithLoading } from '@/components/shared/AuthGuard';
import { Header } from '@/components/shared/Header';
import { Sidebar } from '@/components/shared/Sidebar';
import { AgentPromptCard } from '@/components/agent/AgentPromptCard';
import { api } from '@/lib/trpc/client';

function AgentsContent() {
  const router = useRouter();
  
  console.log('🔍 [AgentsContent] Initiating agentPrompts.list query');
  
  const { data, isLoading, error } = api.agentPrompts.list.useQuery({});
  
  console.log('📊 [AgentsContent] Query state:', {
    isLoading,
    hasError: !!error,
    errorMessage: error?.message,
    hasData: !!data,
    promptCount: data?.prompts?.length,
  });

  return (
    <YStack flex={1}>
      <Header />
      <XStack flex={1}>
        <Sidebar />
        <YStack flex={1} padding="$6" gap="$4">
          <XStack justifyContent="space-between" alignItems="center">
            <H1>Agent Prompts</H1>
            <XStack gap="$3">
              <Button
                variant="outlined"
                onPress={() => router.push('/agents/new')}
              >
                Create Manually
              </Button>
              <Button
                theme="blue"
                onPress={() => router.push('/agents/new/assisted')}
              >
                Build with AI
              </Button>
            </XStack>
          </XStack>

          {isLoading ? (
            <YStack flex={1} alignItems="center" justifyContent="center">
              <Spinner size="large" />
              <Text marginTop="$4">Loading agent prompts...</Text>
            </YStack>
          ) : (
            <YStack gap="$3">
              {data?.prompts.length === 0 ? (
                <YStack padding="$6" alignItems="center">
                  <Text color="$gray11">No agent prompts yet</Text>
                  <Button
                    marginTop="$4"
                    onPress={() => router.push('/agents/new')}
                  >
                    Create Your First Agent
                  </Button>
                </YStack>
              ) : (
                data?.prompts.map((prompt) => (
                  <AgentPromptCard
                    key={prompt.id}
                    prompt={prompt as any}
                    onClick={() => router.push(`/agents/${prompt.id}`)}
                  />
                ))
              )}
            </YStack>
          )}
        </YStack>
      </XStack>
    </YStack>
  );
}

export default function AgentsPage() {
  return (
    <AuthGuardWithLoading>
      <AgentsContent />
    </AuthGuardWithLoading>
  );
}

