import { test, expect } from '@playwright/test';
import { createClient } from '@supabase/supabase-js';

/**
 * E2E Tests for Workflow Creation
 * 
 * Test Case 1: Create workflow when NO projects exist (creates project + queue + workflow)
 * Test Case 2: Create workflow when projects exist (selects existing project, assigns its queue)
 */

const BASE_URL = 'http://localhost:3010';
const TEST_USER = {
  email: 'testuser@example.com',
  password: 'testpassword123',
};

// Initialize Supabase client for test setup/teardown
const supabaseUrl = process.env.NEXT_PUBLIC_SUPABASE_URL || 'http://127.0.0.1:54332';
const supabaseServiceKey = process.env.SUPABASE_SERVICE_ROLE_KEY || '';
const supabase = createClient(supabaseUrl, supabaseServiceKey);

// Helper function to sign in
async function signIn(page) {
  await page.goto(`${BASE_URL}/auth/signin`);
  await page.fill('input[placeholder="you@example.com"]', TEST_USER.email);
  await page.fill('input[type="password"]', TEST_USER.password);
  await page.click('button:has-text("Sign In")');
  await page.waitForURL(`${BASE_URL}/`);
}

// Helper function to clean up test data
async function cleanupUserData(userEmail: string) {
  // Get user record
  const { data: users } = await supabase
    .from('users')
    .select('id')
    .eq('email', userEmail);

  if (users && users.length > 0) {
    const userId = users[0].id;

    // Delete workflows (cascades to nodes, edges)
    await supabase.from('workflows').delete().eq('created_by', userId);

    // Delete projects (cascades to workers, compiled code)
    await supabase.from('projects').delete().eq('created_by', userId);

    // Delete task queues
    await supabase.from('task_queues').delete().eq('created_by', userId);
  }
}

test.describe('Workflow Creation - With Existing Project (Case 2)', () => {
  test.beforeEach(async ({ page }) => {
    // Ensure test data exists: one project with a task queue
    await cleanupUserData(TEST_USER.email);

    // Create a test project directly in the database
    const { data: users } = await supabase
      .from('users')
      .select('id')
      .eq('email', TEST_USER.email)
      .single();

    if (users) {
      const userId = users.id;
      const userIdPrefix = userId.split('-')[0];

      // Create project
      const { data: project } = await supabase
        .from('projects')
        .insert({
          name: 'E2E Test Project',
          description: 'Project for E2E testing',
          created_by: userId,
          task_queue_name: `${userIdPrefix}-e2e-test-project-queue`,
          is_active: true,
        })
        .select()
        .single();

      // Create task queue
      if (project) {
        await supabase.from('task_queues').insert({
          name: `${userIdPrefix}-e2e-test-project-queue`,
          display_name: 'E2E Test Project Task Queue',
          description: 'Default task queue for E2E Test Project',
          created_by: userId,
        });
      }
    }

    await signIn(page);
  });

  test.afterEach(async () => {
    await cleanupUserData(TEST_USER.email);
  });

  test('should create workflow with existing project', async ({ page }) => {
    // Navigate to new workflow page
    await page.goto(`${BASE_URL}/workflows/new`);
    await page.waitForLoadState('networkidle');

    // Fill in workflow details
    await page.fill('input[placeholder="Customer Onboarding"]', 'E2E Test Workflow');

    // Verify kebab name was auto-generated
    const kebabNameInput = page.locator('input[placeholder="customer-onboarding"]');
    await expect(kebabNameInput).toHaveValue('e2e-test-workflow');

    // Select project from dropdown
    // Note: Tamagui Select uses a button-like trigger
    await page.click('text=Select project');
    await page.waitForTimeout(500); // Wait for dropdown to open

    // Click on the E2E Test Project option
    await page.click('text=E2E Test Project');

    // Submit the form
    await page.click('button:has-text("Create & Edit")');

    // Wait for redirect to workflow builder
    await page.waitForURL(/\/workflows\/[a-f0-9-]+\/edit/);

    // Verify we're on the workflow builder page (check URL pattern)
    expect(page.url()).toMatch(/\/workflows\/[a-f0-9-]+\/edit/);

    // Verify workflow was created in database
    const { data: workflow } = await supabase
      .from('workflows')
      .select('*, task_queue:task_queues(name), project:projects(name)')
      .eq('kebab_name', 'e2e-test-workflow')
      .single();

    expect(workflow).not.toBeNull();
    expect(workflow?.display_name).toBe('E2E Test Workflow');
    expect(workflow?.project?.name).toBe('E2E Test Project');
    expect(workflow?.task_queue?.name).toContain('e2e-test-project-queue');
  });
});

test.describe('Workflow Creation - Without Existing Project (Case 1)', () => {
  test.beforeEach(async ({ page }) => {
    // Ensure NO projects exist
    await cleanupUserData(TEST_USER.email);
    await signIn(page);
  });

  test.afterEach(async () => {
    await cleanupUserData(TEST_USER.email);
  });

  test('should create workflow and project when no projects exist', async ({ page }) => {
    // Navigate to new workflow page
    await page.goto(`${BASE_URL}/workflows/new`);
    await page.waitForLoadState('networkidle');

    // Verify "create new project" input is shown (not dropdown)
    await expect(page.locator('input[placeholder="Enter project name"]')).toBeVisible();
    await expect(page.locator('text=Create your first project to organize workflows')).toBeVisible();

    // Fill in workflow details
    await page.fill('input[placeholder="Customer Onboarding"]', 'First Workflow');

    // Verify kebab name was auto-generated
    const kebabNameInput = page.locator('input[placeholder="customer-onboarding"]');
    await expect(kebabNameInput).toHaveValue('first-workflow');

    // Fill in project name
    await page.fill('input[placeholder="Enter project name"]', 'My First Project');

    // Submit the form
    await page.click('button:has-text("Create & Edit")');

    // Wait for redirect to workflow builder
    await page.waitForURL(/\/workflows\/[a-f0-9-]+\/edit/, { timeout: 10000 });

    // Verify we're on the workflow builder page (check URL pattern)
    expect(page.url()).toMatch(/\/workflows\/[a-f0-9-]+\/edit/);

    // Verify both project and workflow were created in database
    const { data: users } = await supabase
      .from('users')
      .select('id')
      .eq('email', TEST_USER.email)
      .single();

    expect(users).not.toBeNull();
    const userId = users!.id;

    // Check project was created
    const { data: project } = await supabase
      .from('projects')
      .select('*')
      .eq('name', 'My First Project')
      .eq('created_by', userId)
      .single();

    expect(project).not.toBeNull();
    expect(project?.name).toBe('My First Project');

    // Check task queue was created
    const { data: taskQueue } = await supabase
      .from('task_queues')
      .select('*')
      .eq('name', project!.task_queue_name)
      .single();

    expect(taskQueue).not.toBeNull();
    expect(taskQueue?.display_name).toBe('My First Project Task Queue');

    // Check workflow was created and linked to project and queue
    const { data: workflow } = await supabase
      .from('workflows')
      .select('*, task_queue:task_queues(name), project:projects(name)')
      .eq('kebab_name', 'first-workflow')
      .single();

    expect(workflow).not.toBeNull();
    expect(workflow?.display_name).toBe('First Workflow');
    expect(workflow?.project?.name).toBe('My First Project');
    expect(workflow?.task_queue?.name).toBe(project!.task_queue_name);
  });
});

