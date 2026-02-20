/**
 * Global Teardown for E2E Tests
 *
 * This script runs ONCE after all tests complete (pass or fail) to:
 * 1. Stop Docker containers
 * 2. Clean up test resources
 * 3. Remove temporary files
 */

import { execSync } from 'child_process';
import * as path from 'path';
import * as fs from 'fs';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const PROJECT_DIR = path.resolve(__dirname, '../../..');
const DOCKER_COMPOSE_FILE = path.join(PROJECT_DIR, 'docker-compose.test.yml');
const SETUP_MARKER_FILE = path.join(PROJECT_DIR, '.test-setup-complete');

async function stopDockerServices(): Promise<void> {
  console.log('\n🐳 Stopping Docker services...');

  try {
    // Check if docker-compose file exists
    if (!fs.existsSync(DOCKER_COMPOSE_FILE)) {
      console.log('⚠️ Docker Compose file not found, skipping container cleanup');
      return;
    }

    // Stop and remove containers, networks, and volumes
    execSync(`docker compose -f "${DOCKER_COMPOSE_FILE}" down --volumes --remove-orphans`, {
      cwd: PROJECT_DIR,
      stdio: 'pipe',
    });

    console.log('✅ Docker services stopped and cleaned up');
  } catch (error) {
    console.warn('⚠️ Error stopping Docker services:', error);
    // Don't throw - we want teardown to complete even if Docker cleanup fails
  }
}

async function cleanupTempFiles(): Promise<void> {
  console.log('🧹 Cleaning up temporary files...');

  try {
    // Remove setup marker file
    if (fs.existsSync(SETUP_MARKER_FILE)) {
      fs.unlinkSync(SETUP_MARKER_FILE);
    }

    // Remove any .env.test.local generated during tests
    const envTestFile = path.join(PROJECT_DIR, '.env.test.local');
    if (fs.existsSync(envTestFile)) {
      fs.unlinkSync(envTestFile);
    }

    console.log('✅ Temporary files cleaned up');
  } catch (error) {
    console.warn('⚠️ Error cleaning up temp files:', error);
  }
}

async function globalTeardown(): Promise<void> {
  console.log('\n' + '='.repeat(60));
  console.log('  🎭 Playwright Global Teardown - Cleaning Up');
  console.log('='.repeat(60) + '\n');

  const startTime = Date.now();

  // Check if setup actually ran (marker file exists)
  const setupRan = fs.existsSync(SETUP_MARKER_FILE);

  if (setupRan) {
    // Stop Docker services
    await stopDockerServices();
  } else {
    console.log('ℹ️ Setup marker not found - Docker services may not have been started');
  }

  // Clean up temp files
  await cleanupTempFiles();

  const duration = ((Date.now() - startTime) / 1000).toFixed(1);
  console.log('\n' + '='.repeat(60));
  console.log(`  ✅ Teardown Complete (${duration}s)`);
  console.log('='.repeat(60) + '\n');
}

export default globalTeardown;
