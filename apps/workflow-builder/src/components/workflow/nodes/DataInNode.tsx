/**
 * Data In Node - Interface component for receiving data via signals
 */

'use client';

import { memo } from 'react';
import { Handle, Position, type NodeProps } from 'react-flow-renderer';
import { YStack, XStack, Text, Card } from 'tamagui';
import { Download, ArrowDown } from 'lucide-react';

export const DataInNode = memo(({ data, selected }: NodeProps) => {
  const endpointPath = data?.config?.endpointPath || data?.endpointPath || '/endpoint';
  const httpMethod = data?.config?.httpMethod || data?.httpMethod || 'POST';
  const componentName = data?.label || data?.componentName || 'Data In';
  const displayName = data?.displayName || componentName;

  const methodColors: Record<string, string> = {
    POST: '$green9',
    PATCH: '$purple9',
    PUT: '$orange9',
  };

  const methodColor = methodColors[httpMethod] || '$green9';

  return (
    <>
      <Handle type="target" position={Position.Top} />
      <Card
        padding="$3"
        backgroundColor={selected ? '$cyan3' : '$background'}
        borderWidth={selected ? 2 : 1}
        borderColor={selected ? '$cyan9' : '$borderColor'}
        borderRadius="$4"
        minWidth={220}
        elevate={selected}
      >
        <XStack gap="$2" alignItems="center">
          <Download size={18} color="var(--cyan9)" />
          <YStack flex={1} gap="$1">
            <Text fontSize="$3" fontWeight="600" color={selected ? '$cyan11' : '$gray12'}>
              {displayName}
            </Text>
            <XStack gap="$2" alignItems="center">
              <Text
                fontSize="$2"
                fontWeight="bold"
                color={methodColor}
                fontFamily="monospace"
              >
                {httpMethod}
              </Text>
              <Text 
                fontSize="$2" 
                color="$gray11" 
                fontFamily="monospace" 
                numberOfLines={1}
                flex={1}
              >
                {endpointPath}
              </Text>
            </XStack>
            <Text fontSize="$1" color="$gray10">
              Receives data via signal
            </Text>
          </YStack>
        </XStack>
      </Card>
      <Handle type="source" position={Position.Bottom} />
    </>
  );
});

DataInNode.displayName = 'DataInNode';

