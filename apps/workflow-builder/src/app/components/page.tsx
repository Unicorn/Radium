/**
 * Components Library Page
 */

'use client';

import { YStack, XStack, H1, Button } from 'tamagui';
import { useRouter } from 'next/navigation';
import { AuthGuardWithLoading } from '@/components/shared/AuthGuard';
import { Header } from '@/components/shared/Header';
import { Sidebar } from '@/components/shared/Sidebar';
import { ComponentList } from '@/components/component/ComponentList';

function ComponentsContent() {
  const router = useRouter();

  return (
    <YStack flex={1}>
      <Header />
      <XStack flex={1}>
        <Sidebar />
        <YStack flex={1} padding="$6" gap="$4">
          <XStack justifyContent="space-between" alignItems="center">
            <H1>Components</H1>
            <Button
              theme="blue"
              onPress={() => router.push('/components/new')}
            >
              New Component
            </Button>
          </XStack>

          <ComponentList />
        </YStack>
      </XStack>
    </YStack>
  );
}

export default function ComponentsPage() {
  return (
    <AuthGuardWithLoading>
      <ComponentsContent />
    </AuthGuardWithLoading>
  );
}

