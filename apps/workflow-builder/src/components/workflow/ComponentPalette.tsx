/**
 * Component Palette - Drag source for workflow components
 * Groups components by utility (what they do) rather than technical type
 */

'use client';

import { useState, useEffect } from 'react';
import { ScrollView, YStack, XStack, Text, Separator } from 'tamagui';
import { ComponentCard } from '../component/ComponentCard';
import { api } from '@/lib/trpc/client';
import { 
  ChevronDown, 
  ChevronRight, 
  Box, 
  Wrench
} from 'lucide-react';
import type { Database } from '@/types/database';
import { categorizeComponent } from '@/lib/component-categorization';
import { useComponentCategories, getPrimaryCategory } from '@/hooks/useComponentCategories';
import * as LucideIcons from 'lucide-react';

type Component = Database['public']['Tables']['components']['Row'] & {
  component_type: { name: string; icon: string | null };
  visibility: { name: string };
};

interface ComponentPaletteProps {
  components: Component[];
  disabled?: boolean;
}

export function ComponentPalette({ components, disabled = false }: ComponentPaletteProps) {
  // Track which sections are expanded
  const [expandedSections, setExpandedSections] = useState<Record<string, boolean>>({});
  const { categoriesFlat, categoryMap, isLoading: categoriesLoading } = useComponentCategories();

  // Optionally discover component files from file system (if workspace path is available)
  // This allows discovering components that aren't yet registered in the database
  const workspacePath = process.env.NEXT_PUBLIC_WORKSPACE_PATH;
  const { data: discoveredFiles } = api.fileOperations.findFiles.useQuery(
    {
      directory: workspacePath || './components',
      pattern: '**/*.{ts,tsx}',
      excludeDirs: ['node_modules', '.git', 'dist', '.next'],
      maxDepth: 3,
    },
    {
      enabled: !!workspacePath && !disabled,
      refetchOnWindowFocus: false,
    }
  );

  // Group by category (use database categories if available, fallback to keyword-based)
  const groupedComponents = components.reduce((acc, comp) => {
    const categoryId = categorizeComponent(comp);
    if (!acc[categoryId]) acc[categoryId] = [];
    acc[categoryId].push(comp);
    return acc;
  }, {} as Record<string, Component[]>);

  // Initialize all sections as collapsed
  if (Object.keys(expandedSections).length === 0 && Object.keys(groupedComponents).length > 0) {
    const initialExpanded: Record<string, boolean> = {};
    Object.keys(groupedComponents).forEach(categoryId => {
      initialExpanded[categoryId] = false; // Start with all collapsed
    });
    setExpandedSections(initialExpanded);
  }

  const toggleSection = (categoryId: string) => {
    setExpandedSections(prev => ({
      ...prev,
      [categoryId]: !prev[categoryId],
    }));
  };

  // Get active categories - use database categories if available, otherwise use fallback
  const activeCategories = categoriesLoading 
    ? [] 
    : categoriesFlat
        .filter(cat => {
          const categoryName = cat.name;
          return groupedComponents[categoryName] && groupedComponents[categoryName].length > 0;
        })
        .sort((a, b) => (a.sort_order || 0) - (b.sort_order || 0))
        .map(cat => {
          // Get icon from lucide-react
          const IconName = cat.icon as keyof typeof LucideIcons;
          const Icon = (IconName && LucideIcons[IconName] as typeof LucideIcons.Activity) || LucideIcons.Package;
          
          return {
            id: cat.name,
            name: cat.display_name,
            icon: Icon,
            color: cat.color || '#3b82f6',
            description: cat.description || '',
          };
        });

  return (
    <YStack
      width={300}
      backgroundColor="$gray1"
      borderRightWidth={1}
      borderColor="$gray6"
      opacity={disabled ? 0.5 : 1}
      pointerEvents={disabled ? 'none' : 'auto'}
    >
      <YStack padding="$4" paddingBottom="$3" borderBottomWidth={1} borderColor="$gray6" backgroundColor="white">
        <XStack alignItems="center" gap="$2">
          <Wrench size={20} color="#6366f1" />
          <Text fontSize="$7" fontWeight="700" color="$gray12" letterSpacing={-0.5}>
            Build Your Service
          </Text>
        </XStack>
        <Text fontSize="$3" color={disabled ? "$red10" : "$gray11"} marginTop="$1.5" lineHeight="$2">
          {disabled ? 'Editing disabled (workflow is active)' : 'Drag components to canvas'}
        </Text>
      </YStack>

      <ScrollView flex={1}>
        <YStack padding="$3" gap="$3.5">
          {activeCategories.map((category) => {
            const comps = groupedComponents[category.id] || [];
            const Icon = category.icon;
            const categoryColor = category.color;
            const isExpanded = expandedSections[category.id] ?? false;

            return (
              <YStack key={category.id} gap="$2.5">
                <XStack
                  paddingHorizontal="$3.5"
                  paddingVertical="$3"
                  alignItems="center"
                  gap="$3"
                  cursor="pointer"
                  backgroundColor="white"
                  hoverStyle={{
                    backgroundColor: '$gray3',
                    borderColor: '$gray8'
                  }}
                  pressStyle={{ backgroundColor: '$gray4' }}
                  borderRadius="$4"
                  borderWidth={1}
                  borderColor="$gray6"
                  onPress={() => toggleSection(category.id)}
                  shadowColor="$gray8"
                  shadowOpacity={0.05}
                  shadowRadius={4}
                  shadowOffset={{ width: 0, height: 1 }}
                >
                  {isExpanded ? (
                    <ChevronDown size={16} color="#64748b" strokeWidth={2.5} />
                  ) : (
                    <ChevronRight size={16} color="#64748b" strokeWidth={2.5} />
                  )}
                  <Icon size={20} color={categoryColor} strokeWidth={2.5} />
                  <YStack flex={1} gap="$0.5">
                    <Text
                      fontSize="$4"
                      fontWeight="700"
                      color={categoryColor}
                      letterSpacing={-0.3}
                    >
                      {category.name}
                    </Text>
                    <Text
                      fontSize="$2"
                      color="$gray10"
                      lineHeight="$1"
                    >
                      {category.description}
                    </Text>
                  </YStack>
                  <XStack
                    backgroundColor="$gray4"
                    borderWidth={1}
                    borderColor="$gray7"
                    paddingHorizontal="$2.5"
                    paddingVertical="$1.5"
                    borderRadius="$3"
                  >
                    <Text fontSize="$2" fontWeight="700" color="$gray11">
                      {comps.length}
                    </Text>
                  </XStack>
                </XStack>

                {isExpanded && comps.map((component) => (
                  <ComponentCard
                    key={component.id}
                    component={component}
                    draggable
                    onDragStart={(e) => {
                      e.dataTransfer.setData(
                        'application/json',
                        JSON.stringify(component)
                      );
                      e.dataTransfer.effectAllowed = 'move';
                    }}
                  />
                ))}

                {category.id !== activeCategories[activeCategories.length - 1].id && (
                  <Separator marginVertical="$2" />
                )}
              </YStack>
            );
          })}

          {components.length === 0 && (
            <YStack padding="$4" alignItems="center">
              <Text fontSize="$3" color="$gray11" textAlign="center">
                No components available. Create components first.
              </Text>
            </YStack>
          )}
        </YStack>
      </ScrollView>
    </YStack>
  );
}

