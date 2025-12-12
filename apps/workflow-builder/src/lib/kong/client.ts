/**
 * Kong Admin API Client
 * 
 * Client for interacting with Kong Admin API to manage services, routes, and plugins.
 */

import { getKongConfig } from './config';

export interface KongService {
  id: string;
  name: string;
  url: string;
}

export interface KongRoute {
  id: string;
  name: string;
  paths: string[];
  methods: string[];
  service: { id: string };
}

export class KongClient {
  private adminUrl: string;
  private apiKey?: string;

  constructor(adminUrl?: string, apiKey?: string) {
    const config = adminUrl ? { adminUrl, apiKey } : getKongConfig();
    this.adminUrl = config.adminUrl;
    this.apiKey = config.adminApiKey;
  }

  /**
   * Get headers for Kong Admin API requests
   */
  private getHeaders(): Record<string, string> {
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
    };

    if (this.apiKey) {
      headers['Kong-Admin-Token'] = this.apiKey;
    }

    return headers;
  }

  /**
   * Create or get a Kong service
   */
  async createService(name: string, url: string): Promise<string> {
    // Check if service already exists
    const existing = await this.getService(name);
    if (existing) {
      return existing.id;
    }

    const response = await fetch(`${this.adminUrl}/services`, {
      method: 'POST',
      headers: this.getHeaders(),
      body: JSON.stringify({
        name,
        url,
        protocol: 'http',
        connect_timeout: 60000,
        write_timeout: 60000,
        read_timeout: 60000,
        retries: 5,
      }),
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(`Failed to create Kong service: ${response.status} ${error}`);
    }

    const data = await response.json();
    return data.id;
  }

  /**
   * Get a service by name
   */
  async getService(name: string): Promise<KongService | null> {
    const response = await fetch(`${this.adminUrl}/services/${name}`, {
      method: 'GET',
      headers: this.getHeaders(),
    });

    if (response.status === 404) {
      return null;
    }

    if (!response.ok) {
      const error = await response.text();
      throw new Error(`Failed to get Kong service: ${response.status} ${error}`);
    }

    const data = await response.json();
    return {
      id: data.id,
      name: data.name,
      url: data.url,
    };
  }

  /**
   * Create a Kong route
   */
  async createRoute(
    serviceId: string,
    name: string,
    paths: string[],
    methods: string[]
  ): Promise<string> {
    const response = await fetch(`${this.adminUrl}/services/${serviceId}/routes`, {
      method: 'POST',
      headers: this.getHeaders(),
      body: JSON.stringify({
        name,
        paths,
        methods,
        strip_path: false,
        preserve_host: false,
        protocols: ['http', 'https'],
      }),
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(`Failed to create Kong route: ${response.status} ${error}`);
    }

    const data = await response.json();
    return data.id;
  }

  /**
   * Get a route by name
   */
  async getRoute(name: string): Promise<KongRoute | null> {
    const response = await fetch(`${this.adminUrl}/routes/${name}`, {
      method: 'GET',
      headers: this.getHeaders(),
    });

    if (response.status === 404) {
      return null;
    }

    if (!response.ok) {
      const error = await response.text();
      throw new Error(`Failed to get Kong route: ${response.status} ${error}`);
    }

    const data = await response.json();
    return {
      id: data.id,
      name: data.name,
      paths: data.paths,
      methods: data.methods,
      service: { id: data.service.id },
    };
  }

  /**
   * Enable a plugin on a route
   */
  async enablePlugin(
    routeId: string,
    pluginName: string,
    config: Record<string, any>
  ): Promise<void> {
    const response = await fetch(`${this.adminUrl}/routes/${routeId}/plugins`, {
      method: 'POST',
      headers: this.getHeaders(),
      body: JSON.stringify({
        name: pluginName,
        config,
      }),
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(`Failed to enable plugin ${pluginName}: ${response.status} ${error}`);
    }
  }

  /**
   * Delete a route
   */
  async deleteRoute(routeId: string): Promise<void> {
    const response = await fetch(`${this.adminUrl}/routes/${routeId}`, {
      method: 'DELETE',
      headers: this.getHeaders(),
    });

    if (response.status === 404) {
      // Already deleted, that's fine
      return;
    }

    if (!response.ok) {
      const error = await response.text();
      throw new Error(`Failed to delete Kong route: ${response.status} ${error}`);
    }
  }

  /**
   * Health check - verify Kong is accessible
   */
  async healthCheck(): Promise<boolean> {
    try {
      const response = await fetch(`${this.adminUrl}/status`, {
        method: 'GET',
        headers: this.getHeaders(),
      });
      return response.ok;
    } catch {
      return false;
    }
  }

  /**
   * Enable JWT authentication plugin on a route
   */
  async enableJwtAuth(
    routeId: string,
    config?: {
      keyClaimName?: string;
      claimsToVerify?: string[];
      maximumExpiration?: number;
    }
  ): Promise<void> {
    const pluginConfig = {
      key_claim_name: config?.keyClaimName || 'sub',
      claims_to_verify: config?.claimsToVerify || ['exp'],
      secret_is_base64: false,
      run_on_preflight: true,
      maximum_expiration: config?.maximumExpiration || 86400, // 24 hours default
    };

    await this.enablePlugin(routeId, 'jwt', pluginConfig);
  }

  /**
   * Create a JWT consumer and credentials
   * This is needed for Kong to validate JWTs
   */
  async createJwtConsumer(
    consumerId: string,
    jwtSecret: string,
    algorithm: 'HS256' | 'HS384' | 'HS512' | 'RS256' | 'RS384' | 'RS512' = 'HS256'
  ): Promise<{ key: string; secret: string }> {
    // First create or get consumer
    let consumer;
    try {
      const getResponse = await fetch(`${this.adminUrl}/consumers/${consumerId}`, {
        method: 'GET',
        headers: this.getHeaders(),
      });

      if (getResponse.ok) {
        consumer = await getResponse.json();
      } else if (getResponse.status === 404) {
        // Create consumer
        const createResponse = await fetch(`${this.adminUrl}/consumers`, {
          method: 'POST',
          headers: this.getHeaders(),
          body: JSON.stringify({ username: consumerId }),
        });

        if (!createResponse.ok) {
          const error = await createResponse.text();
          throw new Error(`Failed to create consumer: ${createResponse.status} ${error}`);
        }

        consumer = await createResponse.json();
      } else {
        const error = await getResponse.text();
        throw new Error(`Failed to get consumer: ${getResponse.status} ${error}`);
      }
    } catch (error) {
      throw error;
    }

    // Create JWT credential for consumer
    const response = await fetch(`${this.adminUrl}/consumers/${consumer.id}/jwt`, {
      method: 'POST',
      headers: this.getHeaders(),
      body: JSON.stringify({
        algorithm,
        secret: jwtSecret,
      }),
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(`Failed to create JWT credential: ${response.status} ${error}`);
    }

    const data = await response.json();
    return {
      key: data.key,
      secret: data.secret,
    };
  }

  /**
   * Enable correlation-id plugin globally or on a route
   */
  async enableCorrelationId(
    routeId?: string,
    config?: {
      headerName?: string;
      generator?: 'uuid' | 'uuid#counter' | 'tracker';
      echoDownstream?: boolean;
    }
  ): Promise<void> {
    const pluginConfig = {
      header_name: config?.headerName || 'X-Request-ID',
      generator: config?.generator || 'uuid',
      echo_downstream: config?.echoDownstream ?? true,
    };

    if (routeId) {
      await this.enablePlugin(routeId, 'correlation-id', pluginConfig);
    } else {
      // Enable globally
      const response = await fetch(`${this.adminUrl}/plugins`, {
        method: 'POST',
        headers: this.getHeaders(),
        body: JSON.stringify({
          name: 'correlation-id',
          config: pluginConfig,
        }),
      });

      if (!response.ok) {
        const error = await response.text();
        throw new Error(`Failed to enable correlation-id plugin: ${response.status} ${error}`);
      }
    }
  }

  /**
   * Get plugin by name on a route
   */
  async getPluginOnRoute(routeId: string, pluginName: string): Promise<any | null> {
    const response = await fetch(`${this.adminUrl}/routes/${routeId}/plugins`, {
      method: 'GET',
      headers: this.getHeaders(),
    });

    if (!response.ok) {
      return null;
    }

    const data = await response.json();
    const plugin = data.data?.find((p: any) => p.name === pluginName);
    return plugin || null;
  }

  /**
   * Delete a plugin from a route
   */
  async deletePlugin(pluginId: string): Promise<void> {
    const response = await fetch(`${this.adminUrl}/plugins/${pluginId}`, {
      method: 'DELETE',
      headers: this.getHeaders(),
    });

    if (response.status === 404) {
      return; // Already deleted
    }

    if (!response.ok) {
      const error = await response.text();
      throw new Error(`Failed to delete plugin: ${response.status} ${error}`);
    }
  }
}

