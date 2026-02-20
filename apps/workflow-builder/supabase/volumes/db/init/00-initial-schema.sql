-- Radium Workflow Builder - Production-Matching Test Database Schema
-- This schema exactly matches the production Supabase schema from src/types/database.ts

-- Create extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- Create roles
DO $$
BEGIN
  IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'anon') THEN
    CREATE ROLE anon NOLOGIN NOINHERIT;
  END IF;
  IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'authenticated') THEN
    CREATE ROLE authenticated NOLOGIN NOINHERIT;
  END IF;
  IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'service_role') THEN
    CREATE ROLE service_role NOLOGIN NOINHERIT BYPASSRLS;
  END IF;
END
$$;

-- Grant schema access
GRANT USAGE ON SCHEMA public TO anon, authenticated, service_role;
GRANT ALL ON SCHEMA public TO postgres;

-- Create auth schema for GoTrue
CREATE SCHEMA IF NOT EXISTS auth;
GRANT USAGE ON SCHEMA auth TO postgres, service_role;
GRANT ALL ON SCHEMA auth TO postgres;

-- =============================================================================
-- LOOKUP TABLES (no foreign key dependencies)
-- =============================================================================

-- User Roles
CREATE TABLE IF NOT EXISTS public.user_roles (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  name TEXT NOT NULL,
  description TEXT,
  permissions JSONB DEFAULT '{}',
  created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Workflow Statuses
CREATE TABLE IF NOT EXISTS public.workflow_statuses (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  name TEXT NOT NULL,
  description TEXT,
  color TEXT
);

-- Component Types
CREATE TABLE IF NOT EXISTS public.component_types (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  name TEXT NOT NULL,
  description TEXT,
  icon TEXT,
  created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Component Visibility
CREATE TABLE IF NOT EXISTS public.component_visibility (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  name TEXT NOT NULL,
  description TEXT
);

-- Activity Categories
CREATE TABLE IF NOT EXISTS public.activity_categories (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  name TEXT NOT NULL,
  description TEXT,
  icon TEXT,
  created_at TIMESTAMPTZ DEFAULT NOW()
);

-- =============================================================================
-- USERS TABLE (depends on user_roles)
-- =============================================================================
CREATE TABLE IF NOT EXISTS public.users (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  auth_user_id UUID UNIQUE NOT NULL,
  email TEXT NOT NULL,
  display_name TEXT,
  role_id UUID NOT NULL REFERENCES public.user_roles(id),
  last_login_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- =============================================================================
-- TASK QUEUES (depends on users)
-- =============================================================================
CREATE TABLE IF NOT EXISTS public.task_queues (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  name TEXT NOT NULL UNIQUE,
  display_name TEXT NOT NULL,
  description TEXT,
  created_by UUID NOT NULL REFERENCES public.users(id),
  is_default BOOLEAN DEFAULT false,
  is_system_queue BOOLEAN DEFAULT false,
  max_concurrent_workflows INTEGER,
  max_concurrent_activities INTEGER,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- =============================================================================
-- PROJECTS (depends on users)
-- =============================================================================
CREATE TABLE IF NOT EXISTS public.projects (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  name TEXT NOT NULL,
  description TEXT,
  created_by UUID NOT NULL REFERENCES public.users(id),
  task_queue_name TEXT NOT NULL,
  is_active BOOLEAN DEFAULT true,
  is_archived BOOLEAN DEFAULT false,
  is_default BOOLEAN DEFAULT false,
  total_workflow_executions INTEGER,
  total_activity_executions INTEGER,
  avg_execution_duration_ms INTEGER,
  last_execution_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- =============================================================================
-- AGENT PROMPTS (depends on users, component_visibility)
-- =============================================================================
CREATE TABLE IF NOT EXISTS public.agent_prompts (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  name TEXT NOT NULL,
  display_name TEXT NOT NULL,
  description TEXT,
  prompt_content TEXT NOT NULL,
  prompt_variables JSONB,
  version TEXT NOT NULL,
  created_by UUID NOT NULL REFERENCES public.users(id),
  visibility_id UUID NOT NULL REFERENCES public.component_visibility(id),
  capabilities TEXT[],
  tags TEXT[],
  recommended_models JSONB,
  deprecated BOOLEAN DEFAULT false,
  deprecated_message TEXT,
  migrate_to_prompt_id UUID REFERENCES public.agent_prompts(id),
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- =============================================================================
-- COMPONENTS (depends on users, component_types, component_visibility, agent_prompts)
-- =============================================================================
CREATE TABLE IF NOT EXISTS public.components (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  name TEXT NOT NULL,
  display_name TEXT NOT NULL,
  description TEXT,
  component_type_id UUID NOT NULL REFERENCES public.component_types(id),
  version TEXT NOT NULL,
  created_by UUID NOT NULL REFERENCES public.users(id),
  visibility_id UUID NOT NULL REFERENCES public.component_visibility(id),
  input_schema JSONB,
  output_schema JSONB,
  config_schema JSONB,
  implementation_path TEXT,
  npm_package TEXT,
  implementation_language TEXT,
  implementation_code TEXT,
  agent_prompt_id UUID REFERENCES public.agent_prompts(id),
  model_provider TEXT,
  model_name TEXT,
  capabilities TEXT[],
  tags TEXT[],
  is_active BOOLEAN DEFAULT true,
  deprecated BOOLEAN DEFAULT false,
  deprecated_message TEXT,
  deprecated_since TEXT,
  migrate_to_component_id UUID REFERENCES public.components(id),
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- =============================================================================
-- WORKFLOWS (depends on users, projects, task_queues, workflow_statuses, component_visibility)
-- =============================================================================
CREATE TABLE IF NOT EXISTS public.workflows (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  name TEXT NOT NULL,
  display_name TEXT NOT NULL,
  kebab_name TEXT,
  description TEXT,
  definition JSONB NOT NULL DEFAULT '{}',
  version TEXT DEFAULT '1.0.0',
  created_by UUID NOT NULL REFERENCES public.users(id),
  project_id UUID NOT NULL REFERENCES public.projects(id) ON DELETE CASCADE,
  task_queue_id UUID NOT NULL REFERENCES public.task_queues(id),
  status_id UUID NOT NULL REFERENCES public.workflow_statuses(id),
  visibility_id UUID NOT NULL REFERENCES public.component_visibility(id),
  parent_workflow_id UUID REFERENCES public.workflows(id),
  compiled_typescript TEXT,
  temporal_workflow_id TEXT,
  temporal_workflow_type TEXT,
  is_archived BOOLEAN DEFAULT false,
  is_scheduled BOOLEAN DEFAULT false,
  schedule_spec TEXT,
  next_run_at TIMESTAMPTZ,
  last_run_at TIMESTAMPTZ,
  run_count INTEGER,
  max_runs INTEGER,
  max_concurrent_executions INTEGER,
  execution_timeout_seconds INTEGER,
  start_immediately BOOLEAN,
  end_with_parent BOOLEAN,
  signal_to_parent_name TEXT,
  query_parent_name TEXT,
  deployed_at TIMESTAMPTZ,
  deployment_status TEXT,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- =============================================================================
-- ACTIVITIES (depends on users)
-- =============================================================================
CREATE TABLE IF NOT EXISTS public.activities (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  name TEXT NOT NULL,
  function_name TEXT NOT NULL,
  module_path TEXT NOT NULL,
  package_name TEXT NOT NULL,
  description TEXT,
  category TEXT,
  input_schema JSONB NOT NULL,
  output_schema JSONB,
  examples JSONB,
  tags TEXT[],
  created_by UUID NOT NULL REFERENCES public.users(id),
  is_active BOOLEAN DEFAULT true,
  usage_count INTEGER DEFAULT 0,
  last_used_at TIMESTAMPTZ,
  deprecated BOOLEAN DEFAULT false,
  deprecated_message TEXT,
  deprecated_since TEXT,
  migrate_to_activity_id UUID REFERENCES public.activities(id),
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- =============================================================================
-- WORKFLOW NODES (depends on workflows, components)
-- =============================================================================
CREATE TABLE IF NOT EXISTS public.workflow_nodes (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  workflow_id UUID NOT NULL REFERENCES public.workflows(id) ON DELETE CASCADE,
  node_id TEXT NOT NULL,
  node_type TEXT NOT NULL,
  position JSONB NOT NULL,
  config JSONB DEFAULT '{}',
  component_id UUID REFERENCES public.components(id),
  signal_to_parent TEXT,
  query_parent TEXT,
  work_queue_target TEXT,
  block_until_queue TEXT,
  block_until_work_items JSONB,
  created_at TIMESTAMPTZ DEFAULT NOW()
);

-- =============================================================================
-- WORKFLOW EDGES (depends on workflows)
-- =============================================================================
CREATE TABLE IF NOT EXISTS public.workflow_edges (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  workflow_id UUID NOT NULL REFERENCES public.workflows(id) ON DELETE CASCADE,
  edge_id TEXT NOT NULL,
  source_node_id TEXT NOT NULL,
  target_node_id TEXT NOT NULL,
  label TEXT,
  config JSONB,
  created_at TIMESTAMPTZ DEFAULT NOW()
);

-- =============================================================================
-- WORKFLOW EXECUTIONS (depends on workflows, users)
-- =============================================================================
CREATE TABLE IF NOT EXISTS public.workflow_executions (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  workflow_id UUID NOT NULL REFERENCES public.workflows(id) ON DELETE CASCADE,
  created_by UUID REFERENCES public.users(id),
  status TEXT NOT NULL,
  temporal_workflow_id TEXT,
  temporal_run_id TEXT,
  input JSONB,
  output JSONB,
  error_message TEXT,
  activities_executed INTEGER,
  duration_ms INTEGER,
  started_at TIMESTAMPTZ DEFAULT NOW(),
  completed_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- =============================================================================
-- WORKFLOW COMPILED CODE (depends on workflows, users)
-- =============================================================================
CREATE TABLE IF NOT EXISTS public.workflow_compiled_code (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  workflow_id UUID NOT NULL REFERENCES public.workflows(id) ON DELETE CASCADE,
  compiled_by UUID REFERENCES public.users(id),
  version TEXT NOT NULL,
  workflow_code TEXT NOT NULL,
  activities_code TEXT NOT NULL,
  worker_code TEXT NOT NULL,
  package_json TEXT NOT NULL,
  tsconfig_json TEXT NOT NULL,
  is_active BOOLEAN DEFAULT true,
  execution_count INTEGER,
  error_count INTEGER,
  avg_execution_duration_ms INTEGER,
  last_executed_at TIMESTAMPTZ,
  compiled_at TIMESTAMPTZ DEFAULT NOW()
);

-- =============================================================================
-- WORKFLOW SIGNALS (depends on workflows, users)
-- =============================================================================
CREATE TABLE IF NOT EXISTS public.workflow_work_queues (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  workflow_id UUID NOT NULL REFERENCES public.workflows(id) ON DELETE CASCADE,
  created_by UUID NOT NULL REFERENCES public.users(id),
  queue_name TEXT NOT NULL,
  signal_name TEXT NOT NULL,
  query_name TEXT NOT NULL,
  description TEXT,
  work_item_schema JSONB,
  priority TEXT DEFAULT 'normal',
  max_size INTEGER,
  deduplicate BOOLEAN DEFAULT false,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS public.workflow_signals (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  workflow_id UUID NOT NULL REFERENCES public.workflows(id) ON DELETE CASCADE,
  created_by UUID NOT NULL REFERENCES public.users(id),
  signal_name TEXT NOT NULL,
  description TEXT,
  parameters JSONB,
  work_queue_id UUID REFERENCES public.workflow_work_queues(id),
  auto_generated BOOLEAN DEFAULT false,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS public.workflow_queries (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  workflow_id UUID NOT NULL REFERENCES public.workflows(id) ON DELETE CASCADE,
  created_by UUID NOT NULL REFERENCES public.users(id),
  query_name TEXT NOT NULL,
  description TEXT,
  return_type JSONB,
  work_queue_id UUID REFERENCES public.workflow_work_queues(id),
  auto_generated BOOLEAN DEFAULT false,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- =============================================================================
-- WORKFLOW STATE VARIABLES
-- =============================================================================
CREATE TABLE IF NOT EXISTS public.workflow_state_variables (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  workflow_id UUID NOT NULL REFERENCES public.workflows(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  type TEXT NOT NULL,
  storage_type TEXT NOT NULL,
  schema JSONB,
  storage_config JSONB,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS public.project_state_variables (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  project_id UUID NOT NULL REFERENCES public.projects(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  type TEXT NOT NULL,
  storage_type TEXT NOT NULL,
  schema JSONB,
  storage_config JSONB,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- =============================================================================
-- WORKFLOW WORKERS (depends on projects)
-- =============================================================================
CREATE TABLE IF NOT EXISTS public.workflow_workers (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  project_id UUID NOT NULL REFERENCES public.projects(id) ON DELETE CASCADE,
  worker_id TEXT NOT NULL,
  task_queue_name TEXT NOT NULL,
  status TEXT NOT NULL,
  host TEXT,
  port INTEGER,
  process_id TEXT,
  metadata JSONB,
  started_at TIMESTAMPTZ,
  stopped_at TIMESTAMPTZ,
  last_heartbeat TIMESTAMPTZ,
  total_tasks_completed INTEGER,
  total_tasks_failed INTEGER,
  avg_task_duration_ms INTEGER,
  memory_usage_mb INTEGER,
  cpu_usage_percent INTEGER,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- =============================================================================
-- CONNECTORS (depends on projects, users)
-- =============================================================================
CREATE TABLE IF NOT EXISTS public.connectors (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  project_id UUID NOT NULL REFERENCES public.projects(id) ON DELETE CASCADE,
  created_by UUID NOT NULL REFERENCES public.users(id),
  name TEXT NOT NULL,
  display_name TEXT NOT NULL,
  description TEXT,
  connector_type TEXT NOT NULL,
  config_schema JSONB NOT NULL,
  config_data JSONB NOT NULL,
  credentials_encrypted TEXT,
  oauth_config JSONB,
  classifications JSONB,
  is_active BOOLEAN DEFAULT true,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS public.connector_classifications (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  connector_id UUID NOT NULL REFERENCES public.connectors(id) ON DELETE CASCADE,
  classification TEXT NOT NULL,
  created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS public.project_connectors (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  source_project_id UUID NOT NULL REFERENCES public.projects(id) ON DELETE CASCADE,
  target_project_id UUID NOT NULL REFERENCES public.projects(id),
  target_service_id UUID NOT NULL REFERENCES public.workflows(id),
  target_interface_id UUID,
  created_by UUID NOT NULL REFERENCES public.users(id),
  name TEXT NOT NULL,
  display_name TEXT NOT NULL,
  description TEXT,
  nexus_endpoint_name TEXT NOT NULL,
  visibility TEXT,
  auth_config JSONB,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- =============================================================================
-- AGENT SESSIONS (depends on users, agent_prompts)
-- =============================================================================
CREATE TABLE IF NOT EXISTS public.agent_builder_sessions (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  user_id UUID NOT NULL REFERENCES public.users(id),
  status TEXT DEFAULT 'active',
  conversation_messages JSONB,
  message_count INTEGER,
  resulting_prompt_id UUID REFERENCES public.agent_prompts(id),
  started_at TIMESTAMPTZ DEFAULT NOW(),
  completed_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS public.agent_test_sessions (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  user_id UUID NOT NULL REFERENCES public.users(id),
  agent_prompt_id UUID NOT NULL REFERENCES public.agent_prompts(id),
  temporal_workflow_id TEXT NOT NULL,
  temporal_run_id TEXT NOT NULL,
  status TEXT DEFAULT 'active',
  conversation_history JSONB,
  message_count INTEGER,
  started_at TIMESTAMPTZ DEFAULT NOW(),
  completed_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- =============================================================================
-- ACTIVITY STATISTICS (depends on projects)
-- =============================================================================
CREATE TABLE IF NOT EXISTS public.activity_statistics (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  project_id UUID NOT NULL REFERENCES public.projects(id) ON DELETE CASCADE,
  activity_name TEXT NOT NULL,
  execution_count INTEGER,
  success_count INTEGER,
  failure_count INTEGER,
  avg_duration_ms INTEGER,
  p95_duration_ms INTEGER,
  p99_duration_ms INTEGER,
  last_executed_at TIMESTAMPTZ,
  requires_dedicated_worker BOOLEAN,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- =============================================================================
-- STATE VARIABLE METRICS
-- =============================================================================
CREATE TABLE IF NOT EXISTS public.state_variable_metrics (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  variable_id UUID NOT NULL,
  scope TEXT NOT NULL,
  size_bytes INTEGER,
  access_count INTEGER,
  last_accessed TIMESTAMPTZ,
  created_at TIMESTAMPTZ DEFAULT NOW()
);

-- =============================================================================
-- METRICS TABLES (from migrations)
-- =============================================================================

-- Component Metrics
CREATE TABLE IF NOT EXISTS public.component_metrics (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  project_id UUID NOT NULL REFERENCES public.projects(id) ON DELETE CASCADE,
  workflow_id UUID NOT NULL REFERENCES public.workflows(id) ON DELETE CASCADE,
  workflow_execution_id UUID REFERENCES public.workflow_executions(id) ON DELETE SET NULL,
  component_type TEXT NOT NULL,
  component_name TEXT NOT NULL,
  component_id UUID,
  node_id TEXT,
  invocation_count INTEGER NOT NULL DEFAULT 1,
  duration_ms INTEGER,
  queue_time_ms INTEGER,
  status TEXT NOT NULL,
  is_retry BOOLEAN DEFAULT false,
  attempt_number INTEGER DEFAULT 1,
  memory_peak_mb INTEGER,
  cpu_time_ms INTEGER,
  io_bytes BIGINT,
  error_type TEXT,
  error_code TEXT,
  started_at TIMESTAMPTZ NOT NULL,
  completed_at TIMESTAMPTZ,
  recorded_at TIMESTAMPTZ DEFAULT NOW(),
  metadata JSONB DEFAULT '{}'
);

-- Component Usage Daily
CREATE TABLE IF NOT EXISTS public.component_usage_daily (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  date DATE NOT NULL,
  project_id UUID NOT NULL REFERENCES public.projects(id) ON DELETE CASCADE,
  component_type TEXT NOT NULL,
  component_name TEXT NOT NULL,
  total_invocations INTEGER NOT NULL DEFAULT 0,
  successful_invocations INTEGER NOT NULL DEFAULT 0,
  failed_invocations INTEGER NOT NULL DEFAULT 0,
  retried_invocations INTEGER NOT NULL DEFAULT 0,
  total_duration_ms BIGINT DEFAULT 0,
  avg_duration_ms INTEGER,
  p50_duration_ms INTEGER,
  p95_duration_ms INTEGER,
  p99_duration_ms INTEGER,
  max_duration_ms INTEGER,
  total_memory_mb BIGINT DEFAULT 0,
  total_cpu_time_ms BIGINT DEFAULT 0,
  updated_at TIMESTAMPTZ DEFAULT NOW(),
  CONSTRAINT component_usage_daily_unique UNIQUE(date, project_id, component_type, component_name)
);

-- Workflow Execution Metrics
CREATE TABLE IF NOT EXISTS public.workflow_execution_metrics (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  project_id UUID NOT NULL REFERENCES public.projects(id) ON DELETE CASCADE,
  workflow_id UUID NOT NULL REFERENCES public.workflows(id) ON DELETE CASCADE,
  workflow_execution_id UUID REFERENCES public.workflow_executions(id) ON DELETE SET NULL,
  workflow_name TEXT NOT NULL,
  workflow_version TEXT,
  task_queue_name TEXT,
  temporal_workflow_id TEXT,
  temporal_run_id TEXT,
  trigger_type TEXT NOT NULL DEFAULT 'manual',
  trigger_source TEXT,
  input_size_bytes INTEGER,
  output_size_bytes INTEGER,
  duration_ms INTEGER,
  queue_time_ms INTEGER,
  activity_count INTEGER DEFAULT 0,
  retry_count INTEGER DEFAULT 0,
  status TEXT NOT NULL DEFAULT 'running',
  error_type TEXT,
  error_message TEXT,
  total_memory_mb INTEGER,
  total_cpu_time_ms INTEGER,
  started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  completed_at TIMESTAMPTZ,
  recorded_at TIMESTAMPTZ DEFAULT NOW(),
  metadata JSONB DEFAULT '{}'
);

-- Workflow Usage Daily
CREATE TABLE IF NOT EXISTS public.workflow_usage_daily (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  date DATE NOT NULL,
  project_id UUID NOT NULL REFERENCES public.projects(id) ON DELETE CASCADE,
  workflow_id UUID NOT NULL REFERENCES public.workflows(id) ON DELETE CASCADE,
  workflow_name TEXT NOT NULL,
  total_executions INTEGER NOT NULL DEFAULT 0,
  successful_executions INTEGER NOT NULL DEFAULT 0,
  failed_executions INTEGER NOT NULL DEFAULT 0,
  cancelled_executions INTEGER NOT NULL DEFAULT 0,
  timeout_executions INTEGER NOT NULL DEFAULT 0,
  total_duration_ms BIGINT DEFAULT 0,
  avg_duration_ms INTEGER,
  p50_duration_ms INTEGER,
  p95_duration_ms INTEGER,
  p99_duration_ms INTEGER,
  max_duration_ms INTEGER,
  total_activities_executed INTEGER DEFAULT 0,
  avg_activities_per_execution INTEGER,
  total_input_bytes BIGINT DEFAULT 0,
  total_output_bytes BIGINT DEFAULT 0,
  manual_triggers INTEGER DEFAULT 0,
  schedule_triggers INTEGER DEFAULT 0,
  webhook_triggers INTEGER DEFAULT 0,
  api_triggers INTEGER DEFAULT 0,
  updated_at TIMESTAMPTZ DEFAULT NOW(),
  CONSTRAINT workflow_usage_daily_unique UNIQUE(date, project_id, workflow_id)
);

-- Resource Events
CREATE TABLE IF NOT EXISTS public.resource_events (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  project_id UUID NOT NULL REFERENCES public.projects(id) ON DELETE CASCADE,
  workflow_id UUID REFERENCES public.workflows(id) ON DELETE SET NULL,
  workflow_execution_id UUID REFERENCES public.workflow_executions(id) ON DELETE SET NULL,
  component_metric_id UUID REFERENCES public.component_metrics(id) ON DELETE SET NULL,
  resource_type TEXT NOT NULL,
  resource_subtype TEXT,
  resource_id UUID,
  resource_name TEXT NOT NULL,
  operation TEXT NOT NULL,
  direction TEXT,
  duration_ms INTEGER,
  latency_ms INTEGER,
  request_size_bytes INTEGER,
  response_size_bytes INTEGER,
  status TEXT NOT NULL DEFAULT 'success',
  error_type TEXT,
  error_code TEXT,
  model_name TEXT,
  prompt_tokens INTEGER,
  completion_tokens INTEGER,
  total_tokens INTEGER,
  target_project_id UUID,
  target_service TEXT,
  started_at TIMESTAMPTZ NOT NULL,
  completed_at TIMESTAMPTZ,
  recorded_at TIMESTAMPTZ DEFAULT NOW(),
  metadata JSONB DEFAULT '{}'
);

-- Resource Usage Daily
CREATE TABLE IF NOT EXISTS public.resource_usage_daily (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  date DATE NOT NULL,
  project_id UUID NOT NULL REFERENCES public.projects(id) ON DELETE CASCADE,
  resource_type TEXT NOT NULL,
  resource_name TEXT NOT NULL,
  total_invocations INTEGER NOT NULL DEFAULT 0,
  successful_invocations INTEGER NOT NULL DEFAULT 0,
  failed_invocations INTEGER NOT NULL DEFAULT 0,
  total_duration_ms BIGINT DEFAULT 0,
  avg_duration_ms INTEGER,
  avg_latency_ms INTEGER,
  total_request_bytes BIGINT DEFAULT 0,
  total_response_bytes BIGINT DEFAULT 0,
  total_prompt_tokens BIGINT DEFAULT 0,
  total_completion_tokens BIGINT DEFAULT 0,
  total_tokens BIGINT DEFAULT 0,
  updated_at TIMESTAMPTZ DEFAULT NOW(),
  CONSTRAINT resource_usage_daily_unique UNIQUE(date, project_id, resource_type, resource_name)
);

-- =============================================================================
-- COMPONENT CATEGORIES (hierarchical component organization)
-- =============================================================================
CREATE TABLE IF NOT EXISTS public.component_categories (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  name TEXT NOT NULL UNIQUE,
  display_name TEXT NOT NULL,
  description TEXT,
  icon TEXT,
  icon_provider TEXT DEFAULT 'lucide',
  color TEXT,
  parent_category_id UUID REFERENCES public.component_categories(id),
  sort_order INTEGER DEFAULT 0,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Component to Category mapping (many-to-many)
CREATE TABLE IF NOT EXISTS public.component_category_mapping (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  component_id UUID NOT NULL REFERENCES public.components(id) ON DELETE CASCADE,
  category_id UUID NOT NULL REFERENCES public.component_categories(id) ON DELETE CASCADE,
  is_primary BOOLEAN DEFAULT false,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  UNIQUE(component_id, category_id)
);

-- Component Keywords (for search)
CREATE TABLE IF NOT EXISTS public.component_keywords (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  component_id UUID NOT NULL REFERENCES public.components(id) ON DELETE CASCADE,
  keyword TEXT NOT NULL,
  relevance_score FLOAT DEFAULT 1.0,
  created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Component Use Cases
CREATE TABLE IF NOT EXISTS public.component_use_cases (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  component_id UUID NOT NULL REFERENCES public.components(id) ON DELETE CASCADE,
  use_case TEXT NOT NULL,
  description TEXT,
  created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Grant permissions for new tables
GRANT SELECT, INSERT, UPDATE, DELETE ON public.component_categories TO authenticated;
GRANT ALL ON public.component_categories TO service_role;
GRANT SELECT ON public.component_categories TO anon;

GRANT SELECT, INSERT, UPDATE, DELETE ON public.component_category_mapping TO authenticated;
GRANT ALL ON public.component_category_mapping TO service_role;
GRANT SELECT ON public.component_category_mapping TO anon;

GRANT SELECT, INSERT, UPDATE, DELETE ON public.component_keywords TO authenticated;
GRANT ALL ON public.component_keywords TO service_role;
GRANT SELECT ON public.component_keywords TO anon;

GRANT SELECT, INSERT, UPDATE, DELETE ON public.component_use_cases TO authenticated;
GRANT ALL ON public.component_use_cases TO service_role;
GRANT SELECT ON public.component_use_cases TO anon;

-- =============================================================================
-- SERVICE INTERFACES (depends on workflows)
-- =============================================================================
CREATE TABLE IF NOT EXISTS public.service_interfaces (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  workflow_id UUID NOT NULL REFERENCES public.workflows(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  display_name TEXT NOT NULL,
  description TEXT,
  interface_type TEXT NOT NULL, -- 'signal', 'query', 'update', 'mcp', 'graphql'
  callable_name TEXT,
  input_schema JSONB,
  output_schema JSONB,
  is_public BOOLEAN DEFAULT false,
  mcp_config JSONB, -- MCP-specific configuration
  graphql_schema TEXT, -- GraphQL schema definition
  endpoint_path TEXT,
  http_method TEXT,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW(),
  UNIQUE(workflow_id, name)
);

-- Public Interfaces (exposed via Kong)
CREATE TABLE IF NOT EXISTS public.public_interfaces (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  service_interface_id UUID NOT NULL REFERENCES public.service_interfaces(id) ON DELETE CASCADE,
  route_path TEXT NOT NULL,
  kong_route_id TEXT,
  kong_service_id TEXT,
  is_active BOOLEAN DEFAULT true,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Service Interface Endpoints
CREATE TABLE IF NOT EXISTS public.service_interface_endpoints (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  service_interface_id UUID NOT NULL REFERENCES public.service_interfaces(id) ON DELETE CASCADE,
  endpoint_path TEXT NOT NULL,
  http_method TEXT NOT NULL,
  description TEXT,
  input_schema JSONB,
  output_schema JSONB,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Grant permissions for service interface tables
GRANT SELECT, INSERT, UPDATE, DELETE ON public.service_interfaces TO authenticated;
GRANT ALL ON public.service_interfaces TO service_role;
GRANT SELECT ON public.service_interfaces TO anon;

GRANT SELECT, INSERT, UPDATE, DELETE ON public.public_interfaces TO authenticated;
GRANT ALL ON public.public_interfaces TO service_role;
GRANT SELECT ON public.public_interfaces TO anon;

GRANT SELECT, INSERT, UPDATE, DELETE ON public.service_interface_endpoints TO authenticated;
GRANT ALL ON public.service_interface_endpoints TO service_role;
GRANT SELECT ON public.service_interface_endpoints TO anon;

-- =============================================================================
-- INDEXES
-- =============================================================================
CREATE INDEX IF NOT EXISTS idx_users_auth_user_id ON public.users(auth_user_id);
CREATE INDEX IF NOT EXISTS idx_users_email ON public.users(email);
CREATE INDEX IF NOT EXISTS idx_workflows_project_id ON public.workflows(project_id);
CREATE INDEX IF NOT EXISTS idx_workflows_created_by ON public.workflows(created_by);
CREATE INDEX IF NOT EXISTS idx_workflow_nodes_workflow_id ON public.workflow_nodes(workflow_id);
CREATE INDEX IF NOT EXISTS idx_workflow_edges_workflow_id ON public.workflow_edges(workflow_id);
CREATE INDEX IF NOT EXISTS idx_workflow_executions_workflow_id ON public.workflow_executions(workflow_id);
CREATE INDEX IF NOT EXISTS idx_projects_created_by ON public.projects(created_by);
CREATE INDEX IF NOT EXISTS idx_components_created_by ON public.components(created_by);
CREATE INDEX IF NOT EXISTS idx_agent_prompts_created_by ON public.agent_prompts(created_by);

-- =============================================================================
-- GRANT PERMISSIONS
-- =============================================================================
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO authenticated;
GRANT ALL ON ALL TABLES IN SCHEMA public TO service_role;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO anon;

-- =============================================================================
-- SEED DATA: Default Lookup Values
-- =============================================================================

-- Default User Roles
INSERT INTO public.user_roles (id, name, description, permissions) VALUES
  ('00000000-0000-0000-0000-000000000001', 'admin', 'Administrator with full access', '{"all": true}'),
  ('00000000-0000-0000-0000-000000000002', 'developer', 'Developer with standard access', '{"read": true, "write": true, "execute": true}'),
  ('00000000-0000-0000-0000-000000000003', 'viewer', 'Read-only access', '{"read": true}')
ON CONFLICT (id) DO NOTHING;

-- Default Workflow Statuses
INSERT INTO public.workflow_statuses (id, name, description, color) VALUES
  ('00000000-0000-0000-0000-000000000001', 'draft', 'Work in progress', '#6B7280'),
  ('00000000-0000-0000-0000-000000000002', 'active', 'Ready for execution', '#10B981'),
  ('00000000-0000-0000-0000-000000000003', 'archived', 'No longer active', '#EF4444'),
  ('00000000-0000-0000-0000-000000000004', 'deploying', 'Currently being deployed', '#F59E0B'),
  ('00000000-0000-0000-0000-000000000005', 'error', 'Has deployment errors', '#DC2626')
ON CONFLICT (id) DO NOTHING;

-- Default Component Types
INSERT INTO public.component_types (id, name, description, icon) VALUES
  ('00000000-0000-0000-0000-000000000001', 'activity', 'Temporal activity component', 'Activity'),
  ('00000000-0000-0000-0000-000000000002', 'agent', 'AI agent component', 'Bot'),
  ('00000000-0000-0000-0000-000000000003', 'transform', 'Data transformation component', 'Transform'),
  ('00000000-0000-0000-0000-000000000004', 'connector', 'External service connector', 'Plug'),
  ('00000000-0000-0000-0000-000000000005', 'trigger', 'Workflow trigger component', 'Zap')
ON CONFLICT (id) DO NOTHING;

-- Default Component Visibility
INSERT INTO public.component_visibility (id, name, description) VALUES
  ('00000000-0000-0000-0000-000000000001', 'private', 'Only visible to creator'),
  ('00000000-0000-0000-0000-000000000002', 'team', 'Visible to team members'),
  ('00000000-0000-0000-0000-000000000003', 'public', 'Visible to everyone')
ON CONFLICT (id) DO NOTHING;

-- Default Activity Categories
INSERT INTO public.activity_categories (id, name, description, icon) VALUES
  ('00000000-0000-0000-0000-000000000001', 'communication', 'Email, SMS, notifications', 'Mail'),
  ('00000000-0000-0000-0000-000000000002', 'data', 'Database operations', 'Database'),
  ('00000000-0000-0000-0000-000000000003', 'integration', 'Third-party integrations', 'Plug'),
  ('00000000-0000-0000-0000-000000000004', 'utility', 'General utilities', 'Tool'),
  ('00000000-0000-0000-0000-000000000005', 'ai', 'AI and ML operations', 'Bot')
ON CONFLICT (id) DO NOTHING;

-- Default Component Categories
INSERT INTO public.component_categories (id, name, display_name, description, icon, color, sort_order) VALUES
  ('c0000001-0000-0000-0000-000000000001', 'communication', 'Communication', 'Email, SMS, messaging components', 'Mail', '#3B82F6', 1),
  ('c0000002-0000-0000-0000-000000000002', 'data', 'Data & Storage', 'Database and storage operations', 'Database', '#10B981', 2),
  ('c0000003-0000-0000-0000-000000000003', 'integration', 'Integrations', 'Third-party service integrations', 'Plug', '#8B5CF6', 3),
  ('c0000004-0000-0000-0000-000000000004', 'utility', 'Utilities', 'General utility components', 'Wrench', '#F59E0B', 4),
  ('c0000005-0000-0000-0000-000000000005', 'ai', 'AI & ML', 'AI and machine learning operations', 'Bot', '#EC4899', 5),
  ('c0000006-0000-0000-0000-000000000006', 'control-flow', 'Control Flow', 'Workflow control and branching', 'GitBranch', '#6366F1', 6)
ON CONFLICT (id) DO NOTHING;

-- =============================================================================
-- FUNCTION: Setup Test User with Full Data
-- =============================================================================
CREATE OR REPLACE FUNCTION public.setup_test_user(test_auth_user_id UUID, test_email TEXT)
RETURNS UUID AS $$
DECLARE
  test_user_id UUID;
  test_project_id UUID;
  test_task_queue_id UUID;
BEGIN
  -- Create or get user (with developer role)
  INSERT INTO public.users (auth_user_id, email, display_name, role_id)
  VALUES (test_auth_user_id, test_email, 'Test User', '00000000-0000-0000-0000-000000000002')
  ON CONFLICT (auth_user_id) DO UPDATE SET email = EXCLUDED.email
  RETURNING id INTO test_user_id;

  -- Create default task queue
  INSERT INTO public.task_queues (id, name, display_name, description, created_by, is_default)
  VALUES (
    '10000000-0000-0000-0000-000000000001',
    'default-task-queue',
    'Default Task Queue',
    'Default task queue for test workflows',
    test_user_id,
    true
  )
  ON CONFLICT (name) DO UPDATE SET display_name = EXCLUDED.display_name
  RETURNING id INTO test_task_queue_id;

  -- Create default project
  INSERT INTO public.projects (id, name, description, created_by, task_queue_name, is_default)
  VALUES (
    '20000000-0000-0000-0000-000000000001',
    'Test Project',
    'Default test project',
    test_user_id,
    'default-task-queue',
    true
  )
  ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name
  RETURNING id INTO test_project_id;

  -- Create sample workflows
  INSERT INTO public.workflows (id, name, display_name, kebab_name, description, definition, created_by, project_id, task_queue_id, status_id, visibility_id)
  VALUES
    (
      '30000001-0000-0000-0000-000000000001',
      'hello-world-demo',
      'Hello World Demo',
      'hello-world-demo',
      'A simple demonstration workflow',
      '{"nodes": [], "edges": []}',
      test_user_id,
      test_project_id,
      test_task_queue_id,
      '00000000-0000-0000-0000-000000000002',
      '00000000-0000-0000-0000-000000000001'
    ),
    (
      '30000002-0000-0000-0000-000000000002',
      'email-notification',
      'Email Notification Workflow',
      'email-notification-workflow',
      'Sends email notifications',
      '{"nodes": [], "edges": []}',
      test_user_id,
      test_project_id,
      test_task_queue_id,
      '00000000-0000-0000-0000-000000000001',
      '00000000-0000-0000-0000-000000000001'
    )
  ON CONFLICT (id) DO NOTHING;

  -- Create sample agent prompts
  INSERT INTO public.agent_prompts (id, name, display_name, description, prompt_content, version, created_by, visibility_id)
  VALUES
    (
      '40000001-0000-0000-0000-000000000001',
      'code-review-assistant',
      'Code Review Assistant',
      'Helps review code',
      'You are a code review assistant.',
      '1.0.0',
      test_user_id,
      '00000000-0000-0000-0000-000000000001'
    )
  ON CONFLICT (id) DO NOTHING;

  -- Create sample components
  INSERT INTO public.components (id, name, display_name, description, component_type_id, version, created_by, visibility_id)
  VALUES
    (
      '50000001-0000-0000-0000-000000000001',
      'http-request',
      'HTTP Request',
      'Make HTTP requests to external APIs',
      '00000000-0000-0000-0000-000000000001',
      '1.0.0',
      test_user_id,
      '00000000-0000-0000-0000-000000000003'
    ),
    (
      '50000002-0000-0000-0000-000000000002',
      'send-email',
      'Send Email',
      'Send emails via SMTP',
      '00000000-0000-0000-0000-000000000001',
      '1.0.0',
      test_user_id,
      '00000000-0000-0000-0000-000000000003'
    )
  ON CONFLICT (id) DO NOTHING;

  RETURN test_user_id;
END;
$$ LANGUAGE plpgsql;

-- Grant execute permission
GRANT EXECUTE ON FUNCTION public.setup_test_user(UUID, TEXT) TO service_role, authenticated;
