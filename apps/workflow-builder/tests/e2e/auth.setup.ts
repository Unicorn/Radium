/**
 * Authentication Setup for E2E Tests
 * 
 * This setup script performs browser-based authentication following Playwright best practices:
 * 1. Ensures test user exists (via generate-test-auth.ts)
 * 2. Performs real browser login through the UI
 * 3. Saves storage state using Playwright's built-in mechanism
 * 4. All subsequent tests reuse this authenticated state
 * 
 * This approach ensures Supabase SSR cookies are set correctly by Supabase's own code,
 * avoiding manual cookie manipulation issues.
 */

import { test as setup, expect } from '@playwright/test';
import { execSync } from 'child_process';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const authFile = path.join(__dirname, '../../playwright/.auth/user.json');
const generateAuthScript = path.join(__dirname, '../../scripts/generate-test-auth.ts');

const TEST_USER = {
  email: 'test@example.com',
  password: 'testpassword123',
};

setup('authenticate', async ({ page }) => {
  setup.setTimeout(120000); // 2 minutes for setup
  
  console.log('🔐 Setting up authentication...');
  
  // Step 1: Ensure test user exists (this script creates the user if needed)
  console.log('📝 Ensuring test user exists...');
  try {
    execSync(`npx tsx ${generateAuthScript}`, {
      stdio: 'pipe', // Don't spam console with generate-test-auth output
      cwd: path.join(__dirname, '../..'),
      env: {
        ...process.env,
        NODE_ENV: 'test',
      },
    });
    console.log('✅ Test user ready');
  } catch (error: any) {
    // If user creation fails, continue - user might already exist
    console.log('⚠️  User creation script had issues (user may already exist):', error.message);
  }
  
  // Step 2: Navigate to sign-in page
  console.log('🌐 Navigating to sign-in page...');
  await page.goto('http://localhost:3010/auth/signin');
  
  // Step 3: Wait for sign-in form to be visible
  console.log('⏳ Waiting for sign-in form...');
  const emailInput = page.getByPlaceholder(/you@example/i).or(
    page.locator('input[type="email"]').first()
  );
  await expect(emailInput).toBeVisible({ timeout: 30000 });
  
  // Step 4: Fill in credentials
  console.log('✍️  Filling in credentials...');
  await emailInput.fill(TEST_USER.email);
  
  const passwordInput = page.getByPlaceholder(/password/i).or(
    page.locator('input[type="password"]').first()
  );
  await expect(passwordInput).toBeVisible({ timeout: 5000 });
  await passwordInput.fill(TEST_USER.password);
  
  // Step 5: Click sign-in button
  console.log('🖱️  Clicking sign-in button...');
  const signInButton = page.getByRole('button', { name: /sign in/i });
  await expect(signInButton).toBeVisible({ timeout: 5000 });
  await expect(signInButton).toBeEnabled({ timeout: 5000 });
  await signInButton.click();
  
  // Step 6: Wait for successful authentication
  // After sign-in, user should be redirected away from /auth/signin
  console.log('⏳ Waiting for authentication to complete...');
  await page.waitForURL(
    (url) => !url.pathname.includes('/auth/signin'),
    { timeout: 30000 }
  );
  
  // Step 7: Verify we're authenticated by checking for authenticated UI elements
  // Look for something that only appears when logged in (e.g., dashboard, workflows list, etc.)
  console.log('✅ Verifying authentication...');
  
  // Wait a moment for auth state to propagate
  await page.waitForTimeout(2000);
  
  // Try to find authenticated content - could be homepage, dashboard, or workflows list
  // If we're still on signin page, auth failed
  const currentUrl = page.url();
  if (currentUrl.includes('/auth/signin')) {
    // Check for error message
    const errorElement = page.locator('text=/error|failed|invalid/i').first();
    if (await errorElement.isVisible({ timeout: 2000 }).catch(() => false)) {
      const errorText = await errorElement.textContent();
      throw new Error(`Authentication failed: ${errorText}`);
    }
    throw new Error('Authentication failed - still on sign-in page after login attempt');
  }
  
  console.log('✅ Authentication successful!');
  console.log('   Redirected to:', currentUrl);
  
  // Step 8: Save authentication state using Playwright's built-in mechanism
  // This captures cookies, localStorage, sessionStorage, and optionally IndexedDB
  console.log('💾 Saving authentication state...');
  await page.context().storageState({ 
    path: authFile,
    // Include IndexedDB if Supabase uses it (it might for some auth features)
    // indexedDB: true 
  });
  
  console.log('✅ Authentication state saved to:', authFile);
  console.log('✨ Setup complete! All tests will use this authenticated session.');
});

