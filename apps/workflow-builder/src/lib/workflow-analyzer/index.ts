/**
 * Workflow Analyzer
 * 
 * Main entry point for workflow analysis functionality.
 * Provides automatic classification, configuration, and state collection.
 */

export { classifyWorkflow, hasScheduledTriggers, type WorkflowType } from './service-classifier';
export { configureContinueAsNew, getContinueAsNewConfig } from './continue-as-new-config';
export { detectLongRunningLoops } from './loop-detector';
export { collectWorkflowState, generateStateObject, type StateMap } from './state-collector';

