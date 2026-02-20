/**
 * Temporal Nexus Client
 * 
 * Client for managing Temporal Nexus services and endpoints.
 * Nexus enables cross-namespace and cross-project communication.
 * 
 * Note: This uses Temporal's HTTP API for Nexus management.
 * For actual workflow calls, use the Temporal client with Nexus endpoints.
 */

import { getTemporalConnection } from '@/lib/temporal/connection';

export interface NexusService {
  name: string;
  namespace: string;
  description?: string;
}

export interface NexusEndpoint {
  name: string;
  service: string;
  namespace: string;
  targetNamespace?: string;
  targetAddress?: string;
  description?: string;
}

export interface NexusOperation {
  name: string;
  endpoint: string;
  method: 'StartOperation' | 'Signal' | 'Query';
}

/**
 * Get Temporal Nexus HTTP API base URL
 */
function getNexusApiUrl(): string {
  const temporalAddress = process.env.TEMPORAL_ADDRESS || 'localhost:7233';
  // Nexus API is typically on the same host but different port or path
  // For now, we'll use the Temporal address and assume Nexus is configured
  // In production, this might be a separate endpoint
  const nexusUrl = process.env.NEXUS_API_URL || temporalAddress.replace(':7233', ':8088');
  return `http://${nexusUrl}`;
}

/**
 * Create a Nexus service
 * 
 * A Nexus service represents a workflow service that can be called from other namespaces.
 */
export async function createNexusService(
  service: NexusService
): Promise<void> {
  // Note: Nexus service creation typically requires Temporal admin API
  // This is a placeholder implementation - actual Nexus API may vary
  const apiUrl = getNexusApiUrl();
  
  try {
    const response = await fetch(`${apiUrl}/api/v1/nexus/services`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        name: service.name,
        namespace: service.namespace,
        description: service.description,
      }),
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(`Failed to create Nexus service: ${response.status} ${error}`);
    }
  } catch (error) {
    // If Nexus API is not available, log warning but don't fail
    // Nexus may be configured differently or not yet available
    console.warn('⚠️  Nexus API not available. Service creation skipped:', error);
    throw new Error(`Nexus service creation failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
  }
}

/**
 * Create a Nexus endpoint
 * 
 * A Nexus endpoint connects to a Nexus service in another namespace/project.
 */
export async function createNexusEndpoint(
  endpoint: NexusEndpoint
): Promise<void> {
  const apiUrl = getNexusApiUrl();
  
  try {
    const response = await fetch(`${apiUrl}/api/v1/nexus/endpoints`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        name: endpoint.name,
        service: endpoint.service,
        namespace: endpoint.namespace,
        target_namespace: endpoint.targetNamespace,
        target_address: endpoint.targetAddress,
        description: endpoint.description,
      }),
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(`Failed to create Nexus endpoint: ${response.status} ${error}`);
    }
  } catch (error) {
    console.warn('⚠️  Nexus API not available. Endpoint creation skipped:', error);
    throw new Error(`Nexus endpoint creation failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
  }
}

/**
 * Delete a Nexus service
 */
export async function deleteNexusService(
  serviceName: string,
  namespace: string
): Promise<void> {
  const apiUrl = getNexusApiUrl();
  
  try {
    const response = await fetch(
      `${apiUrl}/api/v1/nexus/services/${namespace}/${serviceName}`,
      {
        method: 'DELETE',
      }
    );

    if (!response.ok && response.status !== 404) {
      const error = await response.text();
      throw new Error(`Failed to delete Nexus service: ${response.status} ${error}`);
    }
  } catch (error) {
    console.warn('⚠️  Nexus API not available. Service deletion skipped:', error);
  }
}

/**
 * Delete a Nexus endpoint
 */
export async function deleteNexusEndpoint(
  endpointName: string,
  namespace: string
): Promise<void> {
  const apiUrl = getNexusApiUrl();
  
  try {
    const response = await fetch(
      `${apiUrl}/api/v1/nexus/endpoints/${namespace}/${endpointName}`,
      {
        method: 'DELETE',
      }
    );

    if (!response.ok && response.status !== 404) {
      const error = await response.text();
      throw new Error(`Failed to delete Nexus endpoint: ${response.status} ${error}`);
    }
  } catch (error) {
    console.warn('⚠️  Nexus API not available. Endpoint deletion skipped:', error);
  }
}

/**
 * Generate Nexus service name from project and service
 */
export function generateNexusServiceName(projectId: string, serviceId: string): string {
  // Generate deterministic service name
  return `project-${projectId.substring(0, 8)}-service-${serviceId.substring(0, 8)}`;
}

/**
 * Generate Nexus endpoint name for connector
 */
export function generateNexusEndpointName(connectorName: string, sourceProjectId: string): string {
  // Generate deterministic endpoint name
  const sanitized = connectorName.toLowerCase().replace(/[^a-z0-9-]/g, '-');
  return `connector-${sourceProjectId.substring(0, 8)}-${sanitized}`;
}

