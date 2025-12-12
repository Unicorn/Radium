/**
 * State Variable Metrics Collection
 * 
 * Collects metrics for state variables including size, access count, and last accessed time.
 * Provides recommendations based on usage patterns.
 */

import type { SupabaseClient } from '@supabase/supabase-js';
import type { Database } from '@/types/database';

type StateVariableMetrics = Database['public']['Tables']['state_variable_metrics']['Row'];
type StateVariable = Database['public']['Tables']['project_state_variables']['Row'] | 
                    Database['public']['Tables']['workflow_state_variables']['Row'];

export interface StateMetrics {
  variableId: string;
  scope: 'project' | 'workflow';
  sizeBytes: number;
  accessCount: number;
  lastAccessed: Date | null;
  averageAccessInterval?: number; // seconds
  growthRate?: number; // bytes per day
  recommendations: string[];
}

export interface StateAlert {
  variableId: string;
  scope: 'project' | 'workflow';
  severity: 'info' | 'warning' | 'error';
  message: string;
  recommendation: string;
}

/**
 * Record state variable access
 */
export async function recordStateAccess(
  supabase: SupabaseClient,
  variableId: string,
  scope: 'project' | 'workflow',
  sizeBytes: number
): Promise<void> {
  
  // Get existing metrics
  const { data: existing } = await supabase
    .from('state_variable_metrics')
    .select('*')
    .eq('variable_id', variableId)
    .eq('scope', scope)
    .order('created_at', { ascending: false })
    .limit(1)
    .single();

  const now = new Date();
  
  if (existing) {
    // Update existing metrics
    await supabase
      .from('state_variable_metrics')
      .update({
        size_bytes: sizeBytes,
        access_count: existing.access_count + 1,
        last_accessed: now.toISOString(),
        updated_at: now.toISOString(),
      })
      .eq('id', existing.id);
  } else {
    // Create new metrics record
    await supabase
      .from('state_variable_metrics')
      .insert({
        variable_id: variableId,
        scope,
        size_bytes: sizeBytes,
        access_count: 1,
        last_accessed: now.toISOString(),
      });
  }
}

/**
 * Get metrics for a state variable
 */
export async function getStateMetrics(
  supabase: SupabaseClient,
  variableId: string,
  scope: 'project' | 'workflow'
): Promise<StateMetrics | null> {
  
  const { data: metrics } = await supabase
    .from('state_variable_metrics')
    .select('*')
    .eq('variable_id', variableId)
    .eq('scope', scope)
    .order('created_at', { ascending: false })
    .limit(1)
    .single();

  if (!metrics) {
    return null;
  }

  // Get historical data for trend analysis
  const { data: history } = await supabase
    .from('state_variable_metrics')
    .select('size_bytes, created_at')
    .eq('variable_id', variableId)
    .eq('scope', scope)
    .order('created_at', { ascending: false })
    .limit(30); // Last 30 records

  if (!history) {
    return {
      variableId: metrics.variable_id,
      scope: metrics.scope as 'project' | 'workflow',
      sizeBytes: metrics.size_bytes || 0,
      accessCount: metrics.access_count || 0,
      lastAccessed: metrics.last_accessed ? new Date(metrics.last_accessed) : null,
      recommendations: generateRecommendations(metrics, []),
    };
  }

  const recommendations = generateRecommendations(metrics, history || []);

  return {
    variableId: metrics.variable_id,
    scope: metrics.scope as 'project' | 'workflow',
    sizeBytes: metrics.size_bytes || 0,
    accessCount: metrics.access_count || 0,
    lastAccessed: metrics.last_accessed ? new Date(metrics.last_accessed) : null,
    recommendations,
  };
}

/**
 * Generate recommendations based on metrics
 */
function generateRecommendations(
  metrics: StateVariableMetrics,
  history: Array<{ size_bytes: number; created_at: string }>
): string[] {
  const recommendations: string[] = [];
  const sizeMB = metrics.size_bytes / (1024 * 1024);
  const sizeKB = metrics.size_bytes / 1024;

  // Size-based recommendations
  if (sizeMB > 10) {
    recommendations.push(
      `Large state variable (${sizeMB.toFixed(2)} MB). Consider using database or Redis storage instead of in-memory workflow state.`
    );
  } else if (sizeKB > 100) {
    recommendations.push(
      `State variable is ${sizeKB.toFixed(2)} KB. Monitor growth and consider external storage if it exceeds 1 MB.`
    );
  }

  // Access pattern recommendations
  const accessCount = metrics.access_count || 0;
  if (accessCount > 1000) {
    recommendations.push(
      `High access count (${accessCount.toLocaleString()}). Consider caching or using Redis for better performance.`
    );
  }

  // Growth rate analysis
  if (history && history.length >= 2) {
    const oldest = history[history.length - 1];
    const newest = history[0];
    const daysDiff = (new Date(newest.created_at).getTime() - new Date(oldest.created_at).getTime()) / (1000 * 60 * 60 * 24);
    const newestSize = newest.size_bytes || 0;
    const oldestSize = oldest.size_bytes || 0;
    const sizeDiff = newestSize - oldestSize;
    
    if (daysDiff > 0) {
      const growthRate = sizeDiff / daysDiff;
      if (growthRate > 1024 * 1024) { // > 1 MB per day
        recommendations.push(
          `Rapid growth detected (${(growthRate / (1024 * 1024)).toFixed(2)} MB/day). Consider implementing data retention policies or archiving.`
        );
      }
    }
  }

  // Storage type recommendations
  const sizeBytes = metrics.size_bytes || 0;
  if (sizeBytes > 1024 * 1024) { // > 1 MB
    recommendations.push(
      'Consider migrating to database or Redis storage for better scalability and persistence.'
    );
  }

  if (accessCount > 100 && sizeBytes < 1024) { // High access, small size
    recommendations.push(
      'High-frequency access with small data size. Redis caching would provide optimal performance.'
    );
  }

  return recommendations;
}

/**
 * Check for alerts based on metrics
 */
export async function checkStateAlerts(
  supabase: SupabaseClient,
  variableId: string,
  scope: 'project' | 'workflow'
): Promise<StateAlert[]> {
  const metrics = await getStateMetrics(supabase, variableId, scope);
  if (!metrics) {
    return [];
  }

  const alerts: StateAlert[] = [];
  const sizeMB = (metrics.sizeBytes || 0) / (1024 * 1024);

  // Size-based alerts
  if (sizeMB > 50) {
    alerts.push({
      variableId,
      scope,
      severity: 'error',
      message: `State variable exceeds 50 MB (${sizeMB.toFixed(2)} MB). This may cause performance issues.`,
      recommendation: 'Immediately migrate to database or Redis storage. Consider data archiving.',
    });
  } else if (sizeMB > 10) {
    alerts.push({
      variableId,
      scope,
      severity: 'warning',
      message: `State variable is large (${sizeMB.toFixed(2)} MB). Monitor closely.`,
      recommendation: 'Consider migrating to external storage before it grows further.',
    });
  }

  // Access pattern alerts
  if (metrics.accessCount > 10000) {
    alerts.push({
      variableId,
      scope,
      severity: 'warning',
      message: `Very high access count (${metrics.accessCount}). Performance may be impacted.`,
      recommendation: 'Use Redis caching for optimal performance with high-frequency access.',
    });
  }

  // Staleness alerts
  if (metrics.lastAccessed) {
    const daysSinceAccess = (Date.now() - metrics.lastAccessed.getTime()) / (1000 * 60 * 60 * 24);
    const sizeMB = (metrics.sizeBytes || 0) / (1024 * 1024);
    if (daysSinceAccess > 30 && sizeMB > 1) {
      alerts.push({
        variableId,
        scope,
        severity: 'info',
        message: `State variable hasn't been accessed in ${Math.floor(daysSinceAccess)} days but is ${sizeMB.toFixed(2)} MB.`,
        recommendation: 'Consider archiving or cleaning up unused data.',
      });
    }
  }

  return alerts;
}

/**
 * Get all alerts for a project
 */
export async function getProjectStateAlerts(
  supabase: SupabaseClient,
  projectId: string
): Promise<StateAlert[]> {
  // Get all project state variables
  const { data: projectVars } = await supabase
    .from('project_state_variables')
    .select('id')
    .eq('project_id', projectId);

  if (!projectVars) {
    return [];
  }

  const allAlerts: StateAlert[] = [];
  
  for (const variable of projectVars) {
    const alerts = await checkStateAlerts(supabase, variable.id, 'project');
    allAlerts.push(...alerts);
  }

  return allAlerts;
}

