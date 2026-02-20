/**
 * Create New Agent Prompt Page
 */

'use client';

import { YStack, XStack, H1, Button } from 'tamagui';
import { useRouter } from 'next/navigation';
import { AuthGuardWithLoading } from '@/components/shared/AuthGuard';
import { Header } from '@/components/shared/Header';
import { Sidebar } from '@/components/shared/Sidebar';
import { AgentPromptEditor } from '@/components/agent/AgentPromptEditor';

function NewAgentContent() {
  const router = useRouter();

  return (
    <YStack flex={1}>
      <Header />
      <XStack flex={1}>
        <Sidebar />
        <YStack flex={1} padding="$6" gap="$4">
          <XStack justifyContent="space-between" alignItems="center">
            <H1>Create Agent Prompt</H1>
            <Button
              theme="blue"
              onPress={() => router.push('/agents/new/assisted')}
            >
              Build with AI Assistant
            </Button>
          </XStack>
          <AgentPromptEditor />
        </YStack>
      </XStack>
    </YStack>
  );
}

export default function NewAgentPage() {
  return (
    <AuthGuardWithLoading>
      <NewAgentContent />
    </AuthGuardWithLoading>
  );
}

