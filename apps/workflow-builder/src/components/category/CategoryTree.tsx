/**
 * CategoryTree - Hierarchical category display component
 * Supports both expandable/collapsible sections and different interaction modes
 */

'use client';

import { useState, useMemo } from 'react';
import { YStack, XStack, Text } from 'tamagui';
import { ChevronDown, ChevronRight } from 'lucide-react';
import * as LucideIcons from 'lucide-react';

export interface CategoryNode {
  id: string;
  name: string;
  display_name: string;
  description?: string;
  icon?: string;
  icon_provider?: string;
  color?: string;
  parent_category_id?: string | null;
  sort_order?: number;
  children?: CategoryNode[];
}

export interface CategoryTreeProps {
  categories: CategoryNode[];
  itemsByCategory: Record<string, any[]>;
  renderItem: (item: any, categoryId: string) => React.ReactNode;
  onCategoryToggle?: (categoryId: string, isExpanded: boolean) => void;
  defaultExpanded?: boolean | string[]; // true = all, false = none, array = specific IDs
  showEmptyCategories?: boolean;
  indentLevel?: number;
  maxDepth?: number;
}

export function CategoryTree({
  categories,
  itemsByCategory,
  renderItem,
  onCategoryToggle,
  defaultExpanded = false,
  showEmptyCategories = false,
  indentLevel = 0,
  maxDepth = 3,
}: CategoryTreeProps) {
  const [expandedCategories, setExpandedCategories] = useState<Set<string>>(() => {
    if (defaultExpanded === true) {
      return new Set(categories.map(c => c.id));
    }
    if (Array.isArray(defaultExpanded)) {
      return new Set(defaultExpanded);
    }
    return new Set();
  });

  const toggleCategory = (categoryId: string) => {
    setExpandedCategories(prev => {
      const newSet = new Set(prev);
      if (newSet.has(categoryId)) {
        newSet.delete(categoryId);
        onCategoryToggle?.(categoryId, false);
      } else {
        newSet.add(categoryId);
        onCategoryToggle?.(categoryId, true);
      }
      return newSet;
    });
  };

  // Build tree structure from flat categories
  const categoryTree = useMemo(() => {
    const categoryMap = new Map<string, CategoryNode & { children: CategoryNode[] }>();
    const rootCategories: (CategoryNode & { children: CategoryNode[] })[] = [];

    // First pass: create map with children arrays
    categories.forEach(cat => {
      categoryMap.set(cat.id, { ...cat, children: [] });
    });

    // Second pass: build tree
    categories.forEach(cat => {
      const category = categoryMap.get(cat.id)!;
      if (cat.parent_category_id) {
        const parent = categoryMap.get(cat.parent_category_id);
        if (parent) {
          parent.children.push(category);
        } else {
          // Orphaned category, add to root
          rootCategories.push(category);
        }
      } else {
        rootCategories.push(category);
      }
    });

    // Sort categories by sort_order
    const sortCategories = (cats: (CategoryNode & { children: CategoryNode[] })[]) => {
      return cats.sort((a, b) => (a.sort_order || 0) - (b.sort_order || 0));
    };

    const sortRecursive = (cat: CategoryNode & { children: CategoryNode[] }) => {
      cat.children = sortCategories(cat.children);
      cat.children.forEach(sortRecursive);
    };

    rootCategories.forEach(sortRecursive);
    return sortCategories(rootCategories);
  }, [categories]);

  const renderCategory = (
    category: CategoryNode & { children: CategoryNode[] },
    level: number = 0
  ): React.ReactNode => {
    if (level > maxDepth) return null;

    const categoryId = category.id;
    const categoryName = category.name;
    const items = itemsByCategory[categoryName] || [];
    const hasItems = items.length > 0;
    const hasChildren = category.children.length > 0;
    const isExpanded = expandedCategories.has(categoryId);
    const shouldShow = hasItems || hasChildren || showEmptyCategories;

    if (!shouldShow) return null;

    // Get icon from lucide-react
    const IconName = category.icon as keyof typeof LucideIcons;
    const Icon = (IconName && LucideIcons[IconName] as typeof LucideIcons.Activity) || LucideIcons.Package;
    const categoryColor = category.color || '#3b82f6';

    return (
      <YStack key={categoryId} gap="$2" paddingLeft={level * 16}>
        {/* Category Header */}
        <XStack
          paddingHorizontal="$3.5"
          paddingVertical="$3"
          alignItems="center"
          gap="$3"
          cursor="pointer"
          backgroundColor="white"
          hoverStyle={{
            backgroundColor: '$gray3',
            borderColor: '$gray8',
          }}
          pressStyle={{ backgroundColor: '$gray4' }}
          borderRadius="$4"
          borderWidth={1}
          borderColor="$gray6"
          onPress={() => toggleCategory(categoryId)}
          shadowColor="$gray8"
          shadowOpacity={0.05}
          shadowRadius={4}
          shadowOffset={{ width: 0, height: 1 }}
        >
          {(hasChildren || hasItems) && (
            isExpanded ? (
              <ChevronDown size={16} color="#64748b" strokeWidth={2.5} />
            ) : (
              <ChevronRight size={16} color="#64748b" strokeWidth={2.5} />
            )
          )}
          {!hasChildren && !hasItems && <XStack width={16} />}
          <Icon size={20} color={categoryColor} strokeWidth={2.5} />
          <YStack flex={1} gap="$0.5">
            <Text
              fontSize="$4"
              fontWeight="700"
              color={categoryColor}
              letterSpacing={-0.3}
            >
              {category.display_name}
            </Text>
            {category.description && (
              <Text
                fontSize="$2"
                color="$gray10"
                lineHeight="$1"
              >
                {category.description}
              </Text>
            )}
          </YStack>
          {hasItems && (
            <XStack
              backgroundColor="$gray4"
              borderWidth={1}
              borderColor="$gray7"
              paddingHorizontal="$2.5"
              paddingVertical="$1.5"
              borderRadius="$3"
            >
              <Text fontSize="$2" fontWeight="700" color="$gray11">
                {items.length}
              </Text>
            </XStack>
          )}
        </XStack>

        {/* Category Items */}
        {isExpanded && hasItems && (
          <YStack gap="$2" paddingLeft="$2">
            {items.map((item, index) => (
              <YStack key={item.id || index}>
                {renderItem(item, categoryName)}
              </YStack>
            ))}
          </YStack>
        )}

        {/* Child Categories */}
        {isExpanded && hasChildren && (
          <YStack gap="$2" paddingLeft="$2">
            {category.children.map(child => renderCategory(child, level + 1))}
          </YStack>
        )}
      </YStack>
    );
  };

  return (
    <YStack gap="$3.5">
      {categoryTree.map(category => renderCategory(category, indentLevel))}
    </YStack>
  );
}

