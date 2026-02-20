/**
 * Component Categorization - Shared utility for categorizing components
 * Groups components by utility (what they do) rather than technical type
 */

import { 
  Bot, 
  Network, 
  Globe, 
  Download, 
  Upload, 
  GitBranch,
  Package,
} from 'lucide-react';
import type { Database } from '@/types/database';

type Component = Database['public']['Tables']['components']['Row'] & {
  component_type: { name: string; icon: string | null };
  visibility: { name: string };
};

// Utility categories based on what components DO, not what they ARE
import type { LucideIcon } from 'lucide-react';

export interface UtilityCategory {
  id: string;
  name: string;
  icon: LucideIcon;
  color: string;
  description: string;
  keywords: string[]; // Keywords to match components to this category
}

export const UTILITY_CATEGORIES: UtilityCategory[] = [
  {
    id: 'core-actions',
    name: 'Core Actions',
    icon: Package,
    color: '#3b82f6', // blue
    description: 'Basic operations like sending notifications, saving data, fetching APIs',
    keywords: ['notification', 'send', 'email', 'slack', 'save', 'database', 'postgresql', 'redis', 'fetch', 'api', 'http', 'process', 'data', 'transform'],
  },
  {
    id: 'ai-automation',
    name: 'AI & Automation',
    icon: Bot,
    color: '#a855f7', // purple
    description: 'AI agents, decision making, content generation',
    keywords: ['agent', 'ai', 'claude', 'anthropic', 'decision', 'generate', 'content', 'analyze', 'intelligent'],
  },
  {
    id: 'connect-services',
    name: 'Connect to Services',
    icon: Network,
    color: '#10b981', // green
    description: 'Call other services, receive from services',
    keywords: ['service', 'child-workflow', 'start-child', 'call', 'invoke', 'service-to-service'],
  },
  {
    id: 'connect-external',
    name: 'Connect to External',
    icon: Globe,
    color: '#f59e0b', // orange
    description: 'External APIs, webhooks, third-party services',
    keywords: ['external', 'webhook', 'third-party', 'integration', 'connector', 'kong', 'logging', 'cache', 'cors', 'gateway', 'api-gateway'],
  },
  {
    id: 'receive-data',
    name: 'Receive Data',
    icon: Download,
    color: '#06b6d4', // cyan
    description: 'API endpoints, webhook receivers',
    keywords: ['receive', 'endpoint', 'api-endpoint', 'webhook', 'post', 'input', 'trigger'],
  },
  {
    id: 'provide-data',
    name: 'Provide Data',
    icon: Upload,
    color: '#14b8a6', // teal
    description: 'API queries, state queries',
    keywords: ['query', 'get', 'state', 'provide', 'read', 'retrieve'],
  },
  {
    id: 'control-flow',
    name: 'Control Flow',
    icon: GitBranch,
    color: '#8b5cf6', // violet
    description: 'Conditions, loops, retries',
    keywords: ['condition', 'if', 'loop', 'retry', 'repeat', 'while', 'for', 'branch'],
  },
];

/**
 * Get primary category from component's database category mappings
 * Falls back to keyword-based categorization if no database mapping exists
 */
export function categorizeComponent(component: Component & {
  category_mappings?: Array<{
    is_primary: boolean;
    category?: {
      id: string;
      name: string;
      display_name: string;
    };
  }>;
}): string {
  // First, try to get primary category from database mappings
  if (component.category_mappings && component.category_mappings.length > 0) {
    const primaryMapping = component.category_mappings.find(m => m.is_primary);
    if (primaryMapping?.category?.name) {
      return primaryMapping.category.name;
    }
    // If no primary, use first category
    const firstMapping = component.category_mappings[0];
    if (firstMapping?.category?.name) {
      return firstMapping.category.name;
    }
  }

  // Fallback to keyword-based categorization (backward compatibility)
  return categorizeComponentByKeywords(component);
}

/**
 * Categorize a component into a utility category based on its name, description, capabilities, and tags
 * This is the fallback method when database categories are not available
 */
function categorizeComponentByKeywords(component: Component): string {
  const searchText = [
    component.name,
    component.display_name,
    component.description || '',
    ...(component.capabilities || []),
    ...(component.tags || []),
    component.component_type.name,
  ].join(' ').toLowerCase();

  // Special handling for specific component types (check these first)
  if (component.component_type.name === 'agent') {
    return 'ai-automation';
  }
  if (component.component_type.name === 'data-in') {
    return 'receive-data';
  }
  if (component.component_type.name === 'data-out') {
    return 'provide-data';
  }
  if (component.component_type.name === 'kong-logging' || component.component_type.name === 'kong-cache' || component.component_type.name === 'kong-cors' || component.component_type.name === 'graphql-gateway' || component.component_type.name === 'mcp-server') {
    return 'connect-external';
  }
  if (component.component_type.name === 'signal' || component.name.includes('signal')) {
    return 'connect-services';
  }
  if (component.component_type.name === 'query' || component.name.includes('query')) {
    return 'provide-data';
  }
  if (component.component_type.name === 'trigger' || component.name.includes('trigger') || component.name.includes('endpoint')) {
    return 'receive-data';
  }
  if (component.component_type.name === 'condition' || component.name.includes('condition')) {
    return 'control-flow';
  }
  if (component.component_type.name === 'retry' || component.name.includes('retry')) {
    return 'control-flow';
  }

  // Find the category with the most matching keywords
  // Safe to use non-null assertion since UTILITY_CATEGORIES is a constant array with items
  let bestMatch: UtilityCategory = UTILITY_CATEGORIES[0]!; // Default to first category
  let maxMatches = 0;

  for (const category of UTILITY_CATEGORIES) {
    const matches = category.keywords.filter(keyword => 
      searchText.includes(keyword.toLowerCase())
    ).length;
    
    if (matches > maxMatches) {
      maxMatches = matches;
      bestMatch = category;
    }
  }

  return bestMatch.id;
}

