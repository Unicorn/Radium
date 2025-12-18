/**
 * Authentication Helper Functions for E2E Tests
 * 
 * Provides reliable sign-in/sign-out utilities with proper wait strategies
 */

import { Page, expect } from '@playwright/test';

const BASE_URL = process.env.BASE_URL || 'http://localhost:3010';

export interface TestUser {
  email: string;
  password: string;
}

/**
 * Sign in a user and wait for successful authentication
 * 
 * This function uses a reliable wait strategy that explicitly waits
 * for navigation and then verifies dashboard elements.
 */
export async function signIn(page: Page, user: TestUser): Promise<void> {
  // Navigate to sign in page
  await page.goto(`${BASE_URL}/auth/signin`);
  
  // Fill in credentials
  await page.getByRole('textbox', { name: /you@example/i }).fill(user.email);
  await page.getByRole('textbox', { name: /••••••••/i }).fill(user.password);
  
  // Click sign in and wait for navigation
  await Promise.all([
    page.waitForURL(BASE_URL + '/', { timeout: 20000 }),
    page.getByRole('button', { name: 'Sign In' }).click(),
  ]);
  
  // Wait for page to fully load
  await page.waitForLoadState('networkidle', { timeout: 10000 });
  
  // Verify we successfully landed on the dashboard
  await expect(page.getByRole('button', { name: 'Sign Out' }))
    .toBeVisible({ timeout: 5000 });
}

/**
 * Sign out the current user and wait for redirect to sign-in page
 */
export async function signOut(page: Page): Promise<void> {
  // Click sign out button
  await page.getByRole('button', { name: 'Sign Out' }).click();
  
  // Wait for sign in form to appear (indicates successful sign out)
  await expect(page.getByRole('heading', { name: 'Sign In' }))
    .toBeVisible({ timeout: 15000 });
  
  // Verify we're on the sign in page
  await expect(page).toHaveURL(`${BASE_URL}/auth/signin`);
}

/**
 * Check if user is currently authenticated
 */
export async function isAuthenticated(page: Page): Promise<boolean> {
  try {
    await page.getByRole('button', { name: 'Sign Out' }).waitFor({ 
      state: 'visible', 
      timeout: 2000 
    });
    return true;
  } catch {
    return false;
  }
}

/**
 * Ensure user is signed in (sign in if not already authenticated)
 */
export async function ensureSignedIn(page: Page, user: TestUser): Promise<void> {
  const authenticated = await isAuthenticated(page);
  
  if (!authenticated) {
    await signIn(page, user);
  }
}

