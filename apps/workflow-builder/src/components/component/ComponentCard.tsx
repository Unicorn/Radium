/**
 * Component Card - Display a single component
 */

'use client';

import { useState } from 'react';
import { Card, YStack, XStack, Text } from 'tamagui';
import { Badge } from '../shared/Badge';
import { Box, Radio, Zap, PlayCircle, GitBranch, Repeat, ChevronDown, ChevronRight } from 'lucide-react';
import type { Database } from '@/types/database';

type Component = Database['public']['Tables']['components']['Row'] & {
  component_type: { name: string; icon: string | null };
  visibility: { name: string };
};

interface ComponentCardProps {
  component: Component;
  onClick?: () => void;
  draggable?: boolean;
  onDragStart?: (e: React.DragEvent) => void;
}

// Icon mapping for component types
const getTypeIcon = (type: string) => {
  const iconMap: Record<string, any> = {
    activity: Box,
    agent: Radio,
    trigger: Zap,
    signal: PlayCircle,
    condition: GitBranch,
    retry: Repeat,
  };
  return iconMap[type.toLowerCase()] || Box;
};

export function ComponentCard({
  component,
  onClick,
  draggable = false,
  onDragStart,
}: ComponentCardProps) {
  const [isExpanded, setIsExpanded] = useState(false);

  const typeColors: Record<string, string> = {
    activity: '#3b82f6',
    agent: '#a855f7',
    signal: '#22c55e',
    trigger: '#f97316',
    condition: '#eab308',
    retry: '#06b6d4',
  };

  const typeBackgrounds: Record<string, string> = {
    activity: '$blue3',
    agent: '$purple3',
    signal: '$green3',
    trigger: '$orange3',
    condition: '$yellow3',
    retry: '$cyan3',
  };

  const typeColor = typeColors[component.component_type.name] || '#6b7280';
  const typeBackground = typeBackgrounds[component.component_type.name] || '$gray3';
  const Icon = getTypeIcon(component.component_type.name);

  const handleCardClick = (e: React.MouseEvent) => {
    // Don't expand if dragging
    if (draggable) {
      setIsExpanded(!isExpanded);
      e.stopPropagation();
    } else if (onClick) {
      onClick();
    }
  };

  const cardContent = (
    <Card
      padding="$3.5"
      backgroundColor="white"
      borderWidth={1}
      borderColor="$gray6"
      pressStyle={{ scale: 0.98, borderColor: typeColor }}
      hoverStyle={{
        backgroundColor: typeBackground,
        borderColor: typeColor,
        shadowColor: typeColor,
        shadowOpacity: 0.15,
        shadowRadius: 8,
        shadowOffset: { width: 0, height: 2 }
      }}
      cursor={draggable ? 'grab' : 'pointer'}
      onPress={handleCardClick}
      animation="quick"
      borderRadius="$4"
    >
      <YStack gap="$2.5">
        <XStack justifyContent="space-between" alignItems="center" gap="$2">
          <XStack alignItems="center" gap="$2.5" flex={1}>
            <Icon size={20} color={typeColor} strokeWidth={2.5} style={{ flexShrink: 0 }} />
            <Text fontSize="$4" fontWeight="600" color="$gray12" flex={1} lineHeight="$1">
              {component.display_name}
            </Text>
          </XStack>
          {draggable && (
            isExpanded ? (
              <ChevronDown size={16} color="#64748b" strokeWidth={2.5} style={{ flexShrink: 0 }} />
            ) : (
              <ChevronRight size={16} color="#64748b" strokeWidth={2.5} style={{ flexShrink: 0 }} />
            )
          )}
        </XStack>

        {isExpanded && component.description && (
          <Text fontSize="$3" color="$gray11" lineHeight="$2">
            {component.description}
          </Text>
        )}

        {isExpanded && component.capabilities && component.capabilities.length > 0 && (
          <XStack gap="$1.5" flexWrap="wrap" marginTop="$1">
            {component.capabilities.map((cap: string) => (
              <Badge key={cap} size="1" backgroundColor="$gray4" borderWidth={1} borderColor="$gray6" borderRadius="$2">
                <Text fontSize="$2" color="$gray11" fontWeight="500">
                  {cap}
                </Text>
              </Badge>
            ))}
          </XStack>
        )}

        {isExpanded && (
          <XStack justifyContent="space-between" alignItems="center" marginTop="$1">
            <Text fontSize="$2" color="$gray10" fontFamily="$mono">
              v{component.version}
            </Text>
            {component.deprecated && (
              <Badge backgroundColor="$red4" size="1" borderWidth={1} borderColor="$red7">
                <Text fontSize="$2" color="$red11" fontWeight="600">
                  deprecated
                </Text>
              </Badge>
            )}
          </XStack>
        )}
      </YStack>
    </Card>
  );

  // Wrap in native div for HTML5 drag-and-drop support
  if (draggable && onDragStart) {
    return (
      <div
        draggable
        onDragStart={onDragStart}
        style={{ cursor: 'grab' }}
      >
        {cardContent}
      </div>
    );
  }

  return cardContent;
}

