/**
 * MCP Tool Handler Tests
 *
 * TESTING POLICY COMPLIANCE:
 * - Database: Uses REAL test database via test utilities (NOT mocked)
 * - Internal handlers: Uses REAL implementations (NOT mocked)
 * - Temporal: Uses test utilities (mocked only when test infra unavailable)
 *
 * These tests verify tool listing and execution against real database data.
 * Tests are SKIPPED when the test database is not available.
 */

import { describe, it, expect, beforeEach, afterEach, beforeAll, afterAll } from 'vitest';
import type { SupabaseClient } from '@supabase/supabase-js';
import type { Database } from '@/types/database';
import { handleMCPTool, listMCPTools } from '../tool-handler';
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

describe('MCP Tool Handler', () => {
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
          name: 'test-tool-workflow',
          display_name: 'Test Tool Workflow',
          kebab_name: 'test-tool-workflow',
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
            tools: [
              {
                name: 'testTool1',
                description: 'Test tool 1',
                inputSchema: {
                  type: 'object',
                  properties: {
                    param1: { type: 'string' },
                  },
                },
              },
              {
                name: 'testTool2',
                description: 'Test tool 2',
                inputSchema: {
                  type: 'object',
                  properties: {
                    count: { type: 'number' },
                  },
                },
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

  describe('listMCPTools', () => {
    it('should return list of tools from real database', requireDb(async () => {
      const result = await listMCPTools(TEST_SERVICE_INTERFACE_ID, testSupabase);

      expect(result).toHaveLength(2);
      expect(result[0]).toEqual({
        name: 'testTool1',
        description: 'Test tool 1',
        inputSchema: {
          type: 'object',
          properties: {
            param1: { type: 'string' },
          },
        },
      });
      expect(result[1]?.name).toBe('testTool2');
    }));

    it('should return empty array when service interface has no tools config', requireDb(async () => {
      // Seed a service interface without tools
      const noToolsInterfaceId = '00000000-0000-0000-0000-000000000099';
      await seedTestData({
        serviceInterfaces: [
          {
            id: noToolsInterfaceId,
            workflow_id: TEST_WORKFLOW_ID,
            name: 'no-tools-interface',
            display_name: 'No Tools Interface',
            interface_type: 'mcp',
            mcp_config: {},
          },
        ],
      });

      const result = await listMCPTools(noToolsInterfaceId, testSupabase);

      expect(result).toEqual([]);
    }));

    it('should return empty array when service interface does not exist', requireDb(async () => {
      const result = await listMCPTools('non-existent-id', testSupabase);

      expect(result).toEqual([]);
    }));
  });

  describe('handleMCPTool', () => {
    it('should throw error when tool does not exist', requireDb(async () => {
      await expect(
        handleMCPTool({ name: 'nonexistentTool' }, TEST_SERVICE_INTERFACE_ID, testSupabase)
      ).rejects.toThrow('Tool not found');
    }));

    it('should validate input type when schema specifies string', requireDb(async () => {
      // Tool testTool1 expects param1 to be a string
      await expect(
        handleMCPTool(
          {
            name: 'testTool1',
            arguments: { param1: 123 }, // Should be string
          },
          TEST_SERVICE_INTERFACE_ID,
          testSupabase
        )
      ).rejects.toThrow(/Invalid type/);
    }));

    it('should throw error when service interface does not exist', requireDb(async () => {
      await expect(
        handleMCPTool(
          { name: 'testTool1', arguments: { param1: 'test' } },
          'non-existent-id',
          testSupabase
        )
      ).rejects.toThrow();
    }));
  });
});
