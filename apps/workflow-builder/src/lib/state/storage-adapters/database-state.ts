/**
 * Database State Storage Adapter
 * 
 * PostgreSQL-backed state storage using Supabase
 * State is persisted in the database and can be shared across workflows
 */

import type { SupabaseClient } from '@supabase/supabase-js';
import type { Database } from '@/types/database';
import type { StateStorageAdapter } from './external-state';

export interface DatabaseStateConfig {
  tableName?: string; // Custom table name (default: 'state_variable_data')
  connectorId?: string; // Database connector ID (optional, uses default Supabase client if not provided)
}

/**
 * Database State Adapter
 * 
 * Stores state variables in PostgreSQL via Supabase.
 * Suitable for large data or cross-workflow state sharing.
 */
export class DatabaseStateAdapter implements StateStorageAdapter {
  private supabase: SupabaseClient<Database>;
  private tableName: string;

  constructor(
    supabase: SupabaseClient<Database>,
    config?: DatabaseStateConfig
  ) {
    this.supabase = supabase;
    this.tableName = config?.tableName || 'state_variable_data';
  }

  async get(variableId: string, variableName: string): Promise<any> {
    const { data, error } = await this.supabase
      .from(this.tableName)
      .select('value')
      .eq('variable_id', variableId)
      .eq('name', variableName)
      .single();

    if (error) {
      if (error.code === 'PGRST116') {
        // Not found
        return null;
      }
      throw new Error(`Failed to get state variable: ${error.message}`);
    }

    return data?.value ?? null;
  }

  async set(variableId: string, variableName: string, value: any): Promise<void> {
    const { error } = await this.supabase
      .from(this.tableName)
      .upsert({
        variable_id: variableId,
        name: variableName,
        value,
        updated_at: new Date().toISOString(),
      }, {
        onConflict: 'variable_id,name',
      });

    if (error) {
      throw new Error(`Failed to set state variable: ${error.message}`);
    }
  }

  async append(variableId: string, variableName: string, value: any): Promise<void> {
    const current = await this.get(variableId, variableName) || [];
    if (!Array.isArray(current)) {
      throw new Error(`Variable ${variableName} is not an array`);
    }
    current.push(value);
    await this.set(variableId, variableName, current);
  }

  async increment(variableId: string, variableName: string, amount: number = 1): Promise<number> {
    const current = await this.get(variableId, variableName) || 0;
    const newValue = Number(current) + amount;
    await this.set(variableId, variableName, newValue);
    return newValue;
  }

  async decrement(variableId: string, variableName: string, amount: number = 1): Promise<number> {
    const current = await this.get(variableId, variableName) || 0;
    const newValue = Number(current) - amount;
    await this.set(variableId, variableName, newValue);
    return newValue;
  }

  async delete(variableId: string, variableName: string): Promise<void> {
    const { error } = await this.supabase
      .from(this.tableName)
      .delete()
      .eq('variable_id', variableId)
      .eq('name', variableName);

    if (error) {
      throw new Error(`Failed to delete state variable: ${error.message}`);
    }
  }

  async exists(variableId: string, variableName: string): Promise<boolean> {
    const { data, error } = await this.supabase
      .from(this.tableName)
      .select('id')
      .eq('variable_id', variableId)
      .eq('name', variableName)
      .single();

    if (error && error.code !== 'PGRST116') {
      throw new Error(`Failed to check state variable existence: ${error.message}`);
    }

    return !!data;
  }
}

