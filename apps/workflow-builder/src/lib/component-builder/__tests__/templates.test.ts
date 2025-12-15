/**
 * Template Library Tests
 *
 * Tests for the component template system.
 */

import { describe, it, expect, beforeEach } from 'vitest';
import {
  ComponentTemplateLibrary,
  getTemplateLibrary,
  getBuiltInTemplates,
} from '../templates/library';
import { emailSenderTemplate } from '../templates/built-in/email-sender';
import { webhookTemplate } from '../templates/built-in/webhook';
import { databaseQueryTemplate } from '../templates/built-in/database-query';

describe('ComponentTemplateLibrary', () => {
  let library: ComponentTemplateLibrary;

  beforeEach(() => {
    library = new ComponentTemplateLibrary();
  });

  describe('getAll', () => {
    it('should return all built-in templates', () => {
      const templates = library.getAll();

      expect(templates.length).toBeGreaterThanOrEqual(3);
      expect(templates.some((t) => t.id === 'email-sender')).toBe(true);
      expect(templates.some((t) => t.id === 'webhook')).toBe(true);
      expect(templates.some((t) => t.id === 'database-query')).toBe(true);
    });
  });

  describe('get', () => {
    it('should retrieve template by ID', () => {
      const template = library.get('email-sender');

      expect(template).toBeDefined();
      expect(template?.id).toBe('email-sender');
      expect(template?.name).toBe('Email Sender');
    });

    it('should return undefined for non-existent template', () => {
      const template = library.get('non-existent');
      expect(template).toBeUndefined();
    });
  });

  describe('search', () => {
    it('should search by name', () => {
      const results = library.search('email');

      expect(results.templates.length).toBeGreaterThanOrEqual(1);
      expect(results.templates[0]?.id).toBe('email-sender');
    });

    it('should search by tags', () => {
      const results = library.search('smtp');

      expect(results.templates.length).toBeGreaterThanOrEqual(1);
      expect(results.templates.some((t) => t.id === 'email-sender')).toBe(true);
    });

    it('should filter by category', () => {
      const results = library.search('', { category: 'communication' });

      expect(results.templates.every((t) => t.category === 'communication')).toBe(true);
    });

    it('should filter by complexity', () => {
      const results = library.search('', { complexity: 'simple' });

      expect(results.templates.every((t) => t.complexity === 'simple')).toBe(true);
    });

    it('should filter customizable only', () => {
      const results = library.search('', { customizableOnly: true });

      expect(results.templates.every((t) => t.customizable)).toBe(true);
    });
  });

  describe('getByCategory', () => {
    it('should return templates by category', () => {
      const templates = library.getByCategory('communication');

      expect(templates.length).toBeGreaterThanOrEqual(1);
      expect(templates.every((t) => t.category === 'communication')).toBe(true);
    });
  });

  describe('getByComplexity', () => {
    it('should return templates by complexity', () => {
      const templates = library.getByComplexity('moderate');

      expect(templates.length).toBeGreaterThanOrEqual(1);
      expect(templates.every((t) => t.complexity === 'moderate')).toBe(true);
    });
  });

  describe('getAllTags', () => {
    it('should return unique tags', () => {
      const tags = library.getAllTags();

      expect(tags.length).toBeGreaterThan(0);
      expect(tags.includes('email')).toBe(true);
      expect(tags.includes('webhook')).toBe(true);
      // Should be unique
      const uniqueTags = new Set(tags);
      expect(uniqueTags.size).toBe(tags.length);
    });
  });

  describe('addCustomTemplate', () => {
    it('should add custom template', () => {
      const customTemplate = {
        ...emailSenderTemplate,
        id: 'custom-template',
        name: 'Custom Template',
        category: 'custom' as const,
      };

      library.addCustomTemplate(customTemplate);

      const retrieved = library.get('custom-template');
      expect(retrieved).toBeDefined();
      expect(retrieved?.name).toBe('Custom Template');
    });

    it('should throw on ID conflict with built-in', () => {
      const conflicting = {
        ...emailSenderTemplate,
        id: 'email-sender', // Conflicts with built-in
      };

      expect(() => library.addCustomTemplate(conflicting)).toThrow();
    });
  });

  describe('removeCustomTemplate', () => {
    it('should remove custom template', () => {
      const customTemplate = {
        ...emailSenderTemplate,
        id: 'to-remove',
        name: 'To Remove',
        category: 'custom' as const,
      };

      library.addCustomTemplate(customTemplate);
      expect(library.get('to-remove')).toBeDefined();

      const removed = library.removeCustomTemplate('to-remove');
      expect(removed).toBe(true);
      expect(library.get('to-remove')).toBeUndefined();
    });

    it('should return false for non-existent template', () => {
      const removed = library.removeCustomTemplate('non-existent');
      expect(removed).toBe(false);
    });
  });

  describe('applyCustomization', () => {
    it('should customize template', () => {
      const customized = library.applyCustomization({
        templateId: 'email-sender',
        componentName: 'Custom Email',
        fieldCustomizations: [
          {
            originalName: 'to',
            newDescription: 'Updated description',
          },
        ],
        additionalFields: [],
        removedFields: [],
        customValidation: [],
      });

      expect(customized.name).toBe('Custom Email');
      expect(customized.category).toBe('custom');
    });

    it('should throw for non-existent template', () => {
      expect(() =>
        library.applyCustomization({
          templateId: 'non-existent',
          componentName: 'Test',
          fieldCustomizations: [],
          additionalFields: [],
          removedFields: [],
          customValidation: [],
        })
      ).toThrow();
    });
  });

  describe('getLibraryInfo', () => {
    it('should return library metadata', () => {
      const info = library.getLibraryInfo();

      expect(info.version).toBeDefined();
      expect(info.templateCount).toBeGreaterThanOrEqual(3);
      expect(info.byCategory).toBeDefined();
      expect(info.byComplexity).toBeDefined();
    });
  });
});

describe('Built-in Templates', () => {
  describe('emailSenderTemplate', () => {
    it('should have required fields', () => {
      expect(emailSenderTemplate.id).toBe('email-sender');
      expect(emailSenderTemplate.category).toBe('communication');
      expect(emailSenderTemplate.inputSchema.fields.length).toBeGreaterThan(0);
      expect(emailSenderTemplate.outputSchema.fields.length).toBeGreaterThan(0);
    });

    it('should have required input fields', () => {
      const requiredFields = emailSenderTemplate.inputSchema.fields.filter(
        (f) => f.required
      );
      expect(requiredFields.some((f) => f.name === 'to')).toBe(true);
      expect(requiredFields.some((f) => f.name === 'subject')).toBe(true);
      expect(requiredFields.some((f) => f.name === 'body')).toBe(true);
    });
  });

  describe('webhookTemplate', () => {
    it('should have required fields', () => {
      expect(webhookTemplate.id).toBe('webhook');
      expect(webhookTemplate.category).toBe('integration');
      expect(webhookTemplate.inputSchema.fields.length).toBeGreaterThan(0);
    });

    it('should have url as required', () => {
      const urlField = webhookTemplate.inputSchema.fields.find(
        (f) => f.name === 'url'
      );
      expect(urlField).toBeDefined();
      expect(urlField?.required).toBe(true);
    });
  });

  describe('databaseQueryTemplate', () => {
    it('should have required fields', () => {
      expect(databaseQueryTemplate.id).toBe('database-query');
      expect(databaseQueryTemplate.category).toBe('data');
      expect(databaseQueryTemplate.inputSchema.fields.length).toBeGreaterThan(0);
    });

    it('should have query and connection_id as required', () => {
      const queryField = databaseQueryTemplate.inputSchema.fields.find(
        (f) => f.name === 'query'
      );
      const connectionField = databaseQueryTemplate.inputSchema.fields.find(
        (f) => f.name === 'connection_id'
      );
      expect(queryField?.required).toBe(true);
      expect(connectionField?.required).toBe(true);
    });
  });
});

describe('getTemplateLibrary', () => {
  it('should return singleton instance', () => {
    const library1 = getTemplateLibrary();
    const library2 = getTemplateLibrary();

    expect(library1).toBe(library2);
  });
});

describe('getBuiltInTemplates', () => {
  it('should return all built-in templates', () => {
    const templates = getBuiltInTemplates();

    expect(templates.length).toBe(3);
    expect(templates.some((t) => t.id === 'email-sender')).toBe(true);
    expect(templates.some((t) => t.id === 'webhook')).toBe(true);
    expect(templates.some((t) => t.id === 'database-query')).toBe(true);
  });
});
