/**
 * Tests for the Radium API typed HTTP client.
 *
 * Since this is an HTTP client calling an external service, mocking `fetch` is
 * appropriate per project conventions. Mock validation tests are included to
 * verify the mocks match the expected API contract.
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { createRadiumApiClient } from '../client';
import { RadiumApiError } from '../types';
import type { RadiumApiClient } from '../client';

// ---------------------------------------------------------------------------
// Mock setup
// ---------------------------------------------------------------------------

const BASE_URL = 'http://localhost:3020';
const AUTH_TOKEN = 'test-token-abc123';

/** Helper to create a mock Response with JSON body. */
function mockJsonResponse(body: unknown, status = 200): Response {
  return {
    ok: status >= 200 && status < 300,
    status,
    statusText: status === 200 ? 'OK' : 'Error',
    json: () => Promise.resolve(body),
    text: () => Promise.resolve(JSON.stringify(body)),
    headers: new Headers(),
    redirected: false,
    type: 'basic',
    url: '',
    clone: () => mockJsonResponse(body, status),
    body: null,
    bodyUsed: false,
    arrayBuffer: () => Promise.resolve(new ArrayBuffer(0)),
    blob: () => Promise.resolve(new Blob()),
    formData: () => Promise.resolve(new FormData()),
    bytes: () => Promise.resolve(new Uint8Array()),
  } as Response;
}

/** Helper to create a 204 No Content response. */
function mockEmptyResponse(status = 204): Response {
  return mockJsonResponse(null, status);
}

let fetchSpy: ReturnType<typeof vi.spyOn>;

beforeEach(() => {
  fetchSpy = vi.spyOn(globalThis, 'fetch').mockResolvedValue(mockJsonResponse({}));
});

afterEach(() => {
  vi.restoreAllMocks();
});

// ---------------------------------------------------------------------------
// Mock validation tests
// ---------------------------------------------------------------------------

describe('mock validation', () => {
  it('mockJsonResponse returns a valid Response shape', () => {
    const res = mockJsonResponse({ test: true }, 200);
    expect(res.ok).toBe(true);
    expect(res.status).toBe(200);
    expect(typeof res.json).toBe('function');
    expect(typeof res.text).toBe('function');
  });

  it('mockJsonResponse returns non-ok for error status codes', () => {
    const res = mockJsonResponse({}, 404);
    expect(res.ok).toBe(false);
    expect(res.status).toBe(404);
  });

  it('mockEmptyResponse defaults to 204', () => {
    const res = mockEmptyResponse();
    expect(res.ok).toBe(true);
    expect(res.status).toBe(204);
  });

  it('mockJsonResponse json() resolves to the given body', async () => {
    const body = { workflow_id: 'wf-1', status: 'deployed' };
    const res = mockJsonResponse(body);
    const parsed = await res.json();
    expect(parsed).toEqual(body);
  });
});

// ---------------------------------------------------------------------------
// Auth header tests
// ---------------------------------------------------------------------------

describe('auth headers', () => {
  it('includes Authorization header when authToken is provided', async () => {
    fetchSpy.mockResolvedValueOnce(
      mockJsonResponse({ workflow_id: 'wf-1', deployment_status: 'draft', last_deployed_at: null }),
    );
    const client = createRadiumApiClient(BASE_URL, AUTH_TOKEN);
    await client.getStatus('wf-1');

    expect(fetchSpy).toHaveBeenCalledWith(
      `${BASE_URL}/v1/services/wf-1/status`,
      expect.objectContaining({
        headers: expect.objectContaining({
          Authorization: `Bearer ${AUTH_TOKEN}`,
        }),
      }),
    );
  });

  it('omits Authorization header when no authToken is provided', async () => {
    fetchSpy.mockResolvedValueOnce(
      mockJsonResponse({ workflow_id: 'wf-1', deployment_status: 'draft', last_deployed_at: null }),
    );
    const client = createRadiumApiClient(BASE_URL);
    await client.getStatus('wf-1');

    const callHeaders = (fetchSpy.mock.calls[0]?.[1] as RequestInit)?.headers as Record<string, string>;
    expect(callHeaders).not.toHaveProperty('Authorization');
  });

  it('gateway request does not include auth header', async () => {
    fetchSpy.mockResolvedValueOnce(
      mockJsonResponse({ status: 'accepted', request_id: 'req-1', message: 'Queued' }),
    );
    const client = createRadiumApiClient(BASE_URL, AUTH_TOKEN);
    await client.sendGatewayRequest('iface-1', { data: 'test' });

    const callHeaders = (fetchSpy.mock.calls[0]?.[1] as RequestInit)?.headers as Record<string, string>;
    expect(callHeaders).not.toHaveProperty('Authorization');
    expect(callHeaders['Content-Type']).toBe('application/json');
  });
});

// ---------------------------------------------------------------------------
// URL construction tests
// ---------------------------------------------------------------------------

describe('URL construction', () => {
  let client: RadiumApiClient;

  beforeEach(() => {
    client = createRadiumApiClient(BASE_URL, AUTH_TOKEN);
  });

  const urlCases: Array<{
    name: string;
    call: (c: RadiumApiClient) => Promise<unknown>;
    expectedUrl: string;
    expectedMethod: string;
  }> = [
    // Deploy
    {
      name: 'deploy',
      call: (c) => c.deploy('svc-1'),
      expectedUrl: `${BASE_URL}/v1/services/svc-1/deploy`,
      expectedMethod: 'POST',
    },
    {
      name: 'undeploy',
      call: (c) => c.undeploy('svc-1'),
      expectedUrl: `${BASE_URL}/v1/services/svc-1/undeploy`,
      expectedMethod: 'POST',
    },
    {
      name: 'getStatus',
      call: (c) => c.getStatus('svc-1'),
      expectedUrl: `${BASE_URL}/v1/services/svc-1/status`,
      expectedMethod: 'GET',
    },
    // Interfaces
    {
      name: 'createInterface',
      call: (c) =>
        c.createInterface('svc-1', {
          name: 'test',
          interface_type: 'signal',
        }),
      expectedUrl: `${BASE_URL}/v1/services/svc-1/interfaces`,
      expectedMethod: 'POST',
    },
    {
      name: 'listInterfaces',
      call: (c) => c.listInterfaces('svc-1'),
      expectedUrl: `${BASE_URL}/v1/services/svc-1/interfaces`,
      expectedMethod: 'GET',
    },
    {
      name: 'getInterface',
      call: (c) => c.getInterface('svc-1', 'iface-1'),
      expectedUrl: `${BASE_URL}/v1/services/svc-1/interfaces/iface-1`,
      expectedMethod: 'GET',
    },
    {
      name: 'updateInterface',
      call: (c) =>
        c.updateInterface('svc-1', 'iface-1', { name: 'updated' }),
      expectedUrl: `${BASE_URL}/v1/services/svc-1/interfaces/iface-1`,
      expectedMethod: 'PUT',
    },
    {
      name: 'deleteInterface',
      call: (c) => c.deleteInterface('svc-1', 'iface-1'),
      expectedUrl: `${BASE_URL}/v1/services/svc-1/interfaces/iface-1`,
      expectedMethod: 'DELETE',
    },
    {
      name: 'publishInterface',
      call: (c) => c.publishInterface('svc-1', 'iface-1'),
      expectedUrl: `${BASE_URL}/v1/services/svc-1/interfaces/iface-1/publish`,
      expectedMethod: 'POST',
    },
    {
      name: 'unpublishInterface',
      call: (c) => c.unpublishInterface('svc-1', 'iface-1'),
      expectedUrl: `${BASE_URL}/v1/services/svc-1/interfaces/iface-1/unpublish`,
      expectedMethod: 'POST',
    },
    // Gateway
    {
      name: 'sendGatewayRequest',
      call: (c) => c.sendGatewayRequest('iface-1', { key: 'val' }),
      expectedUrl: `${BASE_URL}/v1/gateway/iface-1`,
      expectedMethod: 'POST',
    },
    // Keys
    {
      name: 'createKey',
      call: (c) => c.createKey('iface-1', { name: 'my-key' }),
      expectedUrl: `${BASE_URL}/v1/interfaces/iface-1/keys`,
      expectedMethod: 'POST',
    },
    {
      name: 'listKeys',
      call: (c) => c.listKeys('iface-1'),
      expectedUrl: `${BASE_URL}/v1/interfaces/iface-1/keys`,
      expectedMethod: 'GET',
    },
    {
      name: 'revokeKey',
      call: (c) => c.revokeKey('iface-1', 'key-1'),
      expectedUrl: `${BASE_URL}/v1/interfaces/iface-1/keys/key-1`,
      expectedMethod: 'DELETE',
    },
    // Gateway config
    {
      name: 'getGatewayConfig',
      call: (c) => c.getGatewayConfig('iface-1'),
      expectedUrl: `${BASE_URL}/v1/interfaces/iface-1/config`,
      expectedMethod: 'GET',
    },
    {
      name: 'updateGatewayConfig',
      call: (c) =>
        c.updateGatewayConfig('iface-1', { rate_limit_per_minute: 60 }),
      expectedUrl: `${BASE_URL}/v1/interfaces/iface-1/config`,
      expectedMethod: 'PUT',
    },
    // Dead letters
    {
      name: 'listDeadLetters',
      call: (c) => c.listDeadLetters('iface-1'),
      expectedUrl: `${BASE_URL}/v1/interfaces/iface-1/dead-letters`,
      expectedMethod: 'GET',
    },
    {
      name: 'replayDeadLetter',
      call: (c) => c.replayDeadLetter('iface-1', 'dl-1'),
      expectedUrl: `${BASE_URL}/v1/interfaces/iface-1/dead-letters/dl-1/replay`,
      expectedMethod: 'POST',
    },
    // Versions
    {
      name: 'listVersions',
      call: (c) => c.listVersions('svc-1'),
      expectedUrl: `${BASE_URL}/v1/services/svc-1/versions`,
      expectedMethod: 'GET',
    },
    {
      name: 'getVersion',
      call: (c) => c.getVersion('svc-1', 'v-1'),
      expectedUrl: `${BASE_URL}/v1/services/svc-1/versions/v-1`,
      expectedMethod: 'GET',
    },
    {
      name: 'rollbackService',
      call: (c) => c.rollbackService('svc-1', { target_version: 2 }),
      expectedUrl: `${BASE_URL}/v1/services/svc-1/rollback`,
      expectedMethod: 'POST',
    },
  ];

  for (const tc of urlCases) {
    it(`${tc.name} calls ${tc.expectedMethod} ${tc.expectedUrl}`, async () => {
      // Return appropriate response shape for each method
      const isDelete = tc.expectedMethod === 'DELETE';
      fetchSpy.mockResolvedValueOnce(
        isDelete ? mockEmptyResponse() : mockJsonResponse({}),
      );

      await tc.call(client);

      expect(fetchSpy).toHaveBeenCalledWith(
        tc.expectedUrl,
        expect.objectContaining({ method: tc.expectedMethod }),
      );
    });
  }
});

// ---------------------------------------------------------------------------
// Request body serialization tests
// ---------------------------------------------------------------------------

describe('request body serialization', () => {
  let client: RadiumApiClient;

  beforeEach(() => {
    client = createRadiumApiClient(BASE_URL, AUTH_TOKEN);
  });

  it('createInterface serializes body as JSON', async () => {
    fetchSpy.mockResolvedValueOnce(mockJsonResponse({}));
    const body = { name: 'my-signal', interface_type: 'signal', is_public: true };
    await client.createInterface('svc-1', body);

    const callBody = (fetchSpy.mock.calls[0]?.[1] as RequestInit)?.body;
    expect(JSON.parse(callBody as string)).toEqual(body);
  });

  it('updateInterface serializes partial body', async () => {
    fetchSpy.mockResolvedValueOnce(mockJsonResponse({}));
    const body = { name: 'renamed' };
    await client.updateInterface('svc-1', 'iface-1', body);

    const callBody = (fetchSpy.mock.calls[0]?.[1] as RequestInit)?.body;
    expect(JSON.parse(callBody as string)).toEqual(body);
  });

  it('rollbackService serializes target_version', async () => {
    fetchSpy.mockResolvedValueOnce(mockJsonResponse({}));
    await client.rollbackService('svc-1', { target_version: 3 });

    const callBody = (fetchSpy.mock.calls[0]?.[1] as RequestInit)?.body;
    expect(JSON.parse(callBody as string)).toEqual({ target_version: 3 });
  });

  it('createKey serializes name and optional expires_at', async () => {
    fetchSpy.mockResolvedValueOnce(mockJsonResponse({}));
    const body = { name: 'prod-key', expires_at: '2027-01-01T00:00:00Z' };
    await client.createKey('iface-1', body);

    const callBody = (fetchSpy.mock.calls[0]?.[1] as RequestInit)?.body;
    expect(JSON.parse(callBody as string)).toEqual(body);
  });

  it('updateGatewayConfig serializes partial config', async () => {
    fetchSpy.mockResolvedValueOnce(mockJsonResponse({}));
    const body = { rate_limit_per_minute: 120, timeout_ms: 5000 };
    await client.updateGatewayConfig('iface-1', body);

    const callBody = (fetchSpy.mock.calls[0]?.[1] as RequestInit)?.body;
    expect(JSON.parse(callBody as string)).toEqual(body);
  });

  it('sendGatewayRequest sends payload or undefined body', async () => {
    // With payload
    fetchSpy.mockResolvedValueOnce(
      mockJsonResponse({ status: 'accepted', request_id: 'r-1', message: 'ok' }),
    );
    await client.sendGatewayRequest('iface-1', { order_id: 42 });

    const callBody = (fetchSpy.mock.calls[0]?.[1] as RequestInit)?.body;
    expect(JSON.parse(callBody as string)).toEqual({ order_id: 42 });

    // Without payload
    fetchSpy.mockResolvedValueOnce(
      mockJsonResponse({ status: 'accepted', request_id: 'r-2', message: 'ok' }),
    );
    await client.sendGatewayRequest('iface-1');

    const emptyBody = (fetchSpy.mock.calls[1]?.[1] as RequestInit)?.body;
    expect(emptyBody).toBeUndefined();
  });
});

// ---------------------------------------------------------------------------
// Response parsing tests
// ---------------------------------------------------------------------------

describe('response parsing', () => {
  let client: RadiumApiClient;

  beforeEach(() => {
    client = createRadiumApiClient(BASE_URL, AUTH_TOKEN);
  });

  it('deploy returns typed DeployResponse', async () => {
    const expected = {
      workflow_id: 'wf-1',
      status: 'deployed',
      compiled_at: '2026-03-07T00:00:00Z',
      message: 'Workflow compiled and deployed successfully',
    };
    fetchSpy.mockResolvedValueOnce(mockJsonResponse(expected));
    const result = await client.deploy('wf-1');

    expect(result.workflow_id).toBe('wf-1');
    expect(result.status).toBe('deployed');
    expect(result.compiled_at).toBe('2026-03-07T00:00:00Z');
    expect(result.message).toBe('Workflow compiled and deployed successfully');
  });

  it('getStatus returns typed StatusResponse', async () => {
    const expected = {
      workflow_id: 'wf-2',
      deployment_status: 'compiled',
      last_deployed_at: '2026-03-06T00:00:00Z',
    };
    fetchSpy.mockResolvedValueOnce(mockJsonResponse(expected));
    const result = await client.getStatus('wf-2');

    expect(result.workflow_id).toBe('wf-2');
    expect(result.deployment_status).toBe('compiled');
    expect(result.last_deployed_at).toBe('2026-03-06T00:00:00Z');
  });

  it('listInterfaces returns typed InterfaceListResponse', async () => {
    const expected = {
      interfaces: [
        {
          id: 'i-1',
          workflow_id: 'wf-1',
          name: 'my-signal',
          interface_type: 'signal',
          is_public: true,
          created_at: '2026-01-01T00:00:00Z',
          updated_at: '2026-01-01T00:00:00Z',
        },
      ],
      total: 1,
    };
    fetchSpy.mockResolvedValueOnce(mockJsonResponse(expected));
    const result = await client.listInterfaces('wf-1');

    expect(result.total).toBe(1);
    expect(result.interfaces).toHaveLength(1);
    expect(result.interfaces[0]?.name).toBe('my-signal');
  });
});

// ---------------------------------------------------------------------------
// Error handling tests
// ---------------------------------------------------------------------------

describe('error handling', () => {
  let client: RadiumApiClient;

  beforeEach(() => {
    client = createRadiumApiClient(BASE_URL, AUTH_TOKEN);
  });

  it('throws RadiumApiError on 404', async () => {
    const errorBody = {
      error: { code: 'NOT_FOUND', message: "Workflow 'wf-999' not found", details: [] },
    };
    fetchSpy.mockResolvedValueOnce(mockJsonResponse(errorBody, 404));

    await expect(client.getStatus('wf-999')).rejects.toThrow(RadiumApiError);
  });

  it('RadiumApiError exposes status, code, and details', async () => {
    const errorBody = {
      error: {
        code: 'VALIDATION_FAILED',
        message: 'Workflow validation failed',
        details: ['No start node'],
      },
    };
    fetchSpy.mockResolvedValueOnce(mockJsonResponse(errorBody, 422));

    try {
      await client.deploy('wf-bad');
      expect.fail('Should have thrown');
    } catch (err) {
      expect(err).toBeInstanceOf(RadiumApiError);
      const apiErr = err as RadiumApiError;
      expect(apiErr.status).toBe(422);
      expect(apiErr.code).toBe('VALIDATION_FAILED');
      expect(apiErr.message).toBe('Workflow validation failed');
      expect(apiErr.details).toEqual(['No start node']);
    }
  });

  it('throws RadiumApiError on 401 unauthorized', async () => {
    const errorBody = {
      error: {
        code: 'UNAUTHORIZED',
        message: 'Authorization header with Bearer token is required',
      },
    };
    fetchSpy.mockResolvedValueOnce(mockJsonResponse(errorBody, 401));

    await expect(client.deploy('wf-1')).rejects.toThrow(RadiumApiError);
  });

  it('handles non-JSON error responses gracefully', async () => {
    const res = {
      ok: false,
      status: 500,
      statusText: 'Internal Server Error',
      json: () => Promise.reject(new Error('not json')),
      text: () => Promise.resolve('plain text error'),
      headers: new Headers(),
      redirected: false,
      type: 'basic' as ResponseType,
      url: '',
      clone: () => res,
      body: null,
      bodyUsed: false,
      arrayBuffer: () => Promise.resolve(new ArrayBuffer(0)),
      blob: () => Promise.resolve(new Blob()),
      formData: () => Promise.resolve(new FormData()),
      bytes: () => Promise.resolve(new Uint8Array()),
    } as Response;

    fetchSpy.mockResolvedValueOnce(res);

    try {
      await client.deploy('wf-err');
      expect.fail('Should have thrown');
    } catch (err) {
      expect(err).toBeInstanceOf(RadiumApiError);
      const apiErr = err as RadiumApiError;
      expect(apiErr.status).toBe(500);
      expect(apiErr.code).toBe('UNKNOWN');
      expect(apiErr.message).toBe('plain text error');
    }
  });

  it('deleteInterface throws on error response', async () => {
    const errorBody = {
      error: { code: 'NOT_FOUND', message: 'Interface not found' },
    };
    fetchSpy.mockResolvedValueOnce(mockJsonResponse(errorBody, 404));

    await expect(
      client.deleteInterface('svc-1', 'iface-missing'),
    ).rejects.toThrow(RadiumApiError);
  });
});
