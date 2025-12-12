-- Component Metrics Schema
-- Workflow Observability Platform - Activity Performance Analytics
--
-- This schema supports performance monitoring and analytics for workflow activities.
-- Run this migration in your Supabase SQL editor or via CLI.

-- ============================================================================
-- Core Metrics Table: component_metrics
-- Records individual activity executions for performance analysis
-- ============================================================================
CREATE TABLE IF NOT EXISTS component_metrics (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

  -- Context (for filtering and grouping)
  project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  workflow_id UUID NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
  workflow_execution_id UUID REFERENCES workflow_executions(id) ON DELETE SET NULL,

  -- Component identification
  component_type VARCHAR(50) NOT NULL,  -- 'activity', 'agent', 'transform', etc.
  component_name VARCHAR(255) NOT NULL, -- The activity function name
  component_id UUID,                    -- Reference to components registry if available
  node_id VARCHAR(255),                 -- Node ID in workflow graph

  -- Execution metrics
  invocation_count INTEGER NOT NULL DEFAULT 1,
  duration_ms INTEGER,                  -- Total execution time
  queue_time_ms INTEGER,                -- Time waiting in queue

  -- Status tracking
  status VARCHAR(20) NOT NULL,          -- 'completed', 'failed', 'timeout', 'cancelled'
  is_retry BOOLEAN DEFAULT FALSE,
  attempt_number INTEGER DEFAULT 1,

  -- Resource metrics (reserved for future)
  memory_peak_mb INTEGER,
  cpu_time_ms INTEGER,
  io_bytes BIGINT,

  -- Error tracking
  error_type VARCHAR(100),
  error_code VARCHAR(50),

  -- Timestamps
  started_at TIMESTAMPTZ NOT NULL,
  completed_at TIMESTAMPTZ,
  recorded_at TIMESTAMPTZ DEFAULT NOW(),

  -- Extensibility
  metadata JSONB DEFAULT '{}'
);

-- Performance indexes for common queries
CREATE INDEX IF NOT EXISTS idx_component_metrics_project_component
  ON component_metrics(project_id, component_type, started_at DESC);

CREATE INDEX IF NOT EXISTS idx_component_metrics_workflow_execution
  ON component_metrics(workflow_execution_id, started_at DESC);

CREATE INDEX IF NOT EXISTS idx_component_metrics_reporting_period
  ON component_metrics(project_id, started_at, status);

CREATE INDEX IF NOT EXISTS idx_component_metrics_component_name
  ON component_metrics(component_name, started_at DESC);

-- ============================================================================
-- Aggregation Table: component_usage_daily
-- Daily rollups for dashboard performance (named for observability, not billing)
-- ============================================================================
CREATE TABLE IF NOT EXISTS component_usage_daily (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  date DATE NOT NULL,
  project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  component_type VARCHAR(50) NOT NULL,
  component_name VARCHAR(255) NOT NULL,

  -- Execution counts
  total_invocations INTEGER NOT NULL DEFAULT 0,
  successful_invocations INTEGER NOT NULL DEFAULT 0,
  failed_invocations INTEGER NOT NULL DEFAULT 0,
  retried_invocations INTEGER NOT NULL DEFAULT 0,

  -- Duration metrics (for latency analysis)
  total_duration_ms BIGINT DEFAULT 0,
  avg_duration_ms INTEGER,
  p50_duration_ms INTEGER,
  p95_duration_ms INTEGER,
  p99_duration_ms INTEGER,
  max_duration_ms INTEGER,

  -- Resource metrics (reserved for future)
  total_memory_mb BIGINT DEFAULT 0,
  total_cpu_time_ms BIGINT DEFAULT 0,

  -- Metadata
  updated_at TIMESTAMPTZ DEFAULT NOW(),

  CONSTRAINT component_usage_daily_unique
    UNIQUE(date, project_id, component_type, component_name)
);

-- Index for efficient dashboard queries
CREATE INDEX IF NOT EXISTS idx_component_usage_daily_project_date
  ON component_usage_daily(project_id, date DESC);

CREATE INDEX IF NOT EXISTS idx_component_usage_daily_component
  ON component_usage_daily(component_name, date DESC);

-- ============================================================================
-- Database Functions
-- ============================================================================

-- Atomic metric recording with daily aggregation update
CREATE OR REPLACE FUNCTION record_component_metric(
  p_project_id UUID,
  p_workflow_id UUID,
  p_execution_id UUID,
  p_component_type VARCHAR(50),
  p_component_name VARCHAR(255),
  p_component_id UUID,
  p_node_id VARCHAR(255),
  p_duration_ms INTEGER,
  p_status VARCHAR(20),
  p_is_retry BOOLEAN,
  p_attempt_number INTEGER,
  p_started_at TIMESTAMPTZ,
  p_completed_at TIMESTAMPTZ,
  p_error_type VARCHAR(100) DEFAULT NULL,
  p_metadata JSONB DEFAULT '{}'
) RETURNS UUID AS $$
DECLARE
  v_metric_id UUID;
  v_date DATE;
BEGIN
  v_date := DATE(p_started_at);

  -- Insert raw metric record
  INSERT INTO component_metrics (
    project_id, workflow_id, workflow_execution_id,
    component_type, component_name, component_id, node_id,
    duration_ms, status, is_retry, attempt_number,
    started_at, completed_at, error_type, metadata
  ) VALUES (
    p_project_id, p_workflow_id, p_execution_id,
    p_component_type, p_component_name, p_component_id, p_node_id,
    p_duration_ms, p_status, p_is_retry, p_attempt_number,
    p_started_at, p_completed_at, p_error_type, p_metadata
  ) RETURNING id INTO v_metric_id;

  -- Update daily aggregates (upsert)
  INSERT INTO component_usage_daily (
    date, project_id, component_type, component_name,
    total_invocations, successful_invocations, failed_invocations, retried_invocations,
    total_duration_ms, avg_duration_ms, max_duration_ms,
    updated_at
  ) VALUES (
    v_date, p_project_id, p_component_type, p_component_name,
    1,
    CASE WHEN p_status = 'completed' THEN 1 ELSE 0 END,
    CASE WHEN p_status = 'failed' THEN 1 ELSE 0 END,
    CASE WHEN p_is_retry THEN 1 ELSE 0 END,
    COALESCE(p_duration_ms, 0),
    p_duration_ms,
    p_duration_ms,
    NOW()
  )
  ON CONFLICT (date, project_id, component_type, component_name)
  DO UPDATE SET
    total_invocations = component_usage_daily.total_invocations + 1,
    successful_invocations = component_usage_daily.successful_invocations +
      CASE WHEN p_status = 'completed' THEN 1 ELSE 0 END,
    failed_invocations = component_usage_daily.failed_invocations +
      CASE WHEN p_status = 'failed' THEN 1 ELSE 0 END,
    retried_invocations = component_usage_daily.retried_invocations +
      CASE WHEN p_is_retry THEN 1 ELSE 0 END,
    total_duration_ms = component_usage_daily.total_duration_ms + COALESCE(p_duration_ms, 0),
    avg_duration_ms = (component_usage_daily.total_duration_ms + COALESCE(p_duration_ms, 0)) /
      (component_usage_daily.total_invocations + 1),
    max_duration_ms = GREATEST(component_usage_daily.max_duration_ms, p_duration_ms),
    updated_at = NOW();

  RETURN v_metric_id;
END;
$$ LANGUAGE plpgsql;

-- Function to get activity performance summary
CREATE OR REPLACE FUNCTION get_activity_performance(
  p_project_id UUID,
  p_start_date DATE,
  p_end_date DATE
) RETURNS TABLE (
  component_name VARCHAR(255),
  component_type VARCHAR(50),
  total_executions BIGINT,
  success_rate NUMERIC,
  avg_duration_ms NUMERIC,
  p95_duration_ms INTEGER,
  total_failures BIGINT
) AS $$
BEGIN
  RETURN QUERY
  SELECT
    cud.component_name,
    cud.component_type,
    SUM(cud.total_invocations)::BIGINT as total_executions,
    ROUND(
      SUM(cud.successful_invocations)::NUMERIC /
      NULLIF(SUM(cud.total_invocations), 0) * 100,
      2
    ) as success_rate,
    ROUND(
      SUM(cud.total_duration_ms)::NUMERIC /
      NULLIF(SUM(cud.total_invocations), 0),
      2
    ) as avg_duration_ms,
    MAX(cud.p95_duration_ms) as p95_duration_ms,
    SUM(cud.failed_invocations)::BIGINT as total_failures
  FROM component_usage_daily cud
  WHERE cud.project_id = p_project_id
    AND cud.date BETWEEN p_start_date AND p_end_date
  GROUP BY cud.component_name, cud.component_type
  ORDER BY SUM(cud.total_invocations) DESC;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- Row Level Security (RLS)
-- ============================================================================
ALTER TABLE component_metrics ENABLE ROW LEVEL SECURITY;
ALTER TABLE component_usage_daily ENABLE ROW LEVEL SECURITY;

-- Policies: users can only access metrics for projects they own/have access to
CREATE POLICY "Users can view their project metrics" ON component_metrics
  FOR SELECT
  USING (
    project_id IN (
      SELECT id FROM projects WHERE created_by = auth.uid()
      UNION
      SELECT project_id FROM project_members WHERE user_id = auth.uid()
    )
  );

CREATE POLICY "System can insert metrics" ON component_metrics
  FOR INSERT
  WITH CHECK (true);  -- Service role handles inserts

CREATE POLICY "Users can view their project usage" ON component_usage_daily
  FOR SELECT
  USING (
    project_id IN (
      SELECT id FROM projects WHERE created_by = auth.uid()
      UNION
      SELECT project_id FROM project_members WHERE user_id = auth.uid()
    )
  );

CREATE POLICY "System can manage usage aggregates" ON component_usage_daily
  FOR ALL
  USING (true)
  WITH CHECK (true);  -- Service role handles aggregation

-- ============================================================================
-- Comments (for documentation)
-- ============================================================================
COMMENT ON TABLE component_metrics IS 'Records individual activity execution metrics for performance monitoring';
COMMENT ON TABLE component_usage_daily IS 'Daily aggregated activity performance data for analytics dashboards';
COMMENT ON FUNCTION record_component_metric IS 'Atomically records a component execution metric and updates daily aggregates';
COMMENT ON FUNCTION get_activity_performance IS 'Returns activity performance summary for a project within a date range';
