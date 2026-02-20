/**
 * Auth Guard Component
 * Redirects to sign in if not authenticated
 */

'use client';

import { useEffect, useState } from 'react';
import { useRouter } from 'next/navigation';
import { createClient } from '@/lib/supabase/client';
import { YStack, Spinner, Text } from 'tamagui';

export function AuthGuard({ children }: { children: React.ReactNode }) {
  const router = useRouter();
  const supabase = createClient();

  useEffect(() => {
    const checkAuth = async () => {
      const {
        data: { user },
      } = await supabase.auth.getUser();

      if (!user) {
        router.push('/auth/signin');
      }
    };

    checkAuth();

    // Listen for auth changes
    const {
      data: { subscription },
    } = supabase.auth.onAuthStateChange((event, session) => {
      if (event === 'SIGNED_OUT') {
        router.push('/auth/signin');
      }
    });

    return () => {
      subscription.unsubscribe();
    };
  }, [router, supabase]);

  return <>{children}</>;
}

export function AuthGuardWithLoading({ children }: { children: React.ReactNode }) {
  const router = useRouter();
  const supabase = createClient();
  const [isLoading, setIsLoading] = useState(true);
  const [isAuthenticated, setIsAuthenticated] = useState(false);

  useEffect(() => {
    const checkAuth = async () => {
      console.log('🔐 [AuthGuard] Checking authentication...');
      const {
        data: { user },
        error,
      } = await supabase.auth.getUser();

      if (error) {
        console.error('❌ [AuthGuard] Error checking auth:', error);
      }

      if (!user) {
        console.log('⚠️  [AuthGuard] No user found, redirecting to sign-in');
        router.push('/auth/signin');
      } else {
        console.log('✅ [AuthGuard] User authenticated:', {
          id: user.id,
          email: user.email,
        });
        setIsAuthenticated(true);
      }
      setIsLoading(false);
    };

    checkAuth();

    // Listen for auth changes
    const {
      data: { subscription },
    } = supabase.auth.onAuthStateChange((event, session) => {
      if (event === 'SIGNED_OUT') {
        router.push('/auth/signin');
        setIsAuthenticated(false);
      } else if (event === 'SIGNED_IN') {
        setIsAuthenticated(true);
      }
    });

    return () => {
      subscription.unsubscribe();
    };
  }, [router, supabase]);

  if (isLoading) {
    return (
      <YStack flex={1} alignItems="center" justifyContent="center" gap="$4">
        <Spinner size="large" />
        <Text>Loading...</Text>
      </YStack>
    );
  }

  if (!isAuthenticated) {
    return null;
  }

  return <>{children}</>;
}
