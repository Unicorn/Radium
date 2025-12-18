/**
 * Workflow Test Helper Functions for E2E Tests
 *
 * Provides reusable utilities for workflow creation, compilation, and execution
 */

import type { Page } from '@playwright/test';
import { expect } from '@playwright/test';

const BASE_URL = process.env.BASE_URL || 'http://localhost:3010';

export interface CreateWorkflowOptions {
  name: string;
  description?: string;
  projectName?: string;
  useDefaultProject?: boolean;
}

/**
 * Create a new workflow through the UI
 * Returns the workflow ID
 */
export async function createWorkflow(
  page: Page,
  options: CreateWorkflowOptions
): Promise<string> {
  // Navigate to new workflow page
  await page.goto(`${BASE_URL}/workflows/new`);
  await page.waitForLoadState('networkidle');

  // Fill in workflow name (identifier auto-generates)
  await page.getByLabel(/workflow name/i).fill(options.name);

  // Fill in description if provided
  if (options.description) {
    await page.getByLabel(/description/i).fill(options.description);
  }

  // Handle project selection
  if (options.projectName) {
    // Check if we need to create a new project
    const newProjectOption = page.getByText('+ Create New Project');
    if (await newProjectOption.count() > 0) {
      await newProjectOption.click();
      await page.getByPlaceholder(/enter project name/i).fill(options.projectName);
    }
  } else if (!options.useDefaultProject) {
    // Use first available project
    const projectSelect = page.getByRole('combobox', { name: /project/i });
    if (await projectSelect.count() > 0) {
      await projectSelect.click();
      await page.getByRole('option').first().click();
    }
  }

  // Submit form
  await page.getByRole('button', { name: /create & edit/i }).click();

  // Wait for redirect to workflow edit page
  await page.waitForURL(/\/workflows\/[a-f0-9-]{36}\/edit/, { timeout: 10000 });

  // Extract workflow ID from URL
  const match = page.url().match(/\/workflows\/([a-f0-9-]{36})/);
  if (!match) {
    throw new Error('Failed to extract workflow ID from URL');
  }

  return match[1];
}

/**
 * Navigate to workflow builder page
 */
export async function openWorkflowBuilder(page: Page, workflowId: string): Promise<void> {
  await page.goto(`${BASE_URL}/workflows/${workflowId}/builder`);
  await page.waitForLoadState('networkidle');

  // Verify builder page loaded
  await expect(page.locator('.react-flow')).toBeVisible({ timeout: 10000 });
}

/**
 * Add an activity node to the workflow canvas
 */
export async function addActivityNode(
  page: Page,
  options: {
    name: string;
    timeout?: string;
    retryPolicy?: 'exponential' | 'linear' | 'none';
  }
): Promise<void> {
  // Find activity in component palette
  const activityComponent = page.getByText(/sample.*activity/i).first();
  await expect(activityComponent).toBeVisible();

  // Get canvas bounds for drop calculation
  const canvas = page.locator('.react-flow');
  const canvasBounds = await canvas.boundingBox();
  if (!canvasBounds) {
    throw new Error('Canvas not found');
  }

  // Drag component onto canvas
  await activityComponent.hover();
  await page.mouse.down();
  await page.mouse.move(
    canvasBounds.x + canvasBounds.width / 2,
    canvasBounds.y + canvasBounds.height / 2
  );
  await page.mouse.up();

  // Wait for node to appear
  await page.waitForTimeout(500);

  // Click on the newly added node to select it
  const activityNode = page.locator('[data-node-type="activity"]').last();
  await expect(activityNode).toBeVisible();
  await activityNode.click();

  // Configure in property panel
  await page.waitForTimeout(300);

  // Configure activity name if property panel opens
  const nameInput = page.getByLabel(/activity name/i);
  if (await nameInput.count() > 0) {
    await nameInput.fill(options.name);
  }

  if (options.timeout) {
    const timeoutInput = page.getByLabel(/timeout/i);
    if (await timeoutInput.count() > 0) {
      await timeoutInput.fill(options.timeout);
    }
  }

  if (options.retryPolicy) {
    const retrySelect = page.getByLabel(/retry policy/i);
    if (await retrySelect.count() > 0) {
      await retrySelect.selectOption(options.retryPolicy);
    }
  }

  // Close property panel
  await page.keyboard.press('Escape');
}

/**
 * Connect two nodes on the workflow canvas
 */
export async function connectNodes(
  page: Page,
  sourceNodeId: string,
  targetNodeId: string
): Promise<void> {
  // Find source and target nodes
  const sourceNode = page.locator(`[data-id="${sourceNodeId}"]`);
  const targetNode = page.locator(`[data-id="${targetNodeId}"]`);

  await expect(sourceNode).toBeVisible();
  await expect(targetNode).toBeVisible();

  // Perform drag-and-drop connection
  const sourceHandle = sourceNode.locator('.react-flow__handle-right');
  const targetHandle = targetNode.locator('.react-flow__handle-left');

  await sourceHandle.hover();
  await page.mouse.down();
  await targetHandle.hover();
  await page.mouse.up();

  await page.waitForTimeout(300);
}

/**
 * Save the current workflow
 */
export async function saveWorkflow(page: Page): Promise<void> {
  const saveButton = page.getByRole('button', { name: /^save$/i });
  await saveButton.click();

  // Wait for save to complete
  await page.waitForTimeout(1000);
}

/**
 * Compile the workflow and open code viewer
 */
export async function compileWorkflow(page: Page): Promise<void> {
  const compileButton = page.getByRole('button', { name: /compile/i });
  await expect(compileButton).toBeVisible();
  await compileButton.click();

  // Wait for compilation to complete and modal to appear
  await expect(page.getByText(/generated typescript code/i)).toBeVisible({
    timeout: 15000
  });
}

/**
 * Get the generated code from the code viewer modal
 */
export async function getGeneratedCode(page: Page, tab: 'workflow' | 'activities' | 'worker'): Promise<string> {
  // Switch to the appropriate tab
  const tabButton = page.getByRole('button', { name: new RegExp(tab, 'i') });
  await tabButton.click();
  await page.waitForTimeout(300);

  // Extract code content
  const codeBlock = page.locator('pre code');
  const code = await codeBlock.textContent();

  return code || '';
}

/**
 * Close the code viewer modal
 */
export async function closeCodeViewer(page: Page): Promise<void> {
  const closeButton = page.getByRole('button', { name: /close/i }).last();
  await closeButton.click();

  // Wait for modal to close
  await expect(page.getByText(/generated typescript code/i)).not.toBeVisible();
}

/**
 * Execute the workflow with given input
 * Returns the execution ID
 */
export async function executeWorkflow(page: Page, input: any): Promise<string> {
  // Click Build & Run button
  const executeButton = page.getByRole('button', { name: /build.*workflow/i });
  await expect(executeButton).toBeVisible();
  await executeButton.click();

  // Execution should start
  await page.waitForTimeout(1000);

  // Extract execution ID from execution panel or URL
  // This is a simplified approach - actual implementation may vary
  const executionPanel = page.locator('[data-testid="execution-panel"]');
  const executionId = await executionPanel.getAttribute('data-execution-id');

  if (!executionId) {
    // Fallback: check URL or other indicators
    console.warn('Execution ID not found in panel, using timestamp-based ID');
    return `exec-${Date.now()}`;
  }

  return executionId;
}

/**
 * Wait for execution to complete
 */
export async function waitForExecutionComplete(
  page: Page,
  timeout: number = 30000
): Promise<'completed' | 'failed'> {
  // Wait for execution status to be completed or failed
  const statusLocator = page.locator('[data-testid="execution-status"]');

  try {
    await expect(statusLocator).toHaveText(/completed|failed/i, { timeout });
    const status = await statusLocator.textContent();
    return status?.toLowerCase().includes('completed') ? 'completed' : 'failed';
  } catch (error) {
    throw new Error(`Execution did not complete within ${timeout}ms`);
  }
}

/**
 * Get execution result/output
 */
export async function getExecutionResult(page: Page): Promise<any> {
  const outputLocator = page.locator('[data-testid="execution-output"]');
  const outputText = await outputLocator.textContent();

  if (!outputText) {
    return null;
  }

  try {
    return JSON.parse(outputText);
  } catch {
    return outputText;
  }
}

/**
 * Get execution error if failed
 */
export async function getExecutionError(page: Page): Promise<string | null> {
  const errorLocator = page.locator('[data-testid="execution-error"]');

  if (await errorLocator.count() === 0) {
    return null;
  }

  return await errorLocator.textContent();
}

/**
 * Navigate back to workflow list
 */
export async function navigateToWorkflowList(page: Page): Promise<void> {
  await page.goto(`${BASE_URL}/workflows`);
  await page.waitForLoadState('networkidle');
}

/**
 * Delete a workflow (cleanup helper)
 */
export async function deleteWorkflow(page: Page, workflowId: string): Promise<void> {
  // Navigate to workflow detail page
  await page.goto(`${BASE_URL}/workflows/${workflowId}`);
  await page.waitForLoadState('networkidle');

  // Look for delete button (may be in a menu)
  const deleteButton = page.getByRole('button', { name: /delete|remove/i });

  if (await deleteButton.count() > 0) {
    await deleteButton.click();

    // Confirm deletion if prompted
    const confirmButton = page.getByRole('button', { name: /confirm|yes|delete/i });
    if (await confirmButton.count() > 0) {
      await confirmButton.click();
    }

    // Wait for redirect
    await page.waitForURL(/\/workflows$/, { timeout: 5000 });
  }
}
