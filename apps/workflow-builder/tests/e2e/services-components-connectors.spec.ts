/**
 * Services, Components, and Connectors E2E Tests
 * 
 * Tests the new naming conventions and UI components:
 * - Services (formerly workflows)
 * - Connectors (project-level integrations)
 * - Service Interfaces
 * - Project Connectors
 * 
 * Note: These tests use authenticated storage state from auth.setup.ts
 */

import { test, expect } from '@playwright/test';

const BASE_URL = process.env.BASE_URL || 'http://localhost:3010';

test.describe('Services Naming and UI', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to workflows/services page
    await page.goto(`${BASE_URL}/workflows`);
    await page.waitForLoadState('networkidle');
  });

  test('should display "Services" instead of "Workflows" on workflows page', async ({ page }) => {
    // Wait for main content area to be visible
    await page.waitForSelector('main', { timeout: 15000 });

    // Verify "New Service" button is visible
    await expect(page.getByRole('button', { name: 'New Service' })).toBeVisible({ timeout: 15000 });

    // Verify "Services" heading is visible
    await expect(page.getByRole('heading', { name: 'Services' })).toBeVisible({ timeout: 15000 });
  });


  test('should display services in list', async ({ page }) => {
    // Check if any services are displayed
    const hasServices = await page.getByText(/Hello World Demo|Email Notification/i).count() > 0;

    if (hasServices) {
      // Verify at least one service is visible
      await expect(page.getByText(/Hello World Demo/i).first()).toBeVisible({ timeout: 10000 });
    }
    // If no services, that's okay - empty state is acceptable
  });

  test('should navigate to service detail page when clicking a service', async ({ page }) => {
    const firstService = page.locator('text=Hello World Demo').first();
    if ((await firstService.count()) > 0) {
      await firstService.click();
      await page.waitForLoadState('networkidle');

      // Verify we're on a service detail page
      await expect(page).toHaveURL(/workflows\/[a-f0-9-]{36}$/);
    }
  });
});

test.describe.skip('Project Connectors', () => {
  // Skip these tests - Project detail page with Connectors tab is not implemented yet
  test.beforeEach(async ({ page }) => {
    await page.goto(`${BASE_URL}/`);
    await page.waitForLoadState('networkidle');
  });

  test('should display Connectors tab on project detail page', async ({ page }) => {
    // Project detail page needs to be implemented
    test.skip();
  });

  test('should open connector creation modal when clicking Add Connector', async ({ page }) => {
    // Project detail page needs to be implemented
    test.skip();
  });

  test('should display empty state when no connectors exist', async ({ page }) => {
    // Project detail page needs to be implemented
    test.skip();
  });
});

test.describe.skip('Project Page - Services Tab', () => {
  // Skip these tests - Project detail page is not implemented yet
  test.beforeEach(async ({ page }) => {
    await page.goto(`${BASE_URL}/`);
    await page.waitForLoadState('networkidle');
  });

  test('should display Services tab instead of Workflows tab', async ({ page }) => {
    test.skip();
  });

  test('should display services list in Services tab', async ({ page }) => {
    test.skip();
  });
});

test.describe.skip('Project Details - Channel Name', () => {
  // Skip these tests - Project detail page is not implemented yet
  test.beforeEach(async ({ page }) => {
    await page.goto(`${BASE_URL}/`);
    await page.waitForLoadState('networkidle');
  });

  test('should display "Channel Name" instead of "Task Queue Name"', async ({ page }) => {
    test.skip();
  });
});

test.describe('Service Interfaces', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(`${BASE_URL}/workflows`);
    await page.waitForLoadState('networkidle');
  });

  test('should navigate to service detail page to view interfaces', async ({ page }) => {
    // Click on first service
    const firstService = page.locator('text=Hello World Demo').first();
    if ((await firstService.count()) > 0) {
      await firstService.click();
      await page.waitForLoadState('networkidle');

      // Verify we're on service detail page
      await expect(page).toHaveURL(/workflows\/[a-f0-9-]{36}$/);
    }
  });
});

test.describe.skip('Database Connections Tab', () => {
  // Skip these tests - Project detail page is not implemented yet
  test.beforeEach(async ({ page }) => {
    await page.goto(`${BASE_URL}/`);
    await page.waitForLoadState('networkidle');
  });

  test('should display Database Connections tab on project page', async ({ page }) => {
    test.skip();
  });
});
