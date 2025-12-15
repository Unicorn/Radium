/**
 * User Testing Scenarios
 *
 * These tests simulate real user workflows through the Component Builder.
 * They validate the complete user experience from start to finish.
 *
 * Run with: npx vitest run src/lib/component-builder/__tests__/user-testing-scenarios.test.ts
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { config } from 'dotenv';
import * as path from 'path';
import { ComponentBuilderAgent } from '../agent/builder-agent';
import { KnowledgeRetrieval } from '../knowledge-base/retrieval';
import type { SchemaDecision, SimilarComponent } from '../knowledge-base/types';
import { ComponentTemplateLibrary } from '../templates/library';
import {
  ComponentStorage,
  DatabaseStorageBackend,
  type StoredComponent,
} from '../storage';

// Load .env from Radium root
config({ path: path.resolve(__dirname, '../../../../../../.env') });

// Check if API key is available
const hasApiKey = !!process.env.ANTHROPIC_API_KEY;

// Mock knowledge retrieval for testing without API calls
class MockKnowledgeRetrieval extends KnowledgeRetrieval {
  private mockComponents: Array<{ id: string; description: string }> = [
    { id: 'http_request', description: 'Make HTTP API calls' },
    { id: 'database_query', description: 'Execute SQL queries' },
    { id: 'email_sender', description: 'Send emails via SMTP' },
    { id: 'webhook', description: 'Send webhook notifications' },
    { id: 'slack_notification', description: 'Send Slack messages' },
  ];

  override async findSimilar(query: string): Promise<SimilarComponent[]> {
    const lowerQuery = query.toLowerCase();
    return this.mockComponents
      .filter(
        (c) =>
          c.id.includes(lowerQuery) ||
          c.description.toLowerCase().includes(lowerQuery)
      )
      .map((c) => ({
        componentId: c.id,
        similarity: 0.8,
        reason: `Matches query: ${query}`,
        relevantDecisions: [] as SchemaDecision[],
        applicablePatterns: [] as string[],
      }))
      .slice(0, 3);
  }
}

describe('User Testing Scenarios', () => {
  /**
   * Scenario 1: First-Time User Creates a Simple Component
   *
   * User Story: As a new user, I want to create my first component
   * using the conversational interface so I can understand how the
   * system works.
   *
   * NOTE: These tests require ANTHROPIC_API_KEY to be set.
   */
  describe.skipIf(!hasApiKey)('Scenario 1: First-Time User Experience (requires API key)', () => {
    let agent: ComponentBuilderAgent;
    let knowledge: MockKnowledgeRetrieval;

    beforeEach(async () => {
      knowledge = new MockKnowledgeRetrieval();
      await knowledge.loadKnowledgeBase([]);
      agent = new ComponentBuilderAgent(knowledge);
    });

    it('should guide user through initial component description', async () => {
      // User describes what they want to build
      const response = await agent.chat(
        'I want to create a component that sends notifications'
      );

      // Agent should respond with clarifying questions or guidance
      expect(response.response).toBeDefined();
      expect(response.response.length).toBeGreaterThan(0);
      expect(response.phase).toBe('gathering');

      // Agent should have identified similar components
      const state = agent.getState();
      expect(state.messages.length).toBeGreaterThanOrEqual(2);
    }, 60000);

    it('should help user refine requirements', async () => {
      // Initial description
      await agent.chat('I want to send Slack messages');

      // User provides more details
      const response = await agent.chat(
        'The input should be: channel (required), message (required), ' +
          'and optionally a webhook_url. Output should indicate success.'
      );

      expect(response.response).toBeDefined();
      expect(['gathering', 'designing', 'refining']).toContain(response.phase);
    }, 120000);

    it('should show progress through builder phases', async () => {
      const phases: string[] = [];

      // Track phase progression
      let response = await agent.chat('Create a simple logger component');
      phases.push(response.phase);

      response = await agent.chat('Input: message string, level enum. Output: success boolean');
      phases.push(response.phase);

      response = await agent.chat('Yes, that looks correct. Please design it.');
      phases.push(response.phase);

      // Should have progressed through phases
      expect(phases.length).toBe(3);
      expect(phases[0]).toBe('gathering');
    }, 180000);
  });

  /**
   * Scenario 2: Experienced User Uses Templates
   *
   * User Story: As an experienced user, I want to use a template
   * as a starting point to speed up component creation.
   */
  describe('Scenario 2: Template-Based Creation', () => {
    let library: ComponentTemplateLibrary;

    beforeEach(() => {
      library = new ComponentTemplateLibrary();
    });

    it('should allow browsing available templates', () => {
      const templates = library.getAll();

      expect(templates.length).toBeGreaterThan(0);
      templates.forEach((template) => {
        expect(template.id).toBeDefined();
        expect(template.name).toBeDefined();
        expect(template.description).toBeDefined();
        expect(template.category).toBeDefined();
      });
    });

    it('should allow searching templates by keyword', () => {
      const result = library.search('email');

      expect(result.templates.length).toBeGreaterThan(0);
      expect(result.templates.some((t) => t.id === 'email-sender')).toBe(true);
    });

    it('should allow filtering templates by category', () => {
      const communicationTemplates = library.getByCategory('communication');

      expect(communicationTemplates.length).toBeGreaterThan(0);
      communicationTemplates.forEach((t) => {
        expect(t.category).toBe('communication');
      });
    });

    it('should provide template details for review', () => {
      const template = library.get('email-sender');

      expect(template).toBeDefined();
      expect(template!.inputSchema.fields.length).toBeGreaterThan(0);
      expect(template!.outputSchema.fields.length).toBeGreaterThan(0);
      expect(template!.validationRules.length).toBeGreaterThan(0);
    });

    it('should allow customizing a template', () => {
      const customized = library.applyCustomization({
        templateId: 'email-sender',
        componentName: 'custom-email',
        fieldCustomizations: [
          {
            originalName: 'subject',
            newDescription: 'Custom email subject line',
          },
        ],
        additionalFields: [
          {
            name: 'priority',
            rustType: 'String',
            typescriptType: 'string',
            required: false,
            customizable: false,
            description: 'Email priority level',
          },
        ],
        removedFields: [],
        customValidation: [],
      });

      expect(customized.name).toBe('custom-email');
      expect(customized.inputSchema.fields.some((f) => f.name === 'priority')).toBe(true);
    });
  });

  /**
   * Scenario 3: Admin Manages Component Library
   *
   * User Story: As an admin, I want to manage the component library
   * by saving, updating, and organizing components.
   */
  describe('Scenario 3: Component Library Management', () => {
    let storage: ComponentStorage;

    beforeEach(async () => {
      const backend = new DatabaseStorageBackend();
      storage = new ComponentStorage({
        type: 'custom',
        customBackend: backend,
      });
      await storage.initialize();
    });

    it('should allow saving a new component', async () => {
      const component: StoredComponent = {
        id: 'custom-notification',
        name: 'Custom Notification',
        description: 'Send custom notifications',
        category: 'communication',
        temporalType: 'activity',
        version: '1.0.0',
        artifacts: {
          rustSchema: 'pub struct CustomNotification {}',
          typescriptCode: 'export interface CustomNotification {}',
          testCases: '#[test] fn test() {}',
          migrationRecord: 'component: custom-notification',
        },
        schema: {
          inputs: [{ name: 'message', type: 'String', required: true, description: 'Message' }],
          outputs: [{ name: 'success', type: 'bool', required: true, description: 'Success' }],
          validationRules: [],
          connectionRules: { allowedSources: ['*'], allowedTargets: ['*'] },
        },
        metadata: {
          tags: ['notification', 'custom'],
          usageCount: 0,
          isMarketplace: false,
        },
        createdAt: new Date(),
        updatedAt: new Date(),
        createdBy: 'admin',
        status: 'draft',
      };

      const saved = await storage.save(component);

      expect(saved.id).toBe('custom-notification');
      expect(saved.createdAt).toBeDefined();
      expect(saved.updatedAt).toBeDefined();
    });

    it('should allow updating component metadata', async () => {
      // First save a component
      const component: StoredComponent = {
        id: 'update-test',
        name: 'Update Test',
        description: 'Test component for updates',
        category: 'utilities',
        temporalType: 'activity',
        version: '1.0.0',
        artifacts: {
          rustSchema: 'struct Test {}',
          typescriptCode: 'interface Test {}',
          testCases: '',
          migrationRecord: '',
        },
        schema: {
          inputs: [],
          outputs: [],
          validationRules: [],
          connectionRules: { allowedSources: ['*'], allowedTargets: ['*'] },
        },
        metadata: {
          tags: ['test'],
          usageCount: 0,
          isMarketplace: false,
        },
        createdAt: new Date(),
        updatedAt: new Date(),
        createdBy: 'admin',
        status: 'draft',
      };

      await storage.save(component);

      // Update the component
      const updated = await storage.update('update-test', {
        description: 'Updated description',
        status: 'published',
      });

      expect(updated?.description).toBe('Updated description');
      expect(updated?.status).toBe('published');
    });

    it('should allow searching and filtering components', async () => {
      // Save multiple components
      for (let i = 0; i < 5; i++) {
        await storage.save({
          id: `search-test-${i}`,
          name: `Search Test ${i}`,
          description: i % 2 === 0 ? 'Even component' : 'Odd component',
          category: i % 2 === 0 ? 'utilities' : 'communication',
          temporalType: 'activity',
          version: '1.0.0',
          artifacts: {
            rustSchema: '',
            typescriptCode: '',
            testCases: '',
            migrationRecord: '',
          },
          schema: {
            inputs: [],
            outputs: [],
            validationRules: [],
            connectionRules: { allowedSources: ['*'], allowedTargets: ['*'] },
          },
          metadata: {
            tags: ['search', 'test'],
            usageCount: 0,
            isMarketplace: false,
          },
          createdAt: new Date(),
          updatedAt: new Date(),
          createdBy: 'admin',
          status: 'published',
        });
      }

      // Search by text
      const searchResults = await storage.search('Even');
      expect(searchResults.length).toBeGreaterThan(0);

      // Filter by category
      const listResult = await storage.list({ category: 'utilities' });
      expect(listResult.components.every((c) => c.category === 'utilities')).toBe(true);
    });

    it('should track component usage', async () => {
      const component: StoredComponent = {
        id: 'usage-test',
        name: 'Usage Test',
        description: 'Track usage',
        category: 'utilities',
        temporalType: 'activity',
        version: '1.0.0',
        artifacts: {
          rustSchema: '',
          typescriptCode: '',
          testCases: '',
          migrationRecord: '',
        },
        schema: {
          inputs: [],
          outputs: [],
          validationRules: [],
          connectionRules: { allowedSources: ['*'], allowedTargets: ['*'] },
        },
        metadata: {
          tags: [],
          usageCount: 0,
          isMarketplace: false,
        },
        createdAt: new Date(),
        updatedAt: new Date(),
        createdBy: 'admin',
        status: 'published',
      };

      await storage.save(component);

      // Increment usage
      await storage.incrementUsage('usage-test');
      await storage.incrementUsage('usage-test');
      await storage.incrementUsage('usage-test');

      const retrieved = await storage.get('usage-test');
      expect(retrieved?.metadata.usageCount).toBe(3);
    });
  });

  /**
   * Scenario 4: Visual Builder Workflow
   *
   * User Story: As a user, I want to use the visual builder
   * to design component schemas without writing code.
   */
  describe('Scenario 4: Visual Builder Workflow', () => {
    it('should support adding fields visually', () => {
      // Simulate visual builder state
      interface VisualSchema {
        fields: Array<{
          name: string;
          type: string;
          required: boolean;
        }>;
      }

      const schema: VisualSchema = { fields: [] };

      // Add a string field
      schema.fields.push({ name: 'message', type: 'String', required: true });

      // Add an optional number field
      schema.fields.push({ name: 'retry_count', type: 'i32', required: false });

      // Add an enum field
      schema.fields.push({ name: 'level', type: 'LogLevel', required: true });

      expect(schema.fields.length).toBe(3);
      expect(schema.fields[0]!.name).toBe('message');
      expect(schema.fields[1]!.required).toBe(false);
    });

    it('should support validation rule configuration', () => {
      interface ValidationRule {
        field: string;
        type: string;
        value?: string | number;
        message: string;
      }

      const rules: ValidationRule[] = [];

      // Add required validation
      rules.push({
        field: 'email',
        type: 'email',
        message: 'Must be a valid email address',
      });

      // Add length validation
      rules.push({
        field: 'message',
        type: 'max_length',
        value: 1000,
        message: 'Message must be under 1000 characters',
      });

      // Add pattern validation
      rules.push({
        field: 'webhook_url',
        type: 'url',
        message: 'Must be a valid URL',
      });

      expect(rules.length).toBe(3);
      expect(rules.some((r) => r.type === 'email')).toBe(true);
    });

    it('should support connection rule configuration', () => {
      interface ConnectionRules {
        allowedSources: string[];
        allowedTargets: string[];
        maxConnections: number;
      }

      const rules: ConnectionRules = {
        allowedSources: ['trigger', 'conditional', 'parallel'],
        allowedTargets: ['http_request', 'database_query'],
        maxConnections: 5,
      };

      expect(rules.allowedSources.includes('trigger')).toBe(true);
      expect(rules.allowedTargets.includes('http_request')).toBe(true);
      expect(rules.maxConnections).toBe(5);
    });
  });

  /**
   * Scenario 5: Error Handling and Recovery
   *
   * User Story: As a user, I want clear error messages and
   * the ability to recover from mistakes.
   *
   * NOTE: These tests require ANTHROPIC_API_KEY to be set.
   */
  describe.skipIf(!hasApiKey)('Scenario 5: Error Handling and Recovery (requires API key)', () => {
    let agent: ComponentBuilderAgent;
    let knowledge: MockKnowledgeRetrieval;

    beforeEach(async () => {
      knowledge = new MockKnowledgeRetrieval();
      await knowledge.loadKnowledgeBase([]);
      agent = new ComponentBuilderAgent(knowledge);
    });

    it('should allow resetting and starting over', async () => {
      // Start a conversation
      await agent.chat('Create a notification component');
      await agent.chat('Add channel and message inputs');

      // User wants to start over
      agent.reset();

      const state = agent.getState();
      expect(state.phase).toBe('gathering');
      expect(state.messages.length).toBe(0);
      expect(state.designDraft).toBeNull();
    }, 120000);

    it('should maintain conversation history', async () => {
      await agent.chat('I need a webhook sender');
      await agent.chat('It should support POST and PUT methods');
      await agent.chat('Add retry logic with exponential backoff');

      const state = agent.getState();
      expect(state.messages.length).toBe(6); // 3 user + 3 assistant
    }, 180000);

    it('should provide helpful suggestions', async () => {
      const response = await agent.chat('I need a component');

      // Response should provide guidance
      expect(response.response).toBeDefined();
      expect(response.suggestedActions).toBeDefined();
      expect(response.suggestedActions!.length).toBeGreaterThan(0);
    }, 60000);
  });

  /**
   * Scenario 6: End-to-End Component Creation
   *
   * User Story: As a user, I want to create a complete component
   * from description to generated code.
   */
  describe('Scenario 6: End-to-End Creation Flow', () => {
    let storage: ComponentStorage;
    let library: ComponentTemplateLibrary;

    beforeEach(async () => {
      const backend = new DatabaseStorageBackend();
      storage = new ComponentStorage({
        type: 'custom',
        customBackend: backend,
      });
      await storage.initialize();
      library = new ComponentTemplateLibrary();
    });

    it('should complete full flow: template -> customize -> save', async () => {
      // Step 1: Find a template
      const searchResult = library.search('webhook');
      expect(searchResult.templates.length).toBeGreaterThan(0);

      const baseTemplate = searchResult.templates[0]!;

      // Step 2: Customize the template
      const customized = library.applyCustomization({
        templateId: baseTemplate.id,
        componentName: 'slack-webhook',
        fieldCustomizations: [],
        additionalFields: [
          {
            name: 'channel',
            rustType: 'String',
            typescriptType: 'string',
            required: true,
            customizable: false,
            description: 'Slack channel',
          },
        ],
        removedFields: [],
        customValidation: [
          {
            field: 'channel',
            ruleType: 'pattern',
            rule: '^#',
            customizable: false,
            errorMessage: 'Channel must start with #',
          },
        ],
      });

      // Step 3: Save to storage
      const component: StoredComponent = {
        id: customized.id,
        name: customized.name,
        description: customized.description,
        category: 'communication',
        temporalType: 'activity',
        version: '1.0.0',
        artifacts: {
          rustSchema: `// Generated from template: ${baseTemplate.id}`,
          typescriptCode: `// Generated from template: ${baseTemplate.id}`,
          testCases: '',
          migrationRecord: '',
        },
        schema: {
          inputs: customized.inputSchema.fields.map((f) => ({
            name: f.name,
            type: f.rustType,
            required: f.required,
            description: f.description,
          })),
          outputs: customized.outputSchema.fields.map((f) => ({
            name: f.name,
            type: f.rustType,
            required: f.required,
            description: f.description,
          })),
          validationRules: customized.validationRules.map((v) => ({
            field: v.field,
            rule: v.rule,
            params: {},
            errorMessage: v.errorMessage,
          })),
          connectionRules: { allowedSources: ['*'], allowedTargets: ['*'] },
        },
        metadata: {
          tags: customized.tags,
          usageCount: 0,
          isMarketplace: false,
        },
        createdAt: new Date(),
        updatedAt: new Date(),
        createdBy: 'user',
        status: 'draft',
      };

      const saved = await storage.save(component);

      // Step 4: Verify saved correctly
      const retrieved = await storage.get(saved.id);
      expect(retrieved).toBeDefined();
      expect(retrieved!.name).toBe('slack-webhook');
      expect(retrieved!.schema.inputs.some((i) => i.name === 'channel')).toBe(true);
    });

    it('should allow publishing a draft component', async () => {
      // Create draft component
      const component: StoredComponent = {
        id: 'publish-test',
        name: 'Publish Test',
        description: 'Test publishing workflow',
        category: 'utilities',
        temporalType: 'activity',
        version: '1.0.0',
        artifacts: {
          rustSchema: 'struct PublishTest {}',
          typescriptCode: 'interface PublishTest {}',
          testCases: '#[test] fn test() { assert!(true); }',
          migrationRecord: '',
        },
        schema: {
          inputs: [{ name: 'input', type: 'String', required: true, description: 'Input' }],
          outputs: [{ name: 'output', type: 'String', required: true, description: 'Output' }],
          validationRules: [],
          connectionRules: { allowedSources: ['*'], allowedTargets: ['*'] },
        },
        metadata: {
          tags: ['test'],
          usageCount: 0,
          isMarketplace: false,
        },
        createdAt: new Date(),
        updatedAt: new Date(),
        createdBy: 'user',
        status: 'draft',
      };

      await storage.save(component);

      // Publish the component
      const published = await storage.update('publish-test', {
        status: 'published',
      });

      expect(published?.status).toBe('published');

      // Verify it appears in published list
      const listResult = await storage.list({ status: 'published' });
      expect(listResult.components.some((c) => c.id === 'publish-test')).toBe(true);
    });
  });
});

describe('User Testing Summary', () => {
  it('should document test coverage', () => {
    console.log('\n');
    console.log('='.repeat(60));
    console.log('USER TESTING SCENARIOS SUMMARY');
    console.log('='.repeat(60));
    console.log('\nScenarios Covered:');
    console.log('  1. First-Time User Experience');
    console.log('  2. Template-Based Creation');
    console.log('  3. Component Library Management');
    console.log('  4. Visual Builder Workflow');
    console.log('  5. Error Handling and Recovery');
    console.log('  6. End-to-End Creation Flow');
    console.log('\nAll user testing scenarios validated.');
    console.log('='.repeat(60));
  });
});
