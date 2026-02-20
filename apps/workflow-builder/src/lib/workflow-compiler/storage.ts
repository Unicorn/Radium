/**
 * Workflow Compiled Code Storage
 * 
 * Stores and retrieves compiled workflow code in Supabase database.
 * This allows workers to load code dynamically without file system access.
 */

import { createClient } from '@supabase/supabase-js';
import type { Database } from '@/types/database';

// Server-side Supabase client for code storage
function getStorageClient() {
  return createClient<Database>(
    process.env.NEXT_PUBLIC_SUPABASE_URL!,
    process.env.SUPABASE_SERVICE_ROLE_KEY!
  );
}

export interface CompiledWorkflow {
  workflowCode: string;
  activitiesCode: string;
  workerCode: string;
  packageJson: string;
  tsConfig: string;
}

/**
 * Store compiled workflow code in database
 * Marks previous versions as inactive
 */
export async function storeCompiledCode(
  workflowId: string,
  version: string,
  compiled: CompiledWorkflow,
  userId: string
): Promise<string> {
  const supabase = getStorageClient();
  
  try {
    // Mark previous versions as inactive (only other versions)
    const { error: updateError } = await supabase
      .from('workflow_compiled_code')
      .update({ is_active: false })
      .eq('workflow_id', workflowId)
      .neq('version', version);
    
    if (updateError) {
      console.warn('Warning: Failed to deactivate old versions:', updateError);
    }
    
    // Store new version (upsert to handle existing entries)
    const { data, error } = await supabase
      .from('workflow_compiled_code')
      .upsert({
        workflow_id: workflowId,
        version,
        workflow_code: compiled.workflowCode,
        activities_code: compiled.activitiesCode,
        worker_code: compiled.workerCode,
        package_json: compiled.packageJson,
        tsconfig_json: compiled.tsConfig,
        compiled_by: userId,
        is_active: true,
        compiled_at: new Date().toISOString(),
      }, {
        onConflict: 'workflow_id,version'
      })
      .select()
      .single();
    
    if (error) {
      throw new Error(`Failed to store compiled code: ${error.message}`);
    }
    
    console.log(`✅ Stored compiled code for workflow ${workflowId} version ${version}`);
    return data.id;
  } catch (error) {
    console.error('❌ Error storing compiled code:', error);
    throw error;
  }
}

/**
 * Get compiled code for a workflow
 * Returns active version by default, or specific version if provided
 */
export async function getCompiledCode(
  workflowId: string,
  version?: string
): Promise<CompiledWorkflow & { id: string; compiled_at: string }> {
  const supabase = getStorageClient();
  
  try {
    let query = supabase
      .from('workflow_compiled_code')
      .select('*')
      .eq('workflow_id', workflowId);
    
    if (version) {
      query = query.eq('version', version);
    } else {
      query = query.eq('is_active', true);
    }
    
    const { data, error } = await query.single();
    
    if (error || !data) {
      throw new Error(`No compiled code found for workflow ${workflowId}${version ? ` version ${version}` : ''}`);
    }
    
    return {
      id: data.id,
      workflowCode: data.workflow_code,
      activitiesCode: data.activities_code,
      workerCode: data.worker_code,
      packageJson: data.package_json,
      tsConfig: data.tsconfig_json,
      compiled_at: data.compiled_at,
    };
  } catch (error) {
    console.error('❌ Error retrieving compiled code:', error);
    throw error;
  }
}

/**
 * Load all active workflows for a project
 * Used by workers to load their workflow definitions
 */
export async function loadWorkflowsForProject(projectId: string) {
  const supabase = getStorageClient();
  
  try {
    const { data, error } = await supabase
      .from('workflows')
      .select(`
        id,
        name,
        display_name,
        version,
        compiled_code:workflow_compiled_code!inner(
          id,
          workflow_code,
          activities_code,
          worker_code,
          package_json,
          tsconfig_json,
          compiled_at
        )
      `)
      .eq('project_id', projectId)
      .eq('compiled_code.is_active', true);
    
    if (error) {
      throw new Error(`Failed to load workflows for project: ${error.message}`);
    }
    
    console.log(`✅ Loaded ${data?.length || 0} workflows for project ${projectId}`);
    return data || [];
  } catch (error) {
    console.error('❌ Error loading project workflows:', error);
    throw error;
  }
}

/**
 * List all versions of a workflow's compiled code
 */
export async function listCompiledVersions(workflowId: string) {
  const supabase = getStorageClient();
  
  try {
    const { data, error } = await supabase
      .from('workflow_compiled_code')
      .select('id, version, compiled_at, is_active, execution_count, error_count')
      .eq('workflow_id', workflowId)
      .order('compiled_at', { ascending: false });
    
    if (error) {
      throw new Error(`Failed to list versions: ${error.message}`);
    }
    
    return data || [];
  } catch (error) {
    console.error('❌ Error listing compiled versions:', error);
    throw error;
  }
}

/**
 * Activate a specific version of compiled code
 * Useful for rollbacks
 */
export async function activateVersion(workflowId: string, versionId: string) {
  const supabase = getStorageClient();
  
  try {
    // Deactivate all versions
    await supabase
      .from('workflow_compiled_code')
      .update({ is_active: false })
      .eq('workflow_id', workflowId);
    
    // Activate specified version
    const { data, error } = await supabase
      .from('workflow_compiled_code')
      .update({ is_active: true })
      .eq('id', versionId)
      .eq('workflow_id', workflowId)
      .select()
      .single();
    
    if (error || !data) {
      throw new Error(`Failed to activate version: ${error?.message || 'Version not found'}`);
    }
    
    console.log(`✅ Activated version ${data.version} for workflow ${workflowId}`);
    return data;
  } catch (error) {
    console.error('❌ Error activating version:', error);
    throw error;
  }
}

/**
 * Delete old versions (keep last N versions)
 */
export async function cleanupOldVersions(workflowId: string, keepCount: number = 10) {
  const supabase = getStorageClient();
  
  try {
    // Get all versions, ordered by date
    const { data: versions } = await supabase
      .from('workflow_compiled_code')
      .select('id, compiled_at')
      .eq('workflow_id', workflowId)
      .order('compiled_at', { ascending: false });
    
    if (!versions || versions.length <= keepCount) {
      console.log(`✅ No cleanup needed for workflow ${workflowId}`);
      return;
    }
    
    // Delete older versions
    const toDelete = versions.slice(keepCount).map(v => v.id);
    
    const { error } = await supabase
      .from('workflow_compiled_code')
      .delete()
      .in('id', toDelete);
    
    if (error) {
      throw new Error(`Failed to cleanup versions: ${error.message}`);
    }
    
    console.log(`✅ Cleaned up ${toDelete.length} old versions for workflow ${workflowId}`);
  } catch (error) {
    console.error('❌ Error cleaning up versions:', error);
    throw error;
  }
}

