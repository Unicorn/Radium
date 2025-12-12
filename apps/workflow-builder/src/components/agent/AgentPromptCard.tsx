/**
 * Agent Prompt Card - Display a single agent prompt
 */

'use client';

import { Card, YStack, XStack, Text } from 'tamagui';
import { Badge } from '../shared/Badge';
import type { Database } from '@/types/database';

type AgentPrompt = Database['public']['Tables']['agent_prompts']['Row'] & {
  visibility: { name: string };
};

interface AgentPromptCardProps {
  prompt: AgentPrompt;
  onClick?: () => void;
}

export function AgentPromptCard({ prompt, onClick }: AgentPromptCardProps) {
  return (
    <Card
      padding="$4"
      pressStyle={{ scale: 0.98 }}
      hoverStyle={{ backgroundColor: '$gray3' }}
      cursor="pointer"
      onPress={onClick}
      animation="quick"
    >
      <YStack gap="$3">
        <XStack justifyContent="space-between" alignItems="center">
          <Text fontSize="$6" fontWeight="600">
            {prompt.display_name}
          </Text>
          <Badge backgroundColor="$purple10" size="2">
            <Text fontSize="$2" color="white">
              agent
            </Text>
          </Badge>
        </XStack>

        {prompt.description && (
          <Text fontSize="$3" color="$gray11" numberOfLines={2}>
            {prompt.description}
          </Text>
        )}

        {prompt.capabilities && prompt.capabilities.length > 0 && (
          <XStack gap="$2" flexWrap="wrap">
            <Text fontSize="$2" color="$gray10">
              Capabilities:
            </Text>
            {prompt.capabilities.slice(0, 4).map((cap: string) => (
              <Badge key={cap} size="1" backgroundColor="$purple5">
                <Text fontSize="$1" color="$purple11">
                  {cap}
                </Text>
              </Badge>
            ))}
            {prompt.capabilities.length > 4 && (
              <Badge size="1" backgroundColor="$gray5">
                <Text fontSize="$1" color="$gray12">
                  +{prompt.capabilities.length - 4}
                </Text>
              </Badge>
            )}
          </XStack>
        )}

        <XStack justifyContent="space-between" alignItems="center">
          <Text fontSize="$2" color="$gray10">
            v{prompt.version}
          </Text>
          <XStack gap="$2">
            <Badge backgroundColor="$gray5" size="1">
              <Text fontSize="$1" color="$gray12">
                {prompt.visibility.name}
              </Text>
            </Badge>
            {prompt.deprecated && (
              <Badge backgroundColor="$red5" size="1">
                <Text fontSize="$1" color="$red11">
                  deprecated
                </Text>
              </Badge>
            )}
          </XStack>
        </XStack>
      </YStack>
    </Card>
  );
}

