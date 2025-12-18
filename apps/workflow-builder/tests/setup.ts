/**
 * Vitest Global Setup
 *
 * This file runs before all tests and sets up the test environment.
 *
 * For integration tests that need real infrastructure (DB, Temporal),
 * see tests/utils/test-db.ts and tests/utils/test-temporal.ts
 */

import * as dotenv from 'dotenv';
import * as path from 'path';

// Load .env.test and .env.test.local before anything else
// This must happen before other imports that might use env vars
dotenv.config({ path: path.resolve(__dirname, '..', '.env.test') });
dotenv.config({ path: path.resolve(__dirname, '..', '.env.test.local') });

import '@testing-library/jest-dom';
import { afterAll, beforeAll, beforeEach, vi } from 'vitest';

// Set test environment variables
process.env.NODE_ENV = 'test';
process.env.TEST_SUPABASE_URL = process.env.TEST_SUPABASE_URL || 'http://localhost:54331';
process.env.TEST_TEMPORAL_ADDRESS = process.env.TEST_TEMPORAL_ADDRESS || 'localhost:7233';

// Mock Next.js headers/cookies for server component tests
// These are needed because Next.js server utilities don't work outside Next.js context
vi.mock('next/headers', () => ({
  cookies: vi.fn(() => ({
    get: vi.fn(),
    set: vi.fn(),
    delete: vi.fn(),
  })),
  headers: vi.fn(() => new Map()),
}));

// Mock next/navigation for component tests
vi.mock('next/navigation', () => ({
  useRouter: vi.fn(() => ({
    push: vi.fn(),
    replace: vi.fn(),
    back: vi.fn(),
    forward: vi.fn(),
    refresh: vi.fn(),
    prefetch: vi.fn(),
  })),
  usePathname: vi.fn(() => '/'),
  useSearchParams: vi.fn(() => new URLSearchParams()),
  useParams: vi.fn(() => ({})),
  redirect: vi.fn(),
  notFound: vi.fn(),
}));

// Mock window.matchMedia for component tests that use responsive design
// Only set up if window exists (jsdom environment, not node)
if (typeof window !== 'undefined') {
  Object.defineProperty(window, 'matchMedia', {
    writable: true,
    value: vi.fn().mockImplementation((query: string) => ({
      matches: false,
      media: query,
      onchange: null,
      addListener: vi.fn(), // Deprecated
      removeListener: vi.fn(), // Deprecated
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      dispatchEvent: vi.fn(),
    })),
  });
}

/**
 * Global test setup
 */
beforeAll(async () => {
  // Any global setup needed
  console.log('Test suite starting...');
});

/**
 * Global test teardown
 */
afterAll(async () => {
  // Any global cleanup needed
  console.log('Test suite complete.');
});

/**
 * Clean console warnings about React act() etc.
 */
const originalError = console.error;
beforeAll(() => {
  console.error = (...args: unknown[]) => {
    // Filter out known React testing noise
    if (
      typeof args[0] === 'string' &&
      (args[0].includes('Warning: ReactDOM.render is no longer supported') ||
        args[0].includes('Warning: An update to') ||
        args[0].includes('act(...)'))
    ) {
      return;
    }
    originalError.call(console, ...args);
  };
});

afterAll(() => {
  console.error = originalError;
});
