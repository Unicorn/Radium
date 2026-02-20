/**
 * Component Storage Types
 *
 * Defines interfaces for storing and retrieving custom components.
 * Supports multiple backends: database, filesystem, or custom.
 */

/**
 * Stored artifacts (simpler than GeneratedArtifacts - no validation status)
 */
export interface StoredArtifacts {
  /** Rust schema code */
  rustSchema: string;

  /** TypeScript code */
  typescriptCode: string;

  /** Test cases */
  testCases: string;

  /** Migration record YAML */
  migrationRecord: string;

  /** Handlebars template (if applicable) */
  handlebarsTemplate?: string;
}

/**
 * Stored component definition
 */
export interface StoredComponent {
  /** Unique component identifier (snake_case) */
  id: string;

  /** Human-readable name */
  name: string;

  /** Component description */
  description: string;

  /** Component category */
  category: string;

  /** Temporal type */
  temporalType: 'activity' | 'workflow' | 'signal' | 'query';

  /** Version (semver) */
  version: string;

  /** Generated code artifacts */
  artifacts: StoredArtifacts;

  /** Component schema (JSON) */
  schema: ComponentSchema;

  /** Metadata */
  metadata: ComponentMetadata;

  /** Creation timestamp */
  createdAt: Date;

  /** Last update timestamp */
  updatedAt: Date;

  /** Creator identifier */
  createdBy: string;

  /** Status */
  status: 'draft' | 'published' | 'deprecated';
}

/**
 * Component schema definition
 */
export interface ComponentSchema {
  /** Input field definitions */
  inputs: SchemaField[];

  /** Output field definitions */
  outputs: SchemaField[];

  /** Validation rules */
  validationRules: ValidationRule[];

  /** Connection rules */
  connectionRules: {
    allowedSources: string[];
    allowedTargets: string[];
  };
}

/**
 * Schema field definition
 */
export interface SchemaField {
  name: string;
  type: string;
  required: boolean;
  default?: string;
  description?: string;
  validation?: string;
}

/**
 * Validation rule
 */
export interface ValidationRule {
  field: string;
  rule: string;
  params: Record<string, unknown>;
  errorMessage: string;
}

/**
 * Component metadata
 */
export interface ComponentMetadata {
  /** Tags for search */
  tags: string[];

  /** Icon identifier */
  icon?: string;

  /** Color for UI */
  color?: string;

  /** Usage count */
  usageCount: number;

  /** Average execution time (ms) */
  avgExecutionTime?: number;

  /** Error rate */
  errorRate?: number;

  /** Is this a marketplace component? */
  isMarketplace: boolean;

  /** Marketplace publisher (if applicable) */
  publisher?: string;
}

/**
 * Options for listing components
 */
export interface ListComponentsOptions {
  /** Filter by category */
  category?: string;

  /** Filter by status */
  status?: 'draft' | 'published' | 'deprecated';

  /** Filter by creator */
  createdBy?: string;

  /** Search query */
  search?: string;

  /** Pagination offset */
  offset?: number;

  /** Pagination limit */
  limit?: number;

  /** Sort field */
  sortBy?: 'name' | 'createdAt' | 'updatedAt' | 'usageCount';

  /** Sort direction */
  sortOrder?: 'asc' | 'desc';
}

/**
 * Result of listing components
 */
export interface ListComponentsResult {
  components: StoredComponent[];
  total: number;
  offset: number;
  limit: number;
}

/**
 * Component storage backend interface
 *
 * Implement this interface to support different storage backends.
 */
export interface ComponentStorageBackend {
  /**
   * Initialize the storage backend
   */
  initialize(): Promise<void>;

  /**
   * Save a component
   */
  save(component: StoredComponent): Promise<StoredComponent>;

  /**
   * Get a component by ID
   */
  get(id: string): Promise<StoredComponent | null>;

  /**
   * Get a specific version of a component
   */
  getVersion(id: string, version: string): Promise<StoredComponent | null>;

  /**
   * List components with optional filters
   */
  list(options?: ListComponentsOptions): Promise<ListComponentsResult>;

  /**
   * Update a component
   */
  update(
    id: string,
    updates: Partial<StoredComponent>
  ): Promise<StoredComponent | null>;

  /**
   * Delete a component
   */
  delete(id: string): Promise<boolean>;

  /**
   * Check if a component exists
   */
  exists(id: string): Promise<boolean>;

  /**
   * Get all versions of a component
   */
  getVersions(id: string): Promise<string[]>;

  /**
   * Increment usage count
   */
  incrementUsage(id: string): Promise<void>;

  /**
   * Search components
   */
  search(query: string, limit?: number): Promise<StoredComponent[]>;
}

/**
 * Storage configuration
 */
export interface StorageConfig {
  /** Storage backend type */
  type: 'database' | 'filesystem' | 'custom';

  /** Database connection string (for database backend) */
  databaseUrl?: string;

  /** Base directory for filesystem storage */
  baseDir?: string;

  /** Custom backend instance */
  customBackend?: ComponentStorageBackend;
}
