/**
 * Workflow State Storage Adapter
 * 
 * In-memory state storage using Temporal workflow state
 * This is the default storage type for workflow-level state variables
 */

import type { StateStorageAdapter } from './external-state';

/**
 * Workflow State Adapter
 * 
 * Uses in-memory variables in the Temporal workflow.
 * State is stored in workflow execution history.
 */
export class WorkflowStateAdapter implements StateStorageAdapter {
  private state: Map<string, any> = new Map();

  constructor() {
    // State is managed in-memory within the workflow
  }

  async get(variableId: string, variableName: string): Promise<any> {
    return this.state.get(variableName) ?? null;
  }

  async set(variableId: string, variableName: string, value: any): Promise<void> {
    this.state.set(variableName, value);
  }

  async append(variableId: string, variableName: string, value: any): Promise<void> {
    const current = this.state.get(variableName) || [];
    if (!Array.isArray(current)) {
      throw new Error(`Variable ${variableName} is not an array`);
    }
    current.push(value);
    this.state.set(variableName, current);
  }

  async increment(variableId: string, variableName: string, amount: number = 1): Promise<number> {
    const current = this.state.get(variableName) || 0;
    const newValue = Number(current) + amount;
    this.state.set(variableName, newValue);
    return newValue;
  }

  async decrement(variableId: string, variableName: string, amount: number = 1): Promise<number> {
    const current = this.state.get(variableName) || 0;
    const newValue = Number(current) - amount;
    this.state.set(variableName, newValue);
    return newValue;
  }

  async delete(variableId: string, variableName: string): Promise<void> {
    this.state.delete(variableName);
  }

  async exists(variableId: string, variableName: string): Promise<boolean> {
    return this.state.has(variableName);
  }

  /**
   * Get all state variables (for debugging/monitoring)
   */
  getAll(): Map<string, any> {
    return new Map(this.state);
  }

  /**
   * Clear all state (for testing)
   */
  clear(): void {
    this.state.clear();
  }
}

