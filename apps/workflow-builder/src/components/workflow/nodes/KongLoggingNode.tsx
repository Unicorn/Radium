/**
 * Kong Logging Node - Project-level logging configuration component
 */

'use client';

import { memo } from 'react';
import { Handle, Position, type NodeProps } from 'react-flow-renderer';
import { YStack, XStack, Text, Card, XGroup } from 'tamagui';
import { FileText, Globe } from 'lucide-react';

export const KongLoggingNode = memo(({ data, selected }: NodeProps) => {
  const componentName = data?.label || data?.componentName || 'Kong Logging';
  const displayName = data?.displayName || componentName;
  const connectorName = data?.config?.connectorName || data?.connectorName || 'No connector';
  const enabledCount = Array.isArray(data?.config?.enabledEndpoints) 
    ? data.config.enabledEndpoints.length 
    : 0;

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
          <FileText size={18} color="var(--blue9)" />
          <YStack flex={1} gap="$2">
            <XStack gap="$2" alignItems="center" flexWrap="wrap">
              <Text fontSize="$3" fontWeight="600" color={selected ? '$blue11' : '$gray12'}>
                {displayName}
              </Text>
              <XGroup paddingHorizontal="$2" paddingVertical="$1" backgroundColor="$blue3" borderRadius="$2">
                <Globe size={10} color="var(--blue11)" />
                <Text fontSize="$1" marginLeft="$1" color="$blue11">Project</Text>
              </XGroup>
            </XStack>
            
            <YStack gap="$1">
              <Text fontSize="$2" color="$gray11">
                Connector: <Text fontWeight="600" color="$gray12">{connectorName}</Text>
              </Text>
              <Text fontSize="$2" color="$gray11">
                Endpoints: <Text fontWeight="600" color="$gray12">{enabledCount} enabled</Text>
              </Text>
            </YStack>

            <Text fontSize="$1" color="$gray10" fontStyle="italic">
              Project-level: Visible on all services
            </Text>
          </YStack>
        </XStack>
      </Card>
      <Handle type="source" position={Position.Bottom} />
    </>
  );
});

KongLoggingNode.displayName = 'KongLoggingNode';

