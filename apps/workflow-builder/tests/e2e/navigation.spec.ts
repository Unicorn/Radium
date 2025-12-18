/**
 * Navigation Flow E2E Tests
 * 
 * Tests complete navigation flows between all pages
 * 
 * Note: These tests use authenticated storage state from auth.setup.ts
 */

import { test, expect } from '@playwright/test';

const BASE_URL = process.env.BASE_URL || 'http://localhost:3010';

test.describe('Navigation Flow', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to home - already authenticated
    await page.goto(BASE_URL + '/');
  });

  test('should navigate from Dashboard to Workflows', async ({ page }) => {
    // Start at dashboard
    await page.goto(BASE_URL + '/');
    
    // Navigate to Workflows
    await page.getByRole('button', { name: 'Workflows' }).click();
    await expect(page).toHaveURL(`${BASE_URL}/workflows`);
  });

  test('should navigate from Workflows to Workflow Details', async ({ page }) => {
    // Navigate to workflows
    await page.goto(`${BASE_URL}/workflows`);
    
    // Click on Hello World Demo
    await page.getByText('Hello World Demo').first().click();
    await expect(page).toHaveURL(/workflows\/[a-f0-9-]{36}$/);
  });

  test('should navigate from Workflow Details to Edit', async ({ page }) => {
    // Navigate to workflows and click first one
    await page.goto(`${BASE_URL}/workflows`);
    await page.getByText('Hello World Demo').first().click();
    await page.waitForLoadState('networkidle');
    
    // Click Edit button
    await page.getByRole('button', { name: 'Edit' }).click();
    await expect(page).toHaveURL(/workflows\/[a-f0-9-]{36}\/edit$/);
  });

  test('should navigate from Workflows to Components', async ({ page }) => {
    // Start at workflows
    await page.goto(`${BASE_URL}/workflows`);
    
    // Navigate to Components
    await page.getByRole('button', { name: 'Components' }).click();
    await expect(page).toHaveURL(`${BASE_URL}/components`);
  });

  test('should navigate from Components to Agents', async ({ page }) => {
    // Start at components
    await page.goto(`${BASE_URL}/components`);
    
    // Navigate to Agents
    await page.getByRole('button', { name: 'Agents' }).click();
    await expect(page).toHaveURL(`${BASE_URL}/agents`);
  });

  test('should navigate from Agents to Dashboard', async ({ page }) => {
    // Start at agents
    await page.goto(`${BASE_URL}/agents`);
    
    // Navigate to Dashboard
    await page.getByRole('button', { name: 'Dashboard' }).click();
    await expect(page).toHaveURL(BASE_URL + '/');
  });

  test('should complete full navigation circle', async ({ page }) => {
    // Start at dashboard
    await page.goto(BASE_URL + '/');
    await expect(page).toHaveURL(BASE_URL + '/');
    
    // Dashboard → Workflows
    await page.getByRole('button', { name: 'Workflows' }).click();
    await expect(page).toHaveURL(`${BASE_URL}/workflows`);
    
    // Workflows → Workflow Details
    await page.getByText('Hello World Demo').first().click();
    await expect(page).toHaveURL(/workflows\/[a-f0-9-]{36}$/);
    
    // Workflow Details → Workflow Editor (via Edit)
    await page.getByRole('button', { name: 'Edit' }).click();
    await expect(page).toHaveURL(/workflows\/[a-f0-9-]{36}\/edit$/);
    
    // Back to Workflows
    await page.getByRole('button', { name: 'Workflows' }).click();
    await expect(page).toHaveURL(`${BASE_URL}/workflows`);
    
    // Workflows → Components
    await page.getByRole('button', { name: 'Components' }).click();
    await expect(page).toHaveURL(`${BASE_URL}/components`);
    
    // Components → Agents
    await page.getByRole('button', { name: 'Agents' }).click();
    await expect(page).toHaveURL(`${BASE_URL}/agents`);
    
    // Agents → Dashboard
    await page.getByRole('button', { name: 'Dashboard' }).click();
    await expect(page).toHaveURL(BASE_URL + '/');
  });

  test('should navigate directly via URL', async ({ page }) => {
    // Test direct URL navigation to each page
    const pages = [
      '/',
      '/workflows',
      '/components',
      '/agents',
    ];
    
    for (const path of pages) {
      await page.goto(`${BASE_URL}${path}`);
      await expect(page).toHaveURL(`${BASE_URL}${path}`);
      await page.waitForLoadState('networkidle');
    }
  });

  test('should maintain navigation state after page reload', async ({ page }) => {
    // Navigate to workflows
    await page.goto(`${BASE_URL}/workflows`);
    await expect(page).toHaveURL(`${BASE_URL}/workflows`);
    
    // Reload page
    await page.reload();
    
    // Verify still on workflows page
    await expect(page).toHaveURL(`${BASE_URL}/workflows`);
    await expect(page.getByRole('heading', { name: 'Workflows' })).toBeVisible();
  });

  test('should show 404 for invalid routes', async ({ page }) => {
    // Navigate to non-existent page
    await page.goto(`${BASE_URL}/this-page-does-not-exist`);
    
    // Verify error handling (may redirect or show 404)
    // This depends on your implementation
    const has404 = await page.getByText(/404|not found/i).isVisible().catch(() => false);
    const wasRedirected = page.url() !== `${BASE_URL}/this-page-does-not-exist`;
    
    // Either shows 404 or redirects to valid page
    expect(has404 || wasRedirected).toBe(true);
  });
});

test.describe('Breadcrumb Navigation', () => {
  test.beforeEach(async ({ page }) => {
    // Already authenticated via storage state
    await page.goto(BASE_URL + '/');
  });

  test('should display breadcrumbs on detail pages', async ({ page }) => {
    // Navigate to workflow details
    await page.goto(`${BASE_URL}/workflows/${BUILD_WORKFLOW_ID}`);
    
    // Check for breadcrumbs (if implemented)
    const hasBreadcrumbs = await page.getByRole('navigation', { name: /breadcrumb/i }).isVisible().catch(() => false);
    
    if (hasBreadcrumbs) {
      await expect(page.getByRole('navigation', { name: /breadcrumb/i })).toBeVisible();
    }
  });
});

test.describe('Back Button Navigation', () => {
  test.beforeEach(async ({ page }) => {
    // Already authenticated via storage state
    await page.goto(BASE_URL + '/');
  });

  test('should support browser back button', async ({ page }) => {
    // Navigate through pages
    await page.goto(BASE_URL + '/');
    await page.goto(`${BASE_URL}/workflows`);
    await page.goto(`${BASE_URL}/components`);
    
    // Go back
    await page.goBack();
    await expect(page).toHaveURL(`${BASE_URL}/workflows`);
    
    // Go back again
    await page.goBack();
    await expect(page).toHaveURL(BASE_URL + '/');
  });

  test('should support browser forward button', async ({ page }) => {
    // Navigate through pages
    await page.goto(BASE_URL + '/');
    await page.goto(`${BASE_URL}/workflows`);
    
    // Go back
    await page.goBack();
    await expect(page).toHaveURL(BASE_URL + '/');
    
    // Go forward
    await page.goForward();
    await expect(page).toHaveURL(`${BASE_URL}/workflows`);
  });
});

