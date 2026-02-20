/**
 * Test Database Utilities
 *
 * Provides real Supabase client for integration tests.
 * Uses the test Docker Compose infrastructure (docker-compose.test.yml)
 *
 * IMPORTANT: This creates REAL database connections for testing.
 * We do NOT mock the database - per our testing policy.
 */

import { createClient, SupabaseClient } from '@supabase/supabase-js';
import type { Database } from '@/types/database';

// Test environment configuration
const TEST_SUPABASE_URL = process.env.TEST_SUPABASE_URL || 'http://localhost:54331';
const TEST_SUPABASE_ANON_KEY = process.env.TEST_SUPABASE_ANON_KEY || 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZS1kZW1vIiwicm9sZSI6ImFub24iLCJleHAiOjE5ODM4MTI5OTZ9.CRXP1A7WOeoJeXxjNni43kdQwgnWNReilDMblYTn_I0';
const TEST_SUPABASE_SERVICE_KEY = process.env.TEST_SUPABASE_SERVICE_KEY || 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZS1kZW1vIiwicm9sZSI6InNlcnZpY2Vfcm9sZSIsImV4cCI6MTk4MzgxMjk5Nn0.EGIM96RAZx35lJzdJsyH-qQwv8Hdp7fsn3W0YpN81IU';

let testClientInstance: SupabaseClient<Database> | null = null;
let serviceClientInstance: SupabaseClient<Database> | null = null;

// Default IDs from schema seed data
const DEFAULT_IDS = {
  userRole: {
    admin: '00000000-0000-0000-0000-000000000001',
    developer: '00000000-0000-0000-0000-000000000002',
    viewer: '00000000-0000-0000-0000-000000000003',
  },
  workflowStatus: {
    draft: '00000000-0000-0000-0000-000000000001',
    active: '00000000-0000-0000-0000-000000000002',
    archived: '00000000-0000-0000-0000-000000000003',
  },
  componentVisibility: {
    private: '00000000-0000-0000-0000-000000000001',
    team: '00000000-0000-0000-0000-000000000002',
    public: '00000000-0000-0000-0000-000000000003',
  },
};

/**
 * Interface for test fixtures that can be seeded into the database
 */
export interface TestFixtures {
  users?: Array<{
    id: string;
    auth_user_id: string;
    email: string;
    display_name?: string;
  }>;
  taskQueues?: Array<{
    id: string;
    name: string;
    display_name: string;
    created_by: string;
  }>;
  projects?: Array<{
    id: string;
    name: string;
    created_by: string; // FK to users.id
    task_queue_name: string;
    description?: string;
  }>;
  workflows?: Array<{
    id: string;
    project_id: string;
    created_by: string; // FK to users.id
    task_queue_id: string; // FK to task_queues.id
    name: string;
    display_name: string;
    kebab_name?: string;
    definition?: Record<string, unknown>;
    status_id?: string; // FK to workflow_statuses.id (defaults to draft)
    visibility_id?: string; // FK to component_visibility.id (defaults to private)
  }>;
  serviceInterfaces?: Array<{
    id: string;
    workflow_id: string;
    name: string;
    display_name: string;
    interface_type: string; // 'mcp', 'graphql', 'signal', 'query', 'update'
    mcp_config?: Record<string, unknown>;
    graphql_schema?: string;
    is_public?: boolean;
  }>;
  connectors?: Array<{
    id: string;
    project_id: string;
    created_by: string;
    name: string;
    display_name: string;
    connector_type: string;
    config_schema?: Record<string, unknown>;
    config_data?: Record<string, unknown>;
  }>;
}

/**
 * Create a test Supabase client using anon key
 * Use this for testing user-facing functionality
 */
export function createTestSupabaseClient(): SupabaseClient<Database> {
  if (testClientInstance) {
    return testClientInstance;
  }

  testClientInstance = createClient<Database>(
    TEST_SUPABASE_URL,
    TEST_SUPABASE_ANON_KEY,
    {
      auth: {
        persistSession: false,
        autoRefreshToken: false,
      },
    }
  );

  return testClientInstance;
}

/**
 * Create a test Supabase client using service role key
 * Use this for administrative operations (reset, seed, etc.)
 */
export function createTestServiceClient(): SupabaseClient<Database> {
  if (serviceClientInstance) {
    return serviceClientInstance;
  }

  serviceClientInstance = createClient<Database>(
    TEST_SUPABASE_URL,
    TEST_SUPABASE_SERVICE_KEY,
    {
      auth: {
        persistSession: false,
        autoRefreshToken: false,
      },
    }
  );

  return serviceClientInstance;
}

/**
 * Reset the test database by truncating all user tables
 * Call this before each test to ensure isolation
 */
export async function resetTestDatabase(): Promise<void> {
  const client = createTestServiceClient();

  // Tables in reverse dependency order to avoid FK constraints
  const tables = [
    'public_interfaces',
    'service_interface_endpoints',
    'service_interfaces',
    'workflow_execution_logs',
    'workflow_executions',
    'workflow_edges',
    'workflow_nodes',
    'workflow_compiled_code',
    'workflow_signals',
    'workflow_queries',
    'workflow_work_queues',
    'workflow_state_variables',
    'workflows',
    'connector_classifications',
    'project_connectors',
    'connectors',
    'project_state_variables',
    'workflow_workers',
    'projects',
    'task_queues',
    'users',
  ];

  for (const table of tables) {
    try {
      // Don't delete rows with IDs starting with 00000000 (seed data)
      await (client as any).from(table).delete().not('id', 'like', '00000000%');
    } catch {
      // Table might not exist or might be empty - that's OK
    }
  }
}

/**
 * Seed test data into the database
 * Data must be seeded in dependency order: users -> task_queues -> projects -> workflows -> service_interfaces
 */
export async function seedTestData(fixtures: TestFixtures): Promise<void> {
  const client = createTestServiceClient();

  // Seed users first (other tables depend on users.id)
  if (fixtures.users?.length) {
    const { error } = await (client as any).from('users').upsert(
      fixtures.users.map(u => ({
        id: u.id,
        auth_user_id: u.auth_user_id,
        email: u.email,
        display_name: u.display_name || 'Test User',
        role_id: DEFAULT_IDS.userRole.developer,
      })),
      { onConflict: 'id' }
    );
    if (error) throw new Error(`Failed to seed users: ${error.message}`);
  }

  // Seed task queues (projects and workflows depend on these)
  if (fixtures.taskQueues?.length) {
    const { error } = await (client as any).from('task_queues').upsert(
      fixtures.taskQueues.map(tq => ({
        id: tq.id,
        name: tq.name,
        display_name: tq.display_name,
        created_by: tq.created_by,
      })),
      { onConflict: 'id' }
    );
    if (error) throw new Error(`Failed to seed task queues: ${error.message}`);
  }

  // Seed projects (workflows depend on these)
  if (fixtures.projects?.length) {
    const { error } = await (client as any).from('projects').upsert(
      fixtures.projects.map(p => ({
        id: p.id,
        name: p.name,
        description: p.description || null,
        created_by: p.created_by,
        task_queue_name: p.task_queue_name,
      })),
      { onConflict: 'id' }
    );
    if (error) throw new Error(`Failed to seed projects: ${error.message}`);
  }

  // Seed workflows (service_interfaces depend on these)
  if (fixtures.workflows?.length) {
    const { error } = await (client as any).from('workflows').upsert(
      fixtures.workflows.map(w => ({
        id: w.id,
        project_id: w.project_id,
        created_by: w.created_by,
        task_queue_id: w.task_queue_id,
        name: w.name,
        display_name: w.display_name,
        kebab_name: w.kebab_name || w.name.toLowerCase().replace(/\s+/g, '-'),
        definition: w.definition || {},
        status_id: w.status_id || DEFAULT_IDS.workflowStatus.draft,
        visibility_id: w.visibility_id || DEFAULT_IDS.componentVisibility.private,
      })),
      { onConflict: 'id' }
    );
    if (error) throw new Error(`Failed to seed workflows: ${error.message}`);
  }

  // Seed service interfaces
  if (fixtures.serviceInterfaces?.length) {
    const { error } = await (client as any).from('service_interfaces').upsert(
      fixtures.serviceInterfaces.map(si => ({
        id: si.id,
        workflow_id: si.workflow_id,
        name: si.name,
        display_name: si.display_name,
        interface_type: si.interface_type,
        mcp_config: si.mcp_config || null,
        graphql_schema: si.graphql_schema || null,
        is_public: si.is_public || false,
      })),
      { onConflict: 'id' }
    );
    if (error) throw new Error(`Failed to seed service interfaces: ${error.message}`);
  }

  // Seed connectors
  if (fixtures.connectors?.length) {
    const { error } = await (client as any).from('connectors').upsert(
      fixtures.connectors.map(c => ({
        id: c.id,
        project_id: c.project_id,
        created_by: c.created_by,
        name: c.name,
        display_name: c.display_name,
        connector_type: c.connector_type,
        config_schema: c.config_schema || {},
        config_data: c.config_data || {},
      })),
      { onConflict: 'id' }
    );
    if (error) throw new Error(`Failed to seed connectors: ${error.message}`);
  }
}

/**
 * Create a test user and return authenticated client
 */
export async function createTestUserClient(
  email: string = 'test@example.com',
  password: string = 'testpassword123'
): Promise<{ client: SupabaseClient<Database>; userId: string }> {
  const client = createTestSupabaseClient();

  // Sign up the user (auto-confirmed in test environment)
  const { data: signUpData, error: signUpError } = await client.auth.signUp({
    email,
    password,
  });

  if (signUpError) {
    // User might already exist, try to sign in
    const { data: signInData, error: signInError } = await client.auth.signInWithPassword({
      email,
      password,
    });

    if (signInError) {
      throw new Error(`Failed to authenticate test user: ${signInError.message}`);
    }

    return {
      client,
      userId: signInData.user!.id,
    };
  }

  return {
    client,
    userId: signUpData.user!.id,
  };
}

// Track whether DB is available for the current test run
let dbAvailable: boolean | null = null;

/**
 * Verify database connectivity
 * Use this in test setup to ensure DB is ready
 */
export async function verifyDatabaseConnection(): Promise<boolean> {
  // Return cached result if we've already checked
  if (dbAvailable !== null) {
    return dbAvailable;
  }

  try {
    const client = createTestServiceClient();
    // Check if we can query the workflow_statuses table (always has seed data)
    const { error } = await (client as any).from('workflow_statuses').select('id').limit(1);

    if (error) {
      console.error('Database connection check failed:', error);
      dbAvailable = false;
      return false;
    }

    dbAvailable = true;
    return true;
  } catch (error) {
    console.error('Database connection failed:', error);
    dbAvailable = false;
    return false;
  }
}

/**
 * Check if database is available (synchronous, returns cached result)
 * Returns null if not yet checked
 */
export function isDatabaseAvailable(): boolean | null {
  return dbAvailable;
}

/**
 * Skip test message when DB isn't available
 */
export const DB_UNAVAILABLE_MESSAGE = 'Test skipped: Test database not available. Start docker-compose.test.yml to run integration tests.';

/**
 * Clean up client instances
 * Call this in afterAll
 */
export function cleanupTestClients(): void {
  testClientInstance = null;
  serviceClientInstance = null;
  dbAvailable = null;
}

/**
 * Export default IDs for use in tests
 */
export { DEFAULT_IDS };
