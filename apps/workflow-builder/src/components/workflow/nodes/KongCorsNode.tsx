/**
 * Kong CORS Node - CORS configuration component
 */

'use client';

import { memo } from 'react';
import { Handle, Position, type NodeProps } from 'react-flow-renderer';
import { YStack, XStack, Text, Card, XGroup } from 'tamagui';
import { Globe } from 'lucide-react';

export const KongCorsNode = memo(({ data, selected }: NodeProps) => {
  const componentName = data?.label || data?.componentName || 'Kong CORS';
  const displayName = data?.displayName || componentName;
  const config = data?.config || {};
  const allowedOrigins = config.allowedOrigins || [];
  const allowedMethods = config.allowedMethods || [];
  const allowedHeaders = config.allowedHeaders || [];
  const credentials = config.credentials || false;
  const maxAge = config.maxAge || 3600;

  const originsCount = Array.isArray(allowedOrigins) ? allowedOrigins.length : 0;
  const methodsCount = Array.isArray(allowedMethods) ? allowedMethods.length : 0;

  return (
    <>
      <Handle type="target" position={Position.Top} />
      <Card
        padding="$3"
        backgroundColor={selected ? '$blue3' : '$background'}
        borderWidth={selected ? 2 : 1}
        borderColor={selected ? '$blue9' : '$borderColor'}
        borderRadius="$4"
        minWidth={240}
        elevate={selected}
      >
        <XStack gap="$2" alignItems="flex-start">
          <Globe size={18} color="var(--blue9)" />
          <YStack flex={1} gap="$2">
            <XStack gap="$2" alignItems="center" flexWrap="wrap">
              <Text fontSize="$3" fontWeight="600" color={selected ? '$blue11' : '$gray12'}>
                {displayName}
              </Text>
            </XStack>
            
            <YStack gap="$1">
              <Text fontSize="$2" color="$gray11">
                Origins: <Text fontWeight="600" color="$gray12">{originsCount} configured</Text>
              </Text>
              <Text fontSize="$2" color="$gray11">
                Methods: <Text fontWeight="600" color="$gray12">{methodsCount} allowed</Text>
              </Text>
              {credentials && (
                <XGroup paddingHorizontal="$2" paddingVertical="$1" backgroundColor="$green3" borderRadius="$2">
                  <Text fontSize="$1" color="$green11">Credentials</Text>
                </XGroup>
              )}
              <Text fontSize="$2" color="$gray11">
                Max-Age: <Text fontWeight="600" color="$gray12">{maxAge}s</Text>
              </Text>
            </YStack>

            <Text fontSize="$1" color="$gray10" fontStyle="italic">
              CORS configuration for API endpoints
            </Text>
          </YStack>
        </XStack>
      </Card>
      <Handle type="source" position={Position.Bottom} />
    </>
  );
});

KongCorsNode.displayName = 'KongCorsNode';

