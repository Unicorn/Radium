/**
 * Activities for syncing execution history from Temporal
 * 
 * These activities perform the actual work of fetching history,
 * parsing it, and storing it in the database.
 */

import { createClient } from '@supabase/supabase-js';
import type { Database } from '@/types/database';
import { getTemporalClient } from './connection';
import {
  getWorkflowExecutionHistory,
  storeFullHistory,
  parseComponentExecutions,
  identifyExpectedRetries,
} from './history-query';
import type { WorkflowDefinition } from '@/types/workflow';

/**
 * Get Supabase client with service role (for system operations)
 */
function getSupabaseClient() {
  return createClient<Database>(
    process.env.NEXT_PUBLIC_SUPABASE_URL!,
    process.env.SUPABASE_SERVICE_ROLE_KEY!
  );
}

export interface SyncExecutionInput {
  executionId: string;
  temporalWorkflowId: string;
  runId: string;
  workflowDefinition: WorkflowDefinition;
  systemUserId: string;
}

export interface SyncExecutionResult {
  success: boolean;
  componentExecutionsCount: number;
  error?: string;
}

/**
 * Fetch Temporal history for an execution
 */
export async function fetchTemporalHistoryActivity(
  input: SyncExecutionInput
): Promise<any> {
  const client = await getTemporalClient();
  const history = await getWorkflowExecutionHistory(
    client,
    input.temporalWorkflowId,
    input.runId
  );
  return history;
}

/**
 * Store execution history in database
 */
export async function storeExecutionHistoryActivity(
  input: SyncExecutionInput,
  history: any
): Promise<void> {
  const supabase = getSupabaseClient();
  await storeFullHistory(
    input.executionId,
    history,
    supabase,
    input.systemUserId
  );
}

/**
 * Extract and store component executions
 */
export async function extractComponentExecutionsActivity(
  input: SyncExecutionInput,
  history: any
): Promise<number> {
  const supabase = getSupabaseClient();
  
  // Parse component executions from history
  const componentExecutions = parseComponentExecutions(
    history,
    input.workflowDefinition
  );
  
  // Identify expected retries
  const executionsWithRetryInfo = identifyExpectedRetries(
    componentExecutions,
    input.workflowDefinition
  );
  
  // Store component executions (using system user context)
  const executionsToInsert = executionsWithRetryInfo.map(exec => ({
    workflow_execution_id: input.executionId,
    node_id: exec.nodeId,
    component_id: exec.componentId || null,
    component_name: exec.componentName || null,
    status: exec.status,
    started_at: exec.startedAt?.toISOString() || null,
    completed_at: exec.completedAt?.toISOString() || null,
    duration_ms: exec.durationMs || null,
    input_data: exec.inputData || null,
    output_data: exec.outputData || null,
    error_message: exec.errorMessage || null,
    error_type: exec.errorType || null,
    retry_count: exec.retryCount,
    is_expected_retry: exec.isExpectedRetry,
    temporal_activity_id: exec.temporalActivityId || null,
    temporal_attempt: exec.temporalAttempt,
  }));
  
  if (executionsToInsert.length > 0) {
    // Delete existing component executions for this execution (in case of re-sync)
    await supabase
      .from('component_executions')
      .delete()
      .eq('workflow_execution_id', input.executionId);
    
    // Insert new component executions
    const { error } = await supabase
      .from('component_executions')
      .insert(executionsToInsert);
    
    if (error) {
      throw new Error(`Failed to store component executions: ${error.message}`);
    }
  }
  
  return executionsToInsert.length;
}

/**
 * Check if execution needs syncing (cache check)
 */
export async function checkSyncStatusActivity(
  executionId: string
): Promise<{ needsSync: boolean; syncStatus: string }> {
  const supabase = getSupabaseClient();
  
  const { data, error } = await supabase
    .from('workflow_executions')
    .select('history_sync_status, history_synced_at')
    .eq('id', executionId)
    .single();
  
  if (error || !data) {
    return { needsSync: true, syncStatus: 'pending' };
  }
  
  // If already synced, check if it's recent (within last hour)
  if (data.history_sync_status === 'synced' && data.history_synced_at) {
    const syncedAt = new Date(data.history_synced_at);
    const oneHourAgo = new Date(Date.now() - 60 * 60 * 1000);
    
    if (syncedAt > oneHourAgo) {
      return { needsSync: false, syncStatus: 'synced' };
    }
  }
  
  return { needsSync: true, syncStatus: data.history_sync_status || 'pending' };
}

/**
 * Update execution sync status
 */
export async function updateSyncStatusActivity(
  executionId: string,
  status: 'syncing' | 'synced' | 'failed',
  error?: string
): Promise<void> {
  const supabase = getSupabaseClient();
  
  const update: any = {
    history_sync_status: status,
  };
  
  if (status === 'synced') {
    update.history_synced_at = new Date().toISOString();
  }
  
  if (error) {
    // Could store error in a separate field if needed
  }
  
  const { error: updateError } = await supabase
    .from('workflow_executions')
    .update(update)
    .eq('id', executionId);
  
  if (updateError) {
    throw new Error(`Failed to update sync status: ${updateError.message}`);
  }
}

