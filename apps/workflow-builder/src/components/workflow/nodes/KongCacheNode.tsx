/**
 * Kong Cache Node - Kong proxy caching configuration component
 */

'use client';

import { memo } from 'react';
import { Handle, Position, type NodeProps } from 'react-flow-renderer';
import { YStack, XStack, Text, Card, XGroup } from 'tamagui';
import { Database, Key } from 'lucide-react';

export const KongCacheNode = memo(({ data, selected }: NodeProps) => {
  const componentName = data?.label || data?.componentName || 'Kong Cache';
  const displayName = data?.displayName || componentName;
  const connectorName = data?.config?.connectorName || data?.connectorName || 'No connector';
  const cacheKey = data?.config?.cacheKey || 'Not set';
  const isSaved = data?.config?.isSaved || false;
  const ttl = data?.config?.ttlSeconds || 3600;

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
          <Database size={18} color="var(--blue9)" />
          <YStack flex={1} gap="$2">
            <XStack gap="$2" alignItems="center" flexWrap="wrap">
              <Text fontSize="$3" fontWeight="600" color={selected ? '$blue11' : '$gray12'}>
                {displayName}
              </Text>
              {isSaved && (
                <XGroup paddingHorizontal="$2" paddingVertical="$1" backgroundColor="$green3" borderRadius="$2">
                  <Key size={10} color="var(--green11)" />
                  <Text fontSize="$1" marginLeft="$1" color="$green11">Saved</Text>
                </XGroup>
              )}
            </XStack>
            
            <YStack gap="$1">
              <Text fontSize="$2" color="$gray11">
                Connector: <Text fontWeight="600" color="$gray12">{connectorName}</Text>
              </Text>
              <XStack gap="$2" alignItems="center" flexWrap="wrap">
                <Text fontSize="$2" color="$gray11">
                  Key: <Text fontWeight="600" color="$gray12" fontFamily="monospace" fontSize="$1">
                    {cacheKey.length > 20 ? `${cacheKey.substring(0, 20)}...` : cacheKey}
                  </Text>
                </Text>
                {!isSaved && (
                  <XGroup paddingHorizontal="$2" paddingVertical="$1" backgroundColor="$yellow3" borderRadius="$2">
                    <Text fontSize="$1" color="$yellow11">Editable</Text>
                  </XGroup>
                )}
              </XStack>
              <Text fontSize="$2" color="$gray11">
                TTL: <Text fontWeight="600" color="$gray12">{ttl}s</Text>
              </Text>
            </YStack>

            <Text fontSize="$1" color="$gray10" fontStyle="italic">
              Redis-backed proxy caching
            </Text>
          </YStack>
        </XStack>
      </Card>
      <Handle type="source" position={Position.Bottom} />
    </>
  );
});

KongCacheNode.displayName = 'KongCacheNode';

