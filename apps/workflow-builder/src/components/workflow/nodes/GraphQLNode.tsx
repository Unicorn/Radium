/**
 * GraphQL Gateway Node - GraphQL endpoint configuration component
 */

'use client';

import { memo } from 'react';
import { Handle, Position, type NodeProps } from 'react-flow-renderer';
import { YStack, XStack, Text, Card, XGroup } from 'tamagui';
import { Network } from 'lucide-react';

export const GraphQLNode = memo(({ data, selected }: NodeProps) => {
  const componentName = data?.label || data?.componentName || 'GraphQL Gateway';
  const displayName = data?.displayName || componentName;
  const config = data?.config || {};
  const endpointPath = config.endpointPath || '/graphql';
  const hasSchema = !!config.schema;
  const queryCount = config.queries?.length || 0;
  const mutationCount = config.mutations?.length || 0;

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
          <Network size={18} color="var(--blue9)" />
          <YStack flex={1} gap="$2">
            <XStack gap="$2" alignItems="center" flexWrap="wrap">
              <Text fontSize="$3" fontWeight="600" color={selected ? '$blue11' : '$gray12'}>
                {displayName}
              </Text>
              {hasSchema && (
                <XGroup paddingHorizontal="$2" paddingVertical="$1" backgroundColor="$green3" borderRadius="$2">
                  <Text fontSize="$1" color="$green11">Schema</Text>
                </XGroup>
              )}
            </XStack>
            
            <YStack gap="$1">
              <Text fontSize="$2" color="$gray11">
                Endpoint: <Text fontWeight="600" color="$gray12" fontFamily="monospace" fontSize="$1">
                  {endpointPath}
                </Text>
              </Text>
              <XStack gap="$3" alignItems="center" flexWrap="wrap">
                {queryCount > 0 && (
                  <Text fontSize="$2" color="$gray11">
                    Queries: <Text fontWeight="600" color="$gray12">{queryCount}</Text>
                  </Text>
                )}
                {mutationCount > 0 && (
                  <Text fontSize="$2" color="$gray11">
                    Mutations: <Text fontWeight="600" color="$gray12">{mutationCount}</Text>
                  </Text>
                )}
              </XStack>
              {!hasSchema && (
                <XGroup paddingHorizontal="$2" paddingVertical="$1" backgroundColor="$yellow3" borderRadius="$2">
                  <Text fontSize="$1" color="$yellow11">Schema Required</Text>
                </XGroup>
              )}
            </YStack>

            <Text fontSize="$1" color="$gray10" fontStyle="italic">
              GraphQL endpoint gateway
            </Text>
          </YStack>
        </XStack>
      </Card>
      <Handle type="source" position={Position.Bottom} />
    </>
  );
});

GraphQLNode.displayName = 'GraphQLNode';

