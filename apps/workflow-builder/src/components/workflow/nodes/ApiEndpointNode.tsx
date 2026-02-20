/**
 * API Endpoint Node Component
 * 
 * Visual representation of an API endpoint in the workflow canvas
 */

'use client';

import { Handle, Position, type NodeProps } from 'react-flow-renderer';
import { YStack, XStack, Text, Card } from 'tamagui';
import { Globe } from 'lucide-react';

export function ApiEndpointNode({ data, selected }: NodeProps) {
  const method = data?.config?.method || 'POST';
  const path = data?.config?.endpointPath || '/api/...';
  const targetType = data?.config?.targetType || 'start';

  const methodColors: Record<string, string> = {
    GET: '$blue9',
    POST: '$green9',
    PUT: '$orange9',
    DELETE: '$red9',
    PATCH: '$purple9',
  };

  const methodColor = methodColors[method] || '$gray9';

  return (
    <>
      <Handle type="target" position={Position.Top} />
      <Card
        padding="$3"
        backgroundColor={selected ? '$blue3' : '$background'}
        borderWidth={selected ? 2 : 1}
        borderColor={selected ? '$blue9' : '$borderColor'}
        borderRadius="$4"
        minWidth={200}
        elevate={selected}
      >
        <XStack gap="$2" alignItems="center">
          <Globe size={18} color="var(--color)" />
          <YStack flex={1} gap="$1">
            <XStack gap="$2" alignItems="center">
              <Text
                fontSize="$2"
                fontWeight="bold"
                color={methodColor}
                fontFamily="monospace"
              >
                {method}
              </Text>
              <Text fontSize="$2" color="$gray11" fontFamily="monospace" numberOfLines={1}>
                {path}
              </Text>
            </XStack>
            <Text fontSize="$1" color="$gray10">
              {targetType === 'start' && 'Start Workflow'}
              {targetType === 'signal' && `Signal: ${data?.config?.targetName || ''}`}
              {targetType === 'query' && `Query: ${data?.config?.targetName || ''}`}
            </Text>
          </YStack>
        </XStack>
      </Card>
      <Handle type="source" position={Position.Bottom} />
    </>
  );
}

