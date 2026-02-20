/**
 * Global Setup for E2E Tests
 *
 * This script runs ONCE before all tests to:
 * 1. Start Docker containers (Supabase services)
 * 2. Wait for services to be healthy
 * 3. Run database migrations/seeds
 */

import { execSync } from 'child_process';
import { createClient } from '@supabase/supabase-js';
import * as path from 'path';
import * as fs from 'fs';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const PROJECT_DIR = path.resolve(__dirname, '../../..');
const DOCKER_COMPOSE_FILE = path.join(PROJECT_DIR, 'docker-compose.test.yml');

// Test environment configuration
const TEST_CONFIG = {
  supabaseUrl: 'http://localhost:54331',
  supabaseAnonKey: 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZS1kZW1vIiwicm9sZSI6ImFub24iLCJleHAiOjE5ODM4MTI5OTZ9.CRXP1A7WOeoJeXxjNni43kdQwgnWNReilDMblYTn_I0',
  supabaseServiceKey: 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZS1kZW1vIiwicm9sZSI6InNlcnZpY2Vfcm9sZSIsImV4cCI6MTk4MzgxMjk5Nn0.EGIM96RAZx35lJzdJsyH-qQwv8Hdp7fsn3W0YpN81IU',
  testUser: {
    email: 'test@example.com',
    password: 'testpassword123',
  },
  maxRetries: 60,
  retryInterval: 2000,
};

async function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function waitForService(
  name: string,
  checkFn: () => Promise<boolean>,
  maxRetries = TEST_CONFIG.maxRetries
): Promise<void> {
  console.log(`⏳ Waiting for ${name}...`);

  for (let i = 0; i < maxRetries; i++) {
    try {
      if (await checkFn()) {
        console.log(`✅ ${name} is ready!`);
        return;
      }
    } catch {
      // Ignore errors during health check
    }
    await sleep(TEST_CONFIG.retryInterval);
    if ((i + 1) % 10 === 0) {
      console.log(`   Still waiting for ${name}... (${i + 1}/${maxRetries})`);
    }
  }

  throw new Error(`❌ ${name} failed to become ready within timeout`);
}

async function startDockerServices(): Promise<void> {
  console.log('\n🐳 Starting Docker services...\n');

  // Check if docker-compose file exists
  if (!fs.existsSync(DOCKER_COMPOSE_FILE)) {
    throw new Error(`Docker Compose file not found: ${DOCKER_COMPOSE_FILE}`);
  }

  // Stop any existing containers first
  try {
    console.log('🧹 Cleaning up existing containers...');
    execSync(`docker compose -f "${DOCKER_COMPOSE_FILE}" down --volumes --remove-orphans`, {
      cwd: PROJECT_DIR,
      stdio: 'pipe',
    });
  } catch {
    // Ignore errors if no containers exist
  }

  // Wait a bit to ensure cleanup is complete
  await sleep(2000);

  // Start fresh containers with retry
  console.log('🚀 Starting fresh containers...');
  let retries = 3;
  while (retries > 0) {
    try {
      execSync(`docker compose -f "${DOCKER_COMPOSE_FILE}" up -d`, {
        cwd: PROJECT_DIR,
        stdio: 'inherit',
      });
      break;
    } catch (error) {
      retries--;
      if (retries === 0) {
        throw error;
      }
      console.log(`⚠️ Docker startup failed, retrying... (${retries} attempts left)`);
      await sleep(3000);
    }
  }
}

async function waitForSupabase(): Promise<void> {
  // Wait for Kong (API gateway)
  await waitForService('Kong API Gateway', async () => {
    try {
      const response = await fetch(`${TEST_CONFIG.supabaseUrl}/rest/v1/`, {
        headers: {
          apikey: TEST_CONFIG.supabaseAnonKey,
        },
      });
      return response.status === 200;
    } catch {
      return false;
    }
  });

  // Wait for GoTrue (Auth) - needs API key to go through Kong
  await waitForService('GoTrue Auth', async () => {
    try {
      const response = await fetch(`${TEST_CONFIG.supabaseUrl}/auth/v1/health`, {
        headers: {
          apikey: TEST_CONFIG.supabaseAnonKey,
        },
      });
      return response.ok;
    } catch {
      return false;
    }
  });

  // Wait for PostgREST
  await waitForService('PostgREST', async () => {
    try {
      const supabase = createClient(TEST_CONFIG.supabaseUrl, TEST_CONFIG.supabaseAnonKey);
      const { error } = await supabase.from('components').select('id').limit(1);
      // We expect an error if the table doesn't exist, but no connection error
      return !error || error.code !== 'PGRST301';
    } catch {
      return false;
    }
  });
}

async function setupTestUser(): Promise<void> {
  console.log('\n👤 Setting up test user...\n');

  // Wait for GoTrue to fully initialize its database schema
  // The health endpoint can return OK before the schema is ready
  console.log('⏳ Waiting for GoTrue database to initialize...');
  await sleep(5000);

  const supabase = createClient(TEST_CONFIG.supabaseUrl, TEST_CONFIG.supabaseAnonKey);

  // Retry logic for user creation - GoTrue may need more time after startup
  const maxRetries = 5;
  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      // Try to sign in first (user might already exist)
      const { data: signInData } = await supabase.auth.signInWithPassword({
        email: TEST_CONFIG.testUser.email,
        password: TEST_CONFIG.testUser.password,
      });

      if (signInData?.user) {
        console.log('✅ Test user already exists, signed in successfully');
        await setupTestUserData(signInData.user.id);
        return;
      }

      // Create new user
      console.log(`📝 Creating test user (attempt ${attempt}/${maxRetries})...`);
      const { data: signUpData, error: signUpError } = await supabase.auth.signUp({
        email: TEST_CONFIG.testUser.email,
        password: TEST_CONFIG.testUser.password,
      });

      if (signUpError) {
        // Check if it's a database error that might resolve with retry
        if (signUpError.message.includes('Database error') && attempt < maxRetries) {
          console.log(`⚠️ Database not ready, retrying in 3 seconds...`);
          await sleep(3000);
          continue;
        }
        throw new Error(`Failed to create test user: ${signUpError.message}`);
      }

      if (signUpData?.user) {
        console.log('✅ Test user created successfully');
        await setupTestUserData(signUpData.user.id);
        return;
      }
    } catch (error) {
      if (attempt < maxRetries) {
        console.log(`⚠️ Error creating user, retrying in 3 seconds...`);
        await sleep(3000);
        continue;
      }
      throw error;
    }
  }
}

async function setupTestUserData(authUserId: string): Promise<void> {
  console.log('📊 Setting up test user data...');

  const supabase = createClient(TEST_CONFIG.supabaseUrl, TEST_CONFIG.supabaseServiceKey);

  // Call the setup function we created in the seed data
  const { data, error } = await supabase.rpc('setup_test_user', {
    test_auth_user_id: authUserId,
    test_email: TEST_CONFIG.testUser.email,
  });

  if (error) {
    console.warn(`⚠️ Could not setup test data: ${error.message}`);
    // Don't fail - the tables might not exist yet, and RLS will prevent some operations
  } else {
    console.log('✅ Test user data created');
  }
}

async function globalSetup(): Promise<void> {
  console.log('\n' + '='.repeat(60));
  console.log('  🎭 Playwright Global Setup - Starting Test Infrastructure');
  console.log('='.repeat(60) + '\n');

  const startTime = Date.now();

  try {
    // Start Docker services
    await startDockerServices();

    // Wait for Supabase to be ready
    await waitForSupabase();

    // Setup test user and seed data
    await setupTestUser();

    const duration = ((Date.now() - startTime) / 1000).toFixed(1);
    console.log('\n' + '='.repeat(60));
    console.log(`  ✅ Test Infrastructure Ready (${duration}s)`);
    console.log('='.repeat(60) + '\n');

    // Save a marker file so teardown knows setup completed
    fs.writeFileSync(path.join(PROJECT_DIR, '.test-setup-complete'), Date.now().toString());
  } catch (error) {
    console.error('\n❌ Global setup failed:', error);

    // Try to capture logs for debugging
    try {
      console.log('\n📋 Docker container logs:');
      execSync(`docker compose -f "${DOCKER_COMPOSE_FILE}" logs --tail=50`, {
        cwd: PROJECT_DIR,
        stdio: 'inherit',
      });
    } catch {
      // Ignore
    }

    throw error;
  }
}

export default globalSetup;
