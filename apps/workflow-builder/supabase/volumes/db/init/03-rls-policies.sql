-- =============================================================================
-- ROW LEVEL SECURITY POLICIES
-- =============================================================================
-- Provides defense-in-depth access control at the database level.
-- The Rust API server uses service_role (which bypasses RLS) for all operations,
-- so these policies protect against direct database access from the frontend
-- Supabase client or any other non-service-role connection.
--
-- Key insight: auth.uid() returns the GoTrue UUID, but our tables reference
-- public.users.id (internal UUID) via created_by/user_id columns. These are
-- different UUIDs joined through users.auth_user_id. The current_user_id()
-- helper bridges this gap.
--
-- All UPDATE policies include WITH CHECK to prevent ownership transfer
-- (a user cannot change created_by to another user's ID).
-- =============================================================================

-- =============================================================================
-- HELPER FUNCTIONS
-- =============================================================================

-- Resolve GoTrue auth.uid() -> public.users.id
-- SECURITY DEFINER so it can read users table regardless of RLS on that table.
-- STABLE because the mapping doesn't change within a transaction.
CREATE OR REPLACE FUNCTION public.current_user_id()
RETURNS UUID AS $$
  SELECT id FROM public.users WHERE auth_user_id = auth.uid()
$$ LANGUAGE sql STABLE SECURITY DEFINER;

-- Check if the current user owns a workflow (by workflow ID).
-- Used by child tables that have workflow_id but no created_by column.
CREATE OR REPLACE FUNCTION public.user_owns_workflow(wf_id UUID)
RETURNS BOOLEAN AS $$
  SELECT EXISTS (
    SELECT 1 FROM public.workflows
    WHERE id = wf_id AND created_by = current_user_id()
  )
$$ LANGUAGE sql STABLE SECURITY DEFINER;

-- Check if the current user owns a project (by project ID).
-- Used by project child tables that have project_id but no created_by column.
CREATE OR REPLACE FUNCTION public.user_owns_project(proj_id UUID)
RETURNS BOOLEAN AS $$
  SELECT EXISTS (
    SELECT 1 FROM public.projects
    WHERE id = proj_id AND created_by = current_user_id()
  )
$$ LANGUAGE sql STABLE SECURITY DEFINER;

-- Check if the current user owns a connector (by connector ID).
CREATE OR REPLACE FUNCTION public.user_owns_connector(conn_id UUID)
RETURNS BOOLEAN AS $$
  SELECT EXISTS (
    SELECT 1 FROM public.connectors
    WHERE id = conn_id AND created_by = current_user_id()
  )
$$ LANGUAGE sql STABLE SECURITY DEFINER;

-- Check if the current user owns a service interface's parent workflow.
-- Used by public_interfaces which are children of service_interfaces.
CREATE OR REPLACE FUNCTION public.user_owns_service_interface(si_id UUID)
RETURNS BOOLEAN AS $$
  SELECT EXISTS (
    SELECT 1 FROM public.service_interfaces si
    JOIN public.workflows w ON w.id = si.workflow_id
    WHERE si.id = si_id AND w.created_by = current_user_id()
  )
$$ LANGUAGE sql STABLE SECURITY DEFINER;

-- =============================================================================
-- USERS TABLE
-- =============================================================================
-- Uses auth.uid() = auth_user_id directly since this IS the mapping table.
-- current_user_id() is SECURITY DEFINER so it can read users regardless of RLS.
ALTER TABLE public.users ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can view own profile"
  ON public.users FOR SELECT
  USING (auth.uid() = auth_user_id);

CREATE POLICY "Users can update own profile"
  ON public.users FOR UPDATE
  USING (auth.uid() = auth_user_id)
  WITH CHECK (auth.uid() = auth_user_id);

CREATE POLICY "Service role full access to users"
  ON public.users FOR ALL
  USING (auth.role() = 'service_role');

-- =============================================================================
-- API KEYS TABLE (table defined in 02-api-keys.sql, policies here)
-- =============================================================================
-- Uses current_user_id() to bridge auth.uid() -> users.id since
-- api_keys.user_id references public.users.id, NOT auth.uid() directly.

CREATE POLICY "Users can view own API keys"
  ON public.api_keys FOR SELECT
  USING (current_user_id() = user_id);

CREATE POLICY "Users can create own API keys"
  ON public.api_keys FOR INSERT
  WITH CHECK (current_user_id() = user_id);

CREATE POLICY "Users can update own API keys"
  ON public.api_keys FOR UPDATE
  USING (current_user_id() = user_id)
  WITH CHECK (current_user_id() = user_id);

CREATE POLICY "Users can delete own API keys"
  ON public.api_keys FOR DELETE
  USING (current_user_id() = user_id);

CREATE POLICY "Service role full access to api_keys"
  ON public.api_keys FOR ALL
  USING (auth.role() = 'service_role');

-- =============================================================================
-- WORKFLOWS TABLE
-- =============================================================================
ALTER TABLE public.workflows ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can view own workflows"
  ON public.workflows FOR SELECT
  USING (current_user_id() = created_by);

CREATE POLICY "Users can create own workflows"
  ON public.workflows FOR INSERT
  WITH CHECK (current_user_id() = created_by);

CREATE POLICY "Users can update own workflows"
  ON public.workflows FOR UPDATE
  USING (current_user_id() = created_by)
  WITH CHECK (current_user_id() = created_by);

CREATE POLICY "Users can delete own workflows"
  ON public.workflows FOR DELETE
  USING (current_user_id() = created_by);

CREATE POLICY "Service role full access to workflows"
  ON public.workflows FOR ALL
  USING (auth.role() = 'service_role');

-- =============================================================================
-- WORKFLOW CHILD TABLES (no created_by -- derive access from parent workflow)
-- =============================================================================

-- workflow_nodes
ALTER TABLE public.workflow_nodes ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can view own workflow nodes"
  ON public.workflow_nodes FOR SELECT
  USING (user_owns_workflow(workflow_id));

CREATE POLICY "Users can create own workflow nodes"
  ON public.workflow_nodes FOR INSERT
  WITH CHECK (user_owns_workflow(workflow_id));

CREATE POLICY "Users can update own workflow nodes"
  ON public.workflow_nodes FOR UPDATE
  USING (user_owns_workflow(workflow_id))
  WITH CHECK (user_owns_workflow(workflow_id));

CREATE POLICY "Users can delete own workflow nodes"
  ON public.workflow_nodes FOR DELETE
  USING (user_owns_workflow(workflow_id));

CREATE POLICY "Service role full access to workflow_nodes"
  ON public.workflow_nodes FOR ALL
  USING (auth.role() = 'service_role');

-- workflow_edges
ALTER TABLE public.workflow_edges ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can view own workflow edges"
  ON public.workflow_edges FOR SELECT
  USING (user_owns_workflow(workflow_id));

CREATE POLICY "Users can create own workflow edges"
  ON public.workflow_edges FOR INSERT
  WITH CHECK (user_owns_workflow(workflow_id));

CREATE POLICY "Users can update own workflow edges"
  ON public.workflow_edges FOR UPDATE
  USING (user_owns_workflow(workflow_id))
  WITH CHECK (user_owns_workflow(workflow_id));

CREATE POLICY "Users can delete own workflow edges"
  ON public.workflow_edges FOR DELETE
  USING (user_owns_workflow(workflow_id));

CREATE POLICY "Service role full access to workflow_edges"
  ON public.workflow_edges FOR ALL
  USING (auth.role() = 'service_role');

-- workflow_compiled_code
ALTER TABLE public.workflow_compiled_code ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can view own workflow compiled code"
  ON public.workflow_compiled_code FOR SELECT
  USING (user_owns_workflow(workflow_id));

CREATE POLICY "Users can create own workflow compiled code"
  ON public.workflow_compiled_code FOR INSERT
  WITH CHECK (user_owns_workflow(workflow_id));

CREATE POLICY "Users can update own workflow compiled code"
  ON public.workflow_compiled_code FOR UPDATE
  USING (user_owns_workflow(workflow_id))
  WITH CHECK (user_owns_workflow(workflow_id));

CREATE POLICY "Users can delete own workflow compiled code"
  ON public.workflow_compiled_code FOR DELETE
  USING (user_owns_workflow(workflow_id));

CREATE POLICY "Service role full access to workflow_compiled_code"
  ON public.workflow_compiled_code FOR ALL
  USING (auth.role() = 'service_role');

-- workflow_state_variables
ALTER TABLE public.workflow_state_variables ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can view own workflow state variables"
  ON public.workflow_state_variables FOR SELECT
  USING (user_owns_workflow(workflow_id));

CREATE POLICY "Users can create own workflow state variables"
  ON public.workflow_state_variables FOR INSERT
  WITH CHECK (user_owns_workflow(workflow_id));

CREATE POLICY "Users can update own workflow state variables"
  ON public.workflow_state_variables FOR UPDATE
  USING (user_owns_workflow(workflow_id))
  WITH CHECK (user_owns_workflow(workflow_id));

CREATE POLICY "Users can delete own workflow state variables"
  ON public.workflow_state_variables FOR DELETE
  USING (user_owns_workflow(workflow_id));

CREATE POLICY "Service role full access to workflow_state_variables"
  ON public.workflow_state_variables FOR ALL
  USING (auth.role() = 'service_role');

-- =============================================================================
-- WORKFLOW CHILD TABLES (have created_by or compiled_by)
-- =============================================================================

-- workflow_executions
-- NOTE: Uses user_owns_workflow() instead of current_user_id() = created_by
-- because workflow_executions.created_by is NULLABLE. Rows with NULL created_by
-- would be inaccessible if we checked created_by directly.
ALTER TABLE public.workflow_executions ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can view own workflow executions"
  ON public.workflow_executions FOR SELECT
  USING (user_owns_workflow(workflow_id));

CREATE POLICY "Users can create own workflow executions"
  ON public.workflow_executions FOR INSERT
  WITH CHECK (user_owns_workflow(workflow_id));

CREATE POLICY "Users can update own workflow executions"
  ON public.workflow_executions FOR UPDATE
  USING (user_owns_workflow(workflow_id))
  WITH CHECK (user_owns_workflow(workflow_id));

CREATE POLICY "Users can delete own workflow executions"
  ON public.workflow_executions FOR DELETE
  USING (user_owns_workflow(workflow_id));

CREATE POLICY "Service role full access to workflow_executions"
  ON public.workflow_executions FOR ALL
  USING (auth.role() = 'service_role');

-- workflow_work_queues
ALTER TABLE public.workflow_work_queues ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can view own workflow work queues"
  ON public.workflow_work_queues FOR SELECT
  USING (current_user_id() = created_by);

CREATE POLICY "Users can create own workflow work queues"
  ON public.workflow_work_queues FOR INSERT
  WITH CHECK (current_user_id() = created_by);

CREATE POLICY "Users can update own workflow work queues"
  ON public.workflow_work_queues FOR UPDATE
  USING (current_user_id() = created_by)
  WITH CHECK (current_user_id() = created_by);

CREATE POLICY "Users can delete own workflow work queues"
  ON public.workflow_work_queues FOR DELETE
  USING (current_user_id() = created_by);

CREATE POLICY "Service role full access to workflow_work_queues"
  ON public.workflow_work_queues FOR ALL
  USING (auth.role() = 'service_role');

-- workflow_signals
ALTER TABLE public.workflow_signals ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can view own workflow signals"
  ON public.workflow_signals FOR SELECT
  USING (current_user_id() = created_by);

CREATE POLICY "Users can create own workflow signals"
  ON public.workflow_signals FOR INSERT
  WITH CHECK (current_user_id() = created_by);

CREATE POLICY "Users can update own workflow signals"
  ON public.workflow_signals FOR UPDATE
  USING (current_user_id() = created_by)
  WITH CHECK (current_user_id() = created_by);

CREATE POLICY "Users can delete own workflow signals"
  ON public.workflow_signals FOR DELETE
  USING (current_user_id() = created_by);

CREATE POLICY "Service role full access to workflow_signals"
  ON public.workflow_signals FOR ALL
  USING (auth.role() = 'service_role');

-- workflow_queries
ALTER TABLE public.workflow_queries ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can view own workflow queries"
  ON public.workflow_queries FOR SELECT
  USING (current_user_id() = created_by);

CREATE POLICY "Users can create own workflow queries"
  ON public.workflow_queries FOR INSERT
  WITH CHECK (current_user_id() = created_by);

CREATE POLICY "Users can update own workflow queries"
  ON public.workflow_queries FOR UPDATE
  USING (current_user_id() = created_by)
  WITH CHECK (current_user_id() = created_by);

CREATE POLICY "Users can delete own workflow queries"
  ON public.workflow_queries FOR DELETE
  USING (current_user_id() = created_by);

CREATE POLICY "Service role full access to workflow_queries"
  ON public.workflow_queries FOR ALL
  USING (auth.role() = 'service_role');

-- =============================================================================
-- PROJECTS TABLE
-- =============================================================================
ALTER TABLE public.projects ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can view own projects"
  ON public.projects FOR SELECT
  USING (current_user_id() = created_by);

CREATE POLICY "Users can create own projects"
  ON public.projects FOR INSERT
  WITH CHECK (current_user_id() = created_by);

CREATE POLICY "Users can update own projects"
  ON public.projects FOR UPDATE
  USING (current_user_id() = created_by)
  WITH CHECK (current_user_id() = created_by);

CREATE POLICY "Users can delete own projects"
  ON public.projects FOR DELETE
  USING (current_user_id() = created_by);

CREATE POLICY "Service role full access to projects"
  ON public.projects FOR ALL
  USING (auth.role() = 'service_role');

-- =============================================================================
-- TASK QUEUES
-- =============================================================================
ALTER TABLE public.task_queues ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can view own task queues"
  ON public.task_queues FOR SELECT
  USING (current_user_id() = created_by);

CREATE POLICY "Users can create own task queues"
  ON public.task_queues FOR INSERT
  WITH CHECK (current_user_id() = created_by);

CREATE POLICY "Users can update own task queues"
  ON public.task_queues FOR UPDATE
  USING (current_user_id() = created_by)
  WITH CHECK (current_user_id() = created_by);

CREATE POLICY "Users can delete own task queues"
  ON public.task_queues FOR DELETE
  USING (current_user_id() = created_by);

CREATE POLICY "Service role full access to task_queues"
  ON public.task_queues FOR ALL
  USING (auth.role() = 'service_role');

-- =============================================================================
-- COMPONENTS TABLE
-- =============================================================================
ALTER TABLE public.components ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can view own components"
  ON public.components FOR SELECT
  USING (current_user_id() = created_by);

CREATE POLICY "Users can create own components"
  ON public.components FOR INSERT
  WITH CHECK (current_user_id() = created_by);

CREATE POLICY "Users can update own components"
  ON public.components FOR UPDATE
  USING (current_user_id() = created_by)
  WITH CHECK (current_user_id() = created_by);

CREATE POLICY "Users can delete own components"
  ON public.components FOR DELETE
  USING (current_user_id() = created_by);

CREATE POLICY "Service role full access to components"
  ON public.components FOR ALL
  USING (auth.role() = 'service_role');

-- =============================================================================
-- AGENT PROMPTS TABLE
-- =============================================================================
ALTER TABLE public.agent_prompts ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can view own agent prompts"
  ON public.agent_prompts FOR SELECT
  USING (current_user_id() = created_by);

CREATE POLICY "Users can create own agent prompts"
  ON public.agent_prompts FOR INSERT
  WITH CHECK (current_user_id() = created_by);

CREATE POLICY "Users can update own agent prompts"
  ON public.agent_prompts FOR UPDATE
  USING (current_user_id() = created_by)
  WITH CHECK (current_user_id() = created_by);

CREATE POLICY "Users can delete own agent prompts"
  ON public.agent_prompts FOR DELETE
  USING (current_user_id() = created_by);

CREATE POLICY "Service role full access to agent_prompts"
  ON public.agent_prompts FOR ALL
  USING (auth.role() = 'service_role');

-- =============================================================================
-- ACTIVITIES TABLE
-- =============================================================================
ALTER TABLE public.activities ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can view own activities"
  ON public.activities FOR SELECT
  USING (current_user_id() = created_by);

CREATE POLICY "Users can create own activities"
  ON public.activities FOR INSERT
  WITH CHECK (current_user_id() = created_by);

CREATE POLICY "Users can update own activities"
  ON public.activities FOR UPDATE
  USING (current_user_id() = created_by)
  WITH CHECK (current_user_id() = created_by);

CREATE POLICY "Users can delete own activities"
  ON public.activities FOR DELETE
  USING (current_user_id() = created_by);

CREATE POLICY "Service role full access to activities"
  ON public.activities FOR ALL
  USING (auth.role() = 'service_role');

-- =============================================================================
-- CONNECTORS TABLE
-- =============================================================================
ALTER TABLE public.connectors ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can view own connectors"
  ON public.connectors FOR SELECT
  USING (current_user_id() = created_by);

CREATE POLICY "Users can create own connectors"
  ON public.connectors FOR INSERT
  WITH CHECK (current_user_id() = created_by);

CREATE POLICY "Users can update own connectors"
  ON public.connectors FOR UPDATE
  USING (current_user_id() = created_by)
  WITH CHECK (current_user_id() = created_by);

CREATE POLICY "Users can delete own connectors"
  ON public.connectors FOR DELETE
  USING (current_user_id() = created_by);

CREATE POLICY "Service role full access to connectors"
  ON public.connectors FOR ALL
  USING (auth.role() = 'service_role');

-- =============================================================================
-- PROJECT CONNECTORS TABLE
-- =============================================================================
ALTER TABLE public.project_connectors ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can view own project connectors"
  ON public.project_connectors FOR SELECT
  USING (current_user_id() = created_by);

CREATE POLICY "Users can create own project connectors"
  ON public.project_connectors FOR INSERT
  WITH CHECK (current_user_id() = created_by);

CREATE POLICY "Users can update own project connectors"
  ON public.project_connectors FOR UPDATE
  USING (current_user_id() = created_by)
  WITH CHECK (current_user_id() = created_by);

CREATE POLICY "Users can delete own project connectors"
  ON public.project_connectors FOR DELETE
  USING (current_user_id() = created_by);

CREATE POLICY "Service role full access to project_connectors"
  ON public.project_connectors FOR ALL
  USING (auth.role() = 'service_role');

-- =============================================================================
-- CONNECTOR CLASSIFICATIONS (child of connectors)
-- =============================================================================
ALTER TABLE public.connector_classifications ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can view own connector classifications"
  ON public.connector_classifications FOR SELECT
  USING (user_owns_connector(connector_id));

CREATE POLICY "Users can create own connector classifications"
  ON public.connector_classifications FOR INSERT
  WITH CHECK (user_owns_connector(connector_id));

CREATE POLICY "Users can update own connector classifications"
  ON public.connector_classifications FOR UPDATE
  USING (user_owns_connector(connector_id))
  WITH CHECK (user_owns_connector(connector_id));

CREATE POLICY "Users can delete own connector classifications"
  ON public.connector_classifications FOR DELETE
  USING (user_owns_connector(connector_id));

CREATE POLICY "Service role full access to connector_classifications"
  ON public.connector_classifications FOR ALL
  USING (auth.role() = 'service_role');

-- =============================================================================
-- PROJECT CHILD TABLES (derive access from parent project)
-- =============================================================================

-- project_state_variables
ALTER TABLE public.project_state_variables ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can view own project state variables"
  ON public.project_state_variables FOR SELECT
  USING (user_owns_project(project_id));

CREATE POLICY "Users can create own project state variables"
  ON public.project_state_variables FOR INSERT
  WITH CHECK (user_owns_project(project_id));

CREATE POLICY "Users can update own project state variables"
  ON public.project_state_variables FOR UPDATE
  USING (user_owns_project(project_id))
  WITH CHECK (user_owns_project(project_id));

CREATE POLICY "Users can delete own project state variables"
  ON public.project_state_variables FOR DELETE
  USING (user_owns_project(project_id));

CREATE POLICY "Service role full access to project_state_variables"
  ON public.project_state_variables FOR ALL
  USING (auth.role() = 'service_role');

-- workflow_workers
ALTER TABLE public.workflow_workers ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can view own workflow workers"
  ON public.workflow_workers FOR SELECT
  USING (user_owns_project(project_id));

CREATE POLICY "Users can create own workflow workers"
  ON public.workflow_workers FOR INSERT
  WITH CHECK (user_owns_project(project_id));

CREATE POLICY "Users can update own workflow workers"
  ON public.workflow_workers FOR UPDATE
  USING (user_owns_project(project_id))
  WITH CHECK (user_owns_project(project_id));

CREATE POLICY "Users can delete own workflow workers"
  ON public.workflow_workers FOR DELETE
  USING (user_owns_project(project_id));

CREATE POLICY "Service role full access to workflow_workers"
  ON public.workflow_workers FOR ALL
  USING (auth.role() = 'service_role');

-- =============================================================================
-- AGENT SESSIONS (user_id ownership)
-- =============================================================================

-- agent_builder_sessions
ALTER TABLE public.agent_builder_sessions ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can view own agent builder sessions"
  ON public.agent_builder_sessions FOR SELECT
  USING (current_user_id() = user_id);

CREATE POLICY "Users can create own agent builder sessions"
  ON public.agent_builder_sessions FOR INSERT
  WITH CHECK (current_user_id() = user_id);

CREATE POLICY "Users can update own agent builder sessions"
  ON public.agent_builder_sessions FOR UPDATE
  USING (current_user_id() = user_id)
  WITH CHECK (current_user_id() = user_id);

CREATE POLICY "Users can delete own agent builder sessions"
  ON public.agent_builder_sessions FOR DELETE
  USING (current_user_id() = user_id);

CREATE POLICY "Service role full access to agent_builder_sessions"
  ON public.agent_builder_sessions FOR ALL
  USING (auth.role() = 'service_role');

-- agent_test_sessions
ALTER TABLE public.agent_test_sessions ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can view own agent test sessions"
  ON public.agent_test_sessions FOR SELECT
  USING (current_user_id() = user_id);

CREATE POLICY "Users can create own agent test sessions"
  ON public.agent_test_sessions FOR INSERT
  WITH CHECK (current_user_id() = user_id);

CREATE POLICY "Users can update own agent test sessions"
  ON public.agent_test_sessions FOR UPDATE
  USING (current_user_id() = user_id)
  WITH CHECK (current_user_id() = user_id);

CREATE POLICY "Users can delete own agent test sessions"
  ON public.agent_test_sessions FOR DELETE
  USING (current_user_id() = user_id);

CREATE POLICY "Service role full access to agent_test_sessions"
  ON public.agent_test_sessions FOR ALL
  USING (auth.role() = 'service_role');

-- =============================================================================
-- SERVICE INTERFACES (child of workflows)
-- =============================================================================
ALTER TABLE public.service_interfaces ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can view own service interfaces"
  ON public.service_interfaces FOR SELECT
  USING (user_owns_workflow(workflow_id));

CREATE POLICY "Users can create own service interfaces"
  ON public.service_interfaces FOR INSERT
  WITH CHECK (user_owns_workflow(workflow_id));

CREATE POLICY "Users can update own service interfaces"
  ON public.service_interfaces FOR UPDATE
  USING (user_owns_workflow(workflow_id))
  WITH CHECK (user_owns_workflow(workflow_id));

CREATE POLICY "Users can delete own service interfaces"
  ON public.service_interfaces FOR DELETE
  USING (user_owns_workflow(workflow_id));

CREATE POLICY "Service role full access to service_interfaces"
  ON public.service_interfaces FOR ALL
  USING (auth.role() = 'service_role');

-- =============================================================================
-- PUBLIC INTERFACES (child of service_interfaces -> workflows)
-- =============================================================================
ALTER TABLE public.public_interfaces ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can view own public interfaces"
  ON public.public_interfaces FOR SELECT
  USING (user_owns_service_interface(service_interface_id));

CREATE POLICY "Users can create own public interfaces"
  ON public.public_interfaces FOR INSERT
  WITH CHECK (user_owns_service_interface(service_interface_id));

CREATE POLICY "Users can update own public interfaces"
  ON public.public_interfaces FOR UPDATE
  USING (user_owns_service_interface(service_interface_id))
  WITH CHECK (user_owns_service_interface(service_interface_id));

CREATE POLICY "Users can delete own public interfaces"
  ON public.public_interfaces FOR DELETE
  USING (user_owns_service_interface(service_interface_id));

CREATE POLICY "Service role full access to public_interfaces"
  ON public.public_interfaces FOR ALL
  USING (auth.role() = 'service_role');

-- =============================================================================
-- METRICS AND STATISTICS TABLES (derive access from parent project)
-- =============================================================================

-- activity_statistics
ALTER TABLE public.activity_statistics ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can view own activity statistics"
  ON public.activity_statistics FOR SELECT
  USING (user_owns_project(project_id));

CREATE POLICY "Users can create own activity statistics"
  ON public.activity_statistics FOR INSERT
  WITH CHECK (user_owns_project(project_id));

CREATE POLICY "Users can update own activity statistics"
  ON public.activity_statistics FOR UPDATE
  USING (user_owns_project(project_id))
  WITH CHECK (user_owns_project(project_id));

CREATE POLICY "Users can delete own activity statistics"
  ON public.activity_statistics FOR DELETE
  USING (user_owns_project(project_id));

CREATE POLICY "Service role full access to activity_statistics"
  ON public.activity_statistics FOR ALL
  USING (auth.role() = 'service_role');

-- component_metrics
ALTER TABLE public.component_metrics ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can view own component metrics"
  ON public.component_metrics FOR SELECT
  USING (user_owns_project(project_id));

CREATE POLICY "Users can create own component metrics"
  ON public.component_metrics FOR INSERT
  WITH CHECK (user_owns_project(project_id));

CREATE POLICY "Users can update own component metrics"
  ON public.component_metrics FOR UPDATE
  USING (user_owns_project(project_id))
  WITH CHECK (user_owns_project(project_id));

CREATE POLICY "Users can delete own component metrics"
  ON public.component_metrics FOR DELETE
  USING (user_owns_project(project_id));

CREATE POLICY "Service role full access to component_metrics"
  ON public.component_metrics FOR ALL
  USING (auth.role() = 'service_role');

-- component_usage_daily
ALTER TABLE public.component_usage_daily ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can view own component usage daily"
  ON public.component_usage_daily FOR SELECT
  USING (user_owns_project(project_id));

CREATE POLICY "Users can create own component usage daily"
  ON public.component_usage_daily FOR INSERT
  WITH CHECK (user_owns_project(project_id));

CREATE POLICY "Users can update own component usage daily"
  ON public.component_usage_daily FOR UPDATE
  USING (user_owns_project(project_id))
  WITH CHECK (user_owns_project(project_id));

CREATE POLICY "Users can delete own component usage daily"
  ON public.component_usage_daily FOR DELETE
  USING (user_owns_project(project_id));

CREATE POLICY "Service role full access to component_usage_daily"
  ON public.component_usage_daily FOR ALL
  USING (auth.role() = 'service_role');

-- workflow_execution_metrics
ALTER TABLE public.workflow_execution_metrics ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can view own workflow execution metrics"
  ON public.workflow_execution_metrics FOR SELECT
  USING (user_owns_project(project_id));

CREATE POLICY "Users can create own workflow execution metrics"
  ON public.workflow_execution_metrics FOR INSERT
  WITH CHECK (user_owns_project(project_id));

CREATE POLICY "Users can update own workflow execution metrics"
  ON public.workflow_execution_metrics FOR UPDATE
  USING (user_owns_project(project_id))
  WITH CHECK (user_owns_project(project_id));

CREATE POLICY "Users can delete own workflow execution metrics"
  ON public.workflow_execution_metrics FOR DELETE
  USING (user_owns_project(project_id));

CREATE POLICY "Service role full access to workflow_execution_metrics"
  ON public.workflow_execution_metrics FOR ALL
  USING (auth.role() = 'service_role');

-- workflow_usage_daily
ALTER TABLE public.workflow_usage_daily ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can view own workflow usage daily"
  ON public.workflow_usage_daily FOR SELECT
  USING (user_owns_project(project_id));

CREATE POLICY "Users can create own workflow usage daily"
  ON public.workflow_usage_daily FOR INSERT
  WITH CHECK (user_owns_project(project_id));

CREATE POLICY "Users can update own workflow usage daily"
  ON public.workflow_usage_daily FOR UPDATE
  USING (user_owns_project(project_id))
  WITH CHECK (user_owns_project(project_id));

CREATE POLICY "Users can delete own workflow usage daily"
  ON public.workflow_usage_daily FOR DELETE
  USING (user_owns_project(project_id));

CREATE POLICY "Service role full access to workflow_usage_daily"
  ON public.workflow_usage_daily FOR ALL
  USING (auth.role() = 'service_role');

-- resource_events
ALTER TABLE public.resource_events ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can view own resource events"
  ON public.resource_events FOR SELECT
  USING (user_owns_project(project_id));

CREATE POLICY "Users can create own resource events"
  ON public.resource_events FOR INSERT
  WITH CHECK (user_owns_project(project_id));

CREATE POLICY "Users can update own resource events"
  ON public.resource_events FOR UPDATE
  USING (user_owns_project(project_id))
  WITH CHECK (user_owns_project(project_id));

CREATE POLICY "Users can delete own resource events"
  ON public.resource_events FOR DELETE
  USING (user_owns_project(project_id));

CREATE POLICY "Service role full access to resource_events"
  ON public.resource_events FOR ALL
  USING (auth.role() = 'service_role');

-- resource_usage_daily
ALTER TABLE public.resource_usage_daily ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can view own resource usage daily"
  ON public.resource_usage_daily FOR SELECT
  USING (user_owns_project(project_id));

CREATE POLICY "Users can create own resource usage daily"
  ON public.resource_usage_daily FOR INSERT
  WITH CHECK (user_owns_project(project_id));

CREATE POLICY "Users can update own resource usage daily"
  ON public.resource_usage_daily FOR UPDATE
  USING (user_owns_project(project_id))
  WITH CHECK (user_owns_project(project_id));

CREATE POLICY "Users can delete own resource usage daily"
  ON public.resource_usage_daily FOR DELETE
  USING (user_owns_project(project_id));

CREATE POLICY "Service role full access to resource_usage_daily"
  ON public.resource_usage_daily FOR ALL
  USING (auth.role() = 'service_role');

-- =============================================================================
-- NOTE: The following tables intentionally have NO RLS because they are
-- lookup/reference data with no user ownership:
--   - user_roles
--   - workflow_statuses
--   - component_types
--   - component_visibility
--   - activity_categories
--   - component_categories
--   - component_category_mapping (references components but is public metadata)
--   - component_keywords (references components but is public metadata)
--   - component_use_cases (references components but is public metadata)
--   - service_interface_endpoints (public API documentation)
--   - state_variable_metrics (no user ownership column, aggregate data)
-- =============================================================================

-- =============================================================================
-- PERFORMANCE INDEXES FOR RLS SUBQUERIES
-- =============================================================================

-- Composite index for the user_owns_workflow() function
CREATE INDEX IF NOT EXISTS idx_workflows_id_created_by
  ON public.workflows(id, created_by);

-- Composite index for the user_owns_project() function
CREATE INDEX IF NOT EXISTS idx_projects_id_created_by
  ON public.projects(id, created_by);

-- Composite index for the user_owns_connector() function
CREATE INDEX IF NOT EXISTS idx_connectors_id_created_by
  ON public.connectors(id, created_by);

-- Additional created_by indexes for tables that don't have them yet
CREATE INDEX IF NOT EXISTS idx_activities_created_by
  ON public.activities(created_by);
CREATE INDEX IF NOT EXISTS idx_task_queues_created_by
  ON public.task_queues(created_by);
CREATE INDEX IF NOT EXISTS idx_connectors_created_by
  ON public.connectors(created_by);
CREATE INDEX IF NOT EXISTS idx_project_connectors_created_by
  ON public.project_connectors(created_by);

-- Verify existing indexes exist (idempotent)
CREATE INDEX IF NOT EXISTS idx_users_auth_user_id
  ON public.users(auth_user_id);
CREATE INDEX IF NOT EXISTS idx_workflows_created_by
  ON public.workflows(created_by);
CREATE INDEX IF NOT EXISTS idx_projects_created_by
  ON public.projects(created_by);
CREATE INDEX IF NOT EXISTS idx_components_created_by
  ON public.components(created_by);
CREATE INDEX IF NOT EXISTS idx_agent_prompts_created_by
  ON public.agent_prompts(created_by);
