/**
 * Component List - Grid of component cards organized by utility category
 */

'use client';

import { YStack, XStack, Text, Spinner, Button, Input, Card, Separator, ScrollView } from 'tamagui';
import { useState, useEffect, useRef } from 'react';
import { ComponentCard } from './ComponentCard';
import { useRouter } from 'next/navigation';
import { api } from '@/lib/trpc/client';
import { ChevronDown, ChevronRight } from 'lucide-react';
import { categorizeComponent } from '@/lib/component-categorization';
import { useComponentCategories } from '@/hooks/useComponentCategories';
import * as LucideIcons from 'lucide-react';
import type { Database } from '@/types/database';

type Component = Database['public']['Tables']['components']['Row'] & {
  component_type: { name: string; icon: string | null };
  visibility: { name: string };
};

interface ComponentListProps {
  type?: string;
  capability?: string;
  onComponentClick?: (componentId: string) => void;
}

export function ComponentList({ 
  type, 
  capability,
  onComponentClick 
}: ComponentListProps) {
  const router = useRouter();
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedType, setSelectedType] = useState(type || '');
  
  // ALL HOOKS MUST BE CALLED BEFORE ANY CONDITIONAL RETURNS
  const queryResult = api.components.list.useQuery({
    type: selectedType || undefined,
    capability,
    includeDeprecated: false,
  });

  const { data, isLoading, error } = queryResult;
  const { data: types } = api.components.getTypes.useQuery();
  
  // Track which sections are expanded - initialize with all categories expanded
  const [expandedSections, setExpandedSections] = useState<Record<string, boolean>>({});
  const initializedRef = useRef(false);

  // Filter components by search and type (computed before conditional returns)
  const filteredComponents = (data?.components || []).filter((c) => {
    // Type filter
    if (selectedType && c.component_type.name !== selectedType) {
      return false;
    }
    
    // Search filter
    if (!searchQuery) return true;
    const query = searchQuery.toLowerCase();
    return (
      c.display_name.toLowerCase().includes(query) ||
      c.name.toLowerCase().includes(query) ||
      c.description?.toLowerCase().includes(query)
    );
  }) as Component[];

  // Group by utility category
  const groupedComponents = filteredComponents.reduce((acc, comp) => {
    const categoryId = categorizeComponent(comp);
    if (!acc[categoryId]) acc[categoryId] = [];
    acc[categoryId].push(comp);
    return acc;
  }, {} as Record<string, Component[]>);

  // Get categories that have components, in the order defined in UTILITY_CATEGORIES
  const activeCategories = UTILITY_CATEGORIES.filter(cat => 
    groupedComponents[cat.id] && groupedComponents[cat.id].length > 0
  );

  // Initialize expanded sections when categories are first computed
  useEffect(() => {
    if (activeCategories.length > 0 && !initializedRef.current) {
      const initial: Record<string, boolean> = {};
      activeCategories.forEach(cat => {
        initial[cat.id] = true; // Start with all expanded
      });
      setExpandedSections(initial);
      initializedRef.current = true;
    }
  }, [activeCategories]);

  // NOW we can do conditional returns
  if (isLoading) {
    return (
      <YStack flex={1} alignItems="center" justifyContent="center" padding="$6">
        <Spinner size="large" />
        <Text marginTop="$4">Loading components...</Text>
      </YStack>
    );
  }

  if (error) {
    return (
      <YStack flex={1} alignItems="center" justifyContent="center" padding="$6">
        <Text color="$red10">Error loading components: {error.message}</Text>
      </YStack>
    );
  }

  const toggleSection = (categoryId: string) => {
    setExpandedSections(prev => ({
      ...prev,
      [categoryId]: !prev[categoryId],
    }));
  };

  const totalComponents = filteredComponents.length;

  return (
    <YStack flex={1} gap="$4">
      {/* Filters */}
      <XStack gap="$3" flexWrap="wrap" alignItems="center">
        <Input
          flex={1}
          minWidth={200}
          size="$4"
          placeholder="Search components..."
          value={searchQuery}
          onChangeText={setSearchQuery}
        />

        <XStack gap="$2" flexWrap="wrap">
          <Button
            size="$3"
            theme={selectedType === '' ? 'blue' : undefined}
            onPress={() => setSelectedType('')}
          >
            All
          </Button>
          {types?.map((t) => (
            <Button
              key={t.id}
              size="$3"
              theme={selectedType === t.name ? 'blue' : undefined}
              onPress={() => setSelectedType(t.name)}
            >
              {t.name}
            </Button>
          ))}
        </XStack>
      </XStack>

      {/* Results count */}
      <Text fontSize="$3" color="$gray11">
        {totalComponents} component{totalComponents !== 1 ? 's' : ''} found
      </Text>

      {/* Component categories */}
      {totalComponents === 0 ? (
        <YStack padding="$6" alignItems="center">
          <Text color="$gray11">No components found</Text>
          <Button
            marginTop="$4"
            onPress={() => router.push('/components/new')}
          >
            Create Component
          </Button>
        </YStack>
      ) : (
        <ScrollView flex={1}>
          <YStack gap="$4">
            {activeCategories.map((category) => {
              const comps = groupedComponents[category.id] || [];
              const Icon = category.icon;
              const categoryColor = category.color;
              const isExpanded = expandedSections[category.id] ?? true;

              return (
                <Card key={category.id} padding="$4" elevate>
                  <YStack gap="$3">
                    {/* Category Header */}
                    <XStack
                      alignItems="center"
                      gap="$3"
                      cursor="pointer"
                      onPress={() => toggleSection(category.id)}
                    >
                      {isExpanded ? (
                        <ChevronDown size={18} color="#64748b" />
                      ) : (
                        <ChevronRight size={18} color="#64748b" />
                      )}
                      <Icon size={20} color={categoryColor} />
                      <YStack flex={1} gap="$1">
                        <Text
                          fontSize="$5"
                          fontWeight="700"
                          color={categoryColor}
                        >
                          {category.name}
                        </Text>
                        <Text
                          fontSize="$3"
                          color="$gray11"
                        >
                          {category.description}
                        </Text>
                      </YStack>
                      <XStack
                        backgroundColor="$gray4"
                        paddingHorizontal="$3"
                        paddingVertical="$1.5"
                        borderRadius="$3"
                      >
                        <Text fontSize="$3" fontWeight="700" color="$gray11">
                          {comps.length}
                        </Text>
                      </XStack>
                    </XStack>

                    {/* Components in this category */}
                    {isExpanded && (
                      <YStack gap="$3" paddingLeft="$8">
                        {comps.map((component) => (
                          <ComponentCard
                            key={component.id}
                            component={component as any}
                            onClick={() => {
                              if (onComponentClick) {
                                onComponentClick(component.id);
                              } else {
                                router.push(`/components/${component.id}`);
                              }
                            }}
                          />
                        ))}
                      </YStack>
                    )}
                  </YStack>
                </Card>
              );
            })}
          </YStack>
        </ScrollView>
      )}

      {/* Pagination */}
      {data && data.totalPages > 1 && (
        <XStack justifyContent="center" gap="$2" marginTop="$4">
          <Text fontSize="$3" color="$gray11">
            Page {data.page} of {data.totalPages}
          </Text>
          {/* TODO: Add pagination controls */}
        </XStack>
      )}
    </YStack>
  );
}

