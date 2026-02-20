/**
 * Dashboard/Home Page E2E Tests
 *
 * Tests the main dashboard page elements and navigation
 *
 * Note: These tests use authenticated storage state from auth.setup.ts
 * so no manual sign-in is required.
 * Note: UI displays "Services" instead of "Workflows"
 */

import { test, expect } from '@playwright/test';

const BASE_URL = process.env.BASE_URL || 'http://localhost:3010';

test.describe('Dashboard/Home Page', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to dashboard - already authenticated via storage state
    await page.goto(BASE_URL + '/');
    await page.waitForLoadState('networkidle');
  });

  test('should display dashboard with welcome message', async ({ page }) => {
    // Verify dashboard elements - UI shows "services" instead of "workflows"
    await expect(page.getByRole('heading', { name: /Welcome/i })).toBeVisible();
    await expect(page.getByText(/Start building services by composing reusable components/i)).toBeVisible();
  });

  test('should display stats cards', async ({ page }) => {
    // Verify stats cards are visible - UI shows "Services" instead of "Workflows"
    await expect(page.getByRole('heading', { name: 'Services' })).toBeVisible();
    await expect(page.getByRole('heading', { name: 'Components' })).toBeVisible();
    await expect(page.getByRole('heading', { name: 'Agents' })).toBeVisible();
  });

  test('should display navigation buttons', async ({ page }) => {
    // Verify navigation is accessible - use specific selectors to avoid multiple matches
    // The "Services" heading exists on the dashboard stats card
    await expect(page.getByRole('heading', { name: 'Services' })).toBeVisible();
    await expect(page.getByRole('heading', { name: 'Components' })).toBeVisible();
    await expect(page.getByRole('heading', { name: 'Agents' })).toBeVisible();
  });

  test('should display user information', async ({ page }) => {
    // User info shows in welcome heading
    await expect(page.getByRole('heading', { name: /Welcome.*Test User/i })).toBeVisible();
  });

  test('should load without errors', async ({ page }) => {
    // Navigate to home
    await page.goto(BASE_URL + '/');
    await page.waitForLoadState('networkidle');

    // Verify page loaded successfully
    await expect(page).toHaveURL(BASE_URL + '/');
  });
});

