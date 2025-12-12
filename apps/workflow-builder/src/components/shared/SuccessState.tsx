/**
 * Success State Component
 *
 * Accessible success notification with visual feedback
 */

'use client';

import { YStack, XStack, Text, Button, Card } from 'tamagui';
import { CheckCircle, X } from 'lucide-react';
import { useState, useEffect } from 'react';

interface SuccessStateProps {
  title?: string;
  message: string;
  onDismiss?: () => void;
  autoDismiss?: boolean;
  autoDismissDelay?: number;
}

export function SuccessState({
  title = 'Success',
  message,
  onDismiss,
  autoDismiss = false,
  autoDismissDelay = 5000,
}: SuccessStateProps) {
  const [isVisible, setIsVisible] = useState(true);

  useEffect(() => {
    if (autoDismiss) {
      const timer = setTimeout(() => {
        setIsVisible(false);
        onDismiss?.();
      }, autoDismissDelay);

      return () => clearTimeout(timer);
    }
  }, [autoDismiss, autoDismissDelay, onDismiss]);

  if (!isVisible) return null;

  const handleDismiss = () => {
    setIsVisible(false);
    onDismiss?.();
  };

  return (
    <Card
      p="$4"
      bg="$green2"
      borderWidth={1}
      borderColor="$green6"
      borderRadius="$4"
      role="status"
      aria-live="polite"
      aria-atomic="true"
      animation="quick"
      opacity={isVisible ? 1 : 0}
      scale={isVisible ? 1 : 0.95}
    >
      <XStack ai="center" gap="$3">
        <CheckCircle size={24} color="hsl(140, 70%, 40%)" aria-hidden="true" />
        <YStack f={1} gap="$1">
          <Text fontSize="$5" fontWeight="600" color="$green11">
            {title}
          </Text>
          <Text fontSize="$3" color="$green11">
            {message}
          </Text>
        </YStack>
        {onDismiss && (
          <Button
            size="$3"
            circular
            chromeless
            icon={X}
            onPress={handleDismiss}
            aria-label="Dismiss success message"
          />
        )}
      </XStack>
    </Card>
  );
}
