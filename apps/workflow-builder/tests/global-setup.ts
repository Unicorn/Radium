/**
 * Vitest Global Setup
 *
 * This file runs ONCE before all test files.
 * By default, it starts the test Docker infrastructure.
 *
 * Usage:
 *   npm test                    - Start Docker, run tests (default)
 *   npm run test:quick          - Run tests without starting Docker (unit tests only)
 *   npm run test:db:start       - Just start Docker
 *   npm run test:db:stop        - Just stop Docker
 *
 * Environment variables:
 *   SKIP_TEST_INFRA=true        - Skip starting infrastructure (for quick unit tests)
 */

import { execSync } from 'child_process';
import * as path from 'path';

const PROJECT_ROOT = path.resolve(__dirname, '..');
const START_SCRIPT = path.join(PROJECT_ROOT, 'scripts/test-db-start.sh');
const STOP_SCRIPT = path.join(PROJECT_ROOT, 'scripts/test-db-stop.sh');

// Start infrastructure by default, skip only if explicitly disabled
const SKIP_INFRA = process.env.SKIP_TEST_INFRA === 'true';

export async function setup() {
  if (!SKIP_INFRA) {
    console.log('\n[Global Setup] Starting test infrastructure...\n');
    try {
      execSync(`bash "${START_SCRIPT}"`, { stdio: 'inherit' });
    } catch (error) {
      console.error('Failed to start test infrastructure:', error);
      throw error;
    }
  }
}

export async function teardown() {
  if (!SKIP_INFRA) {
    console.log('\n[Global Teardown] Stopping test infrastructure...\n');
    try {
      execSync(`bash "${STOP_SCRIPT}"`, { stdio: 'inherit' });
    } catch (error) {
      console.error('Failed to stop test infrastructure:', error);
      // Don't throw - we don't want to fail the test run just because cleanup failed
    }
  }
}
