/**
 * State Storage Factory
 * 
 * Factory for creating state storage adapters based on storage type
 */

import type { SupabaseClient } from '@supabase/supabase-js';
import type { Database } from '@/types/database';
import type { StateStorageAdapter, StorageAdapterConfig } from './storage-adapters/external-state';
import { WorkflowStateAdapter } from './storage-adapters/workflow-state';
import { DatabaseStateAdapter } from './storage-adapters/database-state';
import { RedisStateAdapter } from './storage-adapters/redis-state';

/**
 * Create a state storage adapter based on configuration
 */
export async function createStateStorageAdapter(
  supabase: SupabaseClient<Database>,
  config: StorageAdapterConfig
): Promise<StateStorageAdapter> {
  switch (config.storageType) {
    case 'workflow':
      return new WorkflowStateAdapter();

    case 'database':
      return new DatabaseStateAdapter(supabase, config.storageConfig);

    case 'redis':
      if (!config.storageConfig?.connectorId) {
        throw new Error('Redis storage requires connectorId in storageConfig');
      }
      return new RedisStateAdapter(supabase, {
        connectorId: config.storageConfig.connectorId,
        keyPrefix: config.storageConfig.keyPrefix,
      });

    case 'external':
      throw new Error('External storage adapters must be implemented separately');

    default:
      throw new Error(`Unknown storage type: ${config.storageType}`);
  }
}

/**
 * Get storage adapter for a state variable
 */
export async function getStorageAdapterForVariable(
  supabase: SupabaseClient<Database>,
  variableId: string,
  scope: 'project' | 'workflow',
  storageType: string,
  storageConfig?: any
): Promise<StateStorageAdapter> {
  const config: StorageAdapterConfig = {
    storageType: storageType as 'workflow' | 'database' | 'redis' | 'external',
    storageConfig,
  };

  return createStateStorageAdapter(supabase, config);
}

