/**
 * Test Utilities Index
 *
 * Export all test utilities for easy importing in test files.
 */

// Database utilities
export {
  createTestSupabaseClient,
  createTestServiceClient,
  createTestUserClient,
  resetTestDatabase,
  seedTestData,
  verifyDatabaseConnection,
  isDatabaseAvailable,
  DB_UNAVAILABLE_MESSAGE,
  cleanupTestClients,
  type TestFixtures,
} from './test-db';

// Temporal utilities - DISABLED until @temporalio/testing is installed
// To enable: npm install -D @temporalio/testing
// export {
//   getTestTemporalClient,
//   getTestTemporalConnection,
//   setupInMemoryTemporalEnv,
//   getInMemoryTemporalClient,
//   verifyTemporalConnection,
//   waitForWorkflowState,
//   cancelAllTestWorkflows,
//   cleanupTestTemporal,
//   createTestWorkflowId,
// } from './test-temporal';
