/**
 * Supabase Type Helpers
 * 
 * Helper types for better TypeScript inference with Supabase queries
 */

import type { Database } from '@/types/database';

// Extract table row types
export type TableRow<T extends keyof Database['public']['Tables']> = 
  Database['public']['Tables'][T]['Row'];

// Extract table insert types
export type TableInsert<T extends keyof Database['public']['Tables']> = 
  Database['public']['Tables'][T]['Insert'];

// Extract table update types
export type TableUpdate<T extends keyof Database['public']['Tables']> = 
  Database['public']['Tables'][T]['Update'];

// Helper type for queries with joins/aliases that TypeScript can't infer
// Use this when you have complex select statements
export type QueryResult<T> = T extends Array<infer U> ? U[] : T;

// Common query result types
export type ProjectRow = TableRow<'projects'>;
export type WorkflowRow = TableRow<'workflows'>;
export type ComponentRow = TableRow<'components'>;
export type UserRow = TableRow<'users'>;
export type ExecutionRow = TableRow<'workflow_executions'>;

// Extended types for queries with joins
export interface ProjectWithRelations {
  id: string;
  name: string;
  description: string | null;
  created_by: string;
  task_queue_name: string;
  is_active: boolean;
  is_default: boolean;
  is_archived: boolean;
  created_at: string;
  updated_at: string;
  workflow_count?: Array<{ count: number }>;
  active_workers?: Array<{
    id: string;
    status: string;
    last_heartbeat: string | null;
  }>;
  workflows?: Array<{
    id: string;
    name: string;
    display_name: string;
    status_id: string;
  }>;
  workers?: Array<{
    id: string;
    worker_id: string;
    status: string;
    last_heartbeat: string | null;
    started_at: string | null;
  }>;
}

export interface WorkflowWithRelations extends WorkflowRow {
  project?: ProjectRow;
  workflow_nodes?: Array<TableRow<'workflow_nodes'>>;
  workflow_edges?: Array<TableRow<'workflow_edges'>>;
}

export interface ComponentWithRelations extends ComponentRow {
  component_type?: {
    id: string;
    name: string;
    icon: string | null;
  };
  visibility?: {
    id: string;
    name: string;
  };
  created_by_user?: {
    id: string;
    display_name: string | null;
  };
}

