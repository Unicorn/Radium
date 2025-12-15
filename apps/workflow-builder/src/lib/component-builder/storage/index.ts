/**
 * Component Storage
 *
 * Configurable storage system for custom components.
 * Supports both filesystem (for repo-based) and database storage.
 */

import type {
  StorageConfig,
  ComponentStorageBackend,
  StoredComponent,
  ListComponentsOptions,
  ListComponentsResult,
} from './types';
import { FilesystemStorageBackend } from './filesystem-backend';
import { DatabaseStorageBackend } from './database-backend';

export * from './types';
export { FilesystemStorageBackend } from './filesystem-backend';
export { DatabaseStorageBackend } from './database-backend';

/**
 * Component Storage Manager
 *
 * Provides a unified interface for storing and retrieving components,
 * with configurable backend (filesystem or database).
 */
export class ComponentStorage {
  private backend: ComponentStorageBackend;
  private initialized = false;

  constructor(config: StorageConfig) {
    this.backend = this.createBackend(config);
  }

  private createBackend(config: StorageConfig): ComponentStorageBackend {
    switch (config.type) {
      case 'filesystem':
        if (!config.baseDir) {
          throw new Error('baseDir is required for filesystem storage');
        }
        return new FilesystemStorageBackend({ baseDir: config.baseDir });

      case 'database':
        return new DatabaseStorageBackend({
          prisma: undefined, // Will be injected when Prisma is set up
        });

      case 'custom':
        if (!config.customBackend) {
          throw new Error('customBackend is required for custom storage');
        }
        return config.customBackend;

      default:
        throw new Error(`Unknown storage type: ${config.type}`);
    }
  }

  /**
   * Initialize the storage backend
   */
  async initialize(): Promise<void> {
    if (!this.initialized) {
      await this.backend.initialize();
      this.initialized = true;
    }
  }

  /**
   * Ensure storage is initialized before operations
   */
  private async ensureInitialized(): Promise<void> {
    if (!this.initialized) {
      await this.initialize();
    }
  }

  /**
   * Save a component
   */
  async save(component: StoredComponent): Promise<StoredComponent> {
    await this.ensureInitialized();
    return this.backend.save(component);
  }

  /**
   * Get a component by ID
   */
  async get(id: string): Promise<StoredComponent | null> {
    await this.ensureInitialized();
    return this.backend.get(id);
  }

  /**
   * Get a specific version of a component
   */
  async getVersion(id: string, version: string): Promise<StoredComponent | null> {
    await this.ensureInitialized();
    return this.backend.getVersion(id, version);
  }

  /**
   * List components
   */
  async list(options?: ListComponentsOptions): Promise<ListComponentsResult> {
    await this.ensureInitialized();
    return this.backend.list(options);
  }

  /**
   * Update a component
   */
  async update(
    id: string,
    updates: Partial<StoredComponent>
  ): Promise<StoredComponent | null> {
    await this.ensureInitialized();
    return this.backend.update(id, updates);
  }

  /**
   * Delete a component
   */
  async delete(id: string): Promise<boolean> {
    await this.ensureInitialized();
    return this.backend.delete(id);
  }

  /**
   * Check if a component exists
   */
  async exists(id: string): Promise<boolean> {
    await this.ensureInitialized();
    return this.backend.exists(id);
  }

  /**
   * Get all versions of a component
   */
  async getVersions(id: string): Promise<string[]> {
    await this.ensureInitialized();
    return this.backend.getVersions(id);
  }

  /**
   * Increment usage count
   */
  async incrementUsage(id: string): Promise<void> {
    await this.ensureInitialized();
    return this.backend.incrementUsage(id);
  }

  /**
   * Search components
   */
  async search(query: string, limit?: number): Promise<StoredComponent[]> {
    await this.ensureInitialized();
    return this.backend.search(query, limit);
  }

  /**
   * Get the underlying backend (for advanced use cases)
   */
  getBackend(): ComponentStorageBackend {
    return this.backend;
  }
}

/**
 * Default storage configuration from environment
 */
export function getDefaultStorageConfig(): StorageConfig {
  const storageType = process.env.COMPONENT_STORAGE_TYPE || 'filesystem';
  const baseDir =
    process.env.COMPONENT_STORAGE_DIR ||
    './custom-components';

  switch (storageType) {
    case 'database':
      return {
        type: 'database',
        databaseUrl: process.env.DATABASE_URL,
      };

    case 'filesystem':
    default:
      return {
        type: 'filesystem',
        baseDir,
      };
  }
}

/**
 * Create a storage instance with default configuration
 */
export function createDefaultStorage(): ComponentStorage {
  return new ComponentStorage(getDefaultStorageConfig());
}

// Singleton instance
let defaultStorageInstance: ComponentStorage | null = null;

/**
 * Get the default storage instance (singleton)
 */
export function getDefaultStorage(): ComponentStorage {
  if (!defaultStorageInstance) {
    defaultStorageInstance = createDefaultStorage();
  }
  return defaultStorageInstance;
}

/**
 * Reset the default storage instance (for testing)
 */
export function resetDefaultStorage(): void {
  defaultStorageInstance = null;
}
