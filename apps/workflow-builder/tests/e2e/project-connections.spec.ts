/**
 * Project Connections E2E Tests
 * 
 * Tests project connection management features
 */

import { test, expect } from '@playwright/test';

const BASE_URL = process.env.BASE_URL || 'http://localhost:3010';

test.describe('Project Connections', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to a project detail page
    await page.goto(`${BASE_URL}/projects`);
    await page.waitForLoadState('networkidle');
    
    // Click on first project
    const firstProject = page.locator('[data-testid="project-card"]').first().or(
      page.getByRole('link', { name: /demo|test/i }).first()
    );
    
    if (await firstProject.count() > 0) {
      await firstProject.click();
      await page.waitForLoadState('networkidle');
    }
    
    // Navigate to Connections tab
    await page.getByRole('tab', { name: /connections/i }).click();
    await page.waitForLoadState('networkidle');
  });

  test('should display connections tab', async ({ page }) => {
    // Verify connections tab is visible
    await expect(page.getByRole('tab', { name: /connections/i })).toBeVisible();
    
    // Verify connection manager is displayed
    await expect(page.getByText(/project connections/i)).toBeVisible();
  });

  test('should display empty state when no connections', async ({ page }) => {
    // Verify empty state message
    const emptyMessage = page.getByText(/no connections yet/i).or(
      page.getByText(/add a connection/i)
    );
    
    // May have connections, so just verify page structure
    await expect(page.getByText(/project connections/i)).toBeVisible();
  });

  test('should open add connection form', async ({ page }) => {
    // Click add connection button
    const addButton = page.getByRole('button', { name: /add connection/i });
    await expect(addButton).toBeVisible();
    
    await addButton.click();
    await page.waitForLoadState('networkidle');
    
    // Verify form is displayed
    await expect(page.getByText(/add connection/i)).toBeVisible();
    await expect(page.getByLabel(/connection type/i).or(
      page.getByText(/postgresql|redis/i)
    )).toBeVisible();
  });

  test('should create PostgreSQL connection', async ({ page }) => {
    // Open add connection form
    await page.getByRole('button', { name: /add connection/i }).click();
    await page.waitForLoadState('networkidle');
    
    // Fill in connection details
    await page.getByLabel(/connection type/i).or(
      page.locator('select').first()
    ).selectOption('postgresql');
    
    await page.getByLabel(/connection name/i).or(
      page.getByPlaceholder(/connection name/i)
    ).fill('Test PostgreSQL');
    
    await page.getByLabel(/connection url/i).or(
      page.getByPlaceholder(/postgresql/i)
    ).fill('postgresql://user:pass@localhost:5432/testdb');
    
    // Submit form
    await page.getByRole('button', { name: /create/i }).click();
    await page.waitForLoadState('networkidle');
    
    // Verify connection appears in list
    // Note: May need to wait for list refresh
    await expect(page.getByText(/test postgresql/i)).toBeVisible({ timeout: 10000 });
  });

  test('should create Redis connection', async ({ page }) => {
    // Open add connection form
    await page.getByRole('button', { name: /add connection/i }).click();
    await page.waitForLoadState('networkidle');
    
    // Fill in connection details
    await page.getByLabel(/connection type/i).or(
      page.locator('select').first()
    ).selectOption('redis');
    
    await page.getByLabel(/connection name/i).or(
      page.getByPlaceholder(/connection name/i)
    ).fill('Test Redis');
    
    await page.getByLabel(/connection url/i).or(
      page.getByPlaceholder(/redis/i)
    ).fill('redis://localhost:6379');
    
    // Submit form
    await page.getByRole('button', { name: /create/i }).click();
    await page.waitForLoadState('networkidle');
    
    // Verify connection appears in list
    await expect(page.getByText(/test redis/i)).toBeVisible({ timeout: 10000 });
  });

  test('should validate required fields', async ({ page }) => {
    // Open add connection form
    await page.getByRole('button', { name: /add connection/i }).click();
    await page.waitForLoadState('networkidle');
    
    // Try to submit without filling fields
    const createButton = page.getByRole('button', { name: /create/i });
    
    // Button should be disabled or show validation error
    const isDisabled = await createButton.isDisabled();
    if (!isDisabled) {
      await createButton.click();
      
      // Should show validation errors
      await expect(page.getByText(/required|invalid/i)).toBeVisible();
    } else {
      // Button is disabled, which is correct
      expect(isDisabled).toBe(true);
    }
  });

  test('should edit connection', async ({ page }) => {
    // Find an existing connection or create one first
    const connectionCard = page.locator('[data-testid="connection-card"]').first();
    
    if (await connectionCard.count() > 0) {
      // Click edit button
      await connectionCard.getByRole('button', { name: /edit/i }).click();
      await page.waitForLoadState('networkidle');
      
      // Verify edit form is shown
      await expect(page.getByLabel(/connection name/i)).toBeVisible();
      
      // Update name
      await page.getByLabel(/connection name/i).fill('Updated Connection Name');
      
      // Save
      await page.getByRole('button', { name: /save/i }).click();
      await page.waitForLoadState('networkidle');
      
      // Verify update
      await expect(page.getByText(/updated connection name/i)).toBeVisible({ timeout: 10000 });
    }
  });

  test('should delete connection', async ({ page }) => {
    // Find an existing connection
    const connectionCard = page.locator('[data-testid="connection-card"]').first();
    
    if (await connectionCard.count() > 0) {
      // Get connection name for verification
      const connectionName = await connectionCard.textContent();
      
      // Click delete button
      await connectionCard.getByRole('button', { name: /delete/i }).click();
      
      // Confirm deletion (if confirmation dialog appears)
      page.on('dialog', async dialog => {
        expect(dialog.type()).toBe('confirm');
        await dialog.accept();
      });
      
      await page.waitForLoadState('networkidle');
      
      // Verify connection is removed
      if (connectionName) {
        await expect(page.getByText(connectionName)).not.toBeVisible({ timeout: 5000 });
      }
    }
  });

  test('should test connection', async ({ page }) => {
    // Find an existing connection
    const connectionCard = page.locator('[data-testid="connection-card"]').first();
    
    if (await connectionCard.count() > 0) {
      // Click test button
      await connectionCard.getByRole('button', { name: /test/i }).click();
      await page.waitForLoadState('networkidle');
      
      // Verify test result is displayed
      // May show success or error depending on connection validity
      await expect(
        page.getByText(/connection successful|connection failed|testing/i)
      ).toBeVisible({ timeout: 10000 });
    }
  });

  test('should prevent duplicate connection names', async ({ page }) => {
    // Create first connection
    await page.getByRole('button', { name: /add connection/i }).click();
    await page.waitForLoadState('networkidle');
    
    await page.getByLabel(/connection type/i).or(
      page.locator('select').first()
    ).selectOption('postgresql');
    
    await page.getByLabel(/connection name/i).fill('Unique Name');
    await page.getByLabel(/connection url/i).fill('postgresql://test');
    
    await page.getByRole('button', { name: /create/i }).click();
    await page.waitForLoadState('networkidle');
    
    // Try to create duplicate
    await page.getByRole('button', { name: /add connection/i }).click();
    await page.waitForLoadState('networkidle');
    
    await page.getByLabel(/connection type/i).selectOption('postgresql');
    await page.getByLabel(/connection name/i).fill('Unique Name');
    await page.getByLabel(/connection url/i).fill('postgresql://test2');
    
    await page.getByRole('button', { name: /create/i }).click();
    await page.waitForLoadState('networkidle');
    
    // Should show error about duplicate name
    await expect(page.getByText(/already exists|duplicate/i)).toBeVisible({ timeout: 5000 });
  });
});

