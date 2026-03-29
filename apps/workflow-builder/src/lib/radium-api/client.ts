/**
 * Typed HTTP client for the Radium Workflow API.
 *
 * Follows the factory-function pattern used in
 * `src/lib/kong/workflow-compiler-service.ts`.
 */

import type {
  CreateInterfaceRequest,
  CreateKeyRequest,
  CreateKeyResponse,
  DeadLetterListEnvelope,
  DeployResponse,
  GatewayAcceptedResponse,
  GatewayConfig,
  InterfaceListResponse,
  InterfaceResponse,
  ListKeysEnvelope,
  ListVersionsEnvelope,
  PublishResponse,
  ReplayDeadLetterResponse,
  RollbackRequest,
  RollbackResponse,
  ServiceVersion,
  StatusResponse,
  UndeployResponse,
  UpdateGatewayConfigRequest,
  UpdateInterfaceRequest,
} from './types';

import { RadiumApiError } from './types';
import type { ApiErrorEnvelope } from './types';

// ---------------------------------------------------------------------------
// Client interface
// ---------------------------------------------------------------------------

/** Methods exposed by the Radium API client. */
export interface RadiumApiClient {
  // -- Deploy ---------------------------------------------------------------
  deploy(serviceId: string): Promise<DeployResponse>;
  undeploy(serviceId: string): Promise<UndeployResponse>;
  getStatus(serviceId: string): Promise<StatusResponse>;

  // -- Interfaces -----------------------------------------------------------
  createInterface(
    serviceId: string,
    body: CreateInterfaceRequest,
  ): Promise<InterfaceResponse>;
  listInterfaces(serviceId: string): Promise<InterfaceListResponse>;
  getInterface(
    serviceId: string,
    interfaceId: string,
  ): Promise<InterfaceResponse>;
  updateInterface(
    serviceId: string,
    interfaceId: string,
    body: UpdateInterfaceRequest,
  ): Promise<InterfaceResponse>;
  deleteInterface(serviceId: string, interfaceId: string): Promise<void>;
  publishInterface(
    serviceId: string,
    interfaceId: string,
  ): Promise<PublishResponse>;
  unpublishInterface(serviceId: string, interfaceId: string): Promise<void>;

  // -- Gateway (public, no auth) --------------------------------------------
  sendGatewayRequest(
    interfaceId: string,
    payload?: Record<string, unknown>,
  ): Promise<GatewayAcceptedResponse>;

  // -- Interface API keys ---------------------------------------------------
  createKey(
    interfaceId: string,
    body: CreateKeyRequest,
  ): Promise<CreateKeyResponse>;
  listKeys(interfaceId: string): Promise<ListKeysEnvelope>;
  revokeKey(interfaceId: string, keyId: string): Promise<void>;

  // -- Gateway config -------------------------------------------------------
  getGatewayConfig(interfaceId: string): Promise<GatewayConfig>;
  updateGatewayConfig(
    interfaceId: string,
    body: UpdateGatewayConfigRequest,
  ): Promise<GatewayConfig>;

  // -- Dead letters ---------------------------------------------------------
  listDeadLetters(interfaceId: string): Promise<DeadLetterListEnvelope>;
  replayDeadLetter(
    interfaceId: string,
    deadLetterId: string,
  ): Promise<ReplayDeadLetterResponse>;

  // -- Versions -------------------------------------------------------------
  listVersions(serviceId: string): Promise<ListVersionsEnvelope>;
  getVersion(serviceId: string, versionId: string): Promise<ServiceVersion>;
  rollbackService(
    serviceId: string,
    body: RollbackRequest,
  ): Promise<RollbackResponse>;
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/**
 * Build common request headers. Includes `Authorization: Bearer <token>` when
 * an auth token is provided.
 */
function buildHeaders(authToken?: string): Record<string, string> {
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
  };
  if (authToken) {
    headers['Authorization'] = `Bearer ${authToken}`;
  }
  return headers;
}

/**
 * Throw a `RadiumApiError` from a non-2xx response. Attempts to parse a JSON
 * error envelope; falls back to plain text.
 */
async function throwApiError(response: Response): Promise<never> {
  let envelope: ApiErrorEnvelope;
  try {
    envelope = (await response.json()) as ApiErrorEnvelope;
  } catch {
    envelope = {
      error: {
        code: 'UNKNOWN',
        message: await response.text().catch(() => response.statusText),
      },
    };
  }
  throw new RadiumApiError(response.status, envelope);
}

/**
 * Parse a fetch response. Throws `RadiumApiError` for non-2xx responses.
 */
async function parseResponse<T>(response: Response): Promise<T> {
  if (!response.ok) {
    await throwApiError(response);
  }
  return (await response.json()) as T;
}

/**
 * Parse a fetch response that returns no body (204 No Content).
 * Throws `RadiumApiError` for non-2xx responses.
 */
async function parseEmptyResponse(response: Response): Promise<void> {
  if (!response.ok) {
    await throwApiError(response);
  }
}

// ---------------------------------------------------------------------------
// Factory
// ---------------------------------------------------------------------------

/**
 * Create a typed Radium API client.
 *
 * @param baseUrl  Base URL of the Radium workflow service
 *                 (e.g. `http://localhost:3020` or Kong gateway URL).
 * @param authToken  Optional Bearer token for authenticated endpoints.
 */
export function createRadiumApiClient(
  baseUrl: string,
  authToken?: string,
): RadiumApiClient {
  const headers = buildHeaders(authToken);

  return {
    // -- Deploy -------------------------------------------------------------

    async deploy(serviceId: string): Promise<DeployResponse> {
      const res = await fetch(`${baseUrl}/v1/services/${serviceId}/deploy`, {
        method: 'POST',
        headers,
      });
      return parseResponse<DeployResponse>(res);
    },

    async undeploy(serviceId: string): Promise<UndeployResponse> {
      const res = await fetch(`${baseUrl}/v1/services/${serviceId}/undeploy`, {
        method: 'POST',
        headers,
      });
      return parseResponse<UndeployResponse>(res);
    },

    async getStatus(serviceId: string): Promise<StatusResponse> {
      const res = await fetch(`${baseUrl}/v1/services/${serviceId}/status`, {
        method: 'GET',
        headers,
      });
      return parseResponse<StatusResponse>(res);
    },

    // -- Interfaces ---------------------------------------------------------

    async createInterface(
      serviceId: string,
      body: CreateInterfaceRequest,
    ): Promise<InterfaceResponse> {
      const res = await fetch(
        `${baseUrl}/v1/services/${serviceId}/interfaces`,
        {
          method: 'POST',
          headers,
          body: JSON.stringify(body),
        },
      );
      return parseResponse<InterfaceResponse>(res);
    },

    async listInterfaces(serviceId: string): Promise<InterfaceListResponse> {
      const res = await fetch(
        `${baseUrl}/v1/services/${serviceId}/interfaces`,
        {
          method: 'GET',
          headers,
        },
      );
      return parseResponse<InterfaceListResponse>(res);
    },

    async getInterface(
      serviceId: string,
      interfaceId: string,
    ): Promise<InterfaceResponse> {
      const res = await fetch(
        `${baseUrl}/v1/services/${serviceId}/interfaces/${interfaceId}`,
        {
          method: 'GET',
          headers,
        },
      );
      return parseResponse<InterfaceResponse>(res);
    },

    async updateInterface(
      serviceId: string,
      interfaceId: string,
      body: UpdateInterfaceRequest,
    ): Promise<InterfaceResponse> {
      const res = await fetch(
        `${baseUrl}/v1/services/${serviceId}/interfaces/${interfaceId}`,
        {
          method: 'PUT',
          headers,
          body: JSON.stringify(body),
        },
      );
      return parseResponse<InterfaceResponse>(res);
    },

    async deleteInterface(
      serviceId: string,
      interfaceId: string,
    ): Promise<void> {
      const res = await fetch(
        `${baseUrl}/v1/services/${serviceId}/interfaces/${interfaceId}`,
        {
          method: 'DELETE',
          headers,
        },
      );
      return parseEmptyResponse(res);
    },

    async publishInterface(
      serviceId: string,
      interfaceId: string,
    ): Promise<PublishResponse> {
      const res = await fetch(
        `${baseUrl}/v1/services/${serviceId}/interfaces/${interfaceId}/publish`,
        {
          method: 'POST',
          headers,
        },
      );
      return parseResponse<PublishResponse>(res);
    },

    async unpublishInterface(
      serviceId: string,
      interfaceId: string,
    ): Promise<void> {
      const res = await fetch(
        `${baseUrl}/v1/services/${serviceId}/interfaces/${interfaceId}/unpublish`,
        {
          method: 'POST',
          headers,
        },
      );
      return parseEmptyResponse(res);
    },

    // -- Gateway (public) ---------------------------------------------------

    async sendGatewayRequest(
      interfaceId: string,
      payload?: Record<string, unknown>,
    ): Promise<GatewayAcceptedResponse> {
      const res = await fetch(`${baseUrl}/v1/gateway/${interfaceId}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: payload ? JSON.stringify(payload) : undefined,
      });
      return parseResponse<GatewayAcceptedResponse>(res);
    },

    // -- Interface API keys -------------------------------------------------

    async createKey(
      interfaceId: string,
      body: CreateKeyRequest,
    ): Promise<CreateKeyResponse> {
      const res = await fetch(
        `${baseUrl}/v1/interfaces/${interfaceId}/keys`,
        {
          method: 'POST',
          headers,
          body: JSON.stringify(body),
        },
      );
      return parseResponse<CreateKeyResponse>(res);
    },

    async listKeys(interfaceId: string): Promise<ListKeysEnvelope> {
      const res = await fetch(
        `${baseUrl}/v1/interfaces/${interfaceId}/keys`,
        {
          method: 'GET',
          headers,
        },
      );
      return parseResponse<ListKeysEnvelope>(res);
    },

    async revokeKey(interfaceId: string, keyId: string): Promise<void> {
      const res = await fetch(
        `${baseUrl}/v1/interfaces/${interfaceId}/keys/${keyId}`,
        {
          method: 'DELETE',
          headers,
        },
      );
      return parseEmptyResponse(res);
    },

    // -- Gateway config -----------------------------------------------------

    async getGatewayConfig(interfaceId: string): Promise<GatewayConfig> {
      const res = await fetch(
        `${baseUrl}/v1/interfaces/${interfaceId}/config`,
        {
          method: 'GET',
          headers,
        },
      );
      return parseResponse<GatewayConfig>(res);
    },

    async updateGatewayConfig(
      interfaceId: string,
      body: UpdateGatewayConfigRequest,
    ): Promise<GatewayConfig> {
      const res = await fetch(
        `${baseUrl}/v1/interfaces/${interfaceId}/config`,
        {
          method: 'PUT',
          headers,
          body: JSON.stringify(body),
        },
      );
      return parseResponse<GatewayConfig>(res);
    },

    // -- Dead letters -------------------------------------------------------

    async listDeadLetters(
      interfaceId: string,
    ): Promise<DeadLetterListEnvelope> {
      const res = await fetch(
        `${baseUrl}/v1/interfaces/${interfaceId}/dead-letters`,
        {
          method: 'GET',
          headers,
        },
      );
      return parseResponse<DeadLetterListEnvelope>(res);
    },

    async replayDeadLetter(
      interfaceId: string,
      deadLetterId: string,
    ): Promise<ReplayDeadLetterResponse> {
      const res = await fetch(
        `${baseUrl}/v1/interfaces/${interfaceId}/dead-letters/${deadLetterId}/replay`,
        {
          method: 'POST',
          headers,
        },
      );
      return parseResponse<ReplayDeadLetterResponse>(res);
    },

    // -- Versions -----------------------------------------------------------

    async listVersions(serviceId: string): Promise<ListVersionsEnvelope> {
      const res = await fetch(
        `${baseUrl}/v1/services/${serviceId}/versions`,
        {
          method: 'GET',
          headers,
        },
      );
      return parseResponse<ListVersionsEnvelope>(res);
    },

    async getVersion(
      serviceId: string,
      versionId: string,
    ): Promise<ServiceVersion> {
      const res = await fetch(
        `${baseUrl}/v1/services/${serviceId}/versions/${versionId}`,
        {
          method: 'GET',
          headers,
        },
      );
      return parseResponse<ServiceVersion>(res);
    },

    async rollbackService(
      serviceId: string,
      body: RollbackRequest,
    ): Promise<RollbackResponse> {
      const res = await fetch(
        `${baseUrl}/v1/services/${serviceId}/rollback`,
        {
          method: 'POST',
          headers,
          body: JSON.stringify(body),
        },
      );
      return parseResponse<RollbackResponse>(res);
    },
  };
}
