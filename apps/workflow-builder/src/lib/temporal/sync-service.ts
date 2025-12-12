/**
 * Sync Service
 * 
 * Coordinates syncing execution history from Temporal to database.
 * Implements cache-aside pattern: check DB first, sync if missing or stale.
 */

import { createClient } from '@supabase/supabase-js';
import type { Database } from '@/types/database';
import {
  checkSyncStatusActivity,
  fetchTemporalHistoryActivity,
  storeExecutionHistoryActivity,
  extractComponentExecutionsActivity,
  updateSyncStatusActivity,
  type SyncExecutionInput,
} from './sync-execution.activities';
import type { WorkflowDefinition } from '@/types/workflow';

/**
 * Get Supabase client with service role
 */
function getSupabaseClient() {
  return createClient<Database>(
    process.env.NEXT_PUBLIC_SUPABASE_URL!,
    process.env.SUPABASE_SERVICE_ROLE_KEY!
  );
}

export interface SyncRequest {
  executionId: string;
  immediate?: boolean; // If true, wait for sync to complete
}

export interface SyncResult {
  success: boolean;
  synced: boolean; // True if sync was performed, false if already synced
  componentExecutionsCount?: number;
  error?: string;
}

/**
 * Check if execution needs syncing (cache check)
 */
export async function checkSyncStatus(executionId: string): Promise<{
  needsSync: boolean;
  syncStatus: string;
}> {
  return checkSyncStatusActivity(executionId);
}

/**
 * Request sync for an execution
 * Implements cache-aside pattern: checks DB first, syncs if needed
 */
export async function requestSync(
  executionId: string,
  immediate: boolean = false
): Promise<SyncResult> {
  const supabase = getSupabaseClient();
  
  // Get execution details
  const { data: execution, error: execError } = await supabase
    .from('workflow_executions')
    .select(`
      id,
      temporal_workflow_id,
      temporal_run_id,
      workflow_id,
      history_sync_status
    `)
    .eq('id', executionId)
    .single();
  
  if (execError || !execution) {
    return {
      success: false,
      synced: false,
      error: `Execution not found: ${execError?.message || 'Unknown error'}`,
    };
  }
  
  if (!execution.temporal_workflow_id || !execution.temporal_run_id) {
    return {
      success: false,
      synced: false,
      error: 'Execution does not have Temporal workflow ID or run ID',
    };
  }
  
  // Check cache (cache-aside pattern)
  const { needsSync, syncStatus } = await checkSyncStatusActivity(executionId);
  
  if (!needsSync) {
    return {
      success: true,
      synced: false, // Already synced, no work done
    };
  }
  
  // If already syncing and not immediate, return
  if (syncStatus === 'syncing' && !immediate) {
    return {
      success: true,
      synced: false,
      error: 'Sync already in progress',
    };
  }
  
  // Get workflow definition
  const { data: workflow } = await supabase
    .from('workflows')
    .select('definition')
    .eq('id', execution.workflow_id)
    .single();
  
  if (!workflow || !workflow.definition) {
    return {
      success: false,
      synced: false,
      error: 'Workflow definition not found',
    };
  }
  
  // Get system user ID
  const { data: systemUser } = await supabase
    .from('users')
    .select('id')
    .eq('email', 'system@example.com')
    .single();
  
  if (!systemUser) {
    return {
      success: false,
      synced: false,
      error: 'System user not found',
    };
  }
  
  // Perform sync
  try {
    // Update status to syncing
    await updateSyncStatusActivity(executionId, 'syncing');
    
    // Fetch history from Temporal
    const history = await fetchTemporalHistoryActivity({
      executionId,
      temporalWorkflowId: execution.temporal_workflow_id,
      runId: execution.temporal_run_id,
      workflowDefinition: workflow.definition as WorkflowDefinition,
      systemUserId: systemUser.id,
    });
    
    // Store full history
    await storeExecutionHistoryActivity(
      {
        executionId,
        temporalWorkflowId: execution.temporal_workflow_id,
        runId: execution.temporal_run_id,
        workflowDefinition: workflow.definition as WorkflowDefinition,
        systemUserId: systemUser.id,
      },
      history
    );
    
    // Extract and store component executions
    const componentExecutionsCount = await extractComponentExecutionsActivity(
      {
        executionId,
        temporalWorkflowId: execution.temporal_workflow_id,
        runId: execution.temporal_run_id,
        workflowDefinition: workflow.definition as WorkflowDefinition,
        systemUserId: systemUser.id,
      },
      history
    );
    
    // Update status to synced
    await updateSyncStatusActivity(executionId, 'synced');
    
    return {
      success: true,
      synced: true,
      componentExecutionsCount,
    };
  } catch (error: any) {
    // Update status to failed
    await updateSyncStatusActivity(executionId, 'failed', error.message);
    
    return {
      success: false,
      synced: false,
      error: error.message || 'Sync failed',
    };
  }
}

/**
 * Wait for sync to complete (for immediate syncs)
 */
export async function waitForSync(executionId: string, timeoutMs: number = 30000): Promise<SyncResult> {
  const startTime = Date.now();
  
  while (Date.now() - startTime < timeoutMs) {
    const { needsSync, syncStatus } = await checkSyncStatusActivity(executionId);
    
    if (syncStatus === 'synced') {
      return {
        success: true,
        synced: true,
      };
    }
    
    if (syncStatus === 'failed') {
      return {
        success: false,
        synced: false,
        error: 'Sync failed',
      };
    }
    
    // Wait a bit before checking again
    await new Promise(resolve => setTimeout(resolve, 500));
  }
  
  return {
    success: false,
    synced: false,
    error: 'Sync timeout',
  };
}

