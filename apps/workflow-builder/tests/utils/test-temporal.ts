/**
 * Test Temporal Utilities
 *
 * Provides real Temporal client for integration tests.
 * Uses the test Docker Compose infrastructure (docker-compose.test.yml)
 *
 * IMPORTANT: This creates REAL Temporal connections for testing.
 * We do NOT mock Temporal - per our testing policy.
 */

import { Connection, Client, WorkflowHandle } from '@temporalio/client';
import { TestWorkflowEnvironment } from '@temporalio/testing';

// Test environment configuration
const TEST_TEMPORAL_ADDRESS = process.env.TEST_TEMPORAL_ADDRESS || 'localhost:7233';
const TEST_TEMPORAL_NAMESPACE = process.env.TEST_TEMPORAL_NAMESPACE || 'default';

let testClientInstance: Client | null = null;
let testConnectionInstance: Connection | null = null;
let testWorkflowEnv: TestWorkflowEnvironment | null = null;

/**
 * Get or create Temporal client for tests
 * Uses the real Temporal server from docker-compose.test.yml
 */
export async function getTestTemporalClient(): Promise<Client> {
  if (testClientInstance) {
    return testClientInstance;
  }

  try {
    const connection = await getTestTemporalConnection();

    testClientInstance = new Client({
      connection,
      namespace: TEST_TEMPORAL_NAMESPACE,
    });

    console.log('Test Temporal client initialized');
    return testClientInstance;
  } catch (error) {
    console.error('Failed to create test Temporal client:', error);
    throw new Error(
      `Failed to connect to test Temporal: ${error instanceof Error ? error.message : 'Unknown error'}`
    );
  }
}

/**
 * Get or create Temporal connection for tests
 */
export async function getTestTemporalConnection(): Promise<Connection> {
  if (testConnectionInstance) {
    return testConnectionInstance;
  }

  try {
    testConnectionInstance = await Connection.connect({
      address: TEST_TEMPORAL_ADDRESS,
    });

    console.log(`Connected to test Temporal at ${TEST_TEMPORAL_ADDRESS}`);
    return testConnectionInstance;
  } catch (error) {
    console.error('Failed to connect to test Temporal:', error);
    throw error;
  }
}

/**
 * Setup Temporal test environment using Temporal's testing utilities
 * This creates an in-memory Temporal server for fast unit tests
 *
 * Use this for tests that don't need the full Docker infrastructure
 */
export async function setupInMemoryTemporalEnv(): Promise<TestWorkflowEnvironment> {
  if (testWorkflowEnv) {
    return testWorkflowEnv;
  }

  testWorkflowEnv = await TestWorkflowEnvironment.createLocal();
  return testWorkflowEnv;
}

/**
 * Get client from in-memory Temporal environment
 */
export function getInMemoryTemporalClient(): Client {
  if (!testWorkflowEnv) {
    throw new Error('In-memory Temporal environment not initialized. Call setupInMemoryTemporalEnv first.');
  }
  return testWorkflowEnv.client;
}

/**
 * Verify Temporal connectivity
 * Use this in test setup to ensure Temporal is ready
 */
export async function verifyTemporalConnection(): Promise<boolean> {
  try {
    const client = await getTestTemporalClient();

    // Try to list workflows as a health check
    const workflows = client.workflow.list();
    // Just iterate once to verify the connection works
    for await (const _ of workflows) {
      break;
    }

    return true;
  } catch (error) {
    console.error('Temporal connection check failed:', error);
    return false;
  }
}

/**
 * Wait for a workflow to reach a specific state
 */
export async function waitForWorkflowState(
  handle: WorkflowHandle,
  targetStatus: 'RUNNING' | 'COMPLETED' | 'FAILED' | 'CANCELLED',
  timeoutMs: number = 30000
): Promise<boolean> {
  const startTime = Date.now();

  while (Date.now() - startTime < timeoutMs) {
    const description = await handle.describe();
    const status = description.status.name;

    if (status === targetStatus) {
      return true;
    }

    // If workflow completed with different status, return false
    if (['COMPLETED', 'FAILED', 'CANCELLED', 'TERMINATED'].includes(status)) {
      return false;
    }

    // Wait before checking again
    await new Promise(resolve => setTimeout(resolve, 100));
  }

  return false;
}

/**
 * Cancel all running workflows in test namespace
 * Use this for cleanup between tests
 */
export async function cancelAllTestWorkflows(): Promise<void> {
  try {
    const client = await getTestTemporalClient();
    const workflows = client.workflow.list({
      query: 'ExecutionStatus = "Running"',
    });

    for await (const workflow of workflows) {
      try {
        const handle = client.workflow.getHandle(workflow.workflowId, workflow.runId);
        await handle.cancel();
      } catch (error) {
        // Workflow might have completed between list and cancel - that's OK
        console.debug(`Could not cancel workflow ${workflow.workflowId}:`, error);
      }
    }
  } catch (error) {
    console.error('Failed to cancel test workflows:', error);
  }
}

/**
 * Close Temporal connection and clean up
 * Call this in afterAll
 */
export async function cleanupTestTemporal(): Promise<void> {
  // Cancel any running workflows first
  try {
    await cancelAllTestWorkflows();
  } catch (error) {
    // Best effort cleanup
  }

  // Close in-memory environment if used
  if (testWorkflowEnv) {
    await testWorkflowEnv.teardown();
    testWorkflowEnv = null;
  }

  // Close real connection
  if (testClientInstance) {
    await testClientInstance.connection.close();
    testClientInstance = null;
    testConnectionInstance = null;
    console.log('Test Temporal connection closed');
  }
}

/**
 * Create a unique workflow ID for tests
 * Includes timestamp and random suffix to avoid collisions
 */
export function createTestWorkflowId(prefix: string = 'test'): string {
  const timestamp = Date.now();
  const random = Math.random().toString(36).substring(2, 8);
  return `${prefix}-${timestamp}-${random}`;
}
