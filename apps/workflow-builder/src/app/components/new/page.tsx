/**
 * Create New Component Page
 */

'use client';

import { YStack, XStack, H1 } from 'tamagui';
import { AuthGuardWithLoading } from '@/components/shared/AuthGuard';
import { Header } from '@/components/shared/Header';
import { Sidebar } from '@/components/shared/Sidebar';
import { ComponentForm } from '@/components/component/ComponentForm';

function NewComponentContent() {
  return (
    <YStack flex={1}>
      <Header />
      <XStack flex={1}>
        <Sidebar />
        <YStack flex={1} padding="$6" gap="$4">
          <H1>Create Component</H1>
          <ComponentForm />
        </YStack>
      </XStack>
    </YStack>
  );
}

export default function NewComponentPage() {
  return (
    <AuthGuardWithLoading>
      <NewComponentContent />
    </AuthGuardWithLoading>
  );
}

