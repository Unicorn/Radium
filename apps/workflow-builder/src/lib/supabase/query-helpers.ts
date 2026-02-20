/**
 * Supabase Query Helpers
 * 
 * Helper functions to work around TypeScript inference limitations
 * with complex Supabase queries (joins, aliases, nested selects)
 */

import type { SupabaseClient } from '@supabase/supabase-js';
import type { Database } from '@/types/database';

/**
 * Type-safe wrapper for Supabase queries that TypeScript can't infer
 * Use this when you have complex select statements with joins/aliases
 */
export function typedQuery<T>(
  query: any
): Promise<{ data: T | null; error: any }> {
  return query as any;
}

/**
 * Helper to cast supabase client for complex operations
 * Use when TypeScript can't infer update/insert operations
 * 
 * This is a workaround for Supabase's TypeScript inference limitations
 * with complex queries (joins, aliases, nested selects)
 */
export function castSupabaseClient<T = any>(
  client: SupabaseClient<Database> | any
): any {
  return client as any;
}

/**
 * Type assertion helper for query results
 */
export function assertType<T>(value: any): T {
  return value as T;
}

