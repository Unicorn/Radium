/**
 * Error State Component
 *
 * Accessible error display with recovery actions
 */

'use client';

import { YStack, XStack, Text, Button, Card } from 'tamagui';
import { AlertCircle, RefreshCw } from 'lucide-react';

interface ErrorStateProps {
  title?: string;
  message: string;
  onRetry?: () => void;
  onDismiss?: () => void;
  fullscreen?: boolean;
}

export function ErrorState({
  title = 'Something went wrong',
  message,
  onRetry,
  onDismiss,
  fullscreen = false,
}: ErrorStateProps) {
  const Content = (
    <Card
      p="$4"
      bg="$red2"
      borderWidth={1}
      borderColor="$red6"
      borderRadius="$4"
      role="alert"
      aria-live="assertive"
      aria-atomic="true"
    >
      <YStack gap="$3" maxWidth={500}>
        <XStack ai="center" gap="$3">
          <AlertCircle size={24} color="hsl(0, 70%, 50%)" aria-hidden="true" />
          <YStack f={1} gap="$1">
            <Text fontSize="$5" fontWeight="600" color="$red11">
              {title}
            </Text>
            <Text fontSize="$3" color="$red11">
              {message}
            </Text>
          </YStack>
        </XStack>

        {(onRetry || onDismiss) && (
          <XStack gap="$2" jc="flex-end">
            {onDismiss && (
              <Button
                size="$3"
                chromeless
                onPress={onDismiss}
                aria-label="Dismiss error"
              >
                Dismiss
              </Button>
            )}
            {onRetry && (
              <Button
                size="$3"
                themeInverse
                icon={RefreshCw}
                onPress={onRetry}
                aria-label="Retry action"
              >
                Try Again
              </Button>
            )}
          </XStack>
        )}
      </YStack>
    </Card>
  );

  if (fullscreen) {
    return (
      <YStack f={1} ai="center" jc="center" minHeight="100vh" p="$4">
        {Content}
      </YStack>
    );
  }

  return Content;
}
