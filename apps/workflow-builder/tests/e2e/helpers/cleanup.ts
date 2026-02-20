/**
 * Test cleanup utilities for E2E tests
 * 
 * Ensures all test data is properly cleaned up after tests to prevent
 * database bloat and performance issues.
 */

import { createClient } from '@supabase/supabase-js';

const supabaseUrl = process.env.NEXT_PUBLIC_SUPABASE_URL || process.env.SUPABASE_URL || '';
const supabaseServiceKey = process.env.SUPABASE_SERVICE_ROLE_KEY || '';

if (!supabaseUrl || !supabaseServiceKey) {
  console.warn('⚠️  Supabase credentials not found. Cleanup will be skipped.');
}

// Initialize Supabase client with service role key for admin operations
const supabase = supabaseUrl && supabaseServiceKey
  ? createClient(supabaseUrl, supabaseServiceKey, {
      auth: {
        autoRefreshToken: false,
        persistSession: false,
      },
    })
  : null;

/**
 * Clean up all test data for a user
 */
export async function cleanupUserData(userEmail: string): Promise<void> {
  if (!supabase) {
    console.warn('⚠️  Supabase client not available. Skipping cleanup.');
    return;
  }

  try {
    // Get user record
    const { data: users, error: userError } = await supabase
      .from('users')
      .select('id')
      .eq('email', userEmail);

    if (userError) {
      console.warn(`⚠️  Error fetching user for cleanup: ${userError.message}`);
      return;
    }

    if (!users || users.length === 0) {
      return; // No user found, nothing to clean up
    }

    const userId = users[0].id;

    // Delete in order to respect foreign key constraints:
    // 1. Workflow executions (references workflows)
    const { error: execError } = await supabase
      .from('workflow_executions')
      .delete()
      .in('workflow_id', 
        supabase.from('workflows').select('id').eq('created_by', userId)
      );

    if (execError) {
      console.warn(`⚠️  Error cleaning up executions: ${execError.message}`);
    }

    // 2. Workflows (cascades to nodes, edges, connections)
    const { error: workflowError } = await supabase
      .from('workflows')
      .delete()
      .eq('created_by', userId);

    if (workflowError) {
      console.warn(`⚠️  Error cleaning up workflows: ${workflowError.message}`);
    }

    // 3. Projects (cascades to workers, compiled code, task queues)
    const { error: projectError } = await supabase
      .from('projects')
      .delete()
      .eq('created_by', userId);

    if (projectError) {
      console.warn(`⚠️  Error cleaning up projects: ${projectError.message}`);
    }

    // 4. Task queues (if not cascaded)
    const { error: queueError } = await supabase
      .from('task_queues')
      .delete()
      .eq('created_by', userId);

    if (queueError) {
      console.warn(`⚠️  Error cleaning up task queues: ${queueError.message}`);
    }

    console.log(`✅ Cleaned up test data for user: ${userEmail}`);
  } catch (error) {
    console.error(`❌ Error during cleanup: ${error}`);
    // Don't throw - cleanup failures shouldn't fail tests
  }
}

/**
 * Clean up a specific workflow by ID
 */
export async function cleanupWorkflow(workflowId: string): Promise<void> {
  if (!supabase) {
    return;
  }

  try {
    // Delete workflow (cascades to related data)
    const { error } = await supabase
      .from('workflows')
      .delete()
      .eq('id', workflowId);

    if (error) {
      console.warn(`⚠️  Error cleaning up workflow ${workflowId}: ${error.message}`);
    } else {
      console.log(`✅ Cleaned up workflow: ${workflowId}`);
    }
  } catch (error) {
    console.error(`❌ Error cleaning up workflow ${workflowId}: ${error}`);
  }
}

/**
 * Clean up test workflows by name pattern
 */
export async function cleanupTestWorkflows(pattern: string): Promise<void> {
  if (!supabase) {
    return;
  }

  try {
    // Get user ID for test user
    const { data: users } = await supabase
      .from('users')
      .select('id')
      .eq('email', 'test@example.com')
      .single();

    if (!users) {
      return;
    }

    // Delete workflows matching pattern
    const { error } = await supabase
      .from('workflows')
      .delete()
      .eq('created_by', users.id)
      .like('name', pattern);

    if (error) {
      console.warn(`⚠️  Error cleaning up workflows matching ${pattern}: ${error.message}`);
    } else {
      console.log(`✅ Cleaned up workflows matching: ${pattern}`);
    }
  } catch (error) {
    console.error(`❌ Error cleaning up test workflows: ${error}`);
  }
}

