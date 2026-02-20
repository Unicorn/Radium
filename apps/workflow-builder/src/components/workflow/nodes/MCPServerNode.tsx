/**
 * MCP Server Node - MCP (Model Context Protocol) server configuration component
 */

'use client';

import { memo } from 'react';
import { Handle, Position, type NodeProps } from 'react-flow-renderer';
import { YStack, XStack, Text, Card, XGroup } from 'tamagui';
import { Server } from 'lucide-react';

export const MCPServerNode = memo(({ data, selected }: NodeProps) => {
  const componentName = data?.label || data?.componentName || 'MCP Server';
  const displayName = data?.displayName || componentName;
  const config = data?.config || {};
  const serverName = config.serverName || 'Unnamed Server';
  const serverVersion = config.serverVersion || '1.0.0';
  const resourceCount = config.resources?.length || 0;
  const toolCount = config.tools?.length || 0;
  const endpointPath = config.endpointPath || '/mcp';

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
          <Server size={18} color="var(--blue9)" />
          <YStack flex={1} gap="$2">
            <XStack gap="$2" alignItems="center" flexWrap="wrap">
              <Text fontSize="$3" fontWeight="600" color={selected ? '$blue11' : '$gray12'}>
                {displayName}
              </Text>
              <XGroup paddingHorizontal="$2" paddingVertical="$1" backgroundColor="$purple3" borderRadius="$2">
                <Text fontSize="$1" color="$purple11">MCP</Text>
              </XGroup>
            </XStack>
            
            <YStack gap="$1">
              <Text fontSize="$2" color="$gray11">
                Server: <Text fontWeight="600" color="$gray12">{serverName}</Text>
              </Text>
              <Text fontSize="$2" color="$gray11">
                Version: <Text fontWeight="600" color="$gray12">{serverVersion}</Text>
              </Text>
              <Text fontSize="$2" color="$gray11">
                Endpoint: <Text fontWeight="600" color="$gray12" fontFamily="monospace" fontSize="$1">
                  {endpointPath}
                </Text>
              </Text>
              <XStack gap="$3" alignItems="center" flexWrap="wrap">
                {resourceCount > 0 && (
                  <Text fontSize="$2" color="$gray11">
                    Resources: <Text fontWeight="600" color="$gray12">{resourceCount}</Text>
                  </Text>
                )}
                {toolCount > 0 && (
                  <Text fontSize="$2" color="$gray11">
                    Tools: <Text fontWeight="600" color="$gray12">{toolCount}</Text>
                  </Text>
                )}
              </XStack>
              {resourceCount === 0 && toolCount === 0 && (
                <XGroup paddingHorizontal="$2" paddingVertical="$1" backgroundColor="$yellow3" borderRadius="$2">
                  <Text fontSize="$1" color="$yellow11">No Resources/Tools</Text>
                </XGroup>
              )}
            </YStack>

            <Text fontSize="$1" color="$gray10" fontStyle="italic">
              Model Context Protocol server
            </Text>
          </YStack>
        </XStack>
      </Card>
      <Handle type="source" position={Position.Bottom} />
    </>
  );
});

MCPServerNode.displayName = 'MCPServerNode';

