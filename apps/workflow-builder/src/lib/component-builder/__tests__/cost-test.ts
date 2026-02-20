/**
 * Component Builder Cost Test
 *
 * Run a simple component through the builder to measure API costs.
 *
 * Run with: npx tsx src/lib/component-builder/__tests__/cost-test.ts
 */

import { config } from 'dotenv';
import * as path from 'path';
import { fileURLToPath } from 'url';

// ESM equivalent of __dirname
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Load .env from Radium root
config({ path: path.resolve(__dirname, '../../../../../../.env') });

import { MigrationRecordProcessor } from '../knowledge-base/processor';
import { KnowledgeRetrieval } from '../knowledge-base/retrieval';
import { ComponentBuilderAgent } from '../agent/builder-agent';

interface TokenUsage {
  inputTokens: number;
  outputTokens: number;
}

const tokenUsage: TokenUsage = {
  inputTokens: 0,
  outputTokens: 0,
};

// Cost per 1M tokens (Claude Sonnet pricing as of Dec 2024)
const INPUT_COST_PER_M = 3.0; // $3 per 1M input tokens
const OUTPUT_COST_PER_M = 15.0; // $15 per 1M output tokens

function calculateCost(usage: TokenUsage): number {
  const inputCost = (usage.inputTokens / 1_000_000) * INPUT_COST_PER_M;
  const outputCost = (usage.outputTokens / 1_000_000) * OUTPUT_COST_PER_M;
  return inputCost + outputCost;
}

async function runTimestampGeneratorTest() {
  console.log('='.repeat(60));
  console.log('COMPONENT BUILDER COST TEST: Timestamp Generator');
  console.log('='.repeat(60));
  console.log();

  // Check for API key
  if (!process.env.ANTHROPIC_API_KEY) {
    console.error('ERROR: ANTHROPIC_API_KEY not found in environment');
    process.exit(1);
  }

  // Step 1: Load knowledge base
  console.log('Step 1: Loading knowledge base...');
  const recordsDir = path.resolve(
    __dirname,
    '../../../../../../crates/radium-workflow/component-records'
  );
  const processor = new MigrationRecordProcessor({ recordsDir });
  const records = await processor.processAll();
  console.log(`  Loaded ${records.length} migration records`);

  const knowledge = new KnowledgeRetrieval();
  await knowledge.loadKnowledgeBase(records);
  console.log('  Knowledge base initialized');
  console.log();

  // Step 2: Create agent
  console.log('Step 2: Creating component builder agent...');
  const agent = new ComponentBuilderAgent(knowledge);
  console.log(`  Conversation ID: ${agent.getConversationId()}`);
  console.log();

  // Step 3: Run conversation
  console.log('Step 3: Running conversation...');
  console.log('-'.repeat(60));

  const startTime = Date.now();
  let apiCalls = 0;

  // Message 1: Initial description
  console.log('\n[USER] Initial component description:');
  const msg1 = `Create a Timestamp Generator component.
Input: format (optional string, values like "ISO8601", "unix", "rfc2822", default "ISO8601")
Output: timestamp (string - formatted timestamp), unix_ms (number - milliseconds since epoch)
Description: Generates the current timestamp in various formats.`;
  console.log(msg1);

  console.log('\n[AGENT] Response:');
  const res1 = await agent.chat(msg1);
  apiCalls++;
  console.log(res1.response);
  console.log(`\n  Phase: ${res1.phase}`);
  console.log(`  Suggested actions: ${res1.suggestedActions?.join(', ') || 'none'}`);

  // Message 2: Confirm and request design
  if (res1.phase === 'gathering') {
    console.log('\n' + '-'.repeat(60));
    console.log('\n[USER] Confirming requirements:');
    const msg2 = `Yes, that's exactly right. The format field is optional with "ISO8601" as default.
No additional validation needed - just the enum values for format.
Please design the schema.`;
    console.log(msg2);

    console.log('\n[AGENT] Response:');
    const res2 = await agent.chat(msg2);
    apiCalls++;
    console.log(res2.response);
    console.log(`\n  Phase: ${res2.phase}`);
  }

  // Message 3: Approve and generate
  const state = agent.getState();
  if (state.phase === 'designing' || state.phase === 'refining') {
    console.log('\n' + '-'.repeat(60));
    console.log('\n[USER] Approving design:');
    const msg3 = 'Looks good! Please generate the code.';
    console.log(msg3);

    console.log('\n[AGENT] Response:');
    const res3 = await agent.chat(msg3);
    apiCalls++;
    console.log(res3.response);
    console.log(`\n  Phase: ${res3.phase}`);
  }

  const endTime = Date.now();
  const duration = (endTime - startTime) / 1000;

  // Step 4: Show results
  console.log('\n' + '='.repeat(60));
  console.log('RESULTS');
  console.log('='.repeat(60));

  const finalState = agent.getState();
  console.log(`\nFinal Phase: ${finalState.phase}`);
  console.log(`Total API Calls: ${apiCalls}`);
  console.log(`Total Duration: ${duration.toFixed(1)} seconds`);
  console.log(`Messages in conversation: ${finalState.messages.length}`);

  if (finalState.generatedArtifacts) {
    console.log('\n--- Generated Rust Schema ---');
    console.log(finalState.generatedArtifacts.rustSchema);

    console.log('\n--- Generated TypeScript ---');
    console.log(finalState.generatedArtifacts.typescriptCode);

    console.log('\n--- Generated Tests (first 500 chars) ---');
    console.log(finalState.generatedArtifacts.testCases.substring(0, 500) + '...');
  } else {
    console.log('\nNo artifacts generated yet.');
    console.log('Design draft:', finalState.designDraft ? 'Yes' : 'No');
  }

  // Cost estimation
  console.log('\n' + '='.repeat(60));
  console.log('COST ESTIMATION');
  console.log('='.repeat(60));

  // Rough estimate based on typical token counts
  // Each message exchange is roughly:
  // - System prompt: ~1000 tokens
  // - User message: ~100-200 tokens
  // - Knowledge context: ~500-1000 tokens
  // - Response: ~300-800 tokens

  const estimatedInputPerCall = 1500; // tokens
  const estimatedOutputPerCall = 500; // tokens

  const totalInputEstimate = apiCalls * estimatedInputPerCall;
  const totalOutputEstimate = apiCalls * estimatedOutputPerCall;

  const estimatedUsage: TokenUsage = {
    inputTokens: totalInputEstimate,
    outputTokens: totalOutputEstimate,
  };

  const estimatedCost = calculateCost(estimatedUsage);

  console.log(`\nEstimated token usage (rough):`);
  console.log(`  Input tokens:  ~${totalInputEstimate.toLocaleString()}`);
  console.log(`  Output tokens: ~${totalOutputEstimate.toLocaleString()}`);
  console.log(`\nEstimated cost: $${estimatedCost.toFixed(4)}`);
  console.log(`\nNote: Actual costs may vary. Check your Anthropic dashboard for exact usage.`);
  console.log(`Claude Sonnet pricing: $3/1M input, $15/1M output tokens`);
}

// Run the test
runTimestampGeneratorTest().catch(console.error);
