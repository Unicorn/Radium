/**
 * Connector Classification Utilities
 * 
 * Utilities for managing connector classifications (e.g., "redis", "http-log")
 * that allow components to query connectors by type/capability.
 */

import type { SupabaseClient } from '@supabase/supabase-js';
import type { Database } from '@/types/database';

export type ConnectorClassification = 
  | 'redis'
  | 'http-log'
  | 'syslog'
  | 'file-log'
  | 'tcp-log'
  | 'udp-log';

/**
 * Get connectors by classification
 */
export async function getConnectorsByClassification(
  projectId: string,
  classification: ConnectorClassification,
  supabase: SupabaseClient<Database>
): Promise<Array<{
  id: string;
  name: string;
  display_name: string;
  description: string | null;
  connector_type: string;
  is_active: boolean;
  classifications: string[];
}>> {
  const { data, error } = await supabase
    .from('connectors')
    .select(`
      id,
      name,
      display_name,
      description,
      connector_type,
      is_active,
      classifications
    `)
    .eq('project_id', projectId)
    .eq('is_active', true)
    .contains('classifications', [classification])
    .order('name');

  if (error) {
    throw new Error(`Failed to get connectors by classification: ${error.message}`);
  }

  return (data || []).map(connector => ({
    id: connector.id,
    name: connector.name,
    display_name: connector.display_name,
    description: connector.description,
    connector_type: connector.connector_type,
    is_active: connector.is_active,
    classifications: (connector.classifications as string[]) || [],
  }));
}

/**
 * Add classification to a connector
 */
export async function addClassification(
  connectorId: string,
  classification: ConnectorClassification,
  supabase: SupabaseClient<Database>
): Promise<void> {
  // Insert into connector_classifications table
  const { error: insertError } = await supabase
    .from('connector_classifications')
    .insert({
      connector_id: connectorId,
      classification,
    });

  if (insertError) {
    // If it's a unique constraint violation, that's OK (already exists)
    if (insertError.code !== '23505') {
      throw new Error(`Failed to add classification: ${insertError.message}`);
    }
  }

  // The trigger will automatically update the classifications JSONB column
}

/**
 * Remove classification from a connector
 */
export async function removeClassification(
  connectorId: string,
  classification: ConnectorClassification,
  supabase: SupabaseClient<Database>
): Promise<void> {
  const { error } = await supabase
    .from('connector_classifications')
    .delete()
    .eq('connector_id', connectorId)
    .eq('classification', classification);

  if (error) {
    throw new Error(`Failed to remove classification: ${error.message}`);
  }

  // The trigger will automatically update the classifications JSONB column
}

/**
 * Get all classifications for a connector
 */
export async function getConnectorClassifications(
  connectorId: string,
  supabase: SupabaseClient<Database>
): Promise<ConnectorClassification[]> {
  const { data, error } = await supabase
    .from('connector_classifications')
    .select('classification')
    .eq('connector_id', connectorId);

  if (error) {
    throw new Error(`Failed to get connector classifications: ${error.message}`);
  }

  return (data || []).map(row => row.classification as ConnectorClassification);
}

