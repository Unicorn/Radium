/**
 * MCP Resource Handler Tests
 *
 * TESTING POLICY COMPLIANCE:
 * - Database: Uses REAL test database via test utilities (NOT mocked)
 * - Internal handlers: Uses REAL implementations (NOT mocked)
 * - Temporal: Uses test utilities (mocked only when test infra unavailable)
 *
 * These tests verify resource listing and reading against real database data.
 * Tests are SKIPPED when the test database is not available.
 */

import { describe, it, expect, beforeEach, afterEach, beforeAll, afterAll } from 'vitest';
import type { SupabaseClient } from '@supabase/supabase-js';
import type { Database } from '@/types/database';
import { handleMCPResource, listMCPResources } from '../resource-handler';
import {
  createTestSupabaseClient,
  resetTestDatabase,
  seedTestData,
  verifyDatabaseConnection,
  cleanupTestClients,
  DB_UNAVAILABLE_MESSAGE,
} from '@tests/utils';

// Test data constants
const TEST_USER_ID = '00000000-0000-0000-0000-000000000001';
const TEST_PROJECT_ID = '00000000-0000-0000-0000-000000000002';
const TEST_WORKFLOW_ID = '00000000-0000-0000-0000-000000000003';
const TEST_SERVICE_INTERFACE_ID = '00000000-0000-0000-0000-000000000004';

// Check database availability before running tests
let dbAvailable = false;
let testSupabase: SupabaseClient<Database>;

/**
 * Helper to skip test if DB is not available
 */
function requireDb(testFn: () => Promise<void>) {
  return async () => {
    if (!dbAvailable) {
      console.log(DB_UNAVAILABLE_MESSAGE);
      return; // Skip the test
    }
    await testFn();
  };
}

beforeAll(async () => {
  dbAvailable = await verifyDatabaseConnection();
  if (!dbAvailable) {
    console.warn(DB_UNAVAILABLE_MESSAGE);
  } else {
    testSupabase = createTestSupabaseClient();
  }
});

afterAll(async () => {
  cleanupTestClients();
});

describe('MCP Resource Handler', () => {
  beforeEach(async () => {
    if (!dbAvailable) return;

    await resetTestDatabase();

    // Seed test data
    await seedTestData({
      users: [
        {
          id: TEST_USER_ID,
          auth_user_id: TEST_USER_ID,
          email: 'test@example.com',
          display_name: 'Test User',
        },
      ],
      taskQueues: [
        {
          id: '00000000-0000-0000-0000-000000000010',
          name: 'test-task-queue',
          display_name: 'Test Task Queue',
          created_by: TEST_USER_ID,
        },
      ],
      projects: [
        {
          id: TEST_PROJECT_ID,
          name: 'Test Project',
          created_by: TEST_USER_ID,
          task_queue_name: 'test-task-queue',
        },
      ],
      workflows: [
        {
          id: TEST_WORKFLOW_ID,
          project_id: TEST_PROJECT_ID,
          created_by: TEST_USER_ID,
          task_queue_id: '00000000-0000-0000-0000-000000000010',
          name: 'test-resource-workflow',
          display_name: 'Test Resource Workflow',
          kebab_name: 'test-resource-workflow',
          definition: {},
        },
      ],
      serviceInterfaces: [
        {
          id: TEST_SERVICE_INTERFACE_ID,
          workflow_id: TEST_WORKFLOW_ID,
          name: 'test-mcp-interface',
          display_name: 'Test MCP Interface',
          interface_type: 'mcp',
          mcp_config: {
            resources: [
              {
                uri: 'resource://test/resource1',
                name: 'Resource 1',
                description: 'Test resource 1',
                mimeType: 'text/plain',
              },
              {
                uri: 'resource://test/resource2',
                name: 'Resource 2',
                description: 'Test resource 2',
                mimeType: 'application/json',
              },
            ],
          },
        },
      ],
    });
  });

  afterEach(async () => {
    if (!dbAvailable) return;
    await resetTestDatabase();
  });

  describe('listMCPResources', () => {
    it('should return list of resources from real database', requireDb(async () => {
      const result = await listMCPResources(TEST_SERVICE_INTERFACE_ID, testSupabase);

      expect(result).toHaveLength(2);
      expect(result[0]).toEqual({
        uri: 'resource://test/resource1',
        name: 'Resource 1',
        description: 'Test resource 1',
        mimeType: 'text/plain',
      });
      expect(result[1]?.name).toBe('Resource 2');
    }));

    it('should return empty array when service interface has no resources config', requireDb(async () => {
      // Seed a service interface without resources
      const noResourcesInterfaceId = '00000000-0000-0000-0000-000000000099';
      await seedTestData({
        serviceInterfaces: [
          {
            id: noResourcesInterfaceId,
            workflow_id: TEST_WORKFLOW_ID,
            name: 'no-resources-interface',
            display_name: 'No Resources Interface',
            interface_type: 'mcp',
            mcp_config: {},
          },
        ],
      });

      const result = await listMCPResources(noResourcesInterfaceId, testSupabase);

      expect(result).toEqual([]);
    }));

    it('should return empty array when service interface does not exist', requireDb(async () => {
      const result = await listMCPResources('non-existent-id', testSupabase);

      expect(result).toEqual([]);
    }));
  });

  describe('handleMCPResource', () => {
    it('should throw error when resource does not exist', requireDb(async () => {
      await expect(
        handleMCPResource(
          { uri: 'resource://test/nonexistent' },
          TEST_SERVICE_INTERFACE_ID,
          testSupabase
        )
      ).rejects.toThrow('Resource not found');
    }));

    it('should return empty content when resource has no workflow configured', requireDb(async () => {
      // Resource exists but has no workflow to execute
      const result = await handleMCPResource(
        { uri: 'resource://test/resource1' },
        TEST_SERVICE_INTERFACE_ID,
        testSupabase
      );

      expect(result).toEqual({
        contents: [
          {
            uri: 'resource://test/resource1',
            mimeType: 'text/plain',
            text: '',
          },
        ],
      });
    }));

    it('should throw error when service interface does not exist', requireDb(async () => {
      await expect(
        handleMCPResource(
          { uri: 'resource://test/resource1' },
          'non-existent-id',
          testSupabase
        )
      ).rejects.toThrow();
    }));
  });
});
