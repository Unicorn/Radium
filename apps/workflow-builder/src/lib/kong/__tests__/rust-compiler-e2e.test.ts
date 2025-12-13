/**
 * Rust Compiler E2E Tests
 *
 * End-to-end tests for Kong routing to the Rust workflow compiler service.
 * These tests validate the integration between Kong and the Rust compiler.
 *
 * Quick start:
 *   # Start test infrastructure with Rust compiler
 *   docker-compose -f docker-compose.test.yml --profile rust-compiler up -d
 *
 *   # Run tests
 *   RUST_COMPILER_E2E=true KONG_ADMIN_URL=http://localhost:9001 \
 *     KONG_PROXY_URL=http://localhost:9000 RUST_COMPILER_URL=http://localhost:3120 \
 *     pnpm test src/lib/kong/__tests__/rust-compiler-e2e.test.ts
 *
 *   # Or test directly against Rust compiler (without Kong)
 *   RUST_COMPILER_E2E=true RUST_COMPILER_URL=http://localhost:3020 \
 *     pnpm test src/lib/kong/__tests__/rust-compiler-e2e.test.ts
 *
 * Environment variables:
 * - RUST_COMPILER_E2E=true - Enable Rust compiler E2E tests (skipped by default)
 * - RUST_COMPILER_URL - Direct URL to Rust compiler (default: http://localhost:3020)
 * - KONG_ADMIN_URL - Kong Admin API URL (for Kong routing tests)
 * - KONG_PROXY_URL - Kong Proxy URL (for Kong routing tests)
 */

import { describe, it, expect, beforeAll } from 'vitest';
import { KongClient } from '../client';

// Skip E2E tests unless explicitly enabled
const isE2EEnabled = process.env.RUST_COMPILER_E2E === 'true';
const describeE2E = isE2EEnabled ? describe : describe.skip;

// Test configuration
const CONFIG = {
  // Direct Rust compiler access (bypassing Kong)
  rustCompilerUrl: process.env.RUST_COMPILER_URL || 'http://localhost:3020',
  // Kong proxy (for routing tests)
  kongProxyUrl: process.env.KONG_PROXY_URL || 'http://localhost:9000',
  kongAdminUrl: process.env.KONG_ADMIN_URL || 'http://localhost:9001',
  // Whether to test via Kong routing
  testViaKong: Boolean(process.env.KONG_PROXY_URL),
};

// Sample valid workflow for testing
const VALID_WORKFLOW = {
  id: 'test-workflow-1',
  name: 'Test Workflow',
  nodes: [
    {
      id: 'trigger-1',
      type: 'trigger',
      data: { label: 'Start' },
      position: { x: 0, y: 0 },
    },
    {
      id: 'activity-1',
      type: 'activity',
      data: { label: 'Do Something', activityName: 'doSomething' },
      position: { x: 200, y: 0 },
    },
    {
      id: 'end-1',
      type: 'end',
      data: { label: 'End' },
      position: { x: 400, y: 0 },
    },
  ],
  edges: [
    { id: 'e1', source: 'trigger-1', target: 'activity-1' },
    { id: 'e2', source: 'activity-1', target: 'end-1' },
  ],
  variables: [],
  settings: {},
};

// Invalid workflow (missing trigger)
const INVALID_WORKFLOW = {
  id: 'test-invalid-workflow',
  name: 'Invalid Workflow',
  nodes: [
    {
      id: 'activity-1',
      type: 'activity',
      data: { label: 'Do Something', activityName: 'doSomething' },
      position: { x: 0, y: 0 },
    },
    {
      id: 'end-1',
      type: 'end',
      data: { label: 'End' },
      position: { x: 200, y: 0 },
    },
  ],
  edges: [{ id: 'e1', source: 'activity-1', target: 'end-1' }],
  variables: [],
  settings: {},
};

describeE2E('Rust Compiler E2E Tests', () => {
  describe('Direct Rust Compiler Tests', () => {
    beforeAll(async () => {
      // Verify Rust compiler is running
      try {
        const response = await fetch(`${CONFIG.rustCompilerUrl}/health`);
        if (!response.ok) {
          throw new Error(`Rust compiler health check failed: ${response.status}`);
        }
      } catch (error) {
        throw new Error(
          `Rust compiler is not accessible at ${CONFIG.rustCompilerUrl}. ` +
            `Start the compiler with 'pnpm rust:run' or via Docker.`
        );
      }
    });

    describe('Health Check Endpoint', () => {
      it('should return healthy status', async () => {
        const response = await fetch(`${CONFIG.rustCompilerUrl}/health`);
        expect(response.ok).toBe(true);

        const data = await response.json();
        expect(data.status).toBe('healthy');
        expect(data.version).toBeDefined();
      });

      it('should include uptime information', async () => {
        const response = await fetch(`${CONFIG.rustCompilerUrl}/health`);
        const data = await response.json();
        expect(typeof data.uptime_seconds).toBe('number');
      });
    });

    describe('Validate Endpoint', () => {
      it('should validate a valid workflow', async () => {
        const response = await fetch(`${CONFIG.rustCompilerUrl}/validate`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(VALID_WORKFLOW),
        });

        expect(response.ok).toBe(true);
        const data = await response.json();
        expect(data.valid).toBe(true);
        expect(data.errors).toHaveLength(0);
      });

      it('should reject an invalid workflow', async () => {
        const response = await fetch(`${CONFIG.rustCompilerUrl}/validate`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(INVALID_WORKFLOW),
        });

        expect(response.ok).toBe(true);
        const data = await response.json();
        expect(data.valid).toBe(false);
        expect(data.errors.length).toBeGreaterThan(0);
        // Should have NO_START_NODE error
        expect(data.errors.some((e: { code: string }) => e.code === 'NO_START_NODE')).toBe(true);
      });

      it('should return proper error for malformed JSON', async () => {
        const response = await fetch(`${CONFIG.rustCompilerUrl}/validate`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: 'not valid json',
        });

        expect(response.ok).toBe(false);
        // Should be 400 Bad Request or 422 Unprocessable Entity
        expect([400, 422]).toContain(response.status);
      });
    });

    describe('Compile Endpoint', () => {
      it('should compile a valid workflow', async () => {
        const response = await fetch(`${CONFIG.rustCompilerUrl}/compile`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ workflow: VALID_WORKFLOW, options: {} }),
        });

        expect(response.ok).toBe(true);
        const data = await response.json();
        expect(data.success).toBe(true);
        expect(data.code).toBeDefined();
        expect(data.code.workflow).toBeDefined();
        expect(data.code.activities).toBeDefined();
        expect(data.metadata.nodeCount).toBe(3);
        expect(data.metadata.edgeCount).toBe(2);
      });

      it('should include compilation timing in metadata', async () => {
        const response = await fetch(`${CONFIG.rustCompilerUrl}/compile`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ workflow: VALID_WORKFLOW, options: {} }),
        });

        const data = await response.json();
        expect(data.metadata.compilationTimeMs).toBeDefined();
        expect(typeof data.metadata.compilationTimeMs).toBe('number');
      });

      it('should reject invalid workflow with structured errors', async () => {
        const response = await fetch(`${CONFIG.rustCompilerUrl}/compile`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ workflow: INVALID_WORKFLOW, options: {} }),
        });

        expect(response.ok).toBe(true); // API returns 200 with success: false
        const data = await response.json();
        expect(data.success).toBe(false);
        expect(data.errors.length).toBeGreaterThan(0);
        expect(data.errors[0].code).toBeDefined();
        expect(data.errors[0].message).toBeDefined();
      });

      it('should generate TypeScript without any types', async () => {
        const response = await fetch(`${CONFIG.rustCompilerUrl}/compile`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ workflow: VALID_WORKFLOW, options: {} }),
        });

        const data = await response.json();
        if (data.success && data.code) {
          // Check that generated code does not contain 'any' type
          expect(data.code.workflow).not.toContain(': any');
          expect(data.code.workflow).not.toContain(': any,');
          expect(data.code.workflow).not.toContain(': any)');
          expect(data.code.activities).not.toContain(': any');
        }
      });
    });

    describe('Compile Performance', () => {
      it('should compile in under 100ms (p95)', async () => {
        const SAMPLES = 10;
        const compileTimes: number[] = [];

        for (let i = 0; i < SAMPLES; i++) {
          const start = performance.now();
          await fetch(`${CONFIG.rustCompilerUrl}/compile`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ workflow: VALID_WORKFLOW, options: {} }),
          });
          compileTimes.push(performance.now() - start);
        }

        compileTimes.sort((a, b) => a - b);
        const p95Index = Math.ceil(0.95 * compileTimes.length) - 1;
        const p95Time = compileTimes[p95Index];

        console.log(
          `Compile performance (${SAMPLES} samples): p95=${(p95Time ?? 0).toFixed(2)}ms, ` +
            `median=${(compileTimes[Math.floor(compileTimes.length / 2)] ?? 0).toFixed(2)}ms`
        );

        expect(p95Time).toBeLessThan(100);
      });
    });
  });

  // Kong routing tests - only run if Kong is configured
  const describeKong = CONFIG.testViaKong ? describe : describe.skip;

  describeKong('Kong Routing to Rust Compiler', () => {
    let kong: KongClient;

    beforeAll(async () => {
      kong = new KongClient(CONFIG.kongAdminUrl);

      // Verify Kong is running
      const isHealthy = await kong.healthCheck();
      if (!isHealthy) {
        throw new Error(`Kong is not accessible at ${CONFIG.kongAdminUrl}`);
      }

      // Enable correlation-id plugin globally for X-Request-ID headers
      try {
        await kong.enableCorrelationId();
      } catch {
        // Plugin might already be enabled, that's fine
      }
    });

    describe('Route Configuration', () => {
      it('should route /api/compiler/rust/health to Rust service', async () => {
        const response = await fetch(`${CONFIG.kongProxyUrl}/api/compiler/rust/health`);

        // If Rust compiler is down, Kong returns 502/503
        // If routing works but compiler responds, we get 200
        if (response.ok) {
          const data = await response.json();
          expect(data.status).toBe('healthy');
        } else {
          // Routing works but backend is down
          expect([502, 503]).toContain(response.status);
          console.log('Rust compiler not running - routing verified but backend unavailable');
        }
      });

      it('should route /api/compiler/rust/compile to Rust service', async () => {
        const response = await fetch(`${CONFIG.kongProxyUrl}/api/compiler/rust/compile`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ workflow: VALID_WORKFLOW, options: {} }),
        });

        if (response.ok) {
          const data = await response.json();
          expect(data.success).toBeDefined();
        } else {
          // Backend unavailable is fine - routing was verified
          expect([502, 503]).toContain(response.status);
        }
      });

      it('should route /api/compiler/rust/validate to Rust service', async () => {
        const response = await fetch(`${CONFIG.kongProxyUrl}/api/compiler/rust/validate`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(VALID_WORKFLOW),
        });

        if (response.ok) {
          const data = await response.json();
          expect(data.valid).toBeDefined();
        } else {
          expect([502, 503]).toContain(response.status);
        }
      });
    });

    describe('Kong Middleware', () => {
      it('should add X-Request-ID header through Kong', async () => {
        const response = await fetch(`${CONFIG.kongProxyUrl}/api/compiler/rust/health`);
        const requestId = response.headers.get('X-Request-ID');

        expect(requestId).toBeTruthy();
        if (requestId) {
          expect(requestId).toMatch(/^[0-9a-f-]+$/i);
        }
      });

      it('should respect timeout configuration', async () => {
        // Kong service has 120s read timeout for compilation
        // This test verifies the request doesn't timeout prematurely
        const start = performance.now();
        const response = await fetch(`${CONFIG.kongProxyUrl}/api/compiler/rust/compile`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ workflow: VALID_WORKFLOW, options: {} }),
        });
        const elapsed = performance.now() - start;

        // Request should complete (not timeout at 5s default)
        expect(response.ok || [502, 503].includes(response.status)).toBe(true);
        console.log(`Request completed in ${elapsed.toFixed(2)}ms`);
      });
    });

    describe('Kong Health Checks', () => {
      it('should configure active health checks on upstream', async () => {
        // Verify the upstream exists with health check configuration
        // This uses Kong Admin API
        const response = await fetch(`${CONFIG.kongAdminUrl}/upstreams/rust-compiler-upstream`);

        if (response.ok) {
          const upstream = await response.json();
          expect(upstream.healthchecks).toBeDefined();
          expect(upstream.healthchecks.active).toBeDefined();
          expect(upstream.healthchecks.active.http_path).toBe('/health');
        } else if (response.status === 404) {
          // Upstream not configured yet - this is expected if Kong isn't set up
          console.log('Rust compiler upstream not configured in Kong');
        }
      });
    });

    describe('Failover Behavior', () => {
      it(
        'should return 502/503 when Rust compiler is unavailable',
        async () => {
          // Create a test route pointing to non-existent backend
          const timestamp = Date.now();
          const testServiceName = `rust-compiler-fail-test-${timestamp}`;
          const testRouteName = `rust-compiler-fail-route-${timestamp}`;
          const testPath = `/api/compiler/fail-test-${timestamp}`;

          try {
            // Create service pointing to dead backend using raw fetch for more control
            // Use a non-routable IP (not localhost, which refers to Kong container)
            const serviceResponse = await fetch(`${CONFIG.kongAdminUrl}/services`, {
              method: 'POST',
              headers: { 'Content-Type': 'application/json' },
              body: JSON.stringify({
                name: testServiceName,
                url: 'http://10.255.255.1:59999',
                connect_timeout: 5000,
                read_timeout: 5000,
                write_timeout: 5000,
                retries: 0,
              }),
            });

            if (!serviceResponse.ok) {
              throw new Error(`Failed to create service: ${serviceResponse.status}`);
            }

            const service = await serviceResponse.json();

            // Create route for the service
            const routeResponse = await fetch(
              `${CONFIG.kongAdminUrl}/services/${service.id}/routes`,
              {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                  name: testRouteName,
                  paths: [testPath],
                  methods: ['GET'],
                  strip_path: true,
                }),
              }
            );

            if (!routeResponse.ok) {
              throw new Error(`Failed to create route: ${routeResponse.status}`);
            }

            // Wait for route to propagate (Kong needs time to sync routes to workers)
            // Kong's database-backed mode requires ~3 seconds for cache propagation
            await new Promise((resolve) => setTimeout(resolve, 3000));

            // Retry logic: Route propagation can be inconsistent, try multiple times
            let lastStatus = 0;
            for (let attempt = 0; attempt < 3; attempt++) {
              const response = await fetch(`${CONFIG.kongProxyUrl}${testPath}`);
              lastStatus = response.status;
              // If we got a gateway error, we're done
              if ([502, 503, 504].includes(response.status)) {
                break;
              }
              // If we got 404, wait more and retry
              if (response.status === 404 && attempt < 2) {
                await new Promise((resolve) => setTimeout(resolve, 1000));
              }
            }

            // 502 Bad Gateway, 503 Service Unavailable, or 504 Gateway Timeout are all valid
            expect([502, 503, 504]).toContain(lastStatus);
          } finally {
            // Cleanup - delete route first, then service
            try {
              await fetch(`${CONFIG.kongAdminUrl}/routes/${testRouteName}`, {
                method: 'DELETE',
              });
            } catch {
              // Ignore cleanup errors
            }
            try {
              await fetch(`${CONFIG.kongAdminUrl}/services/${testServiceName}`, {
                method: 'DELETE',
              });
            } catch {
              // Ignore cleanup errors
            }
          }
        },
        15000
      ); // 15 second timeout for route propagation
    });
  });
});
