/**
 * Filesystem Storage Backend
 *
 * Stores components as files in the repository.
 * Ideal for open-source deployments where components should be version-controlled.
 */

import * as fs from 'fs/promises';
import * as path from 'path';
import type {
  ComponentStorageBackend,
  StoredComponent,
  ListComponentsOptions,
  ListComponentsResult,
} from './types';

/**
 * Filesystem backend configuration
 */
export interface FilesystemBackendConfig {
  /** Base directory for component storage */
  baseDir: string;

  /** Directory for Rust schemas */
  rustDir?: string;

  /** Directory for TypeScript templates */
  typescriptDir?: string;

  /** Directory for component metadata */
  metadataDir?: string;

  /** Whether to create directories if they don't exist */
  createDirs?: boolean;
}

/**
 * Filesystem storage backend implementation
 */
export class FilesystemStorageBackend implements ComponentStorageBackend {
  private config: Required<FilesystemBackendConfig>;

  constructor(config: FilesystemBackendConfig) {
    this.config = {
      baseDir: config.baseDir,
      rustDir: config.rustDir || path.join(config.baseDir, 'rust'),
      typescriptDir:
        config.typescriptDir || path.join(config.baseDir, 'typescript'),
      metadataDir: config.metadataDir || path.join(config.baseDir, 'metadata'),
      createDirs: config.createDirs ?? true,
    };
  }

  async initialize(): Promise<void> {
    if (this.config.createDirs) {
      await fs.mkdir(this.config.baseDir, { recursive: true });
      await fs.mkdir(this.config.rustDir, { recursive: true });
      await fs.mkdir(this.config.typescriptDir, { recursive: true });
      await fs.mkdir(this.config.metadataDir, { recursive: true });
      // Create additional directories for tests and migration records
      await fs.mkdir(path.join(this.config.baseDir, 'tests'), { recursive: true });
      await fs.mkdir(path.join(this.config.baseDir, 'records'), { recursive: true });
    }
  }

  async save(component: StoredComponent): Promise<StoredComponent> {
    const now = new Date();
    const componentToSave: StoredComponent = {
      ...component,
      createdAt: component.createdAt || now,
      updatedAt: now,
    };

    // Save metadata
    const metadataPath = this.getMetadataPath(component.id);
    await fs.writeFile(
      metadataPath,
      JSON.stringify(componentToSave, null, 2),
      'utf-8'
    );

    // Save Rust schema
    const rustPath = this.getRustPath(component.id);
    await fs.writeFile(rustPath, component.artifacts.rustSchema, 'utf-8');

    // Save TypeScript code
    const tsPath = this.getTypescriptPath(component.id);
    await fs.writeFile(tsPath, component.artifacts.typescriptCode, 'utf-8');

    // Save tests
    const testPath = this.getTestPath(component.id);
    await fs.writeFile(testPath, component.artifacts.testCases, 'utf-8');

    // Save migration record
    const migrationPath = this.getMigrationPath(component.id);
    await fs.writeFile(
      migrationPath,
      component.artifacts.migrationRecord,
      'utf-8'
    );

    return componentToSave;
  }

  async get(id: string): Promise<StoredComponent | null> {
    try {
      const metadataPath = this.getMetadataPath(id);
      const content = await fs.readFile(metadataPath, 'utf-8');
      const component = JSON.parse(content) as StoredComponent;

      // Ensure dates are Date objects
      component.createdAt = new Date(component.createdAt);
      component.updatedAt = new Date(component.updatedAt);

      return component;
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code === 'ENOENT') {
        return null;
      }
      throw error;
    }
  }

  async getVersion(id: string, version: string): Promise<StoredComponent | null> {
    // For filesystem backend, versions are stored in version-specific directories
    const versionDir = path.join(
      this.config.metadataDir,
      'versions',
      id,
      version
    );
    try {
      const metadataPath = path.join(versionDir, `${id}.json`);
      const content = await fs.readFile(metadataPath, 'utf-8');
      const component = JSON.parse(content) as StoredComponent;
      component.createdAt = new Date(component.createdAt);
      component.updatedAt = new Date(component.updatedAt);
      return component;
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code === 'ENOENT') {
        return null;
      }
      throw error;
    }
  }

  async list(options?: ListComponentsOptions): Promise<ListComponentsResult> {
    const {
      category,
      status,
      createdBy,
      search,
      offset = 0,
      limit = 50,
      sortBy = 'name',
      sortOrder = 'asc',
    } = options || {};

    // Read all metadata files
    const files = await fs.readdir(this.config.metadataDir);
    const jsonFiles = files.filter(
      (f) => f.endsWith('.json') && !f.startsWith('.')
    );

    let components: StoredComponent[] = [];

    for (const file of jsonFiles) {
      try {
        const filePath = path.join(this.config.metadataDir, file);
        const content = await fs.readFile(filePath, 'utf-8');
        const component = JSON.parse(content) as StoredComponent;
        component.createdAt = new Date(component.createdAt);
        component.updatedAt = new Date(component.updatedAt);
        components.push(component);
      } catch {
        // Skip invalid files
        console.warn(`Failed to parse component file: ${file}`);
      }
    }

    // Apply filters
    if (category) {
      components = components.filter((c) => c.category === category);
    }
    if (status) {
      components = components.filter((c) => c.status === status);
    }
    if (createdBy) {
      components = components.filter((c) => c.createdBy === createdBy);
    }
    if (search) {
      const searchLower = search.toLowerCase();
      components = components.filter(
        (c) =>
          c.name.toLowerCase().includes(searchLower) ||
          c.description.toLowerCase().includes(searchLower) ||
          c.metadata.tags.some((t) => t.toLowerCase().includes(searchLower))
      );
    }

    // Sort
    components.sort((a, b) => {
      let comparison = 0;
      switch (sortBy) {
        case 'name':
          comparison = a.name.localeCompare(b.name);
          break;
        case 'createdAt':
          comparison = a.createdAt.getTime() - b.createdAt.getTime();
          break;
        case 'updatedAt':
          comparison = a.updatedAt.getTime() - b.updatedAt.getTime();
          break;
        case 'usageCount':
          comparison = a.metadata.usageCount - b.metadata.usageCount;
          break;
      }
      return sortOrder === 'asc' ? comparison : -comparison;
    });

    const total = components.length;

    // Apply pagination
    components = components.slice(offset, offset + limit);

    return {
      components,
      total,
      offset,
      limit,
    };
  }

  async update(
    id: string,
    updates: Partial<StoredComponent>
  ): Promise<StoredComponent | null> {
    const existing = await this.get(id);
    if (!existing) {
      return null;
    }

    const updated: StoredComponent = {
      ...existing,
      ...updates,
      id: existing.id, // Don't allow ID changes
      createdAt: existing.createdAt, // Preserve creation time
      updatedAt: new Date(),
    };

    return this.save(updated);
  }

  async delete(id: string): Promise<boolean> {
    try {
      await fs.unlink(this.getMetadataPath(id));
      await fs.unlink(this.getRustPath(id)).catch(() => {});
      await fs.unlink(this.getTypescriptPath(id)).catch(() => {});
      await fs.unlink(this.getTestPath(id)).catch(() => {});
      await fs.unlink(this.getMigrationPath(id)).catch(() => {});
      return true;
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code === 'ENOENT') {
        return false;
      }
      throw error;
    }
  }

  async exists(id: string): Promise<boolean> {
    try {
      await fs.access(this.getMetadataPath(id));
      return true;
    } catch {
      return false;
    }
  }

  async getVersions(id: string): Promise<string[]> {
    const versionDir = path.join(
      this.config.metadataDir,
      'versions',
      id
    );
    try {
      const versions = await fs.readdir(versionDir);
      return versions.sort().reverse(); // Most recent first
    } catch {
      return [];
    }
  }

  async incrementUsage(id: string): Promise<void> {
    const component = await this.get(id);
    if (component) {
      component.metadata.usageCount++;
      await this.save(component);
    }
  }

  async search(query: string, limit = 10): Promise<StoredComponent[]> {
    const result = await this.list({
      search: query,
      limit,
      status: 'published',
    });
    return result.components;
  }

  // Helper methods for file paths
  private getMetadataPath(id: string): string {
    return path.join(this.config.metadataDir, `${id}.json`);
  }

  private getRustPath(id: string): string {
    return path.join(this.config.rustDir, `${id}.rs`);
  }

  private getTypescriptPath(id: string): string {
    return path.join(this.config.typescriptDir, `${id}.ts`);
  }

  private getTestPath(id: string): string {
    return path.join(this.config.baseDir, 'tests', `${id}_test.rs`);
  }

  private getMigrationPath(id: string): string {
    return path.join(this.config.baseDir, 'records', `${id}.yaml`);
  }
}
