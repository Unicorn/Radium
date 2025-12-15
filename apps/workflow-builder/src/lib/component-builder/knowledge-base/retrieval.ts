/**
 * Knowledge Retrieval System
 *
 * Uses Claude AI to find semantically similar components and retrieve
 * relevant knowledge for component generation.
 */

import Anthropic from '@anthropic-ai/sdk';
import type {
  ProcessedRecord,
  SimilarComponent,
  ComponentPatterns,
  SchemaDecision,
  KnowledgeQueryResult,
  SchemaPattern,
  ValidationPattern,
} from './types';

/** Configuration for knowledge retrieval */
export interface RetrievalConfig {
  /** Anthropic API key (defaults to env var) */
  apiKey?: string;
  /** Model to use for retrieval */
  model?: string;
  /** Maximum similar components to return */
  maxResults?: number;
}

/**
 * Knowledge retrieval system using Claude for semantic search
 */
export class KnowledgeRetrieval {
  private anthropic: Anthropic;
  private knowledgeBase: Map<string, ProcessedRecord>;
  private model: string;
  private maxResults: number;

  constructor(config: RetrievalConfig = {}) {
    this.anthropic = new Anthropic({
      apiKey: config.apiKey,
    });
    this.knowledgeBase = new Map();
    this.model = config.model || 'claude-sonnet-4-20250514';
    this.maxResults = config.maxResults || 5;
  }

  /**
   * Load processed records into the knowledge base
   */
  async loadKnowledgeBase(records: ProcessedRecord[]): Promise<void> {
    for (const record of records) {
      this.knowledgeBase.set(record.id, record);
    }
    console.log(`Loaded ${records.length} components into knowledge base`);
  }

  /**
   * Find similar components based on a natural language query
   */
  async findSimilar(query: string, limit?: number): Promise<SimilarComponent[]> {
    const maxResults = limit || this.maxResults;

    if (this.knowledgeBase.size === 0) {
      console.warn('Knowledge base is empty');
      return [];
    }

    // Build component summaries for Claude
    const componentSummaries = Array.from(this.knowledgeBase.values())
      .map((r) => {
        const inputFields = r.inputSchema?.fields?.map((f) => f.name).join(', ') || 'none';
        const outputFields = r.outputSchema?.fields?.map((f) => f.name).join(', ') || 'none';
        return `- ${r.id} (${r.metadata.category}/${r.metadata.temporalType}): ${r.metadata.description}
    Input fields: ${inputFields}
    Output fields: ${outputFields}`;
      })
      .join('\n');

    const prompt = `You are analyzing workflow components to find ones similar to a user's request.

User's component request:
"${query}"

Available components:
${componentSummaries}

Find the ${maxResults} most relevant components based on:
1. Similar functionality or purpose
2. Similar input/output patterns
3. Same category or temporal type
4. Shared design patterns

Return ONLY a JSON array with this exact format, no other text:
[
  {
    "componentId": "component_name",
    "similarity": 0.85,
    "reason": "Brief explanation of why this is similar"
  }
]

Order by similarity (highest first). Similarity should be 0.0 to 1.0.`;

    try {
      const response = await this.anthropic.messages.create({
        model: this.model,
        max_tokens: 1024,
        messages: [{ role: 'user', content: prompt }],
      });

      const content = response.content[0];
      if (!content || content.type !== 'text') {
        return [];
      }

      // Parse JSON from response
      const jsonMatch = content.text.match(/\[[\s\S]*\]/);
      if (!jsonMatch) {
        console.warn('No JSON array found in response');
        return [];
      }

      const matches = JSON.parse(jsonMatch[0]) as Array<{
        componentId: string;
        similarity: number;
        reason: string;
      }>;

      // Enrich with actual decisions and patterns
      return matches.slice(0, maxResults).map((match) => {
        const record = this.knowledgeBase.get(match.componentId);
        return {
          componentId: match.componentId,
          similarity: match.similarity,
          reason: match.reason,
          relevantDecisions: record?.decisions || [],
          applicablePatterns: this.extractApplicablePatterns(record, query),
        };
      });
    } catch (error) {
      console.error('Error finding similar components:', error);
      return [];
    }
  }

  /**
   * Get detailed knowledge for a specific component
   */
  getComponent(id: string): ProcessedRecord | undefined {
    return this.knowledgeBase.get(id);
  }

  /**
   * Get all components in the knowledge base
   */
  getAllComponents(): ProcessedRecord[] {
    return Array.from(this.knowledgeBase.values());
  }

  /**
   * Query the knowledge base with structured results
   */
  async query(query: string): Promise<KnowledgeQueryResult> {
    const similarComponents = await this.findSimilar(query);

    // Aggregate patterns from similar components
    const extractedPatterns = this.aggregatePatterns(similarComponents);

    // Collect relevant decisions
    const suggestedDecisions = this.aggregateDecisions(similarComponents);

    return {
      query,
      similarComponents,
      extractedPatterns,
      suggestedDecisions,
    };
  }

  /**
   * Get schema patterns for a specific field type
   */
  getPatternsByFieldType(fieldType: string): SchemaPattern[] {
    const patterns: SchemaPattern[] = [];

    for (const record of this.knowledgeBase.values()) {
      for (const pattern of record.patterns.outputSchema) {
        if (
          pattern.rustType.toLowerCase().includes(fieldType.toLowerCase()) ||
          pattern.typescriptType.toLowerCase().includes(fieldType.toLowerCase())
        ) {
          patterns.push(pattern);
        }
      }
    }

    return patterns;
  }

  /**
   * Get validation patterns for a specific validation type
   */
  getValidationPatterns(validationType: string): ValidationPattern[] {
    const patterns: ValidationPattern[] = [];

    for (const record of this.knowledgeBase.values()) {
      for (const pattern of record.patterns.inputValidation) {
        if (pattern.type.toLowerCase().includes(validationType.toLowerCase())) {
          patterns.push(pattern);
        }
      }
    }

    return patterns;
  }

  /**
   * Extract patterns applicable to a query from a component
   */
  private extractApplicablePatterns(
    record: ProcessedRecord | undefined,
    query: string
  ): string[] {
    if (!record) return [];

    const patterns: string[] = [];
    const queryLower = query.toLowerCase();

    // Check for HTTP-related patterns
    if (queryLower.includes('http') || queryLower.includes('api') || queryLower.includes('request')) {
      if (record.id === 'http_request') {
        patterns.push('http_method_enum');
        patterns.push('auth_config');
        patterns.push('timeout_handling');
      }
    }

    // Check for database patterns
    if (queryLower.includes('database') || queryLower.includes('query') || queryLower.includes('sql')) {
      if (record.id === 'database_query') {
        patterns.push('query_parameters');
        patterns.push('result_mapping');
      }
    }

    // Check for control flow patterns
    if (queryLower.includes('condition') || queryLower.includes('if') || queryLower.includes('branch')) {
      if (record.id === 'conditional') {
        patterns.push('condition_expression');
        patterns.push('branch_evaluation');
      }
    }

    // Add Rust patterns
    for (const rustPattern of record.patterns.rustPatterns) {
      patterns.push(rustPattern.patternName);
    }

    return patterns;
  }

  /**
   * Aggregate patterns from multiple similar components
   */
  private aggregatePatterns(similarComponents: SimilarComponent[]): ComponentPatterns {
    const inputValidation: ValidationPattern[] = [];
    const outputSchema: SchemaPattern[] = [];
    const errorHandling: Array<{ errorType: string; handling: string; recovery: string }> = [];
    const typescriptPatterns: Array<{ patternName: string; template: string; usage: string; example: string }> = [];
    const rustPatterns: Array<{
      patternName: string;
      derives: string[];
      structs: string[];
      enums: string[];
      implementation: string;
    }> = [];

    for (const similar of similarComponents) {
      const record = this.knowledgeBase.get(similar.componentId);
      if (!record) continue;

      // Aggregate validation patterns (avoid duplicates)
      for (const pattern of record.patterns.inputValidation) {
        if (!inputValidation.find((p) => p.field === pattern.field && p.type === pattern.type)) {
          inputValidation.push(pattern);
        }
      }

      // Aggregate schema patterns
      for (const pattern of record.patterns.outputSchema) {
        if (!outputSchema.find((p) => p.fieldName === pattern.fieldName)) {
          outputSchema.push(pattern);
        }
      }

      // Aggregate error handling
      for (const pattern of record.patterns.errorHandling) {
        if (!errorHandling.find((p) => p.errorType === pattern.errorType)) {
          errorHandling.push(pattern);
        }
      }

      // Aggregate TypeScript patterns
      for (const pattern of record.patterns.typescriptPatterns) {
        if (!typescriptPatterns.find((p) => p.patternName === pattern.patternName)) {
          typescriptPatterns.push(pattern);
        }
      }

      // Aggregate Rust patterns
      for (const pattern of record.patterns.rustPatterns) {
        if (!rustPatterns.find((p) => p.patternName === pattern.patternName)) {
          rustPatterns.push(pattern);
        }
      }
    }

    return {
      inputValidation,
      outputSchema,
      errorHandling,
      typescriptPatterns,
      rustPatterns,
    };
  }

  /**
   * Aggregate decisions from similar components
   */
  private aggregateDecisions(similarComponents: SimilarComponent[]): SchemaDecision[] {
    const decisions: SchemaDecision[] = [];
    const seenFields = new Set<string>();

    for (const similar of similarComponents) {
      for (const decision of similar.relevantDecisions) {
        if (!seenFields.has(decision.field)) {
          decisions.push(decision);
          seenFields.add(decision.field);
        }
      }
    }

    return decisions;
  }

  /**
   * Get statistics about the knowledge base
   */
  getStats(): {
    totalComponents: number;
    byCategory: Record<string, number>;
    byTemporalType: Record<string, number>;
    totalDecisions: number;
    totalPatterns: number;
  } {
    const byCategory: Record<string, number> = {};
    const byTemporalType: Record<string, number> = {};
    let totalDecisions = 0;
    let totalPatterns = 0;

    for (const record of this.knowledgeBase.values()) {
      // Count by category
      const category = record.metadata.category;
      byCategory[category] = (byCategory[category] || 0) + 1;

      // Count by temporal type
      const type = record.metadata.temporalType;
      byTemporalType[type] = (byTemporalType[type] || 0) + 1;

      // Count decisions
      totalDecisions += record.decisions.length;

      // Count patterns
      totalPatterns +=
        record.patterns.inputValidation.length +
        record.patterns.outputSchema.length +
        record.patterns.errorHandling.length +
        record.patterns.typescriptPatterns.length +
        record.patterns.rustPatterns.length;
    }

    return {
      totalComponents: this.knowledgeBase.size,
      byCategory,
      byTemporalType,
      totalDecisions,
      totalPatterns,
    };
  }
}

/**
 * Create a retrieval instance with default configuration
 */
export function createDefaultRetrieval(): KnowledgeRetrieval {
  return new KnowledgeRetrieval({
    model: 'claude-sonnet-4-20250514',
    maxResults: 5,
  });
}
