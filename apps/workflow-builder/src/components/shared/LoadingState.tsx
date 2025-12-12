/**
 * Loading State Component
 *
 * Accessible loading indicator with proper ARIA attributes
 */

'use client';

import { YStack, Text, Spinner } from 'tamagui';

interface LoadingStateProps {
  message?: string;
  size?: 'small' | 'medium' | 'large';
  fullscreen?: boolean;
}

export function LoadingState({
  message = 'Loading...',
  size = 'medium',
  fullscreen = false,
}: LoadingStateProps) {
  const spinnerSize = {
    small: '$4',
    medium: '$6',
    large: '$8',
  }[size];

  const Content = (
    <YStack
      ai="center"
      jc="center"
      gap="$3"
      role="status"
      aria-live="polite"
      aria-busy="true"
    >
      <Spinner size={spinnerSize} color="$blue10" />
      <Text fontSize="$4" color="$gray11">
        {message}
      </Text>
      <span className="sr-only">{message}</span>
    </YStack>
  );

  if (fullscreen) {
    return (
      <YStack f={1} ai="center" jc="center" minHeight="100vh">
        {Content}
      </YStack>
    );
  }

  return Content;
}
