/**
 * Workflows/Services List and Details E2E Tests
 *
 * Tests workflow listing, viewing details, and metadata display
 *
 * Note: These tests use authenticated storage state from auth.setup.ts
 * Note: UI displays "Services" instead of "Workflows"
 */

import { test, expect } from '@playwright/test';

const BASE_URL = process.env.BASE_URL || 'http://localhost:3010';

test.describe('Workflows List Page', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to workflows - already authenticated via storage state
    await page.goto(`${BASE_URL}/workflows`);
    await page.waitForLoadState('networkidle');
  });

  test('should display workflows list page', async ({ page }) => {
    // Verify page elements - UI shows "Services" instead of "Workflows"
    await expect(page.getByRole('heading', { name: 'Services' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'New Service' })).toBeVisible();
  });

  test('should display demo workflows', async ({ page }) => {
    // Verify demo workflows are visible
    await expect(page.getByText('Hello World Demo')).toBeVisible();
    await expect(page.getByText('Email Notification Workflow')).toBeVisible();
  });

  test('should navigate to workflow details when clicked', async ({ page }) => {
    // Click on Hello World Demo - use link or card
    await page.getByText('Hello World Demo').first().click();

    // Verify navigation to details page (ID will vary)
    await expect(page).toHaveURL(/workflows\/[a-f0-9-]{36}$/);
  });
});

test.describe('Workflow Details Page', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to workflows and click on first workflow
    await page.goto(`${BASE_URL}/workflows`);
    await page.waitForLoadState('networkidle');
    await page.getByText('Hello World Demo').first().click();
    await page.waitForLoadState('networkidle');
  });

  test('should display workflow details', async ({ page }) => {
    // Verify workflow details page - check for workflow name in heading or content
    await expect(page.getByText('Hello World Demo')).toBeVisible();
    // Description might vary - check for any description text
    await expect(page.getByText(/demonstration workflow/i)).toBeVisible();
  });

  test('should display action buttons', async ({ page }) => {
    // Verify action buttons - at least Edit should be present
    await expect(page.getByRole('button', { name: /edit/i })).toBeVisible();
  });

  test('should display workflow metadata', async ({ page }) => {
    // Verify workflow metadata - check for key metadata fields
    // The kebab name should be visible
    await expect(page.getByText('hello-world-demo')).toBeVisible();
  });

  test('should navigate to edit page when Edit clicked', async ({ page }) => {
    // Click Edit button
    await page.getByRole('button', { name: /edit/i }).click();

    // Verify navigation to edit page
    await expect(page).toHaveURL(/workflows\/[a-f0-9-]{36}\/edit$/);
  });
});

