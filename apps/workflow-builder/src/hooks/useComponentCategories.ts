/**
 * React hooks for component categories
 */

import { api } from '@/lib/trpc/client';
import { useMemo } from 'react';

/**
 * Hook to fetch and use component categories from database
 */
export function useComponentCategories() {
  const { data, isLoading, error } = api.components.getCategories.useQuery();

  const categories = useMemo(() => {
    if (!data) return { tree: [], flat: [], map: new Map() };
    
    // Create a map for quick lookups
    const categoryMap = new Map<string, any>();
    data.flat.forEach(cat => {
      categoryMap.set(cat.id, cat);
      categoryMap.set(cat.name, cat);
    });

    return {
      tree: data.categories || [],
      flat: data.flat || [],
      map: categoryMap,
    };
  }, [data]);

  return {
    categories: categories.tree,
    categoriesFlat: categories.flat,
    categoryMap: categories.map,
    isLoading,
    error,
  };
}

/**
 * Hook to get category tree structure
 */
export function useCategoryTree() {
  const { data, isLoading, error } = api.components.getCategoryTree.useQuery();

  return {
    tree: data || [],
    isLoading,
    error,
  };
}

/**
 * Get primary category for a component
 */
export function getPrimaryCategory(component: {
  category_mappings?: Array<{
    is_primary: boolean;
    category?: {
      id: string;
      name: string;
      display_name: string;
      icon?: string;
      color?: string;
    };
  }>;
}): { name: string; display_name: string; icon?: string; color?: string } | null {
  if (!component.category_mappings || component.category_mappings.length === 0) {
    return null;
  }

  const primaryMapping = component.category_mappings.find(m => m.is_primary);
  if (primaryMapping?.category) {
    return {
      name: primaryMapping.category.name,
      display_name: primaryMapping.category.display_name,
      icon: primaryMapping.category.icon,
      color: primaryMapping.category.color,
    };
  }

  // Fallback to first category
  const firstMapping = component.category_mappings[0];
  if (firstMapping?.category) {
    return {
      name: firstMapping.category.name,
      display_name: firstMapping.category.display_name,
      icon: firstMapping.category.icon,
      color: firstMapping.category.color,
    };
  }

  return null;
}

