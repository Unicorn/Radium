/**
 * Redis State Storage Adapter
 * 
 * Redis-backed state storage for high-performance access
 * State is cached in Redis and can be shared across workflows/instances
 */

import type { StateStorageAdapter } from './external-state';
import type { SupabaseClient } from '@supabase/supabase-js';
import type { Database } from '@/types/database';

export interface RedisStateConfig {
  connectorId: string; // Redis connector ID
  keyPrefix?: string; // Prefix for Redis keys (default: 'state:')
}

/**
 * Redis State Adapter
 * 
 * Stores state variables in Redis.
 * Suitable for frequently accessed data or distributed state.
 */
export class RedisStateAdapter implements StateStorageAdapter {
  private connectorId: string;
  private keyPrefix: string;
  private supabase: SupabaseClient<Database>;
  private connectionUrl: string | null = null;

  constructor(
    supabase: SupabaseClient<Database>,
    config: RedisStateConfig
  ) {
    this.supabase = supabase;
    this.connectorId = config.connectorId;
    this.keyPrefix = config.keyPrefix || 'state:';
  }

  /**
   * Get Redis connection URL from connector
   */
  private async getConnectionUrl(): Promise<string> {
    if (this.connectionUrl) {
      return this.connectionUrl;
    }

    const { data: connector, error } = await this.supabase
      .from('connectors')
      .select('config_data, credentials_encrypted')
      .eq('id', this.connectorId)
      .single();

    if (error || !connector) {
      throw new Error(`Failed to get Redis connector: ${error?.message || 'Connector not found'}`);
    }

    // Extract connection URL from config
    // TODO: Decrypt credentials_encrypted if needed
    const config = connector.config_data as any;
    const connectionUrl = config.connectionUrl || config.url || config.redis_url;

    if (!connectionUrl) {
      throw new Error('Redis connector missing connection URL');
    }

    this.connectionUrl = connectionUrl;
    return connectionUrl;
  }

  /**
   * Get Redis key for variable
   */
  private getRedisKey(variableId: string, variableName: string): string {
    return `${this.keyPrefix}${variableId}:${variableName}`;
  }

  /**
   * Execute Redis command via activity
   */
  private async executeRedisCommand(
    command: string,
    key: string,
    value?: string
  ): Promise<any> {
    const connectionUrl = await this.getConnectionUrl();
    
    // Import redis activity dynamically
    const { redisCommandActivity } = await import('@/lib/components/redis-activity');
    
    const result = await redisCommandActivity({
      connectionUrl,
      command,
      key,
      value: value ? JSON.stringify(value) : undefined,
    });

    if (!result.success) {
      throw new Error(result.error || 'Redis command failed');
    }

    // Parse JSON result for GET commands
    if (command === 'GET' && result.result) {
      try {
        return JSON.parse(result.result);
      } catch {
        return result.result;
      }
    }

    return result.result;
  }

  async get(variableId: string, variableName: string): Promise<any> {
    const key = this.getRedisKey(variableId, variableName);
    const result = await this.executeRedisCommand('GET', key);
    return result ?? null;
  }

  async set(variableId: string, variableName: string, value: any): Promise<void> {
    const key = this.getRedisKey(variableId, variableName);
    await this.executeRedisCommand('SET', key, JSON.stringify(value));
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
    const key = this.getRedisKey(variableId, variableName);
    
    // Use INCR for amount = 1, otherwise GET + SET
    if (amount === 1) {
      const result = await this.executeRedisCommand('INCR', key);
      return Number(result);
    } else {
      const current = await this.get(variableId, variableName) || 0;
      const newValue = Number(current) + amount;
      await this.set(variableId, variableName, newValue);
      return newValue;
    }
  }

  async decrement(variableId: string, variableName: string, amount: number = 1): Promise<number> {
    const key = this.getRedisKey(variableId, variableName);
    
    // Use DECR for amount = 1, otherwise GET + SET
    if (amount === 1) {
      const result = await this.executeRedisCommand('DECR', key);
      return Number(result);
    } else {
      const current = await this.get(variableId, variableName) || 0;
      const newValue = Number(current) - amount;
      await this.set(variableId, variableName, newValue);
      return newValue;
    }
  }

  async delete(variableId: string, variableName: string): Promise<void> {
    const key = this.getRedisKey(variableId, variableName);
    await this.executeRedisCommand('DEL', key);
  }

  async exists(variableId: string, variableName: string): Promise<boolean> {
    const key = this.getRedisKey(variableId, variableName);
    const result = await this.executeRedisCommand('EXISTS', key);
    return result === 1;
  }
}

