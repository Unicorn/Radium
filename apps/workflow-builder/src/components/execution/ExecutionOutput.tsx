'use client';

import { useState } from 'react';
import { YStack, Card, Text, Button, ScrollView } from 'tamagui';
import { CheckCircle, Copy, Check } from 'lucide-react';

interface ExecutionOutputProps {
  output: unknown;
}

export function ExecutionOutput({ output }: ExecutionOutputProps) {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(JSON.stringify(output, null, 2));
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      // Silently fail if clipboard copy doesn't work
    }
  };

  return (
    <Card p="$4" borderWidth={1} borderColor="$green6" bg="$green1">
      <YStack gap="$3">
        {/* Header */}
        <YStack gap="$2">
          <YStack gap="$2">
            <Text fontSize="$5" fontWeight="600" color="$green11">
              <CheckCircle size={20} style={{ display: 'inline', marginRight: 8 }} />
              Execution Output
            </Text>
            <Text fontSize="$3" color="$gray11">
              The workflow completed successfully with the following output:
            </Text>
          </YStack>

          <Button
            size="$3"
            icon={copied ? Check : Copy}
            onPress={handleCopy}
            alignSelf="flex-start"
            chromeless
          >
            {copied ? 'Copied!' : 'Copy Output'}
          </Button>
        </YStack>

        {/* Output Display */}
        <ScrollView maxHeight={400}>
          <Card
            p="$4"
            bg="$background"
            borderWidth={1}
            borderColor="$borderColor"
          >
            <Text
              fontSize="$2"
              fontFamily="$mono"
              color="$green11"
              whiteSpace="pre-wrap"
              wordWrap="break-word"
            >
              {typeof output === 'string'
                ? output
                : JSON.stringify(output, null, 2)
              }
            </Text>
          </Card>
        </ScrollView>
      </YStack>
    </Card>
  );
}
