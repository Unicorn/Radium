/**
 * Data Out Node - Interface component for providing data via queries
 */

'use client';

import { memo } from 'react';
import { Handle, Position, type NodeProps } from 'react-flow-renderer';
import { YStack, XStack, Text, Card } from 'tamagui';
import { Upload, ArrowUp } from 'lucide-react';

export const DataOutNode = memo(({ data, selected }: NodeProps) => {
  const endpointPath = data?.config?.endpointPath || data?.endpointPath || '/endpoint';
  const httpMethod = data?.config?.httpMethod || data?.httpMethod || 'GET';
  const componentName = data?.label || data?.componentName || 'Data Out';
  const displayName = data?.displayName || componentName;

  return (
    <>
      <Handle type="target" position={Position.Top} />
      <Card
        padding="$3"
        backgroundColor={selected ? '$teal3' : '$background'}
        borderWidth={selected ? 2 : 1}
        borderColor={selected ? '$teal9' : '$borderColor'}
        borderRadius="$4"
        minWidth={220}
        elevate={selected}
      >
        <XStack gap="$2" alignItems="center">
          <Upload size={18} color="var(--teal9)" />
          <YStack flex={1} gap="$1">
            <Text fontSize="$3" fontWeight="600" color={selected ? '$teal11' : '$gray12'}>
              {displayName}
            </Text>
            <XStack gap="$2" alignItems="center">
              <Text
                fontSize="$2"
                fontWeight="bold"
                color="$blue9"
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
              Provides data via query
            </Text>
          </YStack>
        </XStack>
      </Card>
      <Handle type="source" position={Position.Bottom} />
    </>
  );
});

DataOutNode.displayName = 'DataOutNode';

