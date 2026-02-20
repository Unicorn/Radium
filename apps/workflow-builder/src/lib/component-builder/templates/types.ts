/**
 * Component Template Types
 *
 * Types for the template system that provides reusable
 * component starting points.
 */

/** Template categories */
export type TemplateCategory =
  | 'communication'  // Email, SMS, Slack, webhooks
  | 'data'           // Database, API, transform
  | 'integration'    // Third-party services
  | 'control'        // Flow control, conditions, loops
  | 'ai'             // AI/ML components
  | 'custom';        // User-created templates

/** Template difficulty/complexity */
export type TemplateComplexity = 'simple' | 'moderate' | 'advanced';

/**
 * Component template definition
 */
export interface ComponentTemplate {
  /** Unique template identifier */
  id: string;

  /** Display name */
  name: string;

  /** Template description */
  description: string;

  /** Category */
  category: TemplateCategory;

  /** Template version */
  version: string;

  /** Author/creator */
  author: string;

  /** Complexity level */
  complexity: TemplateComplexity;

  /** Input schema template */
  inputSchema: SchemaTemplate;

  /** Output schema template */
  outputSchema: SchemaTemplate;

  /** Validation rules */
  validationRules: ValidationRuleTemplate[];

  /** Example usage code */
  exampleUsage: string;

  /** Tags for search */
  tags: string[];

  /** Icon name (optional) */
  icon?: string;

  /** Whether template is customizable */
  customizable: boolean;

  /** Customizable fields */
  customizableFields: string[];

  /** Dependencies */
  dependencies: string[];
}

/**
 * Schema template definition
 */
export interface SchemaTemplate {
  /** Fields in the schema */
  fields: FieldTemplate[];

  /** List of customizable field names */
  customizable: string[];
}

/**
 * Field template definition
 */
export interface FieldTemplate {
  /** Field name */
  name: string;

  /** Rust type */
  rustType: string;

  /** TypeScript type */
  typescriptType: string;

  /** Whether required */
  required: boolean;

  /** Whether this field is customizable */
  customizable: boolean;

  /** Field description */
  description: string;

  /** Default value */
  default?: string;

  /** Validation rule */
  validation?: string;
}

/**
 * Validation rule template
 */
export interface ValidationRuleTemplate {
  /** Target field */
  field: string;

  /** Rule type */
  ruleType: string;

  /** Rule expression */
  rule: string;

  /** Whether customizable */
  customizable: boolean;

  /** Error message */
  errorMessage: string;
}

/**
 * Template customization options
 */
export interface TemplateCustomization {
  /** Template ID to customize */
  templateId: string;

  /** New component name */
  componentName: string;

  /** Field customizations */
  fieldCustomizations: FieldCustomization[];

  /** Additional fields to add */
  additionalFields: FieldTemplate[];

  /** Fields to remove */
  removedFields: string[];

  /** Custom validation rules */
  customValidation: ValidationRuleTemplate[];
}

/**
 * Field customization
 */
export interface FieldCustomization {
  /** Original field name */
  originalName: string;

  /** New field name (optional) */
  newName?: string;

  /** New type (optional) */
  newType?: string;

  /** New required status (optional) */
  newRequired?: boolean;

  /** New default value (optional) */
  newDefault?: string;

  /** New description (optional) */
  newDescription?: string;
}

/**
 * Template library metadata
 */
export interface TemplateLibrary {
  /** Library version */
  version: string;

  /** Last updated */
  lastUpdated: string;

  /** Total template count */
  templateCount: number;

  /** Templates by category */
  byCategory: Record<TemplateCategory, number>;

  /** Templates by complexity */
  byComplexity: Record<TemplateComplexity, number>;
}

/**
 * Template search result
 */
export interface TemplateSearchResult {
  /** Matching templates */
  templates: ComponentTemplate[];

  /** Total count */
  total: number;

  /** Search query used */
  query: string;

  /** Filters applied */
  filters: TemplateSearchFilters;
}

/**
 * Template search filters
 */
export interface TemplateSearchFilters {
  /** Filter by category */
  category?: TemplateCategory;

  /** Filter by complexity */
  complexity?: TemplateComplexity;

  /** Filter by tags */
  tags?: string[];

  /** Filter by customizable */
  customizableOnly?: boolean;
}
