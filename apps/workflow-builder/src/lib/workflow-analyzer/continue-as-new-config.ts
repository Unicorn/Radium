/**
 * Continue-as-New Configuration
 * 
 * Automatically configures continue-as-new settings for workflows based on
 * their classification. All configuration is internal and never exposed to users.
 */

import type { WorkflowDefinition, WorkflowSettings } from '../compiler/types';
import { classifyWorkflow, type WorkflowType } from './service-classifier';

/**
 * Default configuration values
 */
const DEFAULT_MAX_HISTORY_EVENTS = 1000;
const DEFAULT_MAX_DURATION_MS = 24 * 60 * 60 * 1000; // 24 hours

/**
 * Configure continue-as-new settings for a workflow
 * 
 * For Services:
 * - Automatically enable continue-as-new
 * - Set max history events to 1000 (default)
 * - Set max duration to 24 hours (default)
 * - Always preserve state
 * 
 * For Tasks:
 * - Disable continue-as-new (not needed for short-running workflows)
 */
export function configureContinueAsNew(
  definition: WorkflowDefinition,
  workflowType?: WorkflowType
): WorkflowDefinition {
  // Classify workflow if type not provided
  const type = workflowType || classifyWorkflow(definition);
  
  // Create updated settings
  const updatedSettings: WorkflowSettings = {
    ...definition.settings,
    _workflowType: type,
    _longRunning: {
      autoContinueAsNew: type === 'service',
      maxHistoryEvents: DEFAULT_MAX_HISTORY_EVENTS,
      maxDurationMs: DEFAULT_MAX_DURATION_MS,
      preserveState: true, // Always preserve all state
    },
  };
  
  // If it's a task, disable continue-as-new
  if (type === 'task') {
    updatedSettings._longRunning = {
      autoContinueAsNew: false,
      maxHistoryEvents: DEFAULT_MAX_HISTORY_EVENTS,
      maxDurationMs: DEFAULT_MAX_DURATION_MS,
      preserveState: true,
    };
  }
  
  return {
    ...definition,
    settings: updatedSettings,
  };
}

/**
 * Get continue-as-new configuration for a workflow
 * Returns undefined if continue-as-new is not enabled
 */
export function getContinueAsNewConfig(settings: WorkflowSettings) {
  return settings._longRunning?.autoContinueAsNew 
    ? settings._longRunning 
    : undefined;
}

