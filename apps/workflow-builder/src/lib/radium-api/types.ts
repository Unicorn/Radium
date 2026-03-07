/**
 * TypeScript type definitions for the Radium Workflow API.
 *
 * These types mirror the Rust response/request structs defined in
 * `crates/radium-workflow/src/api/v1/`. Field names use snake_case to match
 * the JSON produced by serde.
 */

// ---------------------------------------------------------------------------
// Deploy types (from deploy.rs)
// ---------------------------------------------------------------------------

/** Response returned after a successful deploy. */
export interface DeployResponse {
  workflow_id: string;
  status: string;
  compiled_at: string;
  message: string;
}

/** Response returned after a successful undeploy. */
export interface UndeployResponse {
  workflow_id: string;
  status: string;
  message: string;
}

/** Response returned from the deployment status endpoint. */
export interface StatusResponse {
  workflow_id: string;
  deployment_status: string;
  last_deployed_at: string | null;
}

// ---------------------------------------------------------------------------
// Interface types (from interfaces.rs)
// ---------------------------------------------------------------------------

/** Body for creating a new service interface. */
export interface CreateInterfaceRequest {
  name: string;
  display_name?: string;
  description?: string;
  interface_type: string;
  callable_name?: string;
  input_schema?: Record<string, unknown>;
  output_schema?: Record<string, unknown>;
  is_public?: boolean;
}

/** Body for updating an existing service interface. */
export interface UpdateInterfaceRequest {
  name?: string;
  display_name?: string;
  description?: string;
  interface_type?: string;
  callable_name?: unknown;
  input_schema?: Record<string, unknown>;
  output_schema?: Record<string, unknown>;
  is_public?: boolean;
}

/** Full interface response returned from GET and create/update operations. */
export interface InterfaceResponse {
  id: string;
  workflow_id: string;
  name: string;
  display_name?: string;
  description?: string;
  interface_type: string;
  callable_name?: string;
  input_schema?: Record<string, unknown>;
  output_schema?: Record<string, unknown>;
  is_public: boolean;
  created_at: string;
  updated_at: string;
}

/** List response envelope for interfaces. */
export interface InterfaceListResponse {
  interfaces: InterfaceResponse[];
  total: number;
}

/** Response returned after a successful publish. */
export interface PublishResponse {
  id: string;
  service_interface_id: string;
  route_path: string;
  http_method: string;
  kong_route_id?: string;
  kong_service_id?: string;
  gateway_workflow_id?: string;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

// ---------------------------------------------------------------------------
// Gateway types (from gateway.rs)
// ---------------------------------------------------------------------------

/** Response returned when a gateway request is successfully accepted. */
export interface GatewayAcceptedResponse {
  status: string;
  request_id: string;
  message: string;
}

// ---------------------------------------------------------------------------
// Interface API key types (planned P5 -- interface_keys)
// ---------------------------------------------------------------------------

/** Body for creating a new API key for an interface. */
export interface CreateKeyRequest {
  name: string;
  expires_at?: string;
}

/** Response returned after creating a new API key. */
export interface CreateKeyResponse {
  id: string;
  name: string;
  key: string;
  prefix: string;
  interface_id: string;
  created_at: string;
  expires_at: string | null;
}

/** Individual key in a list (key value is masked). */
export interface ListKeyResponse {
  id: string;
  name: string;
  prefix: string;
  interface_id: string;
  created_at: string;
  expires_at: string | null;
  is_active: boolean;
}

/** Envelope for listing keys. */
export interface ListKeysEnvelope {
  keys: ListKeyResponse[];
  total: number;
}

// ---------------------------------------------------------------------------
// Gateway config types (planned P5 -- gateway_config)
// ---------------------------------------------------------------------------

/** Gateway configuration for an interface. */
export interface GatewayConfig {
  interface_id: string;
  rate_limit_per_minute: number;
  rate_limit_per_hour: number;
  timeout_ms: number;
  max_body_size_bytes: number;
  cors_origins: string[];
  updated_at: string;
}

/** Body for updating gateway configuration. */
export interface UpdateGatewayConfigRequest {
  rate_limit_per_minute?: number;
  rate_limit_per_hour?: number;
  timeout_ms?: number;
  max_body_size_bytes?: number;
  cors_origins?: string[];
}

// ---------------------------------------------------------------------------
// Dead letter types (planned P5 -- dead_letters)
// ---------------------------------------------------------------------------

/** A dead letter record for a failed gateway request. */
export interface DeadLetter {
  id: string;
  interface_id: string;
  request_id: string;
  payload: Record<string, unknown>;
  error_message: string;
  failed_at: string;
  replayed_at: string | null;
}

/** Envelope for listing dead letters. */
export interface DeadLetterListEnvelope {
  dead_letters: DeadLetter[];
  total: number;
}

/** Response after replaying a dead letter. */
export interface ReplayDeadLetterResponse {
  id: string;
  status: string;
  replayed_at: string;
}

// ---------------------------------------------------------------------------
// Version types (planned P5 -- versions)
// ---------------------------------------------------------------------------

/** A service version record. */
export interface ServiceVersion {
  id: string;
  service_id: string;
  version_number: number;
  definition: Record<string, unknown>;
  compiled_code: string | null;
  deployed_at: string | null;
  created_at: string;
  is_active: boolean;
}

/** Envelope for listing versions. */
export interface ListVersionsEnvelope {
  versions: ServiceVersion[];
  total: number;
}

/** Body for rolling back to a previous version. */
export interface RollbackRequest {
  target_version: number;
}

/** Response after a successful rollback. */
export interface RollbackResponse {
  service_id: string;
  previous_version: number;
  current_version: number;
  status: string;
  message: string;
}

// ---------------------------------------------------------------------------
// Shared error types
// ---------------------------------------------------------------------------

/** Structured error body used across all endpoints. */
export interface ApiErrorBody {
  code: string;
  message: string;
  details?: string[];
}

/** Standard error envelope wrapping an ApiErrorBody. */
export interface ApiErrorEnvelope {
  error: ApiErrorBody;
}

/** Custom error class for Radium API failures. */
export class RadiumApiError extends Error {
  readonly status: number;
  readonly code: string;
  readonly details: string[];

  constructor(status: number, body: ApiErrorEnvelope) {
    super(body.error.message);
    this.name = 'RadiumApiError';
    this.status = status;
    this.code = body.error.code;
    this.details = body.error.details ?? [];
  }
}
