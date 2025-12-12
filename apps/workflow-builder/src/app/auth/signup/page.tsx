/**
 * Sign Up Page
 */

'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';
import Link from 'next/link';
import { YStack, XStack, Text, Button, Input, H1, Card } from 'tamagui';
import { createClient } from '@/lib/supabase/client';

export default function SignUpPage() {
  const router = useRouter();
  const supabase = createClient();
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [displayName, setDisplayName] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);
  const [success, setSuccess] = useState(false);

  const handleSignUp = async () => {
    setError('');
    setLoading(true);

    // Validation
    if (password !== confirmPassword) {
      setError('Passwords do not match');
      setLoading(false);
      return;
    }

    if (password.length < 8) {
      setError('Password must be at least 8 characters');
      setLoading(false);
      return;
    }

    try {
      const { data, error } = await supabase.auth.signUp({
        email,
        password,
        options: {
          data: {
            display_name: displayName || email.split('@')[0],
          },
        },
      });

      if (error) throw error;

      if (data.user) {
        setSuccess(true);
        // Auto sign in and redirect after a brief moment
        setTimeout(() => {
          router.push('/');
          router.refresh();
        }, 2000);
      }
    } catch (err: any) {
      setError(err.message || 'Failed to sign up');
    } finally {
      setLoading(false);
    }
  };

  if (success) {
    return (
      <YStack
        flex={1}
        alignItems="center"
        justifyContent="center"
        backgroundColor="$gray2"
        padding="$4"
      >
        <Card width={400} maxWidth="100%" padding="$6" elevate>
          <YStack gap="$4" alignItems="center">
            <Text fontSize="$8">✓</Text>
            <H1 fontSize="$6">Account Created!</H1>
            <Text color="$gray11" textAlign="center">
              Welcome to Workflow Builder. Redirecting to dashboard...
            </Text>
          </YStack>
        </Card>
      </YStack>
    );
  }

  return (
    <YStack
      flex={1}
      alignItems="center"
      justifyContent="center"
      backgroundColor="$gray2"
      padding="$4"
    >
      <Card width={400} maxWidth="100%" padding="$6" elevate>
        <YStack gap="$4">
          <YStack gap="$2" alignItems="center">
            <H1>Sign Up</H1>
            <Text color="$gray11">Create your Workflow Builder account</Text>
          </YStack>

          {error && (
            <Card padding="$3" backgroundColor="$red3" borderColor="$red7" borderWidth={1}>
              <Text color="$red11" fontSize="$3">
                {error}
              </Text>
            </Card>
          )}

          <YStack gap="$2">
            <Text fontSize="$3" fontWeight="600">
              Display Name
            </Text>
            <Input
              size="$4"
              placeholder="John Doe"
              value={displayName}
              onChangeText={setDisplayName}
              autoComplete="name"
              disabled={loading}
              onSubmitEditing={handleSignUp}
            />
          </YStack>

          <YStack gap="$2">
            <Text fontSize="$3" fontWeight="600">
              Email
            </Text>
            <Input
              size="$4"
              placeholder="you@example.com"
              value={email}
              onChangeText={setEmail}
              keyboardType="email-address"
              autoCapitalize="none"
              autoComplete="email"
              disabled={loading}
              onSubmitEditing={handleSignUp}
            />
          </YStack>

          <YStack gap="$2">
            <Text fontSize="$3" fontWeight="600">
              Password
            </Text>
            <Input
              size="$4"
              placeholder="••••••••"
              value={password}
              onChangeText={setPassword}
              secureTextEntry
              autoComplete="new-password"
              disabled={loading}
              onSubmitEditing={handleSignUp}
            />
            <Text fontSize="$2" color="$gray11">
              Minimum 8 characters
            </Text>
          </YStack>

          <YStack gap="$2">
            <Text fontSize="$3" fontWeight="600">
              Confirm Password
            </Text>
            <Input
              size="$4"
              placeholder="••••••••"
              value={confirmPassword}
              onChangeText={setConfirmPassword}
              secureTextEntry
              autoComplete="new-password"
              disabled={loading}
              onSubmitEditing={handleSignUp}
            />
          </YStack>

          <Button
            size="$4"
            theme="blue"
            onPress={handleSignUp}
            disabled={loading || !email || !password || !confirmPassword}
          >
            {loading ? 'Creating account...' : 'Sign Up'}
          </Button>

          <XStack gap="$2" justifyContent="center" alignItems="center">
            <Text fontSize="$3" color="$gray11">
              Already have an account?{' '}
            </Text>
            <Link href="/auth/signin" style={{ textDecoration: 'none', color: 'inherit' }}>
              <Text 
                as="span"
                fontSize="$3" 
                color="$blue10" 
                fontWeight="600" 
                cursor="pointer" 
                hoverStyle={{ color: '$blue11' }}
              >
                Sign In
              </Text>
            </Link>
          </XStack>
        </YStack>
      </Card>
    </YStack>
  );
}

