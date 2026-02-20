/**
 * External State Storage Adapter Interface
 * 
 * Base interface for all state storage adapters
 */

export interface StateStorageAdapter {
  /**
   * Get the value of a state variable
   */
  get(variableId: string, variableName: string): Promise<any>;

  /**
   * Set the value of a state variable
   */
  set(variableId: string, variableName: string, value: any): Promise<void>;

  /**
   * Append a value to an array state variable
   */
  append(variableId: string, variableName: string, value: any): Promise<void>;

  /**
   * Increment a numeric state variable
   */
  increment(variableId: string, variableName: string, amount?: number): Promise<number>;

  /**
   * Decrement a numeric state variable
   */
  decrement(variableId: string, variableName: string, amount?: number): Promise<number>;

  /**
   * Delete a state variable
   */
  delete(variableId: string, variableName: string): Promise<void>;

  /**
   * Check if a state variable exists
   */
  exists(variableId: string, variableName: string): Promise<boolean>;
}

/**
 * Storage adapter configuration
 */
export interface StorageAdapterConfig {
  storageType: 'workflow' | 'database' | 'redis' | 'external';
  storageConfig?: {
    connectorId?: string; // For redis/database connectors
    tableName?: string; // For database storage
    keyPrefix?: string; // For redis storage
    [key: string]: any; // Additional config
  };
}

