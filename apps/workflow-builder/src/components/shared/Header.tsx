/**
 * Header Component
 */

'use client';

import { useRouter } from 'next/navigation';
import { XStack, YStack, Text, Button } from 'tamagui';
import { createClient } from '@/lib/supabase/client';
import { api } from '@/lib/trpc/client';

export function Header() {
  const router = useRouter();
  const supabase = createClient();
  const { data: user } = api.users.me.useQuery();

  const handleSignOut = async () => {
    await supabase.auth.signOut();
    router.push('/auth/signin');
  };

  return (
    <XStack
      paddingHorizontal="$4"
      paddingVertical="$3"
      backgroundColor="$background"
      borderBottomWidth={1}
      borderColor="$borderColor"
      alignItems="center"
      justifyContent="space-between"
    >
      <XStack alignItems="center" gap="$4">
        <Text fontSize="$8" fontWeight="bold">
          Workflow Builder
        </Text>
      </XStack>

      <XStack alignItems="center" gap="$3">
        {user && (
          <YStack alignItems="flex-end" gap="$1">
            <Text fontSize="$3" fontWeight="600">
              {user.display_name || user.email}
            </Text>
            <Text fontSize="$2" color="$gray11">
              {user.role.name}
            </Text>
          </YStack>
        )}
        <Button size="$3" onPress={handleSignOut}>
          Sign Out
        </Button>
      </XStack>
    </XStack>
  );
}

