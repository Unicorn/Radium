/**
 * Knowledge Base Tests
 *
 * Tests for the migration record processor and knowledge retrieval system.
 *
 * TESTING POLICY COMPLIANCE:
 * - Anthropic SDK: Mocked (external third-party API - acceptable)
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { MigrationRecordProcessor } from '../knowledge-base/processor';
import { KnowledgeRetrieval } from '../knowledge-base/retrieval';
import type { ProcessedRecord, MigrationRecord } from '../knowledge-base/types';
import * as path from 'path';

// Mock Anthropic SDK (external third-party - acceptable per testing policy)
vi.mock('@anthropic-ai/sdk', () => ({
  default: vi.fn().mockImplementation(() => ({
    messages: {
      create: vi.fn().mockResolvedValue({
        content: [{ type: 'text', text: 'Mock response' }],
      }),
    },
  })),
}));

describe('MigrationRecordProcessor', () => {
  let processor: MigrationRecordProcessor;

  beforeEach(() => {
    // Use the actual component-records directory
    const recordsDir = path.resolve(
      __dirname,
      '../../../../../../crates/radium-workflow/component-records'
    );
    processor = new MigrationRecordProcessor({
      recordsDir,
      includeTestCases: true,
    });
  });

  describe('processAll', () => {
    it('should process all component records', async () => {
      const records = await processor.processAll();

      expect(records).toBeDefined();
      expect(Array.isArray(records)).toBe(true);
      // We have at least some component records
      expect(records.length).toBeGreaterThan(0);
    });

    it('should extract metadata from records', async () => {
      const records = await processor.processAll();

      for (const record of records) {
        expect(record.id).toBeDefined();
        expect(record.metadata).toBeDefined();
        expect(record.metadata.name).toBeDefined();
        expect(record.metadata.category).toBeDefined();
      }
    });

    it('should extract patterns from records', async () => {
      const records = await processor.processAll();

      for (const record of records) {
        expect(record.patterns).toBeDefined();
        expect(record.patterns.inputValidation).toBeDefined();
        expect(record.patterns.outputSchema).toBeDefined();
      }
    });

    it('should create searchable content', async () => {
      const records = await processor.processAll();

      for (const record of records) {
        expect(record.content).toBeDefined();
        expect(record.content.length).toBeGreaterThan(0);
        expect(record.content).toContain('Component:');
      }
    });
  });

  describe('processFile', () => {
    it('should return null for non-existent file', async () => {
      const result = await processor.processFile('/non/existent/file.yaml');
      expect(result).toBeNull();
    });
  });
});

describe('KnowledgeRetrieval', () => {
  let retrieval: KnowledgeRetrieval;
  let mockRecords: ProcessedRecord[];

  beforeEach(() => {
    retrieval = new KnowledgeRetrieval({
      model: 'claude-sonnet-4-20250514',
      maxResults: 3,
    });

    // Create mock records for testing
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
            { name: 'url', rustType: 'String', typescriptType: 'string', required: true, description: '' },
          ],
          validation: [],
        },
        outputSchema: {
          rustStruct: 'HttpRequestOutput',
          typescriptInterface: 'HttpRequestOutput',
          fields: [],
          validation: [],
        },
        decisions: [],
        lessonsLearned: { whatWorkedWell: [], challenges: [], recommendations: [] },
        relatedComponents: [],
      },
      {
        id: 'database_query',
        content: 'Component: database_query\nCategory: data\nDescription: Database query component',
        metadata: {
          name: 'database_query',
          category: 'activities',
          version: '1.0.0',
          description: 'Database query component',
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
          rustStruct: 'DatabaseQueryInput',
          typescriptInterface: 'DatabaseQueryInput',
          fields: [],
          validation: [],
        },
        outputSchema: {
          rustStruct: 'DatabaseQueryOutput',
          typescriptInterface: 'DatabaseQueryOutput',
          fields: [],
          validation: [],
        },
        decisions: [],
        lessonsLearned: { whatWorkedWell: [], challenges: [], recommendations: [] },
        relatedComponents: [],
      },
    ];
  });

  describe('loadKnowledgeBase', () => {
    it('should load records into knowledge base', async () => {
      await retrieval.loadKnowledgeBase(mockRecords);

      const stats = retrieval.getStats();
      expect(stats.totalComponents).toBe(2);
    });
  });

  describe('getComponent', () => {
    it('should retrieve component by ID', async () => {
      await retrieval.loadKnowledgeBase(mockRecords);

      const component = retrieval.getComponent('http_request');
      expect(component).toBeDefined();
      expect(component?.id).toBe('http_request');
    });

    it('should return undefined for non-existent component', async () => {
      await retrieval.loadKnowledgeBase(mockRecords);

      const component = retrieval.getComponent('non_existent');
      expect(component).toBeUndefined();
    });
  });

  describe('getAllComponents', () => {
    it('should return all components', async () => {
      await retrieval.loadKnowledgeBase(mockRecords);

      const components = retrieval.getAllComponents();
      expect(components.length).toBe(2);
    });
  });

  describe('getStats', () => {
    it('should return accurate statistics', async () => {
      await retrieval.loadKnowledgeBase(mockRecords);

      const stats = retrieval.getStats();
      expect(stats.totalComponents).toBe(2);
      expect(stats.byCategory).toBeDefined();
      expect(stats.byTemporalType).toBeDefined();
    });
  });
});
