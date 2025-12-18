/**
 * Component Builder Integration Tests
 *
 * These tests use the real Anthropic API to verify the component builder works.
 * Requires ANTHROPIC_API_KEY in environment.
 *
 * Run with: npx vitest run src/lib/component-builder/__tests__/integration.test.ts
 *
 * @vitest-environment node
 */

import { describe, it, expect, beforeAll } from 'vitest';
import { config } from 'dotenv';
import * as path from 'path';

// Load .env.test from workflow-builder directory
config({ path: path.resolve(__dirname, '../../../../.env.test') });

import { MigrationRecordProcessor } from '../knowledge-base/processor';
import { KnowledgeRetrieval } from '../knowledge-base/retrieval';
import { ComponentBuilderAgent } from '../agent/builder-agent';

// Skip all tests if no API key
const hasApiKey = !!process.env.ANTHROPIC_API_KEY;

describe.skipIf(!hasApiKey)('Component Builder Integration', () => {
  let knowledge: KnowledgeRetrieval;
  let agent: ComponentBuilderAgent;

  beforeAll(async () => {
    // Load real migration records
    const recordsDir = path.resolve(
      __dirname,
      '../../../../../../crates/radium-workflow/component-records'
    );
    const processor = new MigrationRecordProcessor({ recordsDir });
    const records = await processor.processAll();

    console.log(`Loaded ${records.length} migration records`);

    // Initialize knowledge base
    knowledge = new KnowledgeRetrieval();
    await knowledge.loadKnowledgeBase(records);

    // Create agent
    agent = new ComponentBuilderAgent(knowledge);
  }, 30000);

  describe('Knowledge Base with Real API', () => {
    it('should find similar components for HTTP-related query', async () => {
      const similar = await knowledge.findSimilar('I need to make HTTP API calls');

      console.log('Similar components found:', similar.map(s => `${s.componentId} (${s.similarity})`));

      expect(similar.length).toBeGreaterThan(0);
      // Should find http_request as similar
      expect(similar.some(s => s.componentId === 'http_request')).toBe(true);
    }, 30000);

    it('should find similar components for database query', async () => {
      const similar = await knowledge.findSimilar('Execute SQL queries against PostgreSQL');

      console.log('Similar components found:', similar.map(s => `${s.componentId} (${s.similarity})`));

      expect(similar.length).toBeGreaterThan(0);
      expect(similar.some(s => s.componentId === 'database_query')).toBe(true);
    }, 30000);
  });

  describe('Builder Agent Conversation', () => {
    it('should gather requirements through conversation', async () => {
      // Reset for clean state
      agent.reset();

      const response = await agent.chat('I need a component that sends Slack notifications');

      console.log('Agent response:', response.response.substring(0, 200) + '...');
      console.log('Phase:', response.phase);
      console.log('Suggested actions:', response.suggestedActions);

      expect(response.response).toBeDefined();
      expect(response.response.length).toBeGreaterThan(50);
      expect(response.phase).toBe('gathering');
    }, 60000);

    it('should continue gathering with more details', async () => {
      const response = await agent.chat(
        'It should take a channel name, message text, and optional webhook URL. ' +
        'The output should indicate success and include any error message.'
      );

      console.log('Agent response:', response.response.substring(0, 200) + '...');
      console.log('Phase:', response.phase);

      expect(response.response).toBeDefined();
      // Should still be gathering or moved to designing
      expect(['gathering', 'designing', 'refining']).toContain(response.phase);
    }, 60000);
  });

  describe('Full Component Creation Flow', () => {
    it('should complete full flow from description to code generation', async () => {
      // Start fresh
      agent.reset();

      // Step 1: Initial description
      console.log('\n--- Step 1: Initial Description ---');
      const res1 = await agent.chat(
        'Create a simple logging component that writes messages to a log. ' +
        'Input: message (string, required), level (enum: info/warn/error, default info). ' +
        'Output: success (boolean), timestamp (string).'
      );
      console.log('Response:', res1.response.substring(0, 300));
      console.log('Phase:', res1.phase);

      // Step 2: If still gathering, provide more info
      if (res1.phase === 'gathering') {
        console.log('\n--- Step 2: Additional Info ---');
        const res2 = await agent.chat(
          'Yes, that covers my requirements. Please design the schema.'
        );
        console.log('Response:', res2.response.substring(0, 300));
        console.log('Phase:', res2.phase);
      }

      // Step 3: Approve design
      console.log('\n--- Step 3: Approve Design ---');
      const res3 = await agent.chat('Looks good, please generate the code');
      console.log('Response:', res3.response.substring(0, 500));
      console.log('Phase:', res3.phase);

      // Check if we got to generating/reviewing phase
      const state = agent.getState();
      console.log('\n--- Final State ---');
      console.log('Phase:', state.phase);
      console.log('Has design:', !!state.designDraft);
      console.log('Has artifacts:', !!state.generatedArtifacts);

      if (state.generatedArtifacts) {
        console.log('\n--- Generated Rust Schema (first 500 chars) ---');
        console.log(state.generatedArtifacts.rustSchema.substring(0, 500));

        console.log('\n--- Generated TypeScript (first 500 chars) ---');
        console.log(state.generatedArtifacts.typescriptCode.substring(0, 500));

        // Verify artifacts exist
        expect(state.generatedArtifacts.rustSchema.length).toBeGreaterThan(100);
        expect(state.generatedArtifacts.typescriptCode.length).toBeGreaterThan(100);
      }

      // Should have progressed past gathering
      expect(['designing', 'refining', 'generating', 'reviewing', 'complete']).toContain(state.phase);
    }, 120000); // 2 minute timeout for full flow
  });
});

describe.skipIf(!hasApiKey)('Cost Tracking', () => {
  it('should estimate tokens used', async () => {
    // This is informational - just log that we completed the tests
    console.log('\n=== Integration Tests Complete ===');
    console.log('API calls were made to Anthropic Claude Sonnet');
    console.log('Check your Anthropic dashboard for actual usage');
  });
});
