'use client';

import { useState } from 'react';
import { YStack, XStack, Card, Text, Button, ScrollView } from 'tamagui';
import { XCircle, Copy, Check, AlertTriangle } from 'lucide-react';

interface ExecutionErrorProps {
  error: string;
  errorType?: string;
  errorStack?: string;
}

export function ExecutionError({ error, errorType, errorStack }: ExecutionErrorProps) {
  const [copied, setCopied] = useState(false);
  const [showStack, setShowStack] = useState(false);

  const handleCopy = async () => {
    try {
      const errorText = errorStack
        ? `${error}\n\nType: ${errorType || 'Unknown'}\n\nStack Trace:\n${errorStack}`
        : error;

      await navigator.clipboard.writeText(errorText);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      // Silently fail if clipboard copy doesn't work
    }
  };

  return (
    <Card p="$4" borderWidth={1} borderColor="$red6" bg="$red1">
      <YStack gap="$3">
        {/* Header */}
        <YStack gap="$2">
          <XStack ai="center" gap="$2">
            <XCircle size={20} color="$red11" />
            <Text fontSize="$5" fontWeight="600" color="$red11">
              Execution Failed
            </Text>
          </XStack>

          <Text fontSize="$3" color="$gray11">
            The workflow execution encountered an error:
          </Text>
        </YStack>

        {/* Error Type */}
        {errorType && (
          <Card p="$3" bg="$red2" borderWidth={1} borderColor="$red6">
            <XStack ai="center" gap="$2">
              <AlertTriangle size={16} color="$red11" />
              <Text fontSize="$3" fontWeight="600" color="$red11">
                Error Type:
              </Text>
              <Text fontSize="$3" fontFamily="$mono" color="$red11">
                {errorType}
              </Text>
            </XStack>
          </Card>
        )}

        {/* Error Message */}
        <ScrollView maxHeight={200}>
          <Card
            p="$4"
            bg="$background"
            borderWidth={1}
            borderColor="$red6"
          >
            <Text
              fontSize="$3"
              fontFamily="$mono"
              color="$red11"
              whiteSpace="pre-wrap"
              wordWrap="break-word"
            >
              {error}
            </Text>
          </Card>
        </ScrollView>

        {/* Actions */}
        <XStack gap="$3" flexWrap="wrap">
          <Button
            size="$3"
            icon={copied ? Check : Copy}
            onPress={handleCopy}
            chromeless
          >
            {copied ? 'Copied!' : 'Copy Error'}
          </Button>

          {errorStack && (
            <Button
              size="$3"
              onPress={() => setShowStack(!showStack)}
              chromeless
            >
              {showStack ? 'Hide Stack Trace' : 'Show Stack Trace'}
            </Button>
          )}
        </XStack>

        {/* Stack Trace */}
        {showStack && errorStack && (
          <ScrollView maxHeight={300}>
            <Card
              p="$4"
              bg="$gray2"
              borderWidth={1}
              borderColor="$borderColor"
            >
              <Text fontSize="$2" fontWeight="600" color="$gray11" mb="$2">
                Stack Trace:
              </Text>
              <Text
                fontSize="$2"
                fontFamily="$mono"
                color="$gray11"
                whiteSpace="pre-wrap"
                wordWrap="break-word"
              >
                {errorStack}
              </Text>
            </Card>
          </ScrollView>
        )}
      </YStack>
    </Card>
  );
}
