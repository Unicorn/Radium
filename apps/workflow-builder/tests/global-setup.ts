/**
 * Vitest Global Setup
 *
 * This file runs ONCE before all test files.
 * It can optionally start the test Docker infrastructure.
 *
 * Usage:
 *   npm test                    - Run tests (assumes Docker already running)
 *   npm run test:with-db        - Start Docker, run tests, stop Docker
 *   npm run test:db:start       - Just start Docker
 *   npm run test:db:stop        - Just stop Docker
 */

import { execSync } from 'child_process';
import * as path from 'path';

const PROJECT_ROOT = path.resolve(__dirname, '..');
const START_SCRIPT = path.join(PROJECT_ROOT, 'scripts/test-db-start.sh');
const STOP_SCRIPT = path.join(PROJECT_ROOT, 'scripts/test-db-stop.sh');

// Check if we should auto-start Docker
const AUTO_START_DB = process.env.AUTO_START_TEST_DB === 'true';

export async function setup() {
  if (AUTO_START_DB) {
    console.log('\n[Global Setup] Starting test database infrastructure...\n');
    try {
      execSync(`bash "${START_SCRIPT}"`, { stdio: 'inherit' });
    } catch (error) {
      console.error('Failed to start test infrastructure:', error);
      throw error;
    }
  }
}

export async function teardown() {
  if (AUTO_START_DB) {
    console.log('\n[Global Teardown] Stopping test database infrastructure...\n');
    try {
      execSync(`bash "${STOP_SCRIPT}"`, { stdio: 'inherit' });
    } catch (error) {
      console.error('Failed to stop test infrastructure:', error);
      // Don't throw - we don't want to fail the test run just because cleanup failed
    }
  }
}
