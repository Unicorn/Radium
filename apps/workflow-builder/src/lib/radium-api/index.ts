/**
 * Radium Workflow API client and types.
 *
 * @example
 * ```ts
 * import { createRadiumApiClient } from '@/lib/radium-api';
 *
 * const api = createRadiumApiClient('http://localhost:3020', token);
 * const status = await api.getStatus(serviceId);
 * ```
 */

export { createRadiumApiClient } from './client';
export type { RadiumApiClient } from './client';

export type {
  // Deploy
  DeployResponse,
  UndeployResponse,
  StatusResponse,
  // Interfaces
  CreateInterfaceRequest,
  UpdateInterfaceRequest,
  InterfaceResponse,
  InterfaceListResponse,
  PublishResponse,
  // Gateway
  GatewayAcceptedResponse,
  // Keys
  CreateKeyRequest,
  CreateKeyResponse,
  ListKeyResponse,
  ListKeysEnvelope,
  // Gateway config
  GatewayConfig,
  UpdateGatewayConfigRequest,
  // Dead letters
  DeadLetter,
  DeadLetterListEnvelope,
  ReplayDeadLetterResponse,
  // Versions
  ServiceVersion,
  ListVersionsEnvelope,
  RollbackRequest,
  RollbackResponse,
  // Errors
  ApiErrorBody,
  ApiErrorEnvelope,
} from './types';

export { RadiumApiError } from './types';
