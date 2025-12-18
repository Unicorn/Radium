import { defineConfig } from 'vitest/config';
import path from 'path';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  test: {
    globals: true,
    environment: 'jsdom',
    globalSetup: ['./tests/global-setup.ts'],
    setupFiles: ['./tests/setup.ts'],
    include: ['**/__tests__/**/*.test.ts', '**/__tests__/**/*.test.tsx', 'tests/**/*.test.ts'],
    exclude: [
      'node_modules',
      '.next',
      'tests/e2e/**',
      // Exclude slow E2E tests that make API calls (run separately with specific env vars)
      '**/storage-e2e.test.ts',
      '**/component-builder/**/*integration.test.ts',
    ],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      exclude: [
        'node_modules/**',
        '.next/**',
        'tests/**',
        '**/*.d.ts',
        '**/*.config.*',
      ],
    },
    // Timeout for integration tests that hit real infrastructure
    testTimeout: 30000,
    hookTimeout: 30000,
    // Pool configuration for isolation
    pool: 'forks',
    poolOptions: {
      forks: {
        singleFork: true, // Run tests serially for DB isolation
      },
    },
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
      '@tests': path.resolve(__dirname, './tests'),
    },
  },
});
