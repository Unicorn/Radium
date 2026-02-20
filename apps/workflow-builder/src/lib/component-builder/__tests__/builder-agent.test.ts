/**
 * Component Builder Agent Tests
 *
 * Tests for the AI-powered component builder agent.
 * Note: Tests that require the Anthropic API are marked with .skip
 * and can be run manually with ANTHROPIC_API_KEY set.
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { ComponentBuilderAgent } from '../agent/builder-agent';
import { KnowledgeRetrieval } from '../knowledge-base/retrieval';
import type { ProcessedRecord } from '../knowledge-base/types';

// Mock the Anthropic SDK
vi.mock('@anthropic-ai/sdk', () => {
  return {
    default: vi.fn().mockImplementation(() => ({
      messages: {
        create: vi.fn().mockResolvedValue({
          content: [
            {
              type: 'text',
              text: 'I understand you want to create a webhook component. Let me ask some clarifying questions about the inputs and outputs.',
            },
          ],
        }),
      },
    })),
  };
});

describe('ComponentBuilderAgent', () => {
  let agent: ComponentBuilderAgent;
  let knowledge: KnowledgeRetrieval;
  let mockRecords: ProcessedRecord[];

  beforeEach(async () => {
    // Create mock knowledge base
    mockRecords = [
      {
        id: 'http_request',
        content: 'Component: http_request\nCategory: activities\nDescription: HTTP request component',
        metadata: {
          name: 'http_request',
          category: 'activities',
          version: '1.0.0',
          description: 'HTTP request component for external API calls',
          temporalType: 'activity',
          complexity: 'medium',
          migrationDate: '2025-12-15',
          migrationDifficulty: 'medium',
        },
        patterns: {
          inputValidation: [],
          outputSchema: [],
          errorHandling: [],
          typescriptPatterns: [],
          rustPatterns: [],
        },
        inputSchema: {
          rustStruct: 'HttpRequestInput',
          typescriptInterface: 'HttpRequestInput',
          fields: [
            { name: 'url', rustType: 'String', typescriptType: 'string', required: true, description: 'URL to request' },
            { name: 'method', rustType: 'HttpMethod', typescriptType: 'string', required: false, description: 'HTTP method' },
          ],
          validation: [],
        },
        outputSchema: {
          rustStruct: 'HttpRequestOutput',
          typescriptInterface: 'HttpRequestOutput',
          fields: [
            { name: 'status', rustType: 'u16', typescriptType: 'number', required: true, description: 'Status code' },
          ],
          validation: [],
        },
        decisions: [
          {
            field: 'method',
            decision: 'Use enum for HTTP methods',
            rationale: 'Type safety and clear options',
            alternativesConsidered: [],
          },
        ],
        lessonsLearned: { whatWorkedWell: [], challenges: [], recommendations: [] },
        relatedComponents: [],
      },
    ];

    // Create knowledge retrieval with mocked Anthropic
    knowledge = new KnowledgeRetrieval({ model: 'claude-sonnet-4-20250514' });
    await knowledge.loadKnowledgeBase(mockRecords);

    // Create agent
    agent = new ComponentBuilderAgent(knowledge, {
      model: 'claude-sonnet-4-20250514',
    });
  });

  describe('Initial State', () => {
    it('should start in gathering phase', () => {
      expect(agent.getPhase()).toBe('gathering');
    });

    it('should have unique conversation ID', () => {
      const id = agent.getConversationId();
      expect(id).toBeDefined();
      expect(id.length).toBeGreaterThan(0);
    });

    it('should have empty message history initially', () => {
      const state = agent.getState();
      expect(state.messages).toHaveLength(0);
    });

    it('should have no design draft initially', () => {
      const state = agent.getState();
      expect(state.designDraft).toBeNull();
    });

    it('should have no generated artifacts initially', () => {
      const state = agent.getState();
      expect(state.generatedArtifacts).toBeNull();
    });
  });

  describe('chat()', () => {
    it('should return a response', async () => {
      const response = await agent.chat('I need a webhook sender component');

      expect(response).toBeDefined();
      expect(response.response).toBeDefined();
      expect(response.response.length).toBeGreaterThan(0);
    });

    it('should include current phase in response', async () => {
      const response = await agent.chat('I need a component');

      expect(response.phase).toBeDefined();
      expect(['gathering', 'designing', 'refining', 'generating', 'reviewing', 'complete']).toContain(response.phase);
    });

    it('should track phase changes', async () => {
      const response = await agent.chat('I need a component');

      expect(response.phaseChanged).toBeDefined();
      expect(typeof response.phaseChanged).toBe('boolean');
    });

    it('should add messages to history', async () => {
      await agent.chat('Create a webhook component');

      const state = agent.getState();
      expect(state.messages.length).toBe(2); // User message + assistant response
      expect(state.messages[0]?.role).toBe('user');
      expect(state.messages[1]?.role).toBe('assistant');
    });

    it('should include suggested actions', async () => {
      const response = await agent.chat('I need a component');

      expect(response.suggestedActions).toBeDefined();
      expect(Array.isArray(response.suggestedActions)).toBe(true);
    });
  });

  describe('reset()', () => {
    it('should reset to initial state', async () => {
      // Build up some state
      await agent.chat('Create a component');

      // Reset
      agent.reset();

      const state = agent.getState();
      expect(state.phase).toBe('gathering');
      expect(state.messages).toHaveLength(0);
      expect(state.designDraft).toBeNull();
      expect(state.generatedArtifacts).toBeNull();
    });

    it('should generate new conversation ID after reset', async () => {
      const originalId = agent.getConversationId();

      agent.reset();

      const newId = agent.getConversationId();
      expect(newId).not.toBe(originalId);
    });
  });

  describe('getState()', () => {
    it('should return a copy of state', () => {
      const state1 = agent.getState();
      const state2 = agent.getState();

      // Should be equal but not the same object
      expect(state1).toEqual(state2);
      expect(state1).not.toBe(state2);
    });

    it('should include requirement info', () => {
      const state = agent.getState();

      expect(state.requirement).toBeDefined();
      expect(state.requirement.description).toBeDefined();
      expect(state.requirement.inputs).toBeDefined();
      expect(state.requirement.outputs).toBeDefined();
    });
  });
});

describe('ComponentBuilderAgent - State Transitions', () => {
  let agent: ComponentBuilderAgent;
  let knowledge: KnowledgeRetrieval;

  beforeEach(async () => {
    knowledge = new KnowledgeRetrieval({ model: 'claude-sonnet-4-20250514' });
    await knowledge.loadKnowledgeBase([]);
    agent = new ComponentBuilderAgent(knowledge);
  });

  it('should remain in gathering phase for initial messages', async () => {
    await agent.chat('I need a component');
    expect(agent.getPhase()).toBe('gathering');
  });
});

describe('ComponentBuilderAgent - Helper Methods', () => {
  // Test internal helper methods via their effects

  describe('rustToTypeScript mapping', () => {
    let agent: ComponentBuilderAgent;
    let knowledge: KnowledgeRetrieval;

    beforeEach(async () => {
      knowledge = new KnowledgeRetrieval({ model: 'claude-sonnet-4-20250514' });
      await knowledge.loadKnowledgeBase([]);
      agent = new ComponentBuilderAgent(knowledge);
    });

    it('should create agent without errors', () => {
      expect(agent).toBeDefined();
      expect(agent.getPhase()).toBe('gathering');
    });
  });
});

// Integration tests that require actual Anthropic API
// Run with: ANTHROPIC_API_KEY=xxx npx vitest run --testNamePattern="Integration"
describe.skip('ComponentBuilderAgent - Integration Tests', () => {
  let agent: ComponentBuilderAgent;
  let knowledge: KnowledgeRetrieval;

  beforeEach(async () => {
    // Use real knowledge base
    const { MigrationRecordProcessor } = await import('../knowledge-base/processor');
    const path = await import('path');

    const recordsDir = path.resolve(
      __dirname,
      '../../../../../../crates/radium-workflow/component-records'
    );
    const processor = new MigrationRecordProcessor({ recordsDir });
    const records = await processor.processAll();

    knowledge = new KnowledgeRetrieval();
    await knowledge.loadKnowledgeBase(records);

    agent = new ComponentBuilderAgent(knowledge);
  });

  it('should complete full conversation flow', async () => {
    // This test requires ANTHROPIC_API_KEY to be set
    const response1 = await agent.chat('I need a component that sends Slack messages');
    expect(response1.response).toBeDefined();
    expect(response1.phase).toBe('gathering');

    const response2 = await agent.chat('It should take a channel, message, and optional webhook URL');
    expect(response2.response).toBeDefined();

    // Continue conversation until design phase...
  }, 60000); // 60 second timeout for API calls
});
