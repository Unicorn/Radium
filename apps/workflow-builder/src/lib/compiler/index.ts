/**
 * Workflow Compiler
 * Main entry point for pattern-based workflow compilation
 */

import type {
  WorkflowDefinition,
  Pattern,
  CompilerOptions,
  CompilerResult,
  GeneratorContext,
  CodeBlock,
  WorkflowNode,
  WorkflowEdge,
} from './types';
import { validateWorkflow } from './utils/validation';
import {
  generateWorkflowFile,
  generateActivitiesFile,
  generateWorkerFile,
  generatePackageJson,
  generateTsConfig,
} from './generators/typescript-generator';
import { ActivityProxyPattern, StateManagementPattern, InterfaceComponentPattern, ContinueAsNewPattern } from './patterns';
import { configureContinueAsNew } from '../workflow-analyzer';

/**
 * Workflow Compiler Class
 * Compiles workflow definitions into executable TypeScript code using patterns
 */
export class WorkflowCompiler {
  private patterns: Pattern[] = [];
  private options: CompilerOptions;

  constructor(options: CompilerOptions = {}) {
    this.options = {
      includeComments: true,
      strictMode: true,
      optimizationLevel: 'basic',
      ...options,
    };

    // Register default patterns
    this.registerDefaultPatterns();
  }

  /**
   * Register a pattern
   */
  registerPattern(pattern: Pattern): void {
    this.patterns.push(pattern);
    // Sort by priority (higher priority first)
    this.patterns.sort((a, b) => (b.priority || 0) - (a.priority || 0));
  }

  /**
   * Register default patterns
   */
  private registerDefaultPatterns(): void {
    this.registerPattern(ActivityProxyPattern);
    this.registerPattern(StateManagementPattern);
    this.registerPattern(InterfaceComponentPattern);
    this.registerPattern(ContinueAsNewPattern);
  }

  /**
   * Compile a workflow definition
   */
  compile(workflow: WorkflowDefinition): CompilerResult {
    const startTime = Date.now();

    try {
      // Step 1: Validate workflow
      const validationWarnings: CompilerWarning[] = [];
      if (this.options.validateOnly !== false) {
        const validation = validateWorkflow(workflow);

        // Collect warnings to pass through
        validationWarnings.push(...validation.warnings);

        if (!validation.valid) {
          return {
            success: false,
            errors: validation.errors,
            warnings: validation.warnings,
            metadata: {
              nodeCount: workflow.nodes?.length || 0,
              edgeCount: workflow.edges?.length || 0,
              patternsApplied: [],
              compilationTime: Date.now() - startTime,
              version: '1.0.0',
            },
          };
        }

        if (this.options.validateOnly) {
          return {
            success: true,
            warnings: validation.warnings,
            metadata: {
              nodeCount: workflow.nodes?.length || 0,
              edgeCount: workflow.edges?.length || 0,
              patternsApplied: [],
              compilationTime: Date.now() - startTime,
              version: '1.0.0',
            },
          };
        }
      }

      // Step 2: Auto-configure workflow (classify and set continue-as-new)
      const configuredWorkflow = configureContinueAsNew(workflow);

      // Step 3: Build context
      const context = this.buildContext(configuredWorkflow);

      // Step 4: Apply patterns and generate code
      const codeBlocks = this.generateCode(configuredWorkflow, context);

      // Step 5: Generate output files
      const workflowCode = generateWorkflowFile(
        configuredWorkflow,
        codeBlocks,
        context,
        this.options.includeComments
      );

      const activitiesCode = generateActivitiesFile(
        configuredWorkflow,
        this.options.includeComments
      );

      const workerCode = generateWorkerFile(
        configuredWorkflow,
        this.options.includeComments
      );

      const packageJson = generatePackageJson(configuredWorkflow);
      const tsConfig = generateTsConfig(this.options.strictMode);

      // Step 5: Return result
      return {
        success: true,
        workflowCode,
        activitiesCode,
        workerCode,
        packageJson,
        tsConfig,
        errors: [],
        warnings: validationWarnings,
        metadata: {
          nodeCount: configuredWorkflow.nodes?.length || 0,
          edgeCount: configuredWorkflow.edges?.length || 0,
          patternsApplied: Array.from(context.visitedNodes),
          compilationTime: Date.now() - startTime,
          version: '1.0.0',
        },
      };
    } catch (error) {
      return {
        success: false,
        errors: [
          {
            message: error instanceof Error ? error.message : String(error),
            type: 'generation',
            severity: 'fatal',
          },
        ],
        metadata: {
          nodeCount: workflow.nodes?.length || 0,
          edgeCount: workflow.edges?.length || 0,
          patternsApplied: [],
          compilationTime: Date.now() - startTime,
          version: '1.0.0',
        },
      };
    }
  }

  /**
   * Build generator context
   */
  private buildContext(workflow: WorkflowDefinition): GeneratorContext {
    const nodes = workflow.nodes || [];
    const edges = workflow.edges || [];

    // Build node map
    const nodeMap = new Map<string, WorkflowNode>();
    nodes.forEach(node => nodeMap.set(node.id, node));

    // Build edge map (source -> edges)
    const edgeMap = new Map<string, WorkflowEdge[]>();
    edges.forEach(edge => {
      if (!edgeMap.has(edge.source)) {
        edgeMap.set(edge.source, []);
      }
      edgeMap.get(edge.source)!.push(edge);
    });

    return {
      nodes,
      edges,
      variables: workflow.variables || [],
      settings: workflow.settings,
      nodeMap,
      edgeMap,
      visitedNodes: new Set(),
      resultVars: new Map(),
      proxyMap: new Map(),
      currentIndent: 0,
      workflowName: workflow.name,
    };
  }

  /**
   * Generate code by applying patterns
   */
  private generateCode(
    workflow: WorkflowDefinition,
    context: GeneratorContext
  ): CodeBlock[] {
    const codeBlocks: CodeBlock[] = [];

    // Find start node
    const startNode = this.findStartNode(workflow.nodes, workflow.edges || []);
    if (!startNode) {
      throw new Error('No start node found');
    }

    // Traverse and generate code
    this.traverseAndGenerate(startNode, context, codeBlocks);

    return codeBlocks;
  }

  /**
   * Traverse workflow graph and generate code
   */
  private traverseAndGenerate(
    node: WorkflowNode,
    context: GeneratorContext,
    codeBlocks: CodeBlock[]
  ): void {
    // Skip if already visited
    if (context.visitedNodes.has(node.id)) {
      return;
    }

    context.visitedNodes.add(node.id);

    // Find matching pattern
    for (const pattern of this.patterns) {
      if (pattern.detect(node, context)) {
        const codeBlock = pattern.generate(node, context);
        codeBlocks.push(codeBlock);
        break;
      }
    }

    // Get outgoing edges
    const outgoingEdges = context.edgeMap.get(node.id) || [];

    // Traverse connected nodes
    for (const edge of outgoingEdges) {
      const nextNode = context.nodeMap.get(edge.target);
      if (nextNode) {
        this.traverseAndGenerate(nextNode, context, codeBlocks);
      }
    }
  }

  /**
   * Find start node (trigger with no incoming edges)
   */
  private findStartNode(nodes: WorkflowNode[], edges: WorkflowEdge[]): WorkflowNode | null {
    const nodesWithIncoming = new Set(edges.map(e => e.target));

    for (const node of nodes) {
      if (node.type === 'trigger' && !nodesWithIncoming.has(node.id)) {
        return node;
      }
    }

    // Fallback: first trigger node
    return nodes.find(n => n.type === 'trigger') || null;
  }
}

// Export types and utilities
export * from './types';
export * from './patterns';
export * from './generators';
export * from './utils/validation';
