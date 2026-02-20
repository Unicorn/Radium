/**
 * ExecutionService - Manages workflow execution tracking
 *
 * This service provides a high-level API for creating and managing workflow executions.
 * It works with the Supabase workflow_executions table and provides type-safe
 * operations for tracking workflow runs.
 */

import { createClient } from '@supabase/supabase-js';
import type { Database } from '@/types/database';
import type { SupabaseClient } from '@supabase/supabase-js';

// Type aliases for cleaner code
type WorkflowExecution = Database['public']['Tables']['workflow_executions']['Row'];
type WorkflowExecutionInsert = Database['public']['Tables']['workflow_executions']['Insert'];
type WorkflowExecutionUpdate = Database['public']['Tables']['workflow_executions']['Update'];

/**
 * Input for creating a new workflow execution
 */
export interface CreateExecutionInput {
  workflowId: string;
  temporalWorkflowId: string;
  temporalRunId: string;
  input?: Record<string, any>;
  userId: string;
  taskQueue?: string;
}

/**
 * Input for updating an execution
 */
export interface UpdateExecutionInput {
  status?: 'running' | 'completed' | 'failed' | 'cancelled' | 'timed_out' | 'building';
  output?: Record<string, any>;
  error?: string;
  completedAt?: Date;
  durationMs?: number;
  activitiesExecuted?: number;
}

/**
 * Execution with related workflow information
 */
export interface ExecutionWithWorkflow extends WorkflowExecution {
  workflow?: {
    id: string;
    name: string;
    display_name: string;
    description: string | null;
  };
}

/**
 * Execution statistics for a workflow
 */
export interface ExecutionStats {
  total: number;
  running: number;
  completed: number;
  failed: number;
  cancelled: number;
  timedOut: number;
  avgDuration: number;
  minDuration: number | null;
  maxDuration: number | null;
  successRate: number;
}

/**
 * ExecutionService provides methods for managing workflow executions
 */
export class ExecutionService {
  private supabase: SupabaseClient<Database>;

  constructor(supabase: SupabaseClient<Database>) {
    this.supabase = supabase;
  }

  /**
   * Create a new workflow execution record
   */
  async createExecution(data: CreateExecutionInput): Promise<WorkflowExecution> {
    const { data: execution, error } = await this.supabase
      .from('workflow_executions')
      .insert({
        workflow_id: data.workflowId,
        temporal_workflow_id: data.temporalWorkflowId,
        temporal_run_id: data.temporalRunId,
        input: data.input as any,
        status: 'running',
        started_at: new Date().toISOString(),
        created_by: data.userId,
      })
      .select()
      .single();

    if (error) {
      throw new Error(`Failed to create execution: ${error.message}`);
    }

    if (!execution) {
      throw new Error('Failed to create execution: No data returned');
    }

    return execution;
  }

  /**
   * Update an execution by Temporal workflow ID
   */
  async updateExecution(
    temporalWorkflowId: string,
    data: UpdateExecutionInput
  ): Promise<WorkflowExecution> {
    const updateData: WorkflowExecutionUpdate = {};

    if (data.status) {
      updateData.status = data.status;
    }
    if (data.output !== undefined) {
      updateData.output = data.output as any;
    }
    if (data.error !== undefined) {
      updateData.error_message = data.error;
    }
    if (data.completedAt) {
      updateData.completed_at = data.completedAt.toISOString();
    }
    if (data.durationMs !== undefined) {
      updateData.duration_ms = data.durationMs;
    }
    if (data.activitiesExecuted !== undefined) {
      updateData.activities_executed = data.activitiesExecuted;
    }

    const { data: execution, error } = await this.supabase
      .from('workflow_executions')
      .update(updateData)
      .eq('temporal_workflow_id', temporalWorkflowId)
      .select()
      .single();

    if (error) {
      throw new Error(`Failed to update execution: ${error.message}`);
    }

    if (!execution) {
      throw new Error('Failed to update execution: No data returned');
    }

    return execution;
  }

  /**
   * Update an execution by database ID
   */
  async updateExecutionById(
    executionId: string,
    data: UpdateExecutionInput
  ): Promise<WorkflowExecution> {
    const updateData: WorkflowExecutionUpdate = {};

    if (data.status) {
      updateData.status = data.status;
    }
    if (data.output !== undefined) {
      updateData.output = data.output as any;
    }
    if (data.error !== undefined) {
      updateData.error_message = data.error;
    }
    if (data.completedAt) {
      updateData.completed_at = data.completedAt.toISOString();
    }
    if (data.durationMs !== undefined) {
      updateData.duration_ms = data.durationMs;
    }
    if (data.activitiesExecuted !== undefined) {
      updateData.activities_executed = data.activitiesExecuted;
    }

    const { data: execution, error } = await this.supabase
      .from('workflow_executions')
      .update(updateData)
      .eq('id', executionId)
      .select()
      .single();

    if (error) {
      throw new Error(`Failed to update execution: ${error.message}`);
    }

    if (!execution) {
      throw new Error('Failed to update execution: No data returned');
    }

    return execution;
  }

  /**
   * Get an execution by Temporal workflow ID
   */
  async getExecution(temporalWorkflowId: string): Promise<ExecutionWithWorkflow | null> {
    const { data, error } = await this.supabase
      .from('workflow_executions')
      .select(`
        *,
        workflow:workflows!workflow_executions_workflow_id_fkey (
          id,
          name,
          display_name,
          description
        )
      `)
      .eq('temporal_workflow_id', temporalWorkflowId)
      .single();

    if (error) {
      if (error.code === 'PGRST116') {
        return null; // Not found
      }
      throw new Error(`Failed to get execution: ${error.message}`);
    }

    return data as ExecutionWithWorkflow;
  }

  /**
   * Get an execution by database ID
   */
  async getExecutionById(executionId: string): Promise<ExecutionWithWorkflow | null> {
    const { data, error } = await this.supabase
      .from('workflow_executions')
      .select(`
        *,
        workflow:workflows!workflow_executions_workflow_id_fkey (
          id,
          name,
          display_name,
          description
        )
      `)
      .eq('id', executionId)
      .single();

    if (error) {
      if (error.code === 'PGRST116') {
        return null; // Not found
      }
      throw new Error(`Failed to get execution: ${error.message}`);
    }

    return data as ExecutionWithWorkflow;
  }

  /**
   * List executions for a workflow
   */
  async listExecutions(
    workflowId: string,
    options: {
      limit?: number;
      offset?: number;
      status?: string;
      orderBy?: 'started_at' | 'completed_at';
      orderDirection?: 'asc' | 'desc';
    } = {}
  ): Promise<WorkflowExecution[]> {
    const {
      limit = 50,
      offset = 0,
      status,
      orderBy = 'started_at',
      orderDirection = 'desc',
    } = options;

    let query = this.supabase
      .from('workflow_executions')
      .select('*')
      .eq('workflow_id', workflowId)
      .order(orderBy, { ascending: orderDirection === 'asc' })
      .range(offset, offset + limit - 1);

    if (status) {
      query = query.eq('status', status);
    }

    const { data, error } = await query;

    if (error) {
      throw new Error(`Failed to list executions: ${error.message}`);
    }

    return data || [];
  }

  /**
   * List executions for a user
   */
  async listExecutionsByUser(
    userId: string,
    options: {
      limit?: number;
      offset?: number;
      status?: string;
    } = {}
  ): Promise<ExecutionWithWorkflow[]> {
    const { limit = 50, offset = 0, status } = options;

    let query = this.supabase
      .from('workflow_executions')
      .select(`
        *,
        workflow:workflows!workflow_executions_workflow_id_fkey (
          id,
          name,
          display_name,
          description
        )
      `)
      .eq('created_by', userId)
      .order('started_at', { ascending: false })
      .range(offset, offset + limit - 1);

    if (status) {
      query = query.eq('status', status);
    }

    const { data, error } = await query;

    if (error) {
      throw new Error(`Failed to list user executions: ${error.message}`);
    }

    return (data || []) as ExecutionWithWorkflow[];
  }

  /**
   * Get execution statistics for a workflow
   */
  async getStats(workflowId: string): Promise<ExecutionStats> {
    const { data: executions, error } = await this.supabase
      .from('workflow_executions')
      .select('status, duration_ms')
      .eq('workflow_id', workflowId);

    if (error) {
      throw new Error(`Failed to get execution stats: ${error.message}`);
    }

    if (!executions || executions.length === 0) {
      return {
        total: 0,
        running: 0,
        completed: 0,
        failed: 0,
        cancelled: 0,
        timedOut: 0,
        avgDuration: 0,
        minDuration: null,
        maxDuration: null,
        successRate: 0,
      };
    }

    const total = executions.length;
    const running = executions.filter((e) => e.status === 'running').length;
    const completed = executions.filter((e) => e.status === 'completed').length;
    const failed = executions.filter((e) => e.status === 'failed').length;
    const cancelled = executions.filter((e) => e.status === 'cancelled').length;
    const timedOut = executions.filter((e) => e.status === 'timed_out').length;

    const durations = executions
      .filter((e) => e.duration_ms !== null && e.duration_ms !== undefined)
      .map((e) => e.duration_ms!);

    const avgDuration =
      durations.length > 0
        ? durations.reduce((sum, d) => sum + d, 0) / durations.length
        : 0;

    const minDuration = durations.length > 0 ? Math.min(...durations) : null;
    const maxDuration = durations.length > 0 ? Math.max(...durations) : null;

    const successRate = total > 0 ? (completed / total) * 100 : 0;

    return {
      total,
      running,
      completed,
      failed,
      cancelled,
      timedOut,
      avgDuration: Math.round(avgDuration),
      minDuration,
      maxDuration,
      successRate: Math.round(successRate * 100) / 100,
    };
  }

  /**
   * Mark an execution as completed
   */
  async markCompleted(
    temporalWorkflowId: string,
    output?: Record<string, any>,
    durationMs?: number
  ): Promise<WorkflowExecution> {
    return this.updateExecution(temporalWorkflowId, {
      status: 'completed',
      output,
      completedAt: new Date(),
      durationMs,
    });
  }

  /**
   * Mark an execution as failed
   */
  async markFailed(
    temporalWorkflowId: string,
    error: string,
    durationMs?: number
  ): Promise<WorkflowExecution> {
    return this.updateExecution(temporalWorkflowId, {
      status: 'failed',
      error,
      completedAt: new Date(),
      durationMs,
    });
  }

  /**
   * Mark an execution as cancelled
   */
  async markCancelled(
    temporalWorkflowId: string,
    durationMs?: number
  ): Promise<WorkflowExecution> {
    return this.updateExecution(temporalWorkflowId, {
      status: 'cancelled',
      completedAt: new Date(),
      durationMs,
    });
  }

  /**
   * Mark an execution as timed out
   */
  async markTimedOut(
    temporalWorkflowId: string,
    durationMs?: number
  ): Promise<WorkflowExecution> {
    return this.updateExecution(temporalWorkflowId, {
      status: 'timed_out',
      completedAt: new Date(),
      durationMs,
    });
  }

  /**
   * Delete an execution
   */
  async deleteExecution(executionId: string): Promise<void> {
    const { error } = await this.supabase
      .from('workflow_executions')
      .delete()
      .eq('id', executionId);

    if (error) {
      throw new Error(`Failed to delete execution: ${error.message}`);
    }
  }

  /**
   * Delete all executions for a workflow
   */
  async deleteWorkflowExecutions(workflowId: string): Promise<void> {
    const { error } = await this.supabase
      .from('workflow_executions')
      .delete()
      .eq('workflow_id', workflowId);

    if (error) {
      throw new Error(`Failed to delete workflow executions: ${error.message}`);
    }
  }
}

/**
 * Factory function to create an ExecutionService instance
 */
export function createExecutionService(
  supabase: SupabaseClient<Database>
): ExecutionService {
  return new ExecutionService(supabase);
}
