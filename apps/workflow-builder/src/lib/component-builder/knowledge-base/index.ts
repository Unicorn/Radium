/**
 * Knowledge Base Module
 *
 * Provides infrastructure for processing migration records and
 * retrieving relevant component knowledge for the Component Builder Agent.
 */

export * from './types';
export * from './processor';
export * from './retrieval';

import { MigrationRecordProcessor, createDefaultProcessor } from './processor';
import { KnowledgeRetrieval, createDefaultRetrieval } from './retrieval';
import type { ProcessedRecord } from './types';

/**
 * Initialize the knowledge base with all migration records
 */
export async function initializeKnowledgeBase(): Promise<{
  processor: MigrationRecordProcessor;
  retrieval: KnowledgeRetrieval;
  records: ProcessedRecord[];
}> {
  const processor = createDefaultProcessor();
  const retrieval = createDefaultRetrieval();

  // Process all migration records
  const records = await processor.processAll();

  // Load into retrieval system
  await retrieval.loadKnowledgeBase(records);

  // Log statistics
  const stats = retrieval.getStats();
  console.log('Knowledge Base Initialized:');
  console.log(`  - Components: ${stats.totalComponents}`);
  console.log(`  - Categories: ${JSON.stringify(stats.byCategory)}`);
  console.log(`  - Temporal Types: ${JSON.stringify(stats.byTemporalType)}`);
  console.log(`  - Total Decisions: ${stats.totalDecisions}`);
  console.log(`  - Total Patterns: ${stats.totalPatterns}`);

  return { processor, retrieval, records };
}

/**
 * Create a lightweight knowledge base for testing
 */
export function createTestKnowledgeBase(): KnowledgeRetrieval {
  const retrieval = new KnowledgeRetrieval({
    model: 'claude-sonnet-4-20250514',
    maxResults: 3,
  });

  return retrieval;
}
