/**
 * Migration Record Processor
 *
 * Processes YAML migration records from the Rust radium-workflow crate
 * and converts them into structured knowledge for the Component Builder Agent.
 */

import * as yaml from 'yaml';
import * as fs from 'fs';
import * as path from 'path';
import type {
  MigrationRecord,
  ProcessedRecord,
  ComponentMetadata,
  ComponentPatterns,
  ValidationPattern,
  SchemaPattern,
  CodePattern,
  RustPattern,
  SchemaDecision,
  LessonsLearned,
  RelatedComponent,
  ComponentCategory,
  TemporalType,
  DifficultyLevel,
  ComplexityLevel,
} from './types';

/** Configuration for the processor */
export interface ProcessorConfig {
  /** Path to component-records directory */
  recordsDir: string;
  /** Whether to include test case details */
  includeTestCases?: boolean;
  /** Whether to include discovery info */
  includeDiscovery?: boolean;
}

/**
 * Processes migration records from YAML files into structured knowledge
 */
export class MigrationRecordProcessor {
  private recordsDir: string;
  private config: ProcessorConfig;

  constructor(config: ProcessorConfig) {
    this.config = config;
    this.recordsDir = config.recordsDir;
  }

  /**
   * Process all migration records in the directory
   */
  async processAll(): Promise<ProcessedRecord[]> {
    const records: ProcessedRecord[] = [];

    if (!fs.existsSync(this.recordsDir)) {
      console.warn(`Records directory not found: ${this.recordsDir}`);
      return records;
    }

    const files = fs.readdirSync(this.recordsDir).filter((f) => {
      // Skip template files and non-YAML files
      return (
        f.endsWith('.yaml') &&
        !f.startsWith('_') &&
        f !== 'quality-checklist.yaml'
      );
    });

    for (const file of files) {
      try {
        const record = await this.processFile(path.join(this.recordsDir, file));
        if (record) {
          records.push(record);
        }
      } catch (error) {
        console.error(`Error processing ${file}:`, error);
      }
    }

    return records;
  }

  /**
   * Process a single migration record file
   */
  async processFile(filePath: string): Promise<ProcessedRecord | null> {
    if (!fs.existsSync(filePath)) {
      console.warn(`File not found: ${filePath}`);
      return null;
    }

    const content = fs.readFileSync(filePath, 'utf-8');
    const parsed = yaml.parse(content) as MigrationRecord;

    if (!parsed || !parsed.component) {
      console.warn(`Invalid migration record: ${filePath}`);
      return null;
    }

    // Extract structured knowledge
    const metadata = this.extractMetadata(parsed);
    const patterns = this.extractPatterns(parsed);
    const decisions = this.extractDecisions(parsed);
    const lessonsLearned = this.extractLessonsLearned(parsed);
    const relatedComponents = this.extractRelatedComponents(parsed);

    // Create searchable content string
    const searchableContent = this.createSearchableContent(parsed);

    return {
      id: parsed.component.name,
      content: searchableContent,
      metadata,
      patterns,
      inputSchema: parsed.inputSchema,
      outputSchema: parsed.outputSchema,
      decisions,
      lessonsLearned,
      relatedComponents,
    };
  }

  /**
   * Extract component metadata
   */
  private extractMetadata(record: MigrationRecord): ComponentMetadata {
    const category = this.normalizeCategory(record.component.category);
    const temporalType = this.normalizeTemporalType(record.component.temporalType);
    const difficulty = this.normalizeDifficulty(record.migration.difficulty);
    const complexity = this.inferComplexity(record);

    return {
      name: record.component.name,
      category,
      version: record.component.version,
      description: record.component.description,
      temporalType,
      complexity,
      migrationDate: record.migration.migrationDate,
      migrationDifficulty: difficulty,
    };
  }

  /**
   * Extract patterns from the record
   */
  private extractPatterns(record: MigrationRecord): ComponentPatterns {
    const inputValidation = this.extractValidationPatterns(record);
    const outputSchema = this.extractSchemaPatterns(record);
    const errorHandling = this.extractErrorPatterns(record);
    const typescriptPatterns = this.extractTypeScriptPatterns(record);
    const rustPatterns = this.extractRustPatterns(record);

    return {
      inputValidation,
      outputSchema,
      errorHandling,
      typescriptPatterns,
      rustPatterns,
    };
  }

  /**
   * Extract validation patterns from input schema
   */
  private extractValidationPatterns(record: MigrationRecord): ValidationPattern[] {
    const patterns: ValidationPattern[] = [];

    // Extract from validation rules
    for (const rule of record.validationRules || []) {
      patterns.push({
        type: 'custom',
        field: 'various',
        rule,
        rustImplementation: '',
        rationale: 'Defined in validation rules',
      });
    }

    // Extract from input schema fields
    for (const field of record.inputSchema?.fields || []) {
      if (field.required) {
        patterns.push({
          type: 'required',
          field: field.name,
          rule: `${field.name} is required`,
          rustImplementation: `#[validate(required)]`,
          rationale: 'Field marked as required in schema',
        });
      }
    }

    return patterns;
  }

  /**
   * Extract schema patterns from input/output schemas
   */
  private extractSchemaPatterns(record: MigrationRecord): SchemaPattern[] {
    const patterns: SchemaPattern[] = [];

    // Process input fields
    for (const field of record.inputSchema?.fields || []) {
      patterns.push({
        fieldName: field.name,
        fieldType: 'input',
        rustType: field.rustType,
        typescriptType: field.typescriptType,
        required: field.required,
        defaultValue: field.default,
        serdeAnnotations: this.inferSerdeAnnotations(field.rustType),
        description: field.description || '',
      });
    }

    // Process output fields
    for (const field of record.outputSchema?.fields || []) {
      patterns.push({
        fieldName: field.name,
        fieldType: 'output',
        rustType: field.rustType,
        typescriptType: field.typescriptType,
        required: field.required,
        defaultValue: field.default,
        serdeAnnotations: this.inferSerdeAnnotations(field.rustType),
        description: field.description || '',
      });
    }

    return patterns;
  }

  /**
   * Extract error handling patterns
   */
  private extractErrorPatterns(record: MigrationRecord): Array<{
    errorType: string;
    handling: string;
    recovery: string;
  }> {
    const patterns: Array<{
      errorType: string;
      handling: string;
      recovery: string;
    }> = [];

    // Extract from lessons learned challenges
    for (const challenge of record.lessonsLearned?.challenges || []) {
      if (
        challenge.challenge.toLowerCase().includes('error') ||
        challenge.challenge.toLowerCase().includes('fail')
      ) {
        patterns.push({
          errorType: 'runtime',
          handling: challenge.challenge,
          recovery: challenge.solution,
        });
      }
    }

    return patterns;
  }

  /**
   * Extract TypeScript code patterns
   */
  private extractTypeScriptPatterns(record: MigrationRecord): CodePattern[] {
    const patterns: CodePattern[] = [];

    // Extract from template info
    if (record.typescriptTemplate?.keyPatterns) {
      for (const pattern of record.typescriptTemplate.keyPatterns) {
        patterns.push({
          patternName: pattern,
          template: '',
          usage: `Used in ${record.component.name}`,
          example: record.typescriptTemplate.generatedCodeExample || '',
        });
      }
    }

    return patterns;
  }

  /**
   * Extract Rust-specific patterns
   */
  private extractRustPatterns(record: MigrationRecord): RustPattern[] {
    const patterns: RustPattern[] = [];

    if (record.rustSchema) {
      patterns.push({
        patternName: `${record.component.name}_schema`,
        derives: record.rustSchema.derives || [],
        structs: record.rustSchema.structs || [],
        enums: record.rustSchema.enums || [],
        implementation: record.rustSchema.validationImplementation || '',
      });
    }

    return patterns;
  }

  /**
   * Extract schema decisions
   */
  private extractDecisions(record: MigrationRecord): SchemaDecision[] {
    return (record.schemaDecisions || []).map((d) => ({
      field: d.field,
      decision: d.decision,
      rationale: d.rationale,
      alternativesConsidered: d.alternativesConsidered || [],
    }));
  }

  /**
   * Extract lessons learned
   */
  private extractLessonsLearned(record: MigrationRecord): LessonsLearned {
    return {
      whatWorkedWell: record.lessonsLearned?.whatWorkedWell || [],
      challenges: record.lessonsLearned?.challenges || [],
      recommendations: record.lessonsLearned?.recommendations || [],
    };
  }

  /**
   * Extract related components
   */
  private extractRelatedComponents(record: MigrationRecord): RelatedComponent[] {
    return (record.relatedComponents || []).map((r) => ({
      componentId: r.component,
      relationship: r.relationship,
    }));
  }

  /**
   * Create searchable content string for similarity matching
   */
  private createSearchableContent(record: MigrationRecord): string {
    const parts: string[] = [];

    // Component description
    parts.push(`Component: ${record.component.name}`);
    parts.push(`Category: ${record.component.category}`);
    parts.push(`Type: ${record.component.temporalType}`);
    parts.push(`Description: ${record.component.description}`);

    // Schema decisions
    if (record.schemaDecisions?.length > 0) {
      parts.push('\nSchema Decisions:');
      for (const decision of record.schemaDecisions) {
        parts.push(`- ${decision.field}: ${decision.decision}`);
        parts.push(`  Rationale: ${decision.rationale}`);
      }
    }

    // Input schema fields
    if (record.inputSchema?.fields?.length > 0) {
      parts.push('\nInput Fields:');
      for (const field of record.inputSchema.fields) {
        const required = field.required ? '(required)' : '(optional)';
        parts.push(`- ${field.name}: ${field.rustType} ${required}`);
      }
    }

    // Output schema fields
    if (record.outputSchema?.fields?.length > 0) {
      parts.push('\nOutput Fields:');
      for (const field of record.outputSchema.fields) {
        parts.push(`- ${field.name}: ${field.typescriptType}`);
      }
    }

    // Lessons learned
    if (record.lessonsLearned) {
      if (record.lessonsLearned.whatWorkedWell?.length > 0) {
        parts.push('\nWhat Worked Well:');
        for (const item of record.lessonsLearned.whatWorkedWell) {
          parts.push(`- ${item}`);
        }
      }

      if (record.lessonsLearned.challenges?.length > 0) {
        parts.push('\nChallenges & Solutions:');
        for (const challenge of record.lessonsLearned.challenges) {
          parts.push(`- Challenge: ${challenge.challenge}`);
          parts.push(`  Solution: ${challenge.solution}`);
        }
      }

      if (record.lessonsLearned.recommendations?.length > 0) {
        parts.push('\nRecommendations:');
        for (const rec of record.lessonsLearned.recommendations) {
          parts.push(`- ${rec}`);
        }
      }
    }

    return parts.join('\n');
  }

  /**
   * Normalize category string to ComponentCategory type
   */
  private normalizeCategory(category: string): ComponentCategory {
    const normalized = category.toLowerCase();
    if (normalized === 'activities' || normalized === 'activity') {
      return 'activities';
    }
    if (normalized === 'control-flow' || normalized === 'control') {
      return 'control-flow';
    }
    if (normalized === 'triggers' || normalized === 'trigger') {
      return 'triggers';
    }
    if (normalized === 'signals' || normalized === 'signal') {
      return 'signals';
    }
    return 'integrations';
  }

  /**
   * Normalize temporal type string
   */
  private normalizeTemporalType(type: string): TemporalType {
    const normalized = type.toLowerCase().replace(/_/g, '-');
    if (normalized === 'activity') return 'activity';
    if (normalized === 'workflow') return 'workflow';
    if (normalized === 'signal') return 'signal';
    if (normalized === 'query') return 'query';
    if (normalized === 'child-workflow') return 'child-workflow';
    if (normalized === 'timer') return 'timer';
    return 'activity';
  }

  /**
   * Normalize difficulty string
   */
  private normalizeDifficulty(difficulty: string): DifficultyLevel {
    const normalized = difficulty.toLowerCase();
    if (normalized === 'trivial') return 'trivial';
    if (normalized === 'easy') return 'easy';
    if (normalized === 'medium') return 'medium';
    if (normalized === 'hard') return 'hard';
    if (normalized === 'complex') return 'complex';
    return 'medium';
  }

  /**
   * Infer complexity from record characteristics
   */
  private inferComplexity(record: MigrationRecord): ComplexityLevel {
    const inputFieldCount = record.inputSchema?.fields?.length || 0;
    const outputFieldCount = record.outputSchema?.fields?.length || 0;
    const decisionCount = record.schemaDecisions?.length || 0;

    const totalComplexity = inputFieldCount + outputFieldCount + decisionCount;

    if (totalComplexity <= 5) return 'low';
    if (totalComplexity <= 15) return 'medium';
    return 'high';
  }

  /**
   * Infer serde annotations from Rust type
   */
  private inferSerdeAnnotations(rustType: string): string[] {
    const annotations: string[] = [];

    if (rustType.startsWith('Option<')) {
      annotations.push('#[serde(skip_serializing_if = "Option::is_none")]');
    }
    if (rustType.includes('HashMap')) {
      annotations.push('#[serde(default)]');
    }
    if (rustType.includes('Vec<')) {
      annotations.push('#[serde(default)]');
    }

    return annotations;
  }
}

/**
 * Create a processor with default configuration
 */
export function createDefaultProcessor(): MigrationRecordProcessor {
  // Default path relative to the radium-workflow crate
  const defaultRecordsDir = path.resolve(
    __dirname,
    '../../../../../crates/radium-workflow/component-records'
  );

  return new MigrationRecordProcessor({
    recordsDir: defaultRecordsDir,
    includeTestCases: true,
    includeDiscovery: false,
  });
}
