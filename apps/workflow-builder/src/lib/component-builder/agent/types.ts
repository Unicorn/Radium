/**
 * Component Builder Agent Types
 *
 * Types for the AI-powered component builder that guides users
 * through designing and generating new workflow components.
 */

import type { SchemaDecision, ComponentPatterns, TemporalType, ComponentCategory } from '../knowledge-base/types';

/** Builder conversation phases */
export type BuilderPhase =
  | 'gathering'    // Initial requirement gathering
  | 'designing'    // Schema design phase
  | 'refining'     // Design refinement based on feedback
  | 'generating'   // Generating code artifacts
  | 'reviewing'    // Reviewing generated artifacts
  | 'complete';    // Component creation complete

/** State of the component builder conversation */
export interface BuilderState {
  /** Unique conversation identifier */
  conversationId: string;

  /** Current phase of the builder */
  phase: BuilderPhase;

  /** Gathered requirements */
  requirement: ComponentRequirement;

  /** Draft component design */
  designDraft: ComponentDesign | null;

  /** Generated code artifacts */
  generatedArtifacts: GeneratedArtifacts | null;

  /** Conversation history */
  messages: Message[];

  /** Timestamp of creation */
  createdAt: Date;

  /** Timestamp of last update */
  updatedAt: Date;
}

/** Requirements gathered from user */
export interface ComponentRequirement {
  /** Natural language description */
  description: string;

  /** Component category */
  category: ComponentCategory | '';

  /** Target temporal type */
  temporalType: TemporalType | '';

  /** Required input fields */
  inputs: FieldRequirement[];

  /** Expected output fields */
  outputs: FieldRequirement[];

  /** Validation rules to apply */
  validationRules: string[];

  /** Similar components identified */
  similarComponents: string[];

  /** Additional context from user */
  additionalContext: string;

  /** User-specified constraints */
  constraints: string[];
}

/** Field requirement from user */
export interface FieldRequirement {
  /** Field name */
  name: string;

  /** Field description */
  description: string;

  /** Expected type (can be vague like "text" or "number") */
  type: string;

  /** Whether the field is required */
  required: boolean;

  /** Default value if any */
  defaultValue?: string;

  /** Validation constraints */
  constraints?: string[];
}

/** Designed component specification */
export interface ComponentDesign {
  /** Component name (snake_case) */
  name: string;

  /** Display name */
  displayName: string;

  /** Component category */
  category: ComponentCategory;

  /** Temporal construct type */
  temporalType: TemporalType;

  /** Component description */
  description: string;

  /** Designed input schema */
  inputSchema: SchemaDesign;

  /** Designed output schema */
  outputSchema: SchemaDesign;

  /** Validation rules */
  validationRules: ValidationRule[];

  /** Connection rules */
  connections: ComponentConnectionRules;

  /** Design decisions made */
  decisions: SchemaDecision[];

  /** Patterns being applied */
  appliedPatterns: string[];
}

/** Schema design for input/output */
export interface SchemaDesign {
  /** Rust struct name */
  rustStruct: string;

  /** TypeScript interface name */
  typescriptInterface: string;

  /** Field definitions */
  fields: FieldDesign[];
}

/** Designed field specification */
export interface FieldDesign {
  /** Field name (snake_case for Rust, camelCase for TS) */
  name: string;

  /** Rust type */
  rustType: string;

  /** TypeScript type */
  typescriptType: string;

  /** Whether required */
  required: boolean;

  /** Default value */
  default?: string;

  /** Serde annotations */
  serde: string[];

  /** Validation rule */
  validation?: string;

  /** Field description */
  description: string;
}

/** Validation rule definition */
export interface ValidationRule {
  /** Target field */
  field: string;

  /** Rule type */
  ruleType: 'required' | 'format' | 'range' | 'length' | 'custom';

  /** Rule expression */
  rule: string;

  /** Error message */
  errorMessage: string;
}

/** Connection rules for component */
export interface ComponentConnectionRules {
  /** Allowed source component types */
  allowedSources: string[];

  /** Allowed target component types */
  allowedTargets: string[];

  /** Whether multiple inputs allowed */
  multipleInputs: boolean;

  /** Whether multiple outputs allowed */
  multipleOutputs: boolean;
}

/** Generated code artifacts */
export interface GeneratedArtifacts {
  /** Rust schema code */
  rustSchema: string;

  /** TypeScript interface code */
  typescriptCode: string;

  /** Test cases */
  testCases: string;

  /** Migration record YAML */
  migrationRecord: string;

  /** Handlebars template (if applicable) */
  handlebarsTemplate?: string;

  /** Validation status */
  validationStatus: ArtifactValidation;
}

/** Validation status of generated artifacts */
export interface ArtifactValidation {
  /** Whether Rust code is valid */
  rustValid: boolean;

  /** Rust validation errors */
  rustErrors: string[];

  /** Whether TypeScript is valid */
  typescriptValid: boolean;

  /** TypeScript validation errors */
  typescriptErrors: string[];

  /** Overall status */
  isValid: boolean;
}

/** Conversation message */
export interface Message {
  /** Message role */
  role: 'user' | 'assistant' | 'system';

  /** Message content */
  content: string;

  /** Timestamp */
  timestamp: Date;

  /** Phase when message was sent */
  phase: BuilderPhase;

  /** Metadata */
  metadata?: MessageMetadata;
}

/** Message metadata */
export interface MessageMetadata {
  /** Whether this message caused a phase transition */
  phaseTransition?: boolean;

  /** New phase if transitioned */
  newPhase?: BuilderPhase;

  /** Similar components mentioned */
  mentionedComponents?: string[];

  /** Token usage */
  tokenUsage?: {
    input: number;
    output: number;
  };
}

/** Agent response with metadata */
export interface AgentResponse {
  /** Response text */
  response: string;

  /** Current phase */
  phase: BuilderPhase;

  /** Whether phase changed */
  phaseChanged: boolean;

  /** Current state snapshot */
  state: Partial<BuilderState>;

  /** Suggested next actions */
  suggestedActions?: string[];
}

/** Options for the builder agent */
export interface BuilderAgentOptions {
  /** Anthropic API key */
  apiKey?: string;

  /** Model to use */
  model?: string;

  /** Maximum tokens per response */
  maxTokens?: number;

  /** Temperature for generation */
  temperature?: number;

  /** Whether to include debug info */
  debug?: boolean;
}

/** Code generation options */
export interface CodeGenerationOptions {
  /** Include documentation comments */
  includeComments: boolean;

  /** Include validation derives */
  includeValidation: boolean;

  /** Include serde derives */
  includeSerde: boolean;

  /** Target Rust edition */
  rustEdition: '2018' | '2021';

  /** TypeScript strict mode */
  strictTypeScript: boolean;
}

/** Schema suggestion from AI */
export interface SchemaSuggestion {
  /** Field name */
  fieldName: string;

  /** Suggested Rust type */
  rustType: string;

  /** Suggested TypeScript type */
  typescriptType: string;

  /** Confidence score 0-1 */
  confidence: number;

  /** Reasoning */
  reasoning: string;

  /** Alternative suggestions */
  alternatives: Array<{
    rustType: string;
    typescriptType: string;
    reason: string;
  }>;
}
