/**
 * Continue-as-New Pattern
 * 
 * Automatically generates continue-as-new code for long-running workflows.
 * This pattern is workflow-level and injects history reset checks at strategic points.
 * 
 * Principle: All continue-as-new handling is automatic and invisible to users.
 */

import type { Pattern, WorkflowNode, GeneratorContext, CodeBlock } from '../types';
import { collectWorkflowState, generateStateObject } from '../../workflow-analyzer';
import { indent } from '../utils/ast-helpers';

/**
 * Continue-as-New Pattern
 * 
 * Detects: settings._longRunning?.autoContinueAsNew === true
 * Generates: Continue-as-new check code at strategic points
 */
export const ContinueAsNewPattern: Pattern = {
  name: 'continue-as-new',
  priority: 10, // Run last, after all other patterns

  detect: (node: WorkflowNode, context: GeneratorContext): boolean => {
    // This is a workflow-level pattern, so we check context.settings
    // We'll inject this code for loop nodes and signal nodes
    const config = context.settings._longRunning;
    if (!config?.autoContinueAsNew) {
      return false;
    }

    // Inject continue-as-new checks after loops and signal handlers
    return node.type === 'loop' || node.type === 'signal';
  },

  generate: (node: WorkflowNode, context: GeneratorContext): CodeBlock => {
    const config = context.settings._longRunning!;
    const maxEvents = config.maxHistoryEvents || 1000;
    const maxDuration = config.maxDurationMs || 24 * 60 * 60 * 1000;
    const workflowName = context.workflowName;
    
    // Collect all state that needs to be preserved
    const stateMap = collectWorkflowState({
      id: '',
      name: workflowName,
      nodes: context.nodes,
      edges: context.edges,
      variables: context.variables,
      settings: context.settings,
    });
    const stateObject = generateStateObject(stateMap);
    
    const imports: string[] = [];
    const declarations: string[] = [];
    
    // Only add imports once (check if already added)
    if (!context.visitedNodes.has('_continue_as_new_imports')) {
      imports.push(`import { continueAsNew, workflowInfo } from '@temporalio/workflow';`);
      context.visitedNodes.add('_continue_as_new_imports');
    }
    
    // Add declaration for workflow start time (only once, deterministic from workflow info)
    if (!context.visitedNodes.has('_workflow_start_time')) {
      declarations.push('const workflowStartTime: number = workflowInfo().runStartTime.getTime();');
      context.visitedNodes.add('_workflow_start_time');
    }
    
    const indentStr = indent(context.currentIndent);
    
    // Generate continue-as-new check code
    const checkCode = `
${indentStr}// Automatic history management (internal optimization)
${indentStr}const historyLength = workflowInfo().historyLength;
${indentStr}const elapsedTime = Date.now() - workflowStartTime;

${indentStr}if (historyLength > ${maxEvents} || elapsedTime > ${maxDuration}) {
${indentStr}  console.log('[Workflow] Resetting history for optimal performance');
${indentStr}  
${indentStr}  // Preserve all workflow state automatically
${indentStr}  await continueAsNew<typeof ${workflowName}>({
${indentStr}${stateObject}
${indentStr}  });
${indentStr}}`.trim();
    
    return {
      imports,
      declarations,
      code: checkCode,
    };
  },
};
