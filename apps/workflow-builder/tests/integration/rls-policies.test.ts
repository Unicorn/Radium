/**
 * RLS (Row Level Security) Policy Validation Tests
 *
 * These tests verify that database-level RLS policies correctly enforce:
 * - Users can only CRUD their own data
 * - Users cannot access other users' data
 * - Anonymous role can only read lookup/reference tables
 * - Service role can access all data regardless of ownership
 *
 * IMPORTANT: These tests require a running Supabase instance (docker-compose.test.yml).
 * They connect directly to the Supabase PostgREST API with different auth contexts.
 *
 * NOTE: The Rust API server uses service_role (which bypasses RLS), so these tests
 * validate the database-level security layer independently of the application layer.
 */

import { createClient, SupabaseClient } from '@supabase/supabase-js';
import { describe, it, expect, beforeAll, afterAll, beforeEach } from 'vitest';
import {
  createTestServiceClient,
  verifyDatabaseConnection,
  DB_UNAVAILABLE_MESSAGE,
  DEFAULT_IDS,
} from '@tests/utils/test-db';

// Test environment configuration
const SUPABASE_URL = process.env.TEST_SUPABASE_URL || 'http://localhost:54331';
const SUPABASE_ANON_KEY =
  process.env.TEST_SUPABASE_ANON_KEY ||
  'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZS1kZW1vIiwicm9sZSI6ImFub24iLCJleHAiOjE5ODM4MTI5OTZ9.CRXP1A7WOeoJeXxjNni43kdQwgnWNReilDMblYTn_I0';
const SUPABASE_SERVICE_KEY =
  process.env.TEST_SUPABASE_SERVICE_KEY ||
  'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZS1kZW1vIiwicm9sZSI6InNlcnZpY2Vfcm9sZSIsImV4cCI6MTk4MzgxMjk5Nn0.EGIM96RAZx35lJzdJsyH-qQwv8Hdp7fsn3W0YpN81IU';

// Stable test UUIDs to avoid conflicts
const USER_A_INTERNAL_ID = 'a0000000-0000-0000-0000-000000000001';
const USER_B_INTERNAL_ID = 'b0000000-0000-0000-0000-000000000002';
const TASK_QUEUE_A_ID = 'a0000000-0000-0000-0000-0000000000a1';
const TASK_QUEUE_B_ID = 'b0000000-0000-0000-0000-0000000000b1';
const PROJECT_A_ID = 'a0000000-0000-0000-0000-0000000000a2';
const PROJECT_B_ID = 'b0000000-0000-0000-0000-0000000000b2';
const WORKFLOW_A_ID = 'a0000000-0000-0000-0000-0000000000a3';
const WORKFLOW_B_ID = 'b0000000-0000-0000-0000-0000000000b3';

interface TestUser {
  authUserId: string;
  internalUserId: string;
  email: string;
  client: SupabaseClient;
}

let serviceClient: SupabaseClient;
let anonClient: SupabaseClient;
let userA: TestUser;
let userB: TestUser;
let dbAvailable = false;

/**
 * Create a GoTrue user and return an authenticated Supabase client.
 * Also creates the corresponding public.users record.
 */
async function createAuthenticatedUser(
  email: string,
  password: string,
  internalUserId: string
): Promise<TestUser> {
  // Create GoTrue auth user via admin API (service role)
  const adminClient = createClient(SUPABASE_URL, SUPABASE_SERVICE_KEY, {
    auth: { persistSession: false, autoRefreshToken: false },
  });

  // Try to create user, or sign in if already exists
  let authUserId: string;

  const { data: createData, error: createError } =
    await adminClient.auth.admin.createUser({
      email,
      password,
      email_confirm: true,
    });

  if (createError) {
    // User may already exist — try to find them
    const { data: listData } = await adminClient.auth.admin.listUsers();
    const existingUser = listData?.users?.find((u) => u.email === email);
    if (!existingUser) {
      throw new Error(`Failed to create auth user ${email}: ${createError.message}`);
    }
    authUserId = existingUser.id;
  } else {
    authUserId = createData.user.id;
  }

  // Create/update public.users record using service role
  const { error: upsertError } = await adminClient
    .from('users')
    .upsert(
      {
        id: internalUserId,
        auth_user_id: authUserId,
        email,
        display_name: `Test ${email.split('@')[0]}`,
        role_id: DEFAULT_IDS.userRole.developer,
      },
      { onConflict: 'id' }
    );

  if (upsertError) {
    throw new Error(`Failed to create public.users record: ${upsertError.message}`);
  }

  // Create an authenticated client for this user
  const userClient = createClient(SUPABASE_URL, SUPABASE_ANON_KEY, {
    auth: { persistSession: false, autoRefreshToken: false },
  });

  const { error: signInError } = await userClient.auth.signInWithPassword({
    email,
    password,
  });

  if (signInError) {
    throw new Error(`Failed to sign in as ${email}: ${signInError.message}`);
  }

  return {
    authUserId,
    internalUserId,
    email,
    client: userClient,
  };
}

/**
 * Seed test data for a user using service role (bypasses RLS).
 */
async function seedUserData(
  user: TestUser,
  taskQueueId: string,
  projectId: string,
  workflowId: string
): Promise<void> {
  // Task queue
  const { error: tqError } = await serviceClient
    .from('task_queues')
    .upsert(
      {
        id: taskQueueId,
        name: `test-queue-${user.email}`,
        display_name: `Test Queue (${user.email})`,
        created_by: user.internalUserId,
      },
      { onConflict: 'id' }
    );
  if (tqError) throw new Error(`Failed to seed task_queue: ${tqError.message}`);

  // Project
  const { error: projError } = await serviceClient
    .from('projects')
    .upsert(
      {
        id: projectId,
        name: `test-project-${user.email}`,
        description: `Project for ${user.email}`,
        created_by: user.internalUserId,
        task_queue_name: `test-queue-${user.email}`,
      },
      { onConflict: 'id' }
    );
  if (projError) throw new Error(`Failed to seed project: ${projError.message}`);

  // Workflow
  const { error: wfError } = await serviceClient
    .from('workflows')
    .upsert(
      {
        id: workflowId,
        name: `test-workflow-${user.email}`,
        display_name: `Test Workflow (${user.email})`,
        kebab_name: `test-workflow-${user.email.replace('@', '-at-')}`,
        definition: { components: [], connections: [] },
        created_by: user.internalUserId,
        project_id: projectId,
        task_queue_id: taskQueueId,
        status_id: DEFAULT_IDS.workflowStatus.draft,
        visibility_id: DEFAULT_IDS.componentVisibility.private,
      },
      { onConflict: 'id' }
    );
  if (wfError) throw new Error(`Failed to seed workflow: ${wfError.message}`);
}

/**
 * Clean up test data using service role.
 */
async function cleanupTestData(): Promise<void> {
  const testIds = [WORKFLOW_A_ID, WORKFLOW_B_ID];
  const projectIds = [PROJECT_A_ID, PROJECT_B_ID];
  const tqIds = [TASK_QUEUE_A_ID, TASK_QUEUE_B_ID];
  const userIds = [USER_A_INTERNAL_ID, USER_B_INTERNAL_ID];

  // Delete in reverse dependency order
  for (const id of testIds) {
    await serviceClient.from('workflow_nodes').delete().eq('workflow_id', id);
    await serviceClient.from('workflow_edges').delete().eq('workflow_id', id);
    await serviceClient.from('workflow_executions').delete().eq('workflow_id', id);
    await serviceClient.from('workflow_compiled_code').delete().eq('workflow_id', id);
    await serviceClient.from('workflow_state_variables').delete().eq('workflow_id', id);
    await serviceClient.from('workflow_signals').delete().eq('workflow_id', id);
    await serviceClient.from('workflow_queries').delete().eq('workflow_id', id);
    await serviceClient.from('workflow_work_queues').delete().eq('workflow_id', id);
    await serviceClient.from('service_interfaces').delete().eq('workflow_id', id);
    await serviceClient.from('workflows').delete().eq('id', id);
  }

  for (const id of projectIds) {
    await serviceClient.from('project_state_variables').delete().eq('project_id', id);
    await serviceClient.from('workflow_workers').delete().eq('project_id', id);
    await serviceClient.from('projects').delete().eq('id', id);
  }

  for (const id of tqIds) {
    await serviceClient.from('task_queues').delete().eq('id', id);
  }

  for (const id of userIds) {
    await serviceClient.from('api_keys').delete().eq('user_id', id);
    await serviceClient.from('users').delete().eq('id', id);
  }
}

// =============================================================================
// TEST SUITE
// =============================================================================

describe('RLS Policy Validation', () => {
  beforeAll(async () => {
    dbAvailable = await verifyDatabaseConnection();
    if (!dbAvailable) {
      console.warn(DB_UNAVAILABLE_MESSAGE);
      return;
    }

    serviceClient = createTestServiceClient();
    anonClient = createClient(SUPABASE_URL, SUPABASE_ANON_KEY, {
      auth: { persistSession: false, autoRefreshToken: false },
    });

    // Clean up any previous test data
    await cleanupTestData();

    // Create two test users with separate auth accounts
    userA = await createAuthenticatedUser(
      'rls-user-a@test.local',
      'testpasswordA123!',
      USER_A_INTERNAL_ID
    );
    userB = await createAuthenticatedUser(
      'rls-user-b@test.local',
      'testpasswordB123!',
      USER_B_INTERNAL_ID
    );

    // Seed data for both users
    await seedUserData(userA, TASK_QUEUE_A_ID, PROJECT_A_ID, WORKFLOW_A_ID);
    await seedUserData(userB, TASK_QUEUE_B_ID, PROJECT_B_ID, WORKFLOW_B_ID);
  });

  afterAll(async () => {
    if (dbAvailable) {
      await cleanupTestData();

      // Clean up GoTrue users
      const adminClient = createClient(SUPABASE_URL, SUPABASE_SERVICE_KEY, {
        auth: { persistSession: false, autoRefreshToken: false },
      });
      if (userA?.authUserId) {
        await adminClient.auth.admin.deleteUser(userA.authUserId);
      }
      if (userB?.authUserId) {
        await adminClient.auth.admin.deleteUser(userB.authUserId);
      }
    }
  });

  // ---------------------------------------------------------------------------
  // Workflows: Owner access
  // ---------------------------------------------------------------------------

  describe('Workflows - Owner Access', () => {
    it('User A can read their own workflows', async () => {
      if (!dbAvailable) return;

      const { data, error } = await userA.client
        .from('workflows')
        .select('id, name')
        .eq('id', WORKFLOW_A_ID);

      expect(error).toBeNull();
      expect(data).toHaveLength(1);
      expect(data![0].id).toBe(WORKFLOW_A_ID);
    });

    it('User A can update their own workflow', async () => {
      if (!dbAvailable) return;

      const { error } = await userA.client
        .from('workflows')
        .update({ description: 'Updated by User A' })
        .eq('id', WORKFLOW_A_ID);

      expect(error).toBeNull();

      // Verify update persisted
      const { data } = await userA.client
        .from('workflows')
        .select('description')
        .eq('id', WORKFLOW_A_ID)
        .single();

      expect(data?.description).toBe('Updated by User A');
    });
  });

  // ---------------------------------------------------------------------------
  // Workflows: Cross-user isolation
  // ---------------------------------------------------------------------------

  describe('Workflows - Cross-User Isolation', () => {
    it('User A cannot see User B workflows', async () => {
      if (!dbAvailable) return;

      const { data, error } = await userA.client
        .from('workflows')
        .select('id')
        .eq('id', WORKFLOW_B_ID);

      expect(error).toBeNull();
      expect(data).toHaveLength(0);
    });

    it('User B cannot see User A workflows', async () => {
      if (!dbAvailable) return;

      const { data, error } = await userB.client
        .from('workflows')
        .select('id')
        .eq('id', WORKFLOW_A_ID);

      expect(error).toBeNull();
      expect(data).toHaveLength(0);
    });

    it('User A cannot update User B workflows', async () => {
      if (!dbAvailable) return;

      const { data, error } = await userA.client
        .from('workflows')
        .update({ description: 'Hacked by A' })
        .eq('id', WORKFLOW_B_ID)
        .select();

      // RLS should silently return empty (no rows matched)
      expect(error).toBeNull();
      expect(data).toHaveLength(0);
    });

    it('User A cannot delete User B workflows', async () => {
      if (!dbAvailable) return;

      const { data, error } = await userA.client
        .from('workflows')
        .delete()
        .eq('id', WORKFLOW_B_ID)
        .select();

      expect(error).toBeNull();
      expect(data).toHaveLength(0);

      // Verify B's workflow still exists (via service role)
      const { data: check } = await serviceClient
        .from('workflows')
        .select('id')
        .eq('id', WORKFLOW_B_ID);

      expect(check).toHaveLength(1);
    });

    it('User A list only shows their own workflows', async () => {
      if (!dbAvailable) return;

      const { data, error } = await userA.client
        .from('workflows')
        .select('id');

      expect(error).toBeNull();
      const ids = data?.map((w) => w.id) || [];
      expect(ids).toContain(WORKFLOW_A_ID);
      expect(ids).not.toContain(WORKFLOW_B_ID);
    });
  });

  // ---------------------------------------------------------------------------
  // Projects: Cross-user isolation
  // ---------------------------------------------------------------------------

  describe('Projects - Cross-User Isolation', () => {
    it('User A can read their own project', async () => {
      if (!dbAvailable) return;

      const { data, error } = await userA.client
        .from('projects')
        .select('id, name')
        .eq('id', PROJECT_A_ID);

      expect(error).toBeNull();
      expect(data).toHaveLength(1);
    });

    it('User A cannot see User B projects', async () => {
      if (!dbAvailable) return;

      const { data, error } = await userA.client
        .from('projects')
        .select('id')
        .eq('id', PROJECT_B_ID);

      expect(error).toBeNull();
      expect(data).toHaveLength(0);
    });
  });

  // ---------------------------------------------------------------------------
  // Anonymous access restrictions
  // ---------------------------------------------------------------------------

  describe('Anonymous Access', () => {
    it('anon cannot read workflows', async () => {
      if (!dbAvailable) return;

      const { data, error } = await anonClient
        .from('workflows')
        .select('id');

      // RLS should return empty (no policies match anon)
      expect(error).toBeNull();
      expect(data).toHaveLength(0);
    });

    it('anon cannot read projects', async () => {
      if (!dbAvailable) return;

      const { data, error } = await anonClient
        .from('projects')
        .select('id');

      expect(error).toBeNull();
      expect(data).toHaveLength(0);
    });

    it('anon cannot read connectors', async () => {
      if (!dbAvailable) return;

      const { data, error } = await anonClient
        .from('connectors')
        .select('id');

      expect(error).toBeNull();
      expect(data).toHaveLength(0);
    });

    it('anon cannot read users', async () => {
      if (!dbAvailable) return;

      const { data, error } = await anonClient
        .from('users')
        .select('id');

      expect(error).toBeNull();
      expect(data).toHaveLength(0);
    });

    it('anon CAN read workflow_statuses (lookup table)', async () => {
      if (!dbAvailable) return;

      const { data, error } = await anonClient
        .from('workflow_statuses')
        .select('id, name');

      expect(error).toBeNull();
      expect(data!.length).toBeGreaterThan(0);
    });

    it('anon CAN read component_types (lookup table)', async () => {
      if (!dbAvailable) return;

      const { data, error } = await anonClient
        .from('component_types')
        .select('id, name');

      expect(error).toBeNull();
      expect(data!.length).toBeGreaterThan(0);
    });

    it('anon CAN read user_roles (lookup table)', async () => {
      if (!dbAvailable) return;

      const { data, error } = await anonClient
        .from('user_roles')
        .select('id, name');

      expect(error).toBeNull();
      expect(data!.length).toBeGreaterThan(0);
    });

    it('anon CAN read component_visibility (lookup table)', async () => {
      if (!dbAvailable) return;

      const { data, error } = await anonClient
        .from('component_visibility')
        .select('id, name');

      expect(error).toBeNull();
      expect(data!.length).toBeGreaterThan(0);
    });
  });

  // ---------------------------------------------------------------------------
  // Service role access
  // ---------------------------------------------------------------------------

  describe('Service Role Access', () => {
    it('service role can read all workflows', async () => {
      if (!dbAvailable) return;

      const { data, error } = await serviceClient
        .from('workflows')
        .select('id');

      expect(error).toBeNull();
      const ids = data?.map((w) => w.id) || [];
      expect(ids).toContain(WORKFLOW_A_ID);
      expect(ids).toContain(WORKFLOW_B_ID);
    });

    it('service role can read all projects', async () => {
      if (!dbAvailable) return;

      const { data, error } = await serviceClient
        .from('projects')
        .select('id');

      expect(error).toBeNull();
      const ids = data?.map((p) => p.id) || [];
      expect(ids).toContain(PROJECT_A_ID);
      expect(ids).toContain(PROJECT_B_ID);
    });

    it('service role can read all users', async () => {
      if (!dbAvailable) return;

      const { data, error } = await serviceClient
        .from('users')
        .select('id');

      expect(error).toBeNull();
      const ids = data?.map((u) => u.id) || [];
      expect(ids).toContain(USER_A_INTERNAL_ID);
      expect(ids).toContain(USER_B_INTERNAL_ID);
    });
  });

  // ---------------------------------------------------------------------------
  // Users table isolation
  // ---------------------------------------------------------------------------

  describe('Users Table Isolation', () => {
    it('User A can see their own profile', async () => {
      if (!dbAvailable) return;

      const { data, error } = await userA.client
        .from('users')
        .select('id, email')
        .eq('id', USER_A_INTERNAL_ID);

      expect(error).toBeNull();
      expect(data).toHaveLength(1);
      expect(data![0].email).toBe('rls-user-a@test.local');
    });

    it('User A cannot see User B profile', async () => {
      if (!dbAvailable) return;

      const { data, error } = await userA.client
        .from('users')
        .select('id')
        .eq('id', USER_B_INTERNAL_ID);

      expect(error).toBeNull();
      expect(data).toHaveLength(0);
    });
  });

  // ---------------------------------------------------------------------------
  // Workflow child table isolation (nodes, edges)
  // ---------------------------------------------------------------------------

  describe('Workflow Child Tables', () => {
    const NODE_A_ID = 'a0000000-0000-0000-0000-000000000a10';
    const NODE_B_ID = 'b0000000-0000-0000-0000-000000000b10';

    beforeAll(async () => {
      if (!dbAvailable) return;

      // Seed workflow nodes for both users
      await serviceClient.from('workflow_nodes').upsert([
        {
          id: NODE_A_ID,
          workflow_id: WORKFLOW_A_ID,
          node_id: 'start-a',
          node_type: 'trigger',
          position: { x: 0, y: 0 },
        },
        {
          id: NODE_B_ID,
          workflow_id: WORKFLOW_B_ID,
          node_id: 'start-b',
          node_type: 'trigger',
          position: { x: 0, y: 0 },
        },
      ]);
    });

    afterAll(async () => {
      if (!dbAvailable) return;
      await serviceClient.from('workflow_nodes').delete().in('id', [NODE_A_ID, NODE_B_ID]);
    });

    it('User A can see nodes in their own workflow', async () => {
      if (!dbAvailable) return;

      const { data, error } = await userA.client
        .from('workflow_nodes')
        .select('id')
        .eq('workflow_id', WORKFLOW_A_ID);

      expect(error).toBeNull();
      expect(data).toHaveLength(1);
      expect(data![0].id).toBe(NODE_A_ID);
    });

    it('User A cannot see nodes in User B workflow', async () => {
      if (!dbAvailable) return;

      const { data, error } = await userA.client
        .from('workflow_nodes')
        .select('id')
        .eq('workflow_id', WORKFLOW_B_ID);

      expect(error).toBeNull();
      expect(data).toHaveLength(0);
    });

    it('anon cannot see any workflow nodes', async () => {
      if (!dbAvailable) return;

      const { data, error } = await anonClient
        .from('workflow_nodes')
        .select('id');

      expect(error).toBeNull();
      expect(data).toHaveLength(0);
    });
  });

  // ---------------------------------------------------------------------------
  // API Keys RLS (validates the auth.uid() → users.id fix)
  // ---------------------------------------------------------------------------

  describe('API Keys RLS', () => {
    const API_KEY_A_ID = 'a0000000-0000-0000-0000-000000000a20';
    const API_KEY_B_ID = 'b0000000-0000-0000-0000-000000000b20';

    beforeAll(async () => {
      if (!dbAvailable) return;

      // Seed API keys for both users using service role
      await serviceClient.from('api_keys').upsert([
        {
          id: API_KEY_A_ID,
          user_id: USER_A_INTERNAL_ID,
          name: 'test-key-a',
          key_hash: 'hash-a-test-rls',
          key_prefix: 'rk_a_',
          is_active: true,
        },
        {
          id: API_KEY_B_ID,
          user_id: USER_B_INTERNAL_ID,
          name: 'test-key-b',
          key_hash: 'hash-b-test-rls',
          key_prefix: 'rk_b_',
          is_active: true,
        },
      ]);
    });

    afterAll(async () => {
      if (!dbAvailable) return;
      await serviceClient
        .from('api_keys')
        .delete()
        .in('id', [API_KEY_A_ID, API_KEY_B_ID]);
    });

    it('User A can see their own API keys', async () => {
      if (!dbAvailable) return;

      const { data, error } = await userA.client
        .from('api_keys')
        .select('id, name')
        .eq('user_id', USER_A_INTERNAL_ID);

      expect(error).toBeNull();
      expect(data!.length).toBeGreaterThanOrEqual(1);
      const ids = data!.map((k) => k.id);
      expect(ids).toContain(API_KEY_A_ID);
    });

    it('User A cannot see User B API keys', async () => {
      if (!dbAvailable) return;

      const { data, error } = await userA.client
        .from('api_keys')
        .select('id')
        .eq('id', API_KEY_B_ID);

      expect(error).toBeNull();
      expect(data).toHaveLength(0);
    });

    it('anon cannot read API keys', async () => {
      if (!dbAvailable) return;

      const { data, error } = await anonClient
        .from('api_keys')
        .select('id');

      // Should either be empty or error (no grant + RLS)
      if (error) {
        // Permission denied is acceptable
        expect(error.code).toBeTruthy();
      } else {
        expect(data).toHaveLength(0);
      }
    });
  });

  // ---------------------------------------------------------------------------
  // Task Queues isolation
  // ---------------------------------------------------------------------------

  describe('Task Queues Isolation', () => {
    it('User A can see their own task queues', async () => {
      if (!dbAvailable) return;

      const { data, error } = await userA.client
        .from('task_queues')
        .select('id')
        .eq('id', TASK_QUEUE_A_ID);

      expect(error).toBeNull();
      expect(data).toHaveLength(1);
    });

    it('User A cannot see User B task queues', async () => {
      if (!dbAvailable) return;

      const { data, error } = await userA.client
        .from('task_queues')
        .select('id')
        .eq('id', TASK_QUEUE_B_ID);

      expect(error).toBeNull();
      expect(data).toHaveLength(0);
    });
  });

  // ---------------------------------------------------------------------------
  // current_user_id() function validation
  // ---------------------------------------------------------------------------

  describe('current_user_id() helper', () => {
    it('returns the correct internal user ID for authenticated user', async () => {
      if (!dbAvailable) return;

      const { data, error } = await userA.client.rpc('current_user_id');

      expect(error).toBeNull();
      expect(data).toBe(USER_A_INTERNAL_ID);
    });

    it('returns different ID for different user', async () => {
      if (!dbAvailable) return;

      const { data, error } = await userB.client.rpc('current_user_id');

      expect(error).toBeNull();
      expect(data).toBe(USER_B_INTERNAL_ID);
    });

    it('returns null for anon', async () => {
      if (!dbAvailable) return;

      const { data, error } = await anonClient.rpc('current_user_id');

      // Should return null since anon has no auth.uid()
      if (error) {
        // Function might error for anon — that's acceptable
        expect(error).toBeTruthy();
      } else {
        expect(data).toBeNull();
      }
    });
  });
});
