/**
 * Template Library
 *
 * Manages the collection of component templates and provides
 * search/filter functionality.
 */

import type {
  ComponentTemplate,
  TemplateCategory,
  TemplateComplexity,
  TemplateLibrary,
  TemplateSearchResult,
  TemplateSearchFilters,
  TemplateCustomization,
} from './types';

// Import built-in templates
import { emailSenderTemplate } from './built-in/email-sender';
import { webhookTemplate } from './built-in/webhook';
import { databaseQueryTemplate } from './built-in/database-query';

/** Built-in templates collection */
const BUILT_IN_TEMPLATES: ComponentTemplate[] = [
  emailSenderTemplate,
  webhookTemplate,
  databaseQueryTemplate,
];

/**
 * Template Library class
 */
export class ComponentTemplateLibrary {
  private templates: Map<string, ComponentTemplate>;
  private customTemplates: Map<string, ComponentTemplate>;

  constructor() {
    this.templates = new Map();
    this.customTemplates = new Map();

    // Load built-in templates
    for (const template of BUILT_IN_TEMPLATES) {
      this.templates.set(template.id, template);
    }
  }

  /**
   * Get all templates
   */
  getAll(): ComponentTemplate[] {
    return [
      ...Array.from(this.templates.values()),
      ...Array.from(this.customTemplates.values()),
    ];
  }

  /**
   * Get template by ID
   */
  get(id: string): ComponentTemplate | undefined {
    return this.templates.get(id) || this.customTemplates.get(id);
  }

  /**
   * Search templates
   */
  search(query: string, filters?: TemplateSearchFilters): TemplateSearchResult {
    let results = this.getAll();

    // Apply text search
    if (query) {
      const lowerQuery = query.toLowerCase();
      results = results.filter(
        (t) =>
          t.name.toLowerCase().includes(lowerQuery) ||
          t.description.toLowerCase().includes(lowerQuery) ||
          t.tags.some((tag) => tag.toLowerCase().includes(lowerQuery))
      );
    }

    // Apply category filter
    if (filters?.category) {
      results = results.filter((t) => t.category === filters.category);
    }

    // Apply complexity filter
    if (filters?.complexity) {
      results = results.filter((t) => t.complexity === filters.complexity);
    }

    // Apply tags filter
    if (filters?.tags && filters.tags.length > 0) {
      results = results.filter((t) =>
        filters.tags!.some((tag) => t.tags.includes(tag))
      );
    }

    // Apply customizable filter
    if (filters?.customizableOnly) {
      results = results.filter((t) => t.customizable);
    }

    return {
      templates: results,
      total: results.length,
      query,
      filters: filters || {},
    };
  }

  /**
   * Get templates by category
   */
  getByCategory(category: TemplateCategory): ComponentTemplate[] {
    return this.getAll().filter((t) => t.category === category);
  }

  /**
   * Get templates by complexity
   */
  getByComplexity(complexity: TemplateComplexity): ComponentTemplate[] {
    return this.getAll().filter((t) => t.complexity === complexity);
  }

  /**
   * Get all unique tags
   */
  getAllTags(): string[] {
    const tags = new Set<string>();
    for (const template of this.getAll()) {
      for (const tag of template.tags) {
        tags.add(tag);
      }
    }
    return Array.from(tags).sort();
  }

  /**
   * Add a custom template
   */
  addCustomTemplate(template: ComponentTemplate): void {
    if (this.templates.has(template.id)) {
      throw new Error(`Template ID '${template.id}' conflicts with built-in template`);
    }
    this.customTemplates.set(template.id, template);
  }

  /**
   * Remove a custom template
   */
  removeCustomTemplate(id: string): boolean {
    return this.customTemplates.delete(id);
  }

  /**
   * Apply customization to a template
   */
  applyCustomization(customization: TemplateCustomization): ComponentTemplate {
    const baseTemplate = this.get(customization.templateId);
    if (!baseTemplate) {
      throw new Error(`Template '${customization.templateId}' not found`);
    }

    // Clone the template
    const newTemplate: ComponentTemplate = JSON.parse(JSON.stringify(baseTemplate));

    // Apply field customizations
    for (const fieldCustom of customization.fieldCustomizations) {
      const field = newTemplate.inputSchema.fields.find(
        (f) => f.name === fieldCustom.originalName
      );
      if (field) {
        if (fieldCustom.newName) field.name = fieldCustom.newName;
        if (fieldCustom.newType) {
          field.rustType = fieldCustom.newType;
          // TypeScript type would need mapping
        }
        if (fieldCustom.newRequired !== undefined) {
          field.required = fieldCustom.newRequired;
        }
        if (fieldCustom.newDefault !== undefined) {
          field.default = fieldCustom.newDefault;
        }
        if (fieldCustom.newDescription) {
          field.description = fieldCustom.newDescription;
        }
      }
    }

    // Remove fields
    for (const fieldName of customization.removedFields) {
      newTemplate.inputSchema.fields = newTemplate.inputSchema.fields.filter(
        (f) => f.name !== fieldName
      );
    }

    // Add additional fields
    newTemplate.inputSchema.fields.push(...customization.additionalFields);

    // Add custom validation rules
    newTemplate.validationRules.push(...customization.customValidation);

    // Update metadata
    newTemplate.id = `custom-${customization.componentName}`;
    newTemplate.name = customization.componentName;
    newTemplate.author = 'Custom';
    newTemplate.category = 'custom';

    return newTemplate;
  }

  /**
   * Get library metadata
   */
  getLibraryInfo(): TemplateLibrary {
    const templates = this.getAll();

    const byCategory: Record<TemplateCategory, number> = {
      communication: 0,
      data: 0,
      integration: 0,
      control: 0,
      ai: 0,
      custom: 0,
    };

    const byComplexity: Record<TemplateComplexity, number> = {
      simple: 0,
      moderate: 0,
      advanced: 0,
    };

    for (const t of templates) {
      byCategory[t.category]++;
      byComplexity[t.complexity]++;
    }

    return {
      version: '1.0.0',
      lastUpdated: new Date().toISOString(),
      templateCount: templates.length,
      byCategory,
      byComplexity,
    };
  }
}

/** Singleton instance */
let libraryInstance: ComponentTemplateLibrary | null = null;

/**
 * Get the global template library instance
 */
export function getTemplateLibrary(): ComponentTemplateLibrary {
  if (!libraryInstance) {
    libraryInstance = new ComponentTemplateLibrary();
  }
  return libraryInstance;
}

/**
 * Get all built-in templates
 */
export function getBuiltInTemplates(): ComponentTemplate[] {
  return [...BUILT_IN_TEMPLATES];
}
