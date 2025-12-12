/**
 * Kong Configuration Helper
 * 
 * Centralized configuration for Kong API Gateway integration.
 * Supports localhost, ngrok, and production environments.
 */

export interface KongConfig {
  adminUrl: string;
  adminApiKey?: string;
  gatewayUrl: string;
  appUrl: string;
  apiBaseUrl: string;
  environment: 'development' | 'production';
}

/**
 * Get Kong configuration from environment variables
 * 
 * Environment Variables:
 * - KONG_ADMIN_URL: Kong Admin API URL (default: http://localhost:8001)
 * - KONG_ADMIN_API_KEY: Optional admin API key
 * - KONG_GATEWAY_URL: Kong Gateway URL (default: http://localhost:8000)
 * - NEXT_PUBLIC_APP_URL: Application URL (default: http://localhost:3010)
 * - WORKFLOW_API_BASE_URL: Base URL for workflow API routes (default: {appUrl}/api/workflows)
 */
export function getKongConfig(): KongConfig {
  const adminUrl = process.env.KONG_ADMIN_URL || 'http://localhost:8001';
  const adminApiKey = process.env.KONG_ADMIN_API_KEY;
  const gatewayUrl = process.env.KONG_GATEWAY_URL || 'http://localhost:8000';
  const appUrl = process.env.NEXT_PUBLIC_APP_URL || 'http://localhost:3010';
  const apiBaseUrl = process.env.WORKFLOW_API_BASE_URL || `${appUrl}/api/workflows`;
  const environment = (process.env.NODE_ENV || 'development') as 'development' | 'production';

  return {
    adminUrl,
    adminApiKey,
    gatewayUrl,
    appUrl,
    apiBaseUrl,
    environment,
  };
}

/**
 * Check if Kong is enabled (configured)
 */
export function isKongEnabled(): boolean {
  const config = getKongConfig();
  // Kong is enabled if admin URL is configured
  return !!config.adminUrl;
}

