/**
 * System Workflows Registration
 * 
 * Ensures system user exists and registers system workflows in the database
 */

import { createClient } from '@supabase/supabase-js';
import type { Database } from '@/types/database';

/**
 * Get Supabase client with service role
 */
function getSupabaseClient() {
  return createClient<Database>(
    process.env.NEXT_PUBLIC_SUPABASE_URL!,
    process.env.SUPABASE_SERVICE_ROLE_KEY!
  );
}

/**
 * Ensure system user exists
 * Note: Auth user must be created via Supabase Auth API first
 */
export async function ensureSystemUser(): Promise<string | null> {
  const supabase = getSupabaseClient();
  
  // Check if system user exists
  const { data: existingUser } = await supabase
    .from('users')
    .select('id')
    .eq('email', 'system@example.com')
    .single();
  
  if (existingUser) {
    return existingUser.id;
  }
  
  // Try to find auth user
  const { data: authUsers } = await supabase.auth.admin.listUsers();
  const systemAuthUser = authUsers?.users?.find(u => u.email === 'system@example.com');
  
  if (!systemAuthUser) {
    console.warn('⚠️  System auth user not found. Create via Supabase Auth API first.');
    return null;
  }
  
  // Get system role
  const { data: systemRole } = await supabase
    .from('user_roles')
    .select('id')
    .eq('name', 'system')
    .single();
  
  if (!systemRole) {
    console.warn('⚠️  System role not found. Run migration 20251118000001 first.');
    return null;
  }
  
  // Create user record
  const { data: newUser, error } = await supabase
    .from('users')
    .insert({
      auth_user_id: systemAuthUser.id,
      email: 'system@example.com',
      display_name: 'System User',
      role_id: systemRole.id,
    })
    .select('id')
    .single();
  
  if (error) {
    console.error('❌ Failed to create system user:', error);
    return null;
  }
  
  console.log('✅ System user created:', newUser.id);
  return newUser.id;
}

/**
 * Ensure system project exists
 */
export async function ensureSystemProject(systemUserId: string): Promise<string | null> {
  const supabase = getSupabaseClient();
  
  // Check if project exists
  const { data: existingProject } = await supabase
    .from('projects')
    .select('id, task_queue_name')
    .eq('name', 'System Workflows')
    .eq('created_by', systemUserId)
    .single();
  
  if (existingProject) {
    return existingProject.id;
  }
  
  // Create project
  const { data: newProject, error } = await supabase
    .from('projects')
    .insert({
      name: 'System Workflows',
      description: 'System workflows for agent testing and other system operations',
      created_by: systemUserId,
      task_queue_name: 'system-workflows-queue',
      is_active: true,
    })
    .select('id')
    .single();
  
  if (error) {
    console.error('❌ Failed to create system project:', error);
    return null;
  }
  
  console.log('✅ System project created:', newProject.id);
  return newProject.id;
}

/**
 * Register agent tester workflow in database
 */
export async function registerAgentTesterWorkflow(
  systemUserId: string,
  projectId: string
): Promise<string | null> {
  const supabase = getSupabaseClient();
  
  // Get project to find task queue
  const { data: project } = await supabase
    .from('projects')
    .select('task_queue_name')
    .eq('id', projectId)
    .single();
  
  if (!project) {
    console.error('❌ System project not found');
    return null;
  }
  
  // Get task queue ID
  const { data: taskQueue } = await supabase
    .from('task_queues')
    .select('id')
    .eq('name', project.task_queue_name)
    .single();
  
  if (!taskQueue) {
    console.error('❌ System task queue not found');
    return null;
  }
  
  // Check if workflow already exists
  const { data: existing } = await supabase
    .from('workflows')
    .select('id')
    .eq('name', 'agent-tester')
    .eq('created_by', systemUserId)
    .single();
  
  if (existing) {
    return existing.id;
  }
  
  // Get workflow status
  const { data: activeStatus } = await supabase
    .from('workflow_statuses')
    .select('id')
    .eq('name', 'active')
    .single();
  
  if (!activeStatus) {
    console.error('❌ Active workflow status not found');
    return null;
  }
  
  // Get visibility
  const { data: visibility } = await supabase
    .from('component_visibility')
    .select('id')
    .eq('name', 'public')
    .single();
  
  if (!visibility) {
    console.error('❌ Public visibility not found');
    return null;
  }
  
  // Create workflow definition
  const definition = {
    nodes: [
      {
        id: 'start',
        type: 'trigger',
        position: { x: 0, y: 0 },
        data: { label: 'Start' },
      },
    ],
    edges: [],
  };
  
  // Create workflow
  const { data: workflow, error } = await supabase
    .from('workflows')
    .insert({
      name: 'agent-tester',
      kebab_name: 'agent-tester',
      display_name: 'Agent Tester',
      description: 'System workflow for testing agent prompts with human-in-the-loop interaction',
      created_by: systemUserId,
      status_id: activeStatus.id,
      task_queue_id: taskQueue.id,
      project_id: projectId,
      visibility_id: visibility.id,
      version: '1.0.0',
      definition: definition as any,
      temporal_workflow_type: 'agentTesterWorkflow',
    })
    .select('id')
    .single();
  
  if (error) {
    console.error('❌ Failed to register agent tester workflow:', error);
    return null;
  }
  
  console.log('✅ Agent tester workflow registered:', workflow.id);
  return workflow.id;
}

/**
 * Register sync coordinator workflow in database
 */
export async function registerSyncCoordinatorWorkflow(
  systemUserId: string,
  projectId: string
): Promise<string | null> {
  const supabase = getSupabaseClient();
  
  // Get project to find task queue
  const { data: project } = await supabase
    .from('projects')
    .select('task_queue_name')
    .eq('id', projectId)
    .single();
  
  if (!project) {
    console.error('❌ System project not found');
    return null;
  }
  
  // Get task queue ID
  const { data: taskQueue } = await supabase
    .from('task_queues')
    .select('id')
    .eq('name', project.task_queue_name)
    .single();
  
  if (!taskQueue) {
    console.error('❌ System task queue not found');
    return null;
  }
  
  // Check if workflow already exists
  const { data: existing } = await supabase
    .from('workflows')
    .select('id')
    .eq('kebab_name', 'sync-coordinator')
    .eq('created_by', systemUserId)
    .single();
  
  if (existing) {
    return existing.id;
  }
  
  // Get workflow status
  const { data: activeStatus } = await supabase
    .from('workflow_statuses')
    .select('id')
    .eq('name', 'active')
    .single();
  
  if (!activeStatus) {
    console.error('❌ Active workflow status not found');
    return null;
  }
  
  // Get visibility
  const { data: visibility } = await supabase
    .from('component_visibility')
    .select('id')
    .eq('name', 'public')
    .single();
  
  if (!visibility) {
    console.error('❌ Public visibility not found');
    return null;
  }
  
  // Create workflow definition
  const definition = {
    nodes: [
      {
        id: 'start',
        type: 'trigger',
        position: { x: 0, y: 0 },
        data: { label: 'Start' },
      },
    ],
    edges: [],
  };
  
  // Create workflow
  const { data: workflow, error } = await supabase
    .from('workflows')
    .insert({
      name: 'sync-coordinator',
      kebab_name: 'sync-coordinator',
      display_name: 'Sync Coordinator',
      description: 'System workflow that coordinates syncing execution history from Temporal to database. Manages queue of executions to sync and spawns child sync workflows.',
      created_by: systemUserId,
      status_id: activeStatus.id,
      task_queue_id: taskQueue.id,
      project_id: projectId,
      visibility_id: visibility.id,
      version: '1.0.0',
      definition: definition as any,
      temporal_workflow_type: 'syncCoordinatorWorkflow',
    })
    .select('id')
    .single();
  
  if (error) {
    console.error('❌ Failed to register sync coordinator workflow:', error);
    return null;
  }
  
  console.log('✅ Sync coordinator workflow registered:', workflow.id);
  return workflow.id;
}

/**
 * Initialize all system workflows
 * Call this on app startup
 * Note: This is mainly for verification - the seed migration should have created everything
 */
export async function initializeSystemWorkflows(): Promise<void> {
  console.log('🔧 Verifying system workflows setup...');
  
  // Ensure system user exists
  const systemUserId = await ensureSystemUser();
  if (!systemUserId) {
    console.warn('⚠️  System user not available. Run seed migration 20251118000003 after creating auth user.');
    return;
  }
  
  // Ensure system project exists
  const projectId = await ensureSystemProject(systemUserId);
  if (!projectId) {
    console.warn('⚠️  System project not available. Run seed migration 20251118000003.');
    return;
  }
  
  // Register agent tester workflow (idempotent - won't create if exists)
  await registerAgentTesterWorkflow(systemUserId, projectId);
  
  // Register sync coordinator workflow (idempotent - won't create if exists)
  await registerSyncCoordinatorWorkflow(systemUserId, projectId);
  
  console.log('✅ System workflows verified');
}

