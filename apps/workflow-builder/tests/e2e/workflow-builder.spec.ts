/**
 * Workflow Builder E2E Tests
 * 
 * Tests the workflow builder/editor UI and functionality
 * 
 * Note: These tests use authenticated storage state from auth.setup.ts
 */

import { test, expect } from '@playwright/test';

const BASE_URL = process.env.BASE_URL || 'http://localhost:3010';

test.describe('Workflow Editor (Edit Page)', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to workflows and open first workflow editor
    await page.goto(`${BASE_URL}/workflows`);
    await page.waitForLoadState('networkidle');
    await page.getByText('Hello World Demo').first().click();
    await page.waitForLoadState('networkidle');
    
    // Click Edit to go to editor
    await page.getByRole('button', { name: 'Edit' }).click();
    await page.waitForLoadState('networkidle');
  });

  test('should display workflow editor page', async ({ page }) => {
    // Verify page loaded with workflow info
    await expect(page.getByText('Hello World Demo')).toBeVisible();
  });

  test('should display editor controls for active workflow', async ({ page }) => {
    // Verify editor shows that workflow is active and can't be edited
    await expect(page.getByText(/Read-only|Active/i)).toBeVisible();
    await expect(page.getByRole('button', { name: /Pause to Edit/i })).toBeVisible();
  });

  test('should display workflow canvas with React Flow', async ({ page }) => {
    // Verify React Flow canvas is present
    await expect(page.locator('.react-flow')).toBeVisible();
  });

  test('should display component palette sidebar', async ({ page }) => {
    // Verify component palette exists
    await expect(page.getByText('Components')).toBeVisible();
    await expect(page.getByText(/Editing disabled|No components/i)).toBeVisible();
  });
});

test.describe('Workflow Builder (Builder Page)', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to workflows and get first workflow ID
    await page.goto(`${BASE_URL}/workflows`);
    await page.waitForLoadState('networkidle');
    
    // Get the first workflow's URL
    const firstWorkflow = page.getByText('Hello World Demo').first();
    await firstWorkflow.click();
    await page.waitForLoadState('networkidle');
    
    // Get current URL and navigate to builder page
    const currentUrl = page.url();
    const workflowId = currentUrl.match(/workflows\/([a-f0-9-]{36})/)?.[1];
    if (workflowId) {
      await page.goto(`${BASE_URL}/workflows/${workflowId}/builder`);
      await page.waitForLoadState('networkidle');
    }
  });

  test('should display workflow builder page', async ({ page }) => {
    // Verify page loaded
    await expect(page.getByText('Hello World Demo')).toBeVisible();
  });

  test('should display builder controls', async ({ page }) => {
    // Verify builder controls
    await expect(page.getByRole('button', { name: /Settings/i })).toBeVisible();
    await expect(page.getByRole('button', { name: /Build Workflow/i })).toBeVisible();
    await expect(page.getByRole('button', { name: /View Code/i })).toBeVisible();
  });

  test('should display component palette with search', async ({ page }) => {
    // Verify component palette
    await expect(page.getByText('Components')).toBeVisible();
    await expect(page.getByText(/Drag onto canvas/i)).toBeVisible();
    await expect(page.getByRole('textbox', { name: /Search components/i })).toBeVisible();
  });

  test('should display filter buttons', async ({ page }) => {
    // Verify filter buttons exist
    await expect(page.getByRole('button', { name: /All/i })).toBeVisible();
    await expect(page.getByRole('button', { name: /Activities/i })).toBeVisible();
    await expect(page.getByRole('button', { name: /Agents/i })).toBeVisible();
  });

  test('should display execution panel', async ({ page }) => {
    // Verify execution panel
    await expect(page.getByText(/Ready to Run/i)).toBeVisible();
    await expect(page.getByRole('button', { name: /Build & Run Workflow/i })).toBeVisible();
  });

  test('should display workflow canvas placeholder', async ({ page }) => {
    // Verify canvas area
    await expect(page.getByText(/Workflow Builder Canvas/i)).toBeVisible();
  });
});

test.describe('Workflow Code Generation', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to builder
    await page.goto(`${BASE_URL}/workflows`);
    await page.waitForLoadState('networkidle');
    const firstWorkflow = page.getByText('Hello World Demo').first();
    await firstWorkflow.click();
    await page.waitForLoadState('networkidle');
    
    const currentUrl = page.url();
    const workflowId = currentUrl.match(/workflows\/([a-f0-9-]{36})/)?.[1];
    if (workflowId) {
      await page.goto(`${BASE_URL}/workflows/${workflowId}/builder`);
      await page.waitForLoadState('networkidle');
    }
  });

  test('should have View Code button', async ({ page }) => {
    // Verify View Code button exists
    await expect(page.getByRole('button', { name: /View Code/i })).toBeVisible();
  });
});
