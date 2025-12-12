-- Workflow and Resource Events Schema
-- Workflow Observability Platform - Extended Analytics
--
-- This migration adds:
-- 1. workflow_executions_metrics - Track every workflow run (for future service billing)
-- 2. resource_events - Track interface/variable/agent/connector usage
--
-- Run this migration after 001_component_metrics.sql

-- ============================================================================
-- Workflow Executions Metrics Table
-- Records every workflow invocation for performance analytics
-- (Named "metrics" to align with observability framing, but captures billing-ready data)
-- ============================================================================
CREATE TABLE IF NOT EXISTS workflow_execution_metrics (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

  -- Context
  project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  workflow_id UUID NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
  workflow_execution_id UUID REFERENCES workflow_executions(id) ON DELETE SET NULL,

  -- Workflow identification
  workflow_name VARCHAR(255) NOT NULL,
  workflow_version VARCHAR(50),
  task_queue_name VARCHAR(255),
  temporal_workflow_id VARCHAR(255),
  temporal_run_id VARCHAR(255),

  -- Execution context
  trigger_type VARCHAR(50) NOT NULL DEFAULT 'manual',  -- 'manual', 'schedule', 'webhook', 'api', 'signal'
  trigger_source VARCHAR(255),                         -- e.g., 'api-endpoint:/v1/workflows/xyz', 'schedule:daily'

  -- Input/Output sizing (for future compute billing)
  input_size_bytes INTEGER,
  output_size_bytes INTEGER,

  -- Execution metrics
  duration_ms INTEGER,
  queue_time_ms INTEGER,
  activity_count INTEGER DEFAULT 0,
  retry_count INTEGER DEFAULT 0,

  -- Status
  status VARCHAR(20) NOT NULL DEFAULT 'running',  -- 'running', 'completed', 'failed', 'cancelled', 'timeout'
  error_type VARCHAR(100),
  error_message TEXT,

  -- Resource metrics (reserved for future)
  total_memory_mb INTEGER,
  total_cpu_time_ms INTEGER,

  -- Timestamps
  started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  completed_at TIMESTAMPTZ,
  recorded_at TIMESTAMPTZ DEFAULT NOW(),

  -- Extensibility
  metadata JSONB DEFAULT '{}'
);

-- Performance indexes
CREATE INDEX IF NOT EXISTS idx_workflow_exec_metrics_project_time
  ON workflow_execution_metrics(project_id, started_at DESC);

CREATE INDEX IF NOT EXISTS idx_workflow_exec_metrics_workflow
  ON workflow_execution_metrics(workflow_id, started_at DESC);

CREATE INDEX IF NOT EXISTS idx_workflow_exec_metrics_status
  ON workflow_execution_metrics(project_id, status, started_at DESC);

CREATE INDEX IF NOT EXISTS idx_workflow_exec_metrics_temporal
  ON workflow_execution_metrics(temporal_workflow_id);

-- ============================================================================
-- Workflow Usage Daily Aggregation
-- Daily rollups for workflow execution analytics
-- ============================================================================
CREATE TABLE IF NOT EXISTS workflow_usage_daily (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  date DATE NOT NULL,
  project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  workflow_id UUID NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
  workflow_name VARCHAR(255) NOT NULL,

  -- Execution counts
  total_executions INTEGER NOT NULL DEFAULT 0,
  successful_executions INTEGER NOT NULL DEFAULT 0,
  failed_executions INTEGER NOT NULL DEFAULT 0,
  cancelled_executions INTEGER NOT NULL DEFAULT 0,
  timeout_executions INTEGER NOT NULL DEFAULT 0,

  -- Duration metrics
  total_duration_ms BIGINT DEFAULT 0,
  avg_duration_ms INTEGER,
  p50_duration_ms INTEGER,
  p95_duration_ms INTEGER,
  p99_duration_ms INTEGER,
  max_duration_ms INTEGER,

  -- Activity metrics
  total_activities_executed INTEGER DEFAULT 0,
  avg_activities_per_execution INTEGER,

  -- Data volume (for future billing)
  total_input_bytes BIGINT DEFAULT 0,
  total_output_bytes BIGINT DEFAULT 0,

  -- Trigger breakdown
  manual_triggers INTEGER DEFAULT 0,
  schedule_triggers INTEGER DEFAULT 0,
  webhook_triggers INTEGER DEFAULT 0,
  api_triggers INTEGER DEFAULT 0,

  -- Metadata
  updated_at TIMESTAMPTZ DEFAULT NOW(),

  CONSTRAINT workflow_usage_daily_unique
    UNIQUE(date, project_id, workflow_id)
);

-- Index for dashboard queries
CREATE INDEX IF NOT EXISTS idx_workflow_usage_daily_project_date
  ON workflow_usage_daily(project_id, date DESC);

-- ============================================================================
-- Resource Events Table
-- Tracks usage of interfaces, variables, agents, and connectors
-- These events are always paired with component executions but tracked separately
-- ============================================================================
CREATE TABLE IF NOT EXISTS resource_events (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

  -- Context
  project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  workflow_id UUID REFERENCES workflows(id) ON DELETE SET NULL,
  workflow_execution_id UUID REFERENCES workflow_executions(id) ON DELETE SET NULL,
  component_metric_id UUID REFERENCES component_metrics(id) ON DELETE SET NULL,

  -- Resource identification
  resource_type VARCHAR(50) NOT NULL,  -- 'interface', 'variable', 'agent', 'connector'
  resource_subtype VARCHAR(50),        -- e.g., 'service_interface', 'public_interface', 'state_variable', 'project_connector'
  resource_id UUID,                    -- Reference to the specific resource if available
  resource_name VARCHAR(255) NOT NULL,

  -- Usage details
  operation VARCHAR(50) NOT NULL,      -- 'invoke', 'read', 'write', 'call', 'connect'
  direction VARCHAR(20),               -- 'inbound', 'outbound' for interfaces

  -- Performance metrics
  duration_ms INTEGER,
  latency_ms INTEGER,                  -- External call latency for interfaces/connectors

  -- Data volume
  request_size_bytes INTEGER,
  response_size_bytes INTEGER,

  -- Status
  status VARCHAR(20) NOT NULL DEFAULT 'success',  -- 'success', 'failure', 'timeout'
  error_type VARCHAR(100),
  error_code VARCHAR(50),

  -- For agents specifically
  model_name VARCHAR(100),
  prompt_tokens INTEGER,
  completion_tokens INTEGER,
  total_tokens INTEGER,

  -- For connectors specifically
  target_project_id UUID,
  target_service VARCHAR(255),

  -- Timestamps
  started_at TIMESTAMPTZ NOT NULL,
  completed_at TIMESTAMPTZ,
  recorded_at TIMESTAMPTZ DEFAULT NOW(),

  -- Extensibility
  metadata JSONB DEFAULT '{}'
);

-- Performance indexes
CREATE INDEX IF NOT EXISTS idx_resource_events_project_type
  ON resource_events(project_id, resource_type, started_at DESC);

CREATE INDEX IF NOT EXISTS idx_resource_events_workflow
  ON resource_events(workflow_execution_id, started_at DESC);

CREATE INDEX IF NOT EXISTS idx_resource_events_component
  ON resource_events(component_metric_id);

CREATE INDEX IF NOT EXISTS idx_resource_events_resource
  ON resource_events(resource_type, resource_name, started_at DESC);

-- ============================================================================
-- Resource Usage Daily Aggregation
-- Daily rollups for resource usage analytics
-- ============================================================================
CREATE TABLE IF NOT EXISTS resource_usage_daily (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  date DATE NOT NULL,
  project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  resource_type VARCHAR(50) NOT NULL,
  resource_name VARCHAR(255) NOT NULL,

  -- Usage counts
  total_invocations INTEGER NOT NULL DEFAULT 0,
  successful_invocations INTEGER NOT NULL DEFAULT 0,
  failed_invocations INTEGER NOT NULL DEFAULT 0,

  -- Duration metrics
  total_duration_ms BIGINT DEFAULT 0,
  avg_duration_ms INTEGER,
  avg_latency_ms INTEGER,

  -- Data volume
  total_request_bytes BIGINT DEFAULT 0,
  total_response_bytes BIGINT DEFAULT 0,

  -- Agent-specific aggregates
  total_prompt_tokens BIGINT DEFAULT 0,
  total_completion_tokens BIGINT DEFAULT 0,
  total_tokens BIGINT DEFAULT 0,

  -- Metadata
  updated_at TIMESTAMPTZ DEFAULT NOW(),

  CONSTRAINT resource_usage_daily_unique
    UNIQUE(date, project_id, resource_type, resource_name)
);

-- Index for dashboard queries
CREATE INDEX IF NOT EXISTS idx_resource_usage_daily_project_date
  ON resource_usage_daily(project_id, date DESC);

CREATE INDEX IF NOT EXISTS idx_resource_usage_daily_type
  ON resource_usage_daily(resource_type, date DESC);

-- ============================================================================
-- Database Functions
-- ============================================================================

-- Record workflow execution metric
CREATE OR REPLACE FUNCTION record_workflow_execution_metric(
  p_project_id UUID,
  p_workflow_id UUID,
  p_execution_id UUID,
  p_workflow_name VARCHAR(255),
  p_workflow_version VARCHAR(50),
  p_task_queue_name VARCHAR(255),
  p_temporal_workflow_id VARCHAR(255),
  p_temporal_run_id VARCHAR(255),
  p_trigger_type VARCHAR(50),
  p_trigger_source VARCHAR(255),
  p_input_size_bytes INTEGER,
  p_output_size_bytes INTEGER,
  p_duration_ms INTEGER,
  p_activity_count INTEGER,
  p_status VARCHAR(20),
  p_error_type VARCHAR(100) DEFAULT NULL,
  p_started_at TIMESTAMPTZ DEFAULT NOW(),
  p_completed_at TIMESTAMPTZ DEFAULT NULL,
  p_metadata JSONB DEFAULT '{}'
) RETURNS UUID AS $$
DECLARE
  v_metric_id UUID;
  v_date DATE;
BEGIN
  v_date := DATE(p_started_at);

  -- Insert execution metric
  INSERT INTO workflow_execution_metrics (
    project_id, workflow_id, workflow_execution_id,
    workflow_name, workflow_version, task_queue_name,
    temporal_workflow_id, temporal_run_id,
    trigger_type, trigger_source,
    input_size_bytes, output_size_bytes,
    duration_ms, activity_count, status, error_type,
    started_at, completed_at, metadata
  ) VALUES (
    p_project_id, p_workflow_id, p_execution_id,
    p_workflow_name, p_workflow_version, p_task_queue_name,
    p_temporal_workflow_id, p_temporal_run_id,
    p_trigger_type, p_trigger_source,
    p_input_size_bytes, p_output_size_bytes,
    p_duration_ms, p_activity_count, p_status, p_error_type,
    p_started_at, p_completed_at, p_metadata
  ) RETURNING id INTO v_metric_id;

  -- Update daily aggregates
  INSERT INTO workflow_usage_daily (
    date, project_id, workflow_id, workflow_name,
    total_executions, successful_executions, failed_executions,
    cancelled_executions, timeout_executions,
    total_duration_ms, avg_duration_ms, max_duration_ms,
    total_activities_executed,
    total_input_bytes, total_output_bytes,
    manual_triggers, schedule_triggers, webhook_triggers, api_triggers,
    updated_at
  ) VALUES (
    v_date, p_project_id, p_workflow_id, p_workflow_name,
    1,
    CASE WHEN p_status = 'completed' THEN 1 ELSE 0 END,
    CASE WHEN p_status = 'failed' THEN 1 ELSE 0 END,
    CASE WHEN p_status = 'cancelled' THEN 1 ELSE 0 END,
    CASE WHEN p_status = 'timeout' THEN 1 ELSE 0 END,
    COALESCE(p_duration_ms, 0),
    p_duration_ms,
    p_duration_ms,
    COALESCE(p_activity_count, 0),
    COALESCE(p_input_size_bytes, 0),
    COALESCE(p_output_size_bytes, 0),
    CASE WHEN p_trigger_type = 'manual' THEN 1 ELSE 0 END,
    CASE WHEN p_trigger_type = 'schedule' THEN 1 ELSE 0 END,
    CASE WHEN p_trigger_type = 'webhook' THEN 1 ELSE 0 END,
    CASE WHEN p_trigger_type = 'api' THEN 1 ELSE 0 END,
    NOW()
  )
  ON CONFLICT (date, project_id, workflow_id)
  DO UPDATE SET
    total_executions = workflow_usage_daily.total_executions + 1,
    successful_executions = workflow_usage_daily.successful_executions +
      CASE WHEN p_status = 'completed' THEN 1 ELSE 0 END,
    failed_executions = workflow_usage_daily.failed_executions +
      CASE WHEN p_status = 'failed' THEN 1 ELSE 0 END,
    cancelled_executions = workflow_usage_daily.cancelled_executions +
      CASE WHEN p_status = 'cancelled' THEN 1 ELSE 0 END,
    timeout_executions = workflow_usage_daily.timeout_executions +
      CASE WHEN p_status = 'timeout' THEN 1 ELSE 0 END,
    total_duration_ms = workflow_usage_daily.total_duration_ms + COALESCE(p_duration_ms, 0),
    avg_duration_ms = (workflow_usage_daily.total_duration_ms + COALESCE(p_duration_ms, 0)) /
      (workflow_usage_daily.total_executions + 1),
    max_duration_ms = GREATEST(workflow_usage_daily.max_duration_ms, p_duration_ms),
    total_activities_executed = workflow_usage_daily.total_activities_executed + COALESCE(p_activity_count, 0),
    total_input_bytes = workflow_usage_daily.total_input_bytes + COALESCE(p_input_size_bytes, 0),
    total_output_bytes = workflow_usage_daily.total_output_bytes + COALESCE(p_output_size_bytes, 0),
    manual_triggers = workflow_usage_daily.manual_triggers +
      CASE WHEN p_trigger_type = 'manual' THEN 1 ELSE 0 END,
    schedule_triggers = workflow_usage_daily.schedule_triggers +
      CASE WHEN p_trigger_type = 'schedule' THEN 1 ELSE 0 END,
    webhook_triggers = workflow_usage_daily.webhook_triggers +
      CASE WHEN p_trigger_type = 'webhook' THEN 1 ELSE 0 END,
    api_triggers = workflow_usage_daily.api_triggers +
      CASE WHEN p_trigger_type = 'api' THEN 1 ELSE 0 END,
    updated_at = NOW();

  RETURN v_metric_id;
END;
$$ LANGUAGE plpgsql;

-- Record resource event
CREATE OR REPLACE FUNCTION record_resource_event(
  p_project_id UUID,
  p_workflow_id UUID,
  p_execution_id UUID,
  p_component_metric_id UUID,
  p_resource_type VARCHAR(50),
  p_resource_subtype VARCHAR(50),
  p_resource_id UUID,
  p_resource_name VARCHAR(255),
  p_operation VARCHAR(50),
  p_direction VARCHAR(20),
  p_duration_ms INTEGER,
  p_latency_ms INTEGER,
  p_request_size_bytes INTEGER,
  p_response_size_bytes INTEGER,
  p_status VARCHAR(20),
  p_error_type VARCHAR(100) DEFAULT NULL,
  p_model_name VARCHAR(100) DEFAULT NULL,
  p_prompt_tokens INTEGER DEFAULT NULL,
  p_completion_tokens INTEGER DEFAULT NULL,
  p_total_tokens INTEGER DEFAULT NULL,
  p_target_project_id UUID DEFAULT NULL,
  p_target_service VARCHAR(255) DEFAULT NULL,
  p_started_at TIMESTAMPTZ DEFAULT NOW(),
  p_completed_at TIMESTAMPTZ DEFAULT NULL,
  p_metadata JSONB DEFAULT '{}'
) RETURNS UUID AS $$
DECLARE
  v_event_id UUID;
  v_date DATE;
BEGIN
  v_date := DATE(p_started_at);

  -- Insert resource event
  INSERT INTO resource_events (
    project_id, workflow_id, workflow_execution_id, component_metric_id,
    resource_type, resource_subtype, resource_id, resource_name,
    operation, direction, duration_ms, latency_ms,
    request_size_bytes, response_size_bytes,
    status, error_type,
    model_name, prompt_tokens, completion_tokens, total_tokens,
    target_project_id, target_service,
    started_at, completed_at, metadata
  ) VALUES (
    p_project_id, p_workflow_id, p_execution_id, p_component_metric_id,
    p_resource_type, p_resource_subtype, p_resource_id, p_resource_name,
    p_operation, p_direction, p_duration_ms, p_latency_ms,
    p_request_size_bytes, p_response_size_bytes,
    p_status, p_error_type,
    p_model_name, p_prompt_tokens, p_completion_tokens, p_total_tokens,
    p_target_project_id, p_target_service,
    p_started_at, p_completed_at, p_metadata
  ) RETURNING id INTO v_event_id;

  -- Update daily aggregates
  INSERT INTO resource_usage_daily (
    date, project_id, resource_type, resource_name,
    total_invocations, successful_invocations, failed_invocations,
    total_duration_ms, avg_duration_ms, avg_latency_ms,
    total_request_bytes, total_response_bytes,
    total_prompt_tokens, total_completion_tokens, total_tokens,
    updated_at
  ) VALUES (
    v_date, p_project_id, p_resource_type, p_resource_name,
    1,
    CASE WHEN p_status = 'success' THEN 1 ELSE 0 END,
    CASE WHEN p_status = 'failure' THEN 1 ELSE 0 END,
    COALESCE(p_duration_ms, 0),
    p_duration_ms,
    p_latency_ms,
    COALESCE(p_request_size_bytes, 0),
    COALESCE(p_response_size_bytes, 0),
    COALESCE(p_prompt_tokens, 0),
    COALESCE(p_completion_tokens, 0),
    COALESCE(p_total_tokens, 0),
    NOW()
  )
  ON CONFLICT (date, project_id, resource_type, resource_name)
  DO UPDATE SET
    total_invocations = resource_usage_daily.total_invocations + 1,
    successful_invocations = resource_usage_daily.successful_invocations +
      CASE WHEN p_status = 'success' THEN 1 ELSE 0 END,
    failed_invocations = resource_usage_daily.failed_invocations +
      CASE WHEN p_status = 'failure' THEN 1 ELSE 0 END,
    total_duration_ms = resource_usage_daily.total_duration_ms + COALESCE(p_duration_ms, 0),
    avg_duration_ms = (resource_usage_daily.total_duration_ms + COALESCE(p_duration_ms, 0)) /
      (resource_usage_daily.total_invocations + 1),
    avg_latency_ms = CASE
      WHEN p_latency_ms IS NOT NULL THEN
        COALESCE(resource_usage_daily.avg_latency_ms, 0) +
        (p_latency_ms - COALESCE(resource_usage_daily.avg_latency_ms, 0)) /
        (resource_usage_daily.total_invocations + 1)
      ELSE resource_usage_daily.avg_latency_ms
    END,
    total_request_bytes = resource_usage_daily.total_request_bytes + COALESCE(p_request_size_bytes, 0),
    total_response_bytes = resource_usage_daily.total_response_bytes + COALESCE(p_response_size_bytes, 0),
    total_prompt_tokens = resource_usage_daily.total_prompt_tokens + COALESCE(p_prompt_tokens, 0),
    total_completion_tokens = resource_usage_daily.total_completion_tokens + COALESCE(p_completion_tokens, 0),
    total_tokens = resource_usage_daily.total_tokens + COALESCE(p_total_tokens, 0),
    updated_at = NOW();

  RETURN v_event_id;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- Row Level Security (RLS)
-- ============================================================================
ALTER TABLE workflow_execution_metrics ENABLE ROW LEVEL SECURITY;
ALTER TABLE workflow_usage_daily ENABLE ROW LEVEL SECURITY;
ALTER TABLE resource_events ENABLE ROW LEVEL SECURITY;
ALTER TABLE resource_usage_daily ENABLE ROW LEVEL SECURITY;

-- Workflow execution metrics policies
CREATE POLICY "Users can view their workflow metrics" ON workflow_execution_metrics
  FOR SELECT
  USING (
    project_id IN (
      SELECT id FROM projects WHERE created_by = auth.uid()
      UNION
      SELECT project_id FROM project_members WHERE user_id = auth.uid()
    )
  );

CREATE POLICY "System can insert workflow metrics" ON workflow_execution_metrics
  FOR INSERT
  WITH CHECK (true);

-- Workflow usage daily policies
CREATE POLICY "Users can view their workflow usage" ON workflow_usage_daily
  FOR SELECT
  USING (
    project_id IN (
      SELECT id FROM projects WHERE created_by = auth.uid()
      UNION
      SELECT project_id FROM project_members WHERE user_id = auth.uid()
    )
  );

CREATE POLICY "System can manage workflow usage" ON workflow_usage_daily
  FOR ALL
  USING (true)
  WITH CHECK (true);

-- Resource events policies
CREATE POLICY "Users can view their resource events" ON resource_events
  FOR SELECT
  USING (
    project_id IN (
      SELECT id FROM projects WHERE created_by = auth.uid()
      UNION
      SELECT project_id FROM project_members WHERE user_id = auth.uid()
    )
  );

CREATE POLICY "System can insert resource events" ON resource_events
  FOR INSERT
  WITH CHECK (true);

-- Resource usage daily policies
CREATE POLICY "Users can view their resource usage" ON resource_usage_daily
  FOR SELECT
  USING (
    project_id IN (
      SELECT id FROM projects WHERE created_by = auth.uid()
      UNION
      SELECT project_id FROM project_members WHERE user_id = auth.uid()
    )
  );

CREATE POLICY "System can manage resource usage" ON resource_usage_daily
  FOR ALL
  USING (true)
  WITH CHECK (true);

-- ============================================================================
-- Comments
-- ============================================================================
COMMENT ON TABLE workflow_execution_metrics IS 'Records individual workflow execution metrics for performance and service analytics';
COMMENT ON TABLE workflow_usage_daily IS 'Daily aggregated workflow execution data for analytics dashboards';
COMMENT ON TABLE resource_events IS 'Records interface, variable, agent, and connector usage events';
COMMENT ON TABLE resource_usage_daily IS 'Daily aggregated resource usage data for analytics';
COMMENT ON FUNCTION record_workflow_execution_metric IS 'Atomically records a workflow execution metric and updates daily aggregates';
COMMENT ON FUNCTION record_resource_event IS 'Atomically records a resource usage event and updates daily aggregates';
