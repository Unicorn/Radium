/**
 * Import Statement Generator
 * Generates and manages import statements for workflow code
 */

import type { WorkflowNode, ImportStatement } from '../types';

/**
 * Generate base Temporal imports based on node types
 */
export function generateBaseImports(nodes: WorkflowNode[]): string[] {
  const imports = new Set<string>();
  const temporalImports = new Set<string>();

  // Always include proxyActivities for activities
  const hasActivities = nodes.some(n => n.type === 'activity' || n.type === 'agent');
  if (hasActivities) {
    temporalImports.add('proxyActivities');
  }

  // Check for child workflows
  const hasChildWorkflows = nodes.some(n => n.type === 'child-workflow');
  if (hasChildWorkflows) {
    temporalImports.add('startChild');
    temporalImports.add('executeChild');
  }

  // Check for signals
  const hasSignals = nodes.some(n => n.type === 'signal');
  if (hasSignals) {
    temporalImports.add('defineSignal');
    temporalImports.add('setHandler');
  }

  // Check for conditions/wait
  const hasConditions = nodes.some(n => n.type === 'condition' || n.type === 'loop');
  if (hasConditions) {
    temporalImports.add('condition');
  }

  // Check for sleep/delays
  const hasDelays = nodes.some(n =>
    n.type === 'retry' || (n.data.config && 'delay' in n.data.config)
  );
  if (hasDelays) {
    temporalImports.add('sleep');
  }

  // Generate Temporal import
  if (temporalImports.size > 0) {
    imports.add(`import { ${Array.from(temporalImports).sort().join(', ')} } from '@temporalio/workflow';`);
  }

  // Generate activities import
  if (hasActivities) {
    imports.add(`import type * as activities from './activities';`);
  }

  return Array.from(imports);
}

/**
 * Generate import for specific pattern
 */
export function generatePatternImport(patternName: string): string[] {
  const imports: string[] = [];

  switch (patternName) {
    case 'activity-proxy':
      imports.push(`import { proxyActivities } from '@temporalio/workflow';`);
      break;

    case 'child-workflow':
      imports.push(`import { startChild, executeChild } from '@temporalio/workflow';`);
      break;

    case 'signal-handler':
      imports.push(`import { defineSignal, setHandler } from '@temporalio/workflow';`);
      break;

    case 'query-handler':
      imports.push(`import { defineQuery, setHandler } from '@temporalio/workflow';`);
      break;

    case 'state-management':
      // No special imports needed
      break;

    case 'condition':
      imports.push(`import { condition } from '@temporalio/workflow';`);
      break;

    case 'timer':
      imports.push(`import { sleep } from '@temporalio/workflow';`);
      break;
  }

  return imports;
}

/**
 * Create import statement structure
 */
export function createImport(
  module: string,
  items: string[],
  type: 'named' | 'default' | 'namespace' = 'named'
): ImportStatement {
  return {
    module,
    items,
    type,
  };
}

/**
 * Add type import qualifier
 */
export function makeTypeImport(importStmt: string): string {
  if (importStmt.includes('import type')) {
    return importStmt;
  }

  return importStmt.replace('import {', 'import type {');
}
