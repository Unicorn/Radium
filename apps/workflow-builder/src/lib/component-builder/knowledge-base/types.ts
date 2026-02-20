/**
 * Knowledge Base Types for Component Builder
 *
 * These types define the structure of the knowledge base used by the
 * Component Builder Agent to understand existing patterns and generate
 * new components.
 */

/** Categories of workflow components */
export type ComponentCategory =
  | 'activities'
  | 'control-flow'
  | 'triggers'
  | 'signals'
  | 'integrations';

/** Temporal construct types */
export type TemporalType =
  | 'activity'
  | 'workflow'
  | 'signal'
  | 'query'
  | 'child-workflow'
  | 'timer';

/** Complexity levels for components */
export type ComplexityLevel = 'low' | 'medium' | 'high';

/** Difficulty levels for migration */
export type DifficultyLevel = 'trivial' | 'easy' | 'medium' | 'hard' | 'complex';

/** Component knowledge extracted from migration records */
export interface ComponentKnowledge {
  /** Unique component identifier */
  componentId: string;

  /** Component metadata */
  metadata: ComponentMetadata;

  /** Extracted patterns for AI learning */
  patterns: ComponentPatterns;

  /** Schema design decisions with rationale */
  decisions: SchemaDecision[];

  /** Related component identifiers */
  relatedComponents: RelatedComponent[];

  /** Lessons learned during migration */
  lessonsLearned: LessonsLearned;

  /** Future improvement suggestions */
  futureImprovements: FutureImprovement[];
}

/** Component metadata */
export interface ComponentMetadata {
  name: string;
  category: ComponentCategory;
  version: string;
  description: string;
  temporalType: TemporalType;
  complexity: ComplexityLevel;
  migrationDate: string;
  migrationDifficulty: DifficultyLevel;
}

/** Extracted patterns from component */
export interface ComponentPatterns {
  /** Input validation patterns */
  inputValidation: ValidationPattern[];

  /** Output schema patterns */
  outputSchema: SchemaPattern[];

  /** Error handling patterns */
  errorHandling: ErrorPattern[];

  /** TypeScript generation patterns */
  typescriptPatterns: CodePattern[];

  /** Rust schema patterns */
  rustPatterns: RustPattern[];
}

/** Validation pattern extracted from component */
export interface ValidationPattern {
  type: string;
  field: string;
  rule: string;
  rustImplementation: string;
  rationale: string;
}

/** Schema field pattern */
export interface SchemaPattern {
  fieldName: string;
  fieldType: string;
  rustType: string;
  typescriptType: string;
  required: boolean;
  defaultValue?: string;
  serdeAnnotations: string[];
  description: string;
}

/** Error handling pattern */
export interface ErrorPattern {
  errorType: string;
  handling: string;
  recovery: string;
}

/** Code generation pattern */
export interface CodePattern {
  patternName: string;
  template: string;
  usage: string;
  example: string;
}

/** Rust-specific pattern */
export interface RustPattern {
  patternName: string;
  derives: string[];
  structs: string[];
  enums: string[];
  implementation: string;
}

/** Schema design decision */
export interface SchemaDecision {
  field: string;
  decision: string;
  rationale: string;
  alternativesConsidered: Alternative[];
}

/** Alternative approach considered */
export interface Alternative {
  approach: string;
  pros: string[];
  cons: string[];
  whyRejected: string;
}

/** Related component reference */
export interface RelatedComponent {
  componentId: string;
  relationship: string;
}

/** Lessons learned from migration */
export interface LessonsLearned {
  whatWorkedWell: string[];
  challenges: Challenge[];
  recommendations: string[];
}

/** Challenge encountered during migration */
export interface Challenge {
  challenge: string;
  solution: string;
  timeSpent: string;
}

/** Future improvement suggestion */
export interface FutureImprovement {
  improvement: string;
  priority: 'low' | 'medium' | 'high';
  effort: 'low' | 'medium' | 'high';
}

/** Input schema definition */
export interface InputSchema {
  rustStruct: string;
  typescriptInterface: string;
  fields: FieldDefinition[];
  validation: string[];
}

/** Output schema definition */
export interface OutputSchema {
  rustStruct: string;
  typescriptInterface: string;
  fields: FieldDefinition[];
  validation: string[];
}

/** Field definition in a schema */
export interface FieldDefinition {
  name: string;
  rustType: string;
  typescriptType: string;
  required: boolean;
  default?: string;
  description: string;
}

/** Connection rules for component */
export interface ConnectionRules {
  allowedSources: string[];
  allowedTargets: string[];
  connectionValidation: string;
}

/** Test case definition */
export interface TestCase {
  name: string;
  category: 'unit' | 'integration' | 'behavior';
  input: string;
  expectedOutput: string;
  passed: boolean;
}

/** Rust schema information */
export interface RustSchemaInfo {
  filePath: string;
  structs: string[];
  enums: string[];
  derives: string[];
  validationImplementation: string;
}

/** TypeScript template information */
export interface TypeScriptTemplateInfo {
  templatePath: string;
  generatedCodeExample: string;
  keyPatterns: string[];
}

/** Raw migration record from YAML */
export interface MigrationRecord {
  component: {
    name: string;
    category: string;
    version: string;
    description: string;
    temporalType: string;
  };
  migration: {
    migratedBy: string;
    migrationDate: string;
    durationHours: number;
    difficulty: string;
    breakingChanges: boolean;
    filesCreated: string[];
    filesModified: string[];
  };
  discovery?: {
    originalTypescriptFile: string;
    linesOfCode: number;
    existingTests: string[];
    usageLocations: string[];
    dependencies: string[];
  };
  schemaDecisions: Array<{
    field: string;
    decision: string;
    rationale: string;
    alternativesConsidered: Alternative[];
  }>;
  inputSchema: InputSchema;
  outputSchema: OutputSchema;
  validationRules: string[];
  connections: ConnectionRules;
  rustSchema: RustSchemaInfo;
  typescriptTemplate: TypeScriptTemplateInfo;
  testCases: TestCase[];
  lessonsLearned: {
    whatWorkedWell: string[];
    challenges: Challenge[];
    recommendations: string[];
  };
  relatedComponents: Array<{
    component: string;
    relationship: string;
  }>;
  futureImprovements: FutureImprovement[];
}

/** Processed record for knowledge base */
export interface ProcessedRecord {
  id: string;
  content: string;
  metadata: ComponentMetadata;
  patterns: ComponentPatterns;
  inputSchema: InputSchema;
  outputSchema: OutputSchema;
  decisions: SchemaDecision[];
  lessonsLearned: LessonsLearned;
  relatedComponents: RelatedComponent[];
}

/** Similar component result from search */
export interface SimilarComponent {
  componentId: string;
  similarity: number;
  relevantDecisions: SchemaDecision[];
  applicablePatterns: string[];
  reason: string;
}

/** Knowledge base query result */
export interface KnowledgeQueryResult {
  query: string;
  similarComponents: SimilarComponent[];
  extractedPatterns: ComponentPatterns;
  suggestedDecisions: SchemaDecision[];
}
