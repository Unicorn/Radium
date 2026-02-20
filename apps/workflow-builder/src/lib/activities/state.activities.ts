/**
 * State Variable Activities
 * 
 * Temporal activities for managing state variables with different storage backends
 * (workflow/in-memory, database, redis)
 */

import { createClient } from '@supabase/supabase-js';
import type { Database } from '@/types/database';
import { getStorageAdapterForVariable } from '@/lib/state/state-factory';

/**
 * Get Supabase client for activities (uses service role key)
 */
function getSupabaseClient() {
  return createClient<Database>(
    process.env.NEXT_PUBLIC_SUPABASE_URL!,
    process.env.SUPABASE_SERVICE_ROLE_KEY!
  );
}

// ============================================================================
// Type Definitions
// ============================================================================

export interface StateVariableInput {
  variableId: string;
  variableName: string;
  scope: 'project' | 'workflow';
  storageType: 'workflow' | 'database' | 'redis' | 'external';
  storageConfig?: {
    connectorId?: string;
    tableName?: string;
    keyPrefix?: string;
    [key: string]: any;
  };
}

export interface GetStateVariableInput extends StateVariableInput {}

export interface SetStateVariableInput extends StateVariableInput {
  value: any;
}

export interface AppendStateVariableInput extends StateVariableInput {
  value: any;
}

export interface IncrementStateVariableInput extends StateVariableInput {
  amount?: number;
}

export interface DecrementStateVariableInput extends StateVariableInput {
  amount?: number;
}

// ============================================================================
// State Variable Activities
// ============================================================================

/**
 * Get state variable value
 */
export async function getStateVariable(
  input: GetStateVariableInput
): Promise<any> {
  const supabase = getSupabaseClient();
  const adapter = await getStorageAdapterForVariable(
    supabase,
    input.variableId,
    input.scope,
    input.storageType,
    input.storageConfig
  );

  return adapter.get(input.variableId, input.variableName);
}

/**
 * Set state variable value
 */
export async function setStateVariable(
  input: SetStateVariableInput
): Promise<void> {
  const supabase = getSupabaseClient();
  const adapter = await getStorageAdapterForVariable(
    supabase,
    input.variableId,
    input.scope,
    input.storageType,
    input.storageConfig
  );

  return adapter.set(input.variableId, input.variableName, input.value);
}

/**
 * Append value to array state variable
 */
export async function appendStateVariable(
  input: AppendStateVariableInput
): Promise<void> {
  const supabase = getSupabaseClient();
  const adapter = await getStorageAdapterForVariable(
    supabase,
    input.variableId,
    input.scope,
    input.storageType,
    input.storageConfig
  );

  return adapter.append(input.variableId, input.variableName, input.value);
}

/**
 * Increment numeric state variable
 */
export async function incrementStateVariable(
  input: IncrementStateVariableInput
): Promise<number> {
  const supabase = getSupabaseClient();
  const adapter = await getStorageAdapterForVariable(
    supabase,
    input.variableId,
    input.scope,
    input.storageType,
    input.storageConfig
  );

  return adapter.increment(input.variableId, input.variableName, input.amount);
}

/**
 * Decrement numeric state variable
 */
export async function decrementStateVariable(
  input: DecrementStateVariableInput
): Promise<number> {
  const supabase = getSupabaseClient();
  const adapter = await getStorageAdapterForVariable(
    supabase,
    input.variableId,
    input.scope,
    input.storageType,
    input.storageConfig
  );

  return adapter.decrement(input.variableId, input.variableName, input.amount);
}

/**
 * Export all state activities as an object for proxyActivities
 */
export const stateActivities = {
  getStateVariable,
  setStateVariable,
  appendStateVariable,
  incrementStateVariable,
  decrementStateVariable,
};

