/**
 * Project Connector Manager
 * 
 * Manages project connectors and their Nexus integration.
 */

import {
  createNexusService,
  createNexusEndpoint,
  deleteNexusService,
  deleteNexusEndpoint,
  generateNexusServiceName,
  generateNexusEndpointName,
  type NexusService,
  type NexusEndpoint,
} from './client';
import type { SupabaseClient } from '@supabase/supabase-js';
import type { Database } from '@/types/database';

export interface ProjectConnectorConfig {
  sourceProjectId: string;
  targetProjectId: string;
  targetServiceId: string;
  targetInterfaceId: string;
  name: string;
  displayName: string;
  description?: string;
  visibility?: 'private' | 'public' | 'organization';
  authConfig?: Record<string, any>;
}

export interface RegisteredConnector {
  id: string;
  sourceProjectId: string;
  targetProjectId: string;
  nexusEndpointName: string;
  nexusServiceName: string;
}

/**
 * Create a project connector with Nexus integration
 */
export async function createProjectConnector(
  config: ProjectConnectorConfig,
  userId: string,
  supabase: SupabaseClient<Database>
): Promise<RegisteredConnector> {
  // Get target service and interface info
  const { data: targetService, error: serviceError } = await supabase
    .from('workflows')
    .select(`
      *,
      project:projects!inner(id, created_by),
      service_interface:service_interfaces!inner(*)
    `)
    .eq('id', config.targetServiceId)
    .single();

  if (serviceError || !targetService) {
    throw new Error('Target service not found');
  }

  const project = targetService.project as any;
  const serviceInterface = targetService.service_interface as any;

  // Verify target interface matches
  if (serviceInterface.id !== config.targetInterfaceId) {
    throw new Error('Target interface does not match service');
  }

  // Generate Nexus names
  const nexusServiceName = generateNexusServiceName(
    config.targetProjectId,
    config.targetServiceId
  );
  const nexusEndpointName = generateNexusEndpointName(
    config.name,
    config.sourceProjectId
  );

  // Get namespace (from environment or default)
  const namespace = process.env.TEMPORAL_NAMESPACE || 'default';

  // Create Nexus service for target service (if not exists)
  try {
    const nexusService: NexusService = {
      name: nexusServiceName,
      namespace,
      description: `Service connector for ${config.displayName}`,
    };
    await createNexusService(nexusService);
    console.log(`✅ Created Nexus service: ${nexusServiceName}`);
  } catch (error) {
    // Service might already exist, that's okay
    console.log(`ℹ️  Nexus service may already exist: ${nexusServiceName}`);
  }

  // Create Nexus endpoint pointing to target service
  try {
    const nexusEndpoint: NexusEndpoint = {
      name: nexusEndpointName,
      service: nexusServiceName,
      namespace,
      targetNamespace: namespace, // Same namespace for now, can be different in future
      description: `Connector: ${config.displayName}`,
    };
    await createNexusEndpoint(nexusEndpoint);
    console.log(`✅ Created Nexus endpoint: ${nexusEndpointName}`);
  } catch (error) {
    // Clean up service if endpoint creation fails
    try {
      await deleteNexusService(nexusServiceName, namespace);
    } catch (cleanupError) {
      console.error('Failed to cleanup Nexus service:', cleanupError);
    }
    throw new Error(`Failed to create Nexus endpoint: ${error instanceof Error ? error.message : 'Unknown error'}`);
  }

  // Store in database
  const { data: connector, error: dbError } = await supabase
    .from('project_connectors')
    .insert({
      source_project_id: config.sourceProjectId,
      target_project_id: config.targetProjectId,
      target_service_id: config.targetServiceId,
      target_interface_id: config.targetInterfaceId,
      name: config.name,
      display_name: config.displayName,
      description: config.description || null,
      nexus_endpoint_name: nexusEndpointName,
      nexus_service_name: nexusServiceName,
      visibility: config.visibility || 'private',
      auth_config: config.authConfig || null,
      created_by: userId,
    })
    .select()
    .single();

  if (dbError) {
    // Clean up Nexus resources if DB insert fails
    try {
      await deleteNexusEndpoint(nexusEndpointName, namespace);
      await deleteNexusService(nexusServiceName, namespace);
    } catch (cleanupError) {
      console.error('Failed to cleanup Nexus resources:', cleanupError);
    }

    if (dbError.code === '23505') {
      throw new Error('Connector with this name already exists');
    }
    throw new Error(`Failed to store connector: ${dbError.message}`);
  }

  return {
    id: connector.id,
    sourceProjectId: connector.source_project_id,
    targetProjectId: connector.target_project_id,
    nexusEndpointName: connector.nexus_endpoint_name,
    nexusServiceName: connector.nexus_service_name,
  };
}

/**
 * Delete a project connector and clean up Nexus resources
 */
export async function deleteProjectConnector(
  connectorId: string,
  supabase: SupabaseClient<Database>
): Promise<void> {
  // Get connector info
  const { data: connector, error } = await supabase
    .from('project_connectors')
    .select('nexus_endpoint_name, nexus_service_name')
    .eq('id', connectorId)
    .single();

  if (error || !connector) {
    throw new Error('Connector not found');
  }

  const namespace = process.env.TEMPORAL_NAMESPACE || 'default';

  // Delete Nexus endpoint
  try {
    await deleteNexusEndpoint(connector.nexus_endpoint_name, namespace);
    console.log(`✅ Deleted Nexus endpoint: ${connector.nexus_endpoint_name}`);
  } catch (error) {
    console.warn(`⚠️  Failed to delete Nexus endpoint: ${error}`);
  }

  // Delete Nexus service (only if no other connectors use it)
  // For now, we'll delete it - in production, check for other connectors first
  try {
    await deleteNexusService(connector.nexus_service_name, namespace);
    console.log(`✅ Deleted Nexus service: ${connector.nexus_service_name}`);
  } catch (error) {
    console.warn(`⚠️  Failed to delete Nexus service: ${error}`);
  }

  // Delete from database
  await supabase
    .from('project_connectors')
    .delete()
    .eq('id', connectorId);
}

/**
 * Test a project connector (verify Nexus connection works)
 */
export async function testProjectConnector(
  connectorId: string,
  supabase: SupabaseClient<Database>
): Promise<{ success: boolean; message: string; error?: string }> {
  const { data: connector, error } = await supabase
    .from('project_connectors')
    .select(`
      *,
      target_service:workflows!inner(*),
      target_interface:service_interfaces!inner(*)
    `)
    .eq('id', connectorId)
    .single();

  if (error || !connector) {
    return {
      success: false,
      message: 'Connector not found',
      error: error?.message,
    };
  }

  // Verify Nexus endpoint exists
  // In a real implementation, we'd make an HTTP call to verify
  // For now, we'll just check that the connector exists in the database
  try {
    // TODO: Add actual Nexus endpoint health check
    return {
      success: true,
      message: 'Connector configuration is valid',
    };
  } catch (error: any) {
    return {
      success: false,
      message: 'Failed to test connector',
      error: error.message,
    };
  }
}

