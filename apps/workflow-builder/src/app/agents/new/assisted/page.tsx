/**
 * AI-Assisted Agent Builder Page
 */

'use client';

import { YStack, XStack, H1, Button } from 'tamagui';
import { AuthGuardWithLoading } from '@/components/shared/AuthGuard';
import { Header } from '@/components/shared/Header';
import { Sidebar } from '@/components/shared/Sidebar';
import { AgentBuilderChat } from '@/components/agent/AgentBuilderChat';
import { useRouter } from 'next/navigation';
import { ArrowLeft } from 'lucide-react';

function AssistedAgentContent() {
  const router = useRouter();

  return (
    <YStack flex={1}>
      <Header />
      <XStack flex={1}>
        <Sidebar />
        <YStack flex={1} padding="$6" gap="$4">
          <XStack alignItems="center" gap="$3">
            <Button
              size="$3"
              variant="outlined"
              onPress={() => router.push('/agents/new')}
              icon={ArrowLeft}
            >
              Back to Manual Editor
            </Button>
            <H1>Build Agent with AI Assistant</H1>
          </XStack>
          
          <AgentBuilderChat
            onSuccess={(promptId) => {
              router.push(`/agents/${promptId}`);
            }}
          />
        </YStack>
      </XStack>
    </YStack>
  );
}

export default function AssistedAgentPage() {
  return (
    <AuthGuardWithLoading>
      <AssistedAgentContent />
    </AuthGuardWithLoading>
  );
}

