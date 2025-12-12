/**
 * Project State Manager
 * 
 * Manages project-level state variables that are shared across all services
 */

import type { SupabaseClient } from '@supabase/supabase-js';
import type { Database } from '@/types/database';
import { getStorageAdapterForVariable } from './state-factory';

export interface ProjectStateVariable {
  id: string;
  project_id: string;
  name: string;
  type: 'string' | 'number' | 'boolean' | 'object' | 'array';
  storage_type: 'workflow' | 'database' | 'redis' | 'external';
  storage_config?: any;
  schema?: any;
}

/**
 * Get project-level state variable
 */
export async function getProjectStateVariable(
  projectId: string,
  variableName: string,
  supabase: SupabaseClient<Database>
): Promise<ProjectStateVariable | null> {
  const { data, error } = await supabase
    .from('project_state_variables')
    .select('*')
    .eq('project_id', projectId)
    .eq('name', variableName)
    .single();

  if (error) {
    if (error.code === 'PGRST116') {
      return null;
    }
    throw new Error(`Failed to get project state variable: ${error.message}`);
  }

  return data;
}

/**
 * Get all project-level state variables
 */
export async function getProjectStateVariables(
  projectId: string,
  supabase: SupabaseClient<Database>
): Promise<ProjectStateVariable[]> {
  const { data, error } = await supabase
    .from('project_state_variables')
    .select('*')
    .eq('project_id', projectId)
    .order('name');

  if (error) {
    throw new Error(`Failed to get project state variables: ${error.message}`);
  }

  return data || [];
}

/**
 * Create project-level state variable
 */
export async function createProjectStateVariable(
  projectId: string,
  variable: Omit<ProjectStateVariable, 'id' | 'project_id' | 'created_at' | 'updated_at'>,
  supabase: SupabaseClient<Database>
): Promise<ProjectStateVariable> {
  const { data, error } = await supabase
    .from('project_state_variables')
    .insert({
      project_id: projectId,
      name: variable.name,
      type: variable.type,
      storage_type: variable.storage_type,
      storage_config: variable.storage_config,
      schema: variable.schema,
    })
    .select()
    .single();

  if (error) {
    throw new Error(`Failed to create project state variable: ${error.message}`);
  }

  return data;
}

/**
 * Update project-level state variable
 */
export async function updateProjectStateVariable(
  variableId: string,
  updates: Partial<Omit<ProjectStateVariable, 'id' | 'project_id' | 'created_at'>>,
  supabase: SupabaseClient<Database>
): Promise<ProjectStateVariable> {
  const { data, error } = await supabase
    .from('project_state_variables')
    .update(updates)
    .eq('id', variableId)
    .select()
    .single();

  if (error) {
    throw new Error(`Failed to update project state variable: ${error.message}`);
  }

  return data;
}

/**
 * Delete project-level state variable
 */
export async function deleteProjectStateVariable(
  variableId: string,
  supabase: SupabaseClient<Database>
): Promise<void> {
  const { error } = await supabase
    .from('project_state_variables')
    .delete()
    .eq('id', variableId);

  if (error) {
    throw new Error(`Failed to delete project state variable: ${error.message}`);
  }
}

/**
 * Get value of a project-level state variable using appropriate storage adapter
 */
export async function getProjectStateVariableValue(
  variableId: string,
  variableName: string,
  storageType: string,
  storageConfig: any,
  supabase: SupabaseClient<Database>
): Promise<any> {
  const adapter = await getStorageAdapterForVariable(
    supabase,
    variableId,
    'project',
    storageType,
    storageConfig
  );

  return adapter.get(variableId, variableName);
}

/**
 * Set value of a project-level state variable using appropriate storage adapter
 */
export async function setProjectStateVariableValue(
  variableId: string,
  variableName: string,
  value: any,
  storageType: string,
  storageConfig: any,
  supabase: SupabaseClient<Database>
): Promise<void> {
  const adapter = await getStorageAdapterForVariable(
    supabase,
    variableId,
    'project',
    storageType,
    storageConfig
  );

  return adapter.set(variableId, variableName, value);
}

