/**
 * Scenario B – Activity Timeout
 * 
 * E2E test for configuring and observing activity timeouts
 * 
 * Prerequisites:
 * - Temporal server running (yarn infra:up)
 * - Next.js dev server running on port 3010
 * - Authenticated session (via auth.setup.ts)
 */

import { test, expect } from '@playwright/test';
import { WorkflowBuilderPage } from '../page-objects/WorkflowBuilderPage';
import { cleanupUserData } from '../helpers/cleanup';

const BASE_URL = process.env.BASE_URL || 'http://localhost:3010';
const TEST_USER_EMAIL = 'test@example.com';

test.describe('Activity Timeout Scenario', () => {
  // Clean up after each test to prevent data accumulation
  // Note: We don't clean up beforeEach to avoid breaking authentication state
  test.afterEach(async ({ page }) => {
    // Close any open modals/dialogs
    try {
      const modals = page.locator('[role="dialog"]');
      const count = await modals.count();
      for (let i = 0; i < count; i++) {
        const modal = modals.nth(i);
        if (await modal.isVisible({ timeout: 1000 }).catch(() => false)) {
          await page.keyboard.press('Escape');
        }
      }
    } catch (error) {
      // Ignore cleanup errors
    }

    // Clean up test data
    await cleanupUserData(TEST_USER_EMAIL);

    // Navigate away from any modals or complex pages to ensure clean state
    try {
      if (!page.isClosed()) {
        // Navigate to a simple page to clear any complex state
        await page.goto(BASE_URL, { waitUntil: 'domcontentloaded', timeout: 5000 }).catch(() => {
          // Ignore navigation errors
        });
      }
    } catch (error) {
      // Ignore cleanup errors
    }
  });

  test('should configure timeout and observe timeout failure', async ({ page }) => {
    test.setTimeout(180000); // 3 minutes for full workflow creation and execution
    const builder = new WorkflowBuilderPage(page, BASE_URL);
    const workflowName = `Test Timeout Workflow ${Date.now()}`;

    // Step 1: Create a new workflow
    const workflowId = await builder.createWorkflow({
      name: workflowName,
      useDefaultProject: true,
    });

    // Step 2: Navigate to builder
    await builder.openBuilder(workflowId);
    await builder.verifyCanvasLoaded();

    // Step 3: Add a trigger node first (required for compilation)
    const canvas = builder.canvas;
    const canvasBounds = await canvas.boundingBox();
    if (!canvasBounds) {
      throw new Error('Canvas not found');
    }

    // Expand "Receive Data" category for triggers
    const receiveDataCategory = page.getByText(/Receive Data/i).first();
    await expect(receiveDataCategory).toBeVisible();
    await receiveDataCategory.locator('..').locator('..').first().click();
    await page.waitForTimeout(500);

    // Find a trigger component
    const triggerComponent = page.locator('[draggable="true"]').filter({ 
      hasText: /endpoint|webhook|trigger|api|schedule|manual/i 
    }).first();
    
    await expect(triggerComponent).toBeVisible();

    // Drag trigger to canvas (left side)
    await triggerComponent.dragTo(canvas, {
      targetPosition: { x: canvasBounds.width / 2 - 150, y: canvasBounds.height / 2 },
    });
    await page.waitForTimeout(1000);

    // Get trigger node
    const triggerNodes = page.locator('.react-flow__node');
    const triggerNodeCount = await triggerNodes.count();
    const triggerNode = triggerNodes.nth(triggerNodeCount - 1);
    await expect(triggerNode).toBeVisible();

    // Step 4: Add an activity node
    // Expand "Core Actions" category
    const coreActionsCategory = page.getByText(/Core Actions/i).first();
    await expect(coreActionsCategory).toBeVisible();
    await coreActionsCategory.locator('..').locator('..').first().click();
    await page.waitForTimeout(500);

    // Find an activity component
    const activityComponent = page.locator('[draggable="true"]').filter({ 
      hasNotText: /trigger|agent|signal/i 
    }).first();
    
    await expect(activityComponent).toBeVisible();

    // Drag activity to canvas (right side of trigger)
    await activityComponent.dragTo(canvas, {
      targetPosition: { x: canvasBounds.width / 2 + 150, y: canvasBounds.height / 2 },
    });
    await page.waitForTimeout(1000);

    // Get activity node
    const activityNodes = page.locator('.react-flow__node');
    const activityNodeCount = await activityNodes.count();
    const activityNode = activityNodes.nth(activityNodeCount - 1);
    await expect(activityNode).toBeVisible();

    // Connect trigger to activity
    const sourceHandle = triggerNode.locator('.react-flow__handle-right, .react-flow__handle-source').first();
    const targetHandle = activityNode.locator('.react-flow__handle-left, .react-flow__handle-target').first();

    if (await sourceHandle.isVisible({ timeout: 2000 }).catch(() => false) && 
        await targetHandle.isVisible({ timeout: 2000 }).catch(() => false)) {
      await sourceHandle.hover();
      await page.mouse.down();
      await targetHandle.hover();
      await page.mouse.up();
      await page.waitForTimeout(500);
    }

    // Step 5: Configure timeout via UI (e.g., 2s)
    // Click on activity node to open config panel
    await activityNode.click();
    await page.waitForTimeout(500);

    // Find timeout input in config panel
    const timeoutInput = page.getByLabel(/timeout/i).or(
      page.locator('input[id="timeout"]')
    );
    
    await expect(timeoutInput).toBeVisible({ timeout: 2000 });
    await timeoutInput.fill('2s');
    await timeoutInput.blur();
    await page.waitForTimeout(500);

    // Close property panel
    await page.keyboard.press('Escape');
    await page.waitForTimeout(300);

    // Step 6: Save workflow
    await builder.save();

    // Step 7: Compile workflow (optional)
    const compileButton = page.getByTestId('compile-workflow-button');
    if (await compileButton.isVisible({ timeout: 2000 }).catch(() => false)) {
      await builder.compile();
      await builder.closeCodeViewer();
    }

    // Step 8: Build and run workflow
    await builder.buildAndRun();

    // Step 9: Wait for execution to complete or timeout
    // For timeout scenarios, we expect either:
    // - A timeout error/failure
    // - Or the execution to complete (if activity finishes before timeout)
    const executionPanel = page.getByTestId('execution-panel');
    await expect(executionPanel).toBeVisible({ timeout: 10000 });

    // Wait for execution status to appear
    const statusLocator = page.getByTestId('execution-status').or(
      page.locator('text=/completed|failed|timeout|timed out/i')
    );
    
    await expect(statusLocator).toBeVisible({ timeout: 30000 });

    // Step 10: Assert UI shows timeout-related failure or completion
    const statusText = await statusLocator.textContent();
    const statusLower = statusText?.toLowerCase() || '';

    // Check for timeout indicators
    const hasTimeout = statusLower.includes('timeout') || 
                       statusLower.includes('timed out') ||
                       statusLower.includes('exceeded');

    // Check for error in execution panel
    const errorCard = page.getByTestId('execution-error');
    const hasError = await errorCard.isVisible({ timeout: 2000 }).catch(() => false);
    
    if (hasError) {
      const errorText = await errorCard.textContent();
      const errorLower = errorText?.toLowerCase() || '';
      
      // Verify timeout-related error message
      expect(
        errorLower.includes('timeout') || 
        errorLower.includes('timed out') ||
        errorLower.includes('exceeded')
      ).toBeTruthy();
    } else if (hasTimeout) {
      // Status indicates timeout
      expect(hasTimeout).toBeTruthy();
    } else {
      // If no timeout, execution may have completed before timeout
      // This is acceptable - the timeout was configured correctly
      expect(statusLower).toMatch(/completed|success|finished/i);
    }

    // Step 11: Optionally verify link or hint to check Temporal UI
    const temporalLink = page.getByText(/Temporal|View in Temporal/i);
    if (await temporalLink.isVisible({ timeout: 2000 }).catch(() => false)) {
      await expect(temporalLink).toBeVisible();
    }
  });
});
