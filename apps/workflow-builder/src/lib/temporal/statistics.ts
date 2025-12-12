/**
 * Statistics Collection Module
 * 
 * Records workflow and activity execution metrics for performance analysis
 * and future optimization (e.g., identifying activities that need dedicated workers).
 */

import { createClient } from '@supabase/supabase-js';
import type { Database } from '@/types/database';

// Server-side Supabase client
function getStorageClient() {
  return createClient<Database>(
    process.env.NEXT_PUBLIC_SUPABASE_URL!,
    process.env.SUPABASE_SERVICE_ROLE_KEY!
  );
}

/**
 * Record a workflow execution
 * Updates project statistics atomically
 */
export async function recordWorkflowExecution(
  projectId: string,
  workflowId: string,
  durationMs: number,
  success: boolean
): Promise<void> {
  const supabase = getStorageClient();
  
  try {
    // Call database function to update project stats atomically
    const { error: projectError } = await supabase.rpc('increment_project_stats', {
      p_project_id: projectId,
      p_duration_ms: durationMs,
    });
    
    if (projectError) {
      console.error('⚠️  Failed to update project statistics:', projectError);
    } else {
      console.log(`📊 Updated project statistics for ${projectId}`);
    }
    
    // Call database function to update compiled code stats
    const { error: codeError } = await supabase.rpc('increment_code_stats', {
      p_workflow_id: workflowId,
      p_duration_ms: durationMs,
      p_success: success,
    });
    
    if (codeError) {
      console.error('⚠️  Failed to update code statistics:', codeError);
    } else {
      console.log(`📊 Updated code statistics for workflow ${workflowId}`);
    }
  } catch (error) {
    console.error('❌ Error recording workflow execution:', error);
    // Don't throw - statistics are non-critical
  }
}

/**
 * Record an activity execution
 * Updates activity statistics for future optimization
 */
export async function recordActivityExecution(
  projectId: string,
  activityName: string,
  durationMs: number,
  success: boolean
): Promise<void> {
  const supabase = getStorageClient();
  
  try {
    const { error } = await supabase.rpc('update_activity_stats', {
      p_project_id: projectId,
      p_activity_name: activityName,
      p_duration_ms: durationMs,
      p_success: success,
    });
    
    if (error) {
      console.error('⚠️  Failed to update activity statistics:', error);
    } else {
      console.log(`📊 Updated activity statistics for ${activityName}`);
    }
  } catch (error) {
    console.error('❌ Error recording activity execution:', error);
    // Don't throw - statistics are non-critical
  }
}

/**
 * Analyze activity performance for a project
 * Returns activities that might benefit from dedicated workers
 */
export async function analyzeActivityPerformance(projectId: string) {
  const supabase = getStorageClient();
  
  try {
    const { data, error } = await supabase
      .from('activity_statistics')
      .select('*')
      .eq('project_id', projectId)
      .order('execution_count', { ascending: false })
      .limit(20);
    
    if (error) {
      console.error('❌ Error analyzing activity performance:', error);
      return [];
    }
    
    // Identify activities that might need dedicated workers
    const candidates = data?.filter(activity => {
      // Thresholds for dedicated worker consideration:
      const HIGH_USAGE = 1000; // > 1000 executions
      const HIGH_LATENCY = 5000; // > 5 seconds avg
      const HIGH_FAILURE_RATE = 0.05; // > 5% failure rate
      
      const failureRate = activity.execution_count > 0
        ? activity.failure_count / activity.execution_count
        : 0;
      
      return (
        activity.execution_count > HIGH_USAGE ||
        (activity.avg_duration_ms || 0) > HIGH_LATENCY ||
        failureRate > HIGH_FAILURE_RATE
      );
    }) || [];
    
    if (candidates.length > 0) {
      console.log(`🔍 Found ${candidates.length} activities that might benefit from optimization:`);
      candidates.forEach(a => {
        console.log(`   - ${a.activity_name}: ${a.execution_count} executions, ${a.avg_duration_ms}ms avg`);
      });
    }
    
    return candidates;
  } catch (error) {
    console.error('❌ Error analyzing activity performance:', error);
    return [];
  }
}

/**
 * Get project performance summary
 */
export async function getProjectPerformanceSummary(projectId: string) {
  const supabase = getStorageClient();
  
  try {
    const { data: project, error: projectError } = await supabase
      .from('projects')
      .select('*')
      .eq('id', projectId)
      .single();
    
    if (projectError || !project) {
      throw new Error('Project not found');
    }
    
    const { data: activities, error: activitiesError } = await supabase
      .from('activity_statistics')
      .select('*')
      .eq('project_id', projectId)
      .order('execution_count', { ascending: false });
    
    if (activitiesError) {
      console.warn('⚠️  Failed to fetch activity statistics:', activitiesError);
    }
    
    return {
      project: {
        id: project.id,
        name: project.name,
        totalWorkflowExecutions: project.total_workflow_executions,
        totalActivityExecutions: project.total_activity_executions,
        avgExecutionDuration: project.avg_execution_duration_ms,
        lastExecution: project.last_execution_at,
      },
      activities: activities || [],
      topActivities: (activities || []).slice(0, 10),
      recommendedOptimizations: await analyzeActivityPerformance(projectId),
    };
  } catch (error) {
    console.error('❌ Error getting project performance summary:', error);
    throw error;
  }
}

/**
 * Update worker statistics
 * Called by worker heartbeat or on task completion
 */
export async function updateWorkerStatistics(
  workerId: string,
  stats: {
    tasksCompleted?: number;
    tasksFailed?: number;
    avgTaskDuration?: number;
    cpuUsage?: number;
    memoryUsage?: number;
  }
): Promise<void> {
  const supabase = getStorageClient();
  
  try {
    const updates: any = {
      updated_at: new Date().toISOString(),
    };
    
    if (stats.tasksCompleted !== undefined) {
      updates.total_tasks_completed = stats.tasksCompleted;
    }
    
    if (stats.tasksFailed !== undefined) {
      updates.total_tasks_failed = stats.tasksFailed;
    }
    
    if (stats.avgTaskDuration !== undefined) {
      updates.avg_task_duration_ms = stats.avgTaskDuration;
    }
    
    if (stats.cpuUsage !== undefined) {
      updates.cpu_usage_percent = stats.cpuUsage;
    }
    
    if (stats.memoryUsage !== undefined) {
      updates.memory_usage_mb = stats.memoryUsage;
    }
    
    const { error } = await supabase
      .from('workflow_workers')
      .update(updates)
      .eq('worker_id', workerId);
    
    if (error) {
      console.error('⚠️  Failed to update worker statistics:', error);
    }
  } catch (error) {
    console.error('❌ Error updating worker statistics:', error);
    // Don't throw - statistics are non-critical
  }
}

/**
 * Get worker performance metrics
 */
export async function getWorkerMetrics(projectId: string) {
  const supabase = getStorageClient();
  
  try {
    const { data, error } = await supabase
      .from('workflow_workers')
      .select('*')
      .eq('project_id', projectId)
      .order('created_at', { ascending: false });
    
    if (error) {
      throw new Error(`Failed to get worker metrics: ${error.message}`);
    }
    
    return data || [];
  } catch (error) {
    console.error('❌ Error getting worker metrics:', error);
    throw error;
  }
}

/**
 * Flag an activity for dedicated worker consideration
 */
export async function flagActivityForOptimization(
  projectId: string,
  activityName: string,
  requiresDedicatedWorker: boolean
): Promise<void> {
  const supabase = getStorageClient();
  
  try {
    const { error } = await supabase
      .from('activity_statistics')
      .update({ requires_dedicated_worker: requiresDedicatedWorker })
      .eq('project_id', projectId)
      .eq('activity_name', activityName);
    
    if (error) {
      throw new Error(`Failed to flag activity: ${error.message}`);
    }
    
    console.log(`✅ Activity ${activityName} ${requiresDedicatedWorker ? 'flagged for' : 'unflagged from'} dedicated worker`);
  } catch (error) {
    console.error('❌ Error flagging activity:', error);
    throw error;
  }
}

