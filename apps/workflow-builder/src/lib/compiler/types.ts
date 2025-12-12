/**
 * Core Compiler Types
 * Types and interfaces for the pattern-based workflow compiler
 */

/**
 * Workflow definition structure
 */
export interface WorkflowDefinition {
  id: string;
  name: string;
  nodes: WorkflowNode[];
  edges: WorkflowEdge[];
  variables: WorkflowVariable[];
  settings: WorkflowSettings;
}

/**
 * Workflow node structure
 */
export interface WorkflowNode {
  id: string;
  type: 'trigger' | 'activity' | 'agent' | 'conditional' | 'loop' | 'child-workflow' | 'signal' | 'phase' | 'retry' | 'state-variable' | 'api-endpoint' | 'condition' | 'end' | 'data-in' | 'data-out' | 'kong-logging' | 'kong-cache' | 'kong-cors' | 'graphql-gateway' | 'mcp-server';
  data: NodeData;
  position: { x: number; y: number };
}

/**
 * Node data structure
 */
export interface NodeData {
  label: string;
  componentId?: string;
  componentName?: string;
  activityName?: string;
  signalName?: string;
  config?: Record<string, any>;
  timeout?: string;
  retryPolicy?: RetryPolicy;
}

/**
 * Retry policy configuration
 */
export interface RetryPolicy {
  strategy: 'keep-trying' | 'fail-after-x' | 'exponential-backoff' | 'none';
  maxAttempts?: number;
  initialInterval?: string;
  maxInterval?: string;
  backoffCoefficient?: number;
}

/**
 * Workflow edge structure
 */
export interface WorkflowEdge {
  id: string;
  source: string;
  target: string;
  sourceHandle?: string;
  targetHandle?: string;
  label?: string;
  type?: string;
}

/**
 * Workflow variable definition
 */
export interface WorkflowVariable {
  name: string;
  type: 'string' | 'number' | 'boolean' | 'array' | 'object';
  initialValue?: any;
  description?: string;
}

/**
 * Workflow settings
 */
export interface WorkflowSettings {
  timeout?: string;
  retryPolicy?: RetryPolicy;
  taskQueue?: string;
  description?: string;
  version?: string;
  // Internal: Auto-configured, never exposed to UI
  _longRunning?: {
    autoContinueAsNew: boolean;
    maxHistoryEvents: number;      // Default: 1000
    maxDurationMs: number;         // Default: 24 hours
    preserveState: boolean;        // Always true - preserve all variables
  };
  // Internal: Auto-detected workflow type
  _workflowType?: 'service' | 'task';
}

/**
 * Pattern interface for code generation
 */
export interface Pattern {
  name: string;
  detect: (node: WorkflowNode, context: GeneratorContext) => boolean;
  generate: (node: WorkflowNode, context: GeneratorContext) => CodeBlock;
  dependencies?: string[];
  priority?: number;
}

/**
 * Generator context provides access to workflow state during code generation
 */
export interface GeneratorContext {
  nodes: WorkflowNode[];
  edges: WorkflowEdge[];
  variables: WorkflowVariable[];
  settings: WorkflowSettings;
  nodeMap: Map<string, WorkflowNode>;
  edgeMap: Map<string, WorkflowEdge[]>;
  visitedNodes: Set<string>;
  resultVars: Map<string, string>;
  proxyMap?: Map<string, string>; // Maps node ID to proxy variable name
  currentIndent: number;
  workflowName: string; // Workflow function name for code generation
}

/**
 * Generated code block
 */
export interface CodeBlock {
  code: string;
  imports?: string[];
  declarations?: string[];
  resultVar?: string;
}

/**
 * Compiler result containing generated code and metadata
 */
export interface CompilerResult {
  success: boolean;
  workflowCode?: string;
  activitiesCode?: string;
  workerCode?: string;
  packageJson?: string;
  tsConfig?: string;
  errors?: CompilerError[];
  warnings?: CompilerWarning[];
  metadata?: CompilerMetadata;
}

/**
 * Compiler error
 */
export interface CompilerError {
  message: string;
  nodeId?: string;
  type: 'validation' | 'generation' | 'pattern' | 'dependency';
  severity: 'error' | 'fatal';
}

/**
 * Compiler warning
 */
export interface CompilerWarning {
  message: string;
  nodeId?: string;
  type: 'optimization' | 'deprecation' | 'best-practice';
}

/**
 * Compiler metadata
 */
export interface CompilerMetadata {
  nodeCount: number;
  edgeCount: number;
  patternsApplied: string[];
  compilationTime: number;
  version: string;
}

/**
 * Compiler options
 */
export interface CompilerOptions {
  packageName?: string;
  outputPath?: string;
  includeComments?: boolean;
  strictMode?: boolean;
  validateOnly?: boolean;
  optimizationLevel?: 'none' | 'basic' | 'aggressive';
}

/**
 * Validation result
 */
export interface ValidationResult {
  valid: boolean;
  errors: CompilerError[];
  warnings: CompilerWarning[];
}

/**
 * Import statement structure
 */
export interface ImportStatement {
  module: string;
  items: string[];
  type?: 'named' | 'default' | 'namespace';
}
