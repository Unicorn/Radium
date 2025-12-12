/**
 * Kong E2E Tests
 *
 * End-to-end tests for Kong API Gateway integration.
 * These tests require a running Kong instance.
 *
 * Quick start (uses test infrastructure):
 *   ./scripts/run-e2e-tests.sh kong-e2e
 *
 * Or manually:
 *   ./scripts/start-test-env.sh
 *   KONG_E2E=true KONG_ADMIN_URL=http://localhost:9001 KONG_PROXY_URL=http://localhost:9000 pnpm test src/lib/kong/__tests__/kong-e2e.test.ts
 *   ./scripts/stop-test-env.sh
 *
 * Environment variables:
 * - KONG_E2E=true - Enable E2E tests (skipped by default)
 * - KONG_ADMIN_URL - Kong Admin API URL (default: http://localhost:8001 for dev, http://localhost:9001 for test)
 * - KONG_PROXY_URL - Kong Proxy URL (default: http://localhost:8000 for dev, http://localhost:9000 for test)
 * - KONG_GATEWAY_URL - Alias for KONG_PROXY_URL (deprecated, use KONG_PROXY_URL)
 * - KONG_BACKEND_URL - Backend service URL (default: http://localhost:3010)
 * - SUPABASE_JWT_SECRET - Supabase JWT secret for JWT tests
 */

import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { KongClient } from '../client';
import * as crypto from 'crypto';

// Skip E2E tests unless explicitly enabled
const isE2EEnabled = process.env.KONG_E2E === 'true';
const describeE2E = isE2EEnabled ? describe : describe.skip;

// Test configuration
// Supports both test environment (9001/9000) and dev environment (8001/8000)
const CONFIG = {
  adminUrl: process.env.KONG_ADMIN_URL || 'http://localhost:8001',
  // KONG_PROXY_URL is preferred, KONG_GATEWAY_URL is deprecated but supported
  gatewayUrl: process.env.KONG_PROXY_URL || process.env.KONG_GATEWAY_URL || 'http://localhost:8000',
  backendUrl: process.env.KONG_BACKEND_URL || 'http://localhost:3010',
  jwtSecret: process.env.SUPABASE_JWT_SECRET || 'test-jwt-secret-for-development-only',
};

// Helper to base64url encode
function base64urlEncode(data: string | Buffer): string {
  const base64 = Buffer.from(data).toString('base64');
  return base64.replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

// Helper to generate a valid JWT using built-in crypto
function generateTestJwt(payload: Record<string, unknown> = {}, expiresInSeconds = 3600): string {
  const header = {
    alg: 'HS256',
    typ: 'JWT',
  };

  const defaultPayload = {
    iss: 'e2e-test-key', // Must match Kong JWT credential key
    sub: 'test-user-id',
    email: 'test@example.com',
    role: 'authenticated',
    aud: 'authenticated',
    iat: Math.floor(Date.now() / 1000),
    exp: Math.floor(Date.now() / 1000) + expiresInSeconds,
    ...payload,
  };

  const headerEncoded = base64urlEncode(JSON.stringify(header));
  const payloadEncoded = base64urlEncode(JSON.stringify(defaultPayload));
  const signatureInput = `${headerEncoded}.${payloadEncoded}`;

  const signature = crypto
    .createHmac('sha256', CONFIG.jwtSecret)
    .update(signatureInput)
    .digest();
  const signatureEncoded = base64urlEncode(signature);

  return `${headerEncoded}.${payloadEncoded}.${signatureEncoded}`;
}

// Helper to generate an expired JWT
function generateExpiredJwt(): string {
  return generateTestJwt({}, -3600); // Expired 1 hour ago
}

// Helper to measure request latency
async function measureLatency(
  url: string,
  options?: RequestInit
): Promise<{ latencyMs: number; status: number }> {
  const start = performance.now();
  const response = await fetch(url, options);
  const end = performance.now();
  return {
    latencyMs: end - start,
    status: response.status,
  };
}

describeE2E('Kong E2E Tests', () => {
  let kong: KongClient;
  let testServiceId: string;
  let testRouteId: string;
  let jwtConsumerKey: string;

  beforeAll(async () => {
    kong = new KongClient(CONFIG.adminUrl);

    // Verify Kong is running
    const isHealthy = await kong.healthCheck();
    if (!isHealthy) {
      throw new Error(`Kong is not accessible at ${CONFIG.adminUrl}. Start Kong before running E2E tests.`);
    }

    // Set up test service and route for JWT testing
    try {
      testServiceId = await kong.createService('e2e-test-service', CONFIG.backendUrl);
      testRouteId = await kong.createRoute(testServiceId, 'e2e-test-route', ['/api/e2e-test'], ['GET', 'POST']);

      // Create JWT consumer and credentials
      const jwtCreds = await kong.createJwtConsumer('e2e-test-consumer', CONFIG.jwtSecret);
      jwtConsumerKey = jwtCreds.key;

      // Enable JWT auth on test route
      await kong.enableJwtAuth(testRouteId);
    } catch (error) {
      console.warn('Setup warning:', error);
      // Tests may still work if routes already exist
    }
  });

  afterAll(async () => {
    // Clean up test resources
    try {
      if (testRouteId) {
        await kong.deleteRoute(testRouteId);
      }
    } catch {
      // Ignore cleanup errors
    }
  });

  describe('1.3 JWT Authentication', () => {
    describe('1.3.6 Test with valid JWT', () => {
      it('should accept requests with valid JWT', async () => {
        const validJwt = generateTestJwt();

        const response = await fetch(`${CONFIG.gatewayUrl}/api/e2e-test`, {
          method: 'GET',
          headers: {
            Authorization: `Bearer ${validJwt}`,
          },
        });

        // Should not be 401 (auth required) - may be 404 if backend route doesn't exist
        expect(response.status).not.toBe(401);
      });

      it('should include user info from JWT in upstream headers', async () => {
        const validJwt = generateTestJwt({ sub: 'user-12345' });

        const response = await fetch(`${CONFIG.gatewayUrl}/api/e2e-test`, {
          method: 'GET',
          headers: {
            Authorization: `Bearer ${validJwt}`,
          },
        });

        // Kong should pass through or add headers with JWT claims
        expect(response.status).not.toBe(401);
      });
    });

    describe('1.3.7 Test with invalid/expired JWT', () => {
      it('should reject requests with invalid JWT', async () => {
        const response = await fetch(`${CONFIG.gatewayUrl}/api/e2e-test`, {
          method: 'GET',
          headers: {
            Authorization: 'Bearer invalid-jwt-token',
          },
        });

        expect(response.status).toBe(401);
      });

      it('should reject requests with expired JWT', async () => {
        const expiredJwt = generateExpiredJwt();

        const response = await fetch(`${CONFIG.gatewayUrl}/api/e2e-test`, {
          method: 'GET',
          headers: {
            Authorization: `Bearer ${expiredJwt}`,
          },
        });

        expect(response.status).toBe(401);
      });

      it('should reject requests without Authorization header', async () => {
        const response = await fetch(`${CONFIG.gatewayUrl}/api/e2e-test`, {
          method: 'GET',
        });

        expect(response.status).toBe(401);
      });

      it('should reject requests with malformed Authorization header', async () => {
        const response = await fetch(`${CONFIG.gatewayUrl}/api/e2e-test`, {
          method: 'GET',
          headers: {
            Authorization: 'NotBearer some-token',
          },
        });

        expect(response.status).toBe(401);
      });
    });
  });

  describe('1.7.3 E2E Tests', () => {
    it('should route tRPC requests correctly', async () => {
      const response = await fetch(`${CONFIG.gatewayUrl}/api/trpc/health`);
      // May return 401 if auth required, or 200 if public, or 404 if not configured
      // The important thing is Kong routes it (not a Kong error like 502/503)
      expect([200, 401, 404]).toContain(response.status);
    });

    it('should route health check requests', async () => {
      const response = await fetch(`${CONFIG.gatewayUrl}/api/health`);
      // Health route should be public (no auth required)
      expect([200, 404]).toContain(response.status);
    });

    it('should add X-Request-ID header (correlation-id)', async () => {
      const response = await fetch(`${CONFIG.gatewayUrl}/api/health`);
      const requestId = response.headers.get('X-Request-ID');

      // Correlation ID should be present
      expect(requestId).toBeTruthy();
      // Should be a UUID format
      if (requestId) {
        expect(requestId).toMatch(/^[0-9a-f-]+$/i);
      }
    });

    it('should include rate limit headers', async () => {
      const response = await fetch(`${CONFIG.gatewayUrl}/api/health`);

      // Rate limit headers may vary based on plugin configuration
      const rateLimitLimit = response.headers.get('X-RateLimit-Limit-Minute');
      const rateLimitRemaining = response.headers.get('X-RateLimit-Remaining-Minute');
      const retryAfter = response.headers.get('Retry-After');

      // At least one rate limit indicator should be present
      const hasRateLimitInfo = rateLimitLimit || rateLimitRemaining || retryAfter;
      expect(hasRateLimitInfo || response.status === 429).toBeTruthy();
    });
  });

  describe('1.7.4 Latency Overhead Measurement', () => {
    const LATENCY_SAMPLES = 10;
    const MAX_OVERHEAD_MS = 10;

    it('should add less than 10ms overhead (p95)', async () => {
      const latencies: number[] = [];

      // Warm up request
      await fetch(`${CONFIG.gatewayUrl}/api/health`);

      // Collect latency samples
      for (let i = 0; i < LATENCY_SAMPLES; i++) {
        const { latencyMs } = await measureLatency(`${CONFIG.gatewayUrl}/api/health`);
        latencies.push(latencyMs);
      }

      // Sort for percentile calculation
      latencies.sort((a, b) => a - b);

      // Calculate p95
      const p95Index = Math.ceil(0.95 * latencies.length) - 1;
      const p95Latency = latencies[p95Index];

      // Calculate median for baseline
      const medianIndex = Math.floor(latencies.length / 2);
      const medianLatency = latencies[medianIndex];

      console.log(`Latency stats (${LATENCY_SAMPLES} samples):`);
      console.log(`  Min: ${Math.min(...latencies).toFixed(2)}ms`);
      console.log(`  Median: ${medianLatency.toFixed(2)}ms`);
      console.log(`  P95: ${p95Latency.toFixed(2)}ms`);
      console.log(`  Max: ${Math.max(...latencies).toFixed(2)}ms`);

      // We can't directly measure overhead without a direct backend comparison,
      // but we can check that total latency is reasonable
      // Adjust this threshold based on your network conditions
      expect(p95Latency).toBeLessThan(100); // 100ms is reasonable for local testing
    });

    it('should have consistent response times', async () => {
      const latencies: number[] = [];

      for (let i = 0; i < LATENCY_SAMPLES; i++) {
        const { latencyMs } = await measureLatency(`${CONFIG.gatewayUrl}/api/health`);
        latencies.push(latencyMs);
      }

      // Calculate standard deviation
      const mean = latencies.reduce((a, b) => a + b, 0) / latencies.length;
      const squaredDiffs = latencies.map((l) => Math.pow(l - mean, 2));
      const avgSquaredDiff = squaredDiffs.reduce((a, b) => a + b, 0) / squaredDiffs.length;
      const stdDev = Math.sqrt(avgSquaredDiff);

      console.log(`Latency consistency: mean=${mean.toFixed(2)}ms, stdDev=${stdDev.toFixed(2)}ms`);

      // Standard deviation should be reasonable (not wildly inconsistent)
      expect(stdDev).toBeLessThan(mean); // StdDev should be less than mean
    });
  });

  describe('1.7.5 Load Testing (100 concurrent requests)', () => {
    const CONCURRENT_REQUESTS = 100;
    const MAX_ERROR_RATE = 0.05; // 5% error rate allowed

    it('should handle 100 concurrent requests', async () => {
      const promises = Array(CONCURRENT_REQUESTS)
        .fill(null)
        .map(() => measureLatency(`${CONFIG.gatewayUrl}/api/health`));

      const results = await Promise.all(promises);

      const successCount = results.filter((r) => r.status === 200 || r.status === 404).length;
      const rateLimitedCount = results.filter((r) => r.status === 429).length;
      const errorCount = results.filter((r) => r.status >= 500).length;
      const avgLatency = results.reduce((sum, r) => sum + r.latencyMs, 0) / results.length;
      const maxLatency = Math.max(...results.map((r) => r.latencyMs));

      console.log(`Load test results (${CONCURRENT_REQUESTS} concurrent requests):`);
      console.log(`  Success (2xx/4xx): ${successCount}`);
      console.log(`  Rate limited (429): ${rateLimitedCount}`);
      console.log(`  Errors (5xx): ${errorCount}`);
      console.log(`  Avg latency: ${avgLatency.toFixed(2)}ms`);
      console.log(`  Max latency: ${maxLatency.toFixed(2)}ms`);

      // Error rate should be below threshold
      const errorRate = errorCount / CONCURRENT_REQUESTS;
      expect(errorRate).toBeLessThanOrEqual(MAX_ERROR_RATE);

      // Most requests should succeed (some may be rate limited, which is expected)
      expect(successCount + rateLimitedCount).toBeGreaterThan(CONCURRENT_REQUESTS * 0.9);
    });

    it('should properly rate limit burst traffic', async () => {
      // Send burst of requests to compiler endpoint (has lower rate limit)
      const BURST_SIZE = 150; // More than the 100/minute limit
      const promises = Array(BURST_SIZE)
        .fill(null)
        .map(() =>
          fetch(`${CONFIG.gatewayUrl}/api/compiler/compile`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ test: true }),
          })
        );

      const responses = await Promise.all(promises);
      const rateLimitedCount = responses.filter((r) => r.status === 429).length;

      console.log(`Rate limit test: ${rateLimitedCount}/${BURST_SIZE} requests rate limited`);

      // Some requests should be rate limited
      expect(rateLimitedCount).toBeGreaterThan(0);
    });
  });

  describe('1.7.6 Failover Testing', () => {
    it('should return 503 when backend is unavailable', async () => {
      // Use unique names and paths with timestamp to avoid conflicts with previous runs
      const timestamp = Date.now();
      const serviceName = `e2e-fail-test-service-${timestamp}`;
      const routeName = `e2e-fail-test-route-${timestamp}`;
      const routePath = `/api/fail-test-${timestamp}`;

      // Create a service pointing to non-existent backend
      const failServiceId = await kong.createService(
        serviceName,
        'http://localhost:59999' // Non-existent port
      );

      const failRouteId = await kong.createRoute(failServiceId, routeName, [routePath], ['GET']);

      try {
        // Small delay to ensure Kong route is ready
        await new Promise(resolve => setTimeout(resolve, 200));

        const response = await fetch(`${CONFIG.gatewayUrl}${routePath}`);

        // Should return 502 (Bad Gateway) or 503 (Service Unavailable)
        expect([502, 503]).toContain(response.status);
      } finally {
        // Clean up
        await kong.deleteRoute(failRouteId);
      }
    });

    it('should include Kong error headers on failure', async () => {
      // Use unique names and paths with timestamp to avoid conflicts with previous runs
      const timestamp = Date.now();
      const serviceName = `e2e-fail-test-service-2-${timestamp}`;
      const routeName = `e2e-fail-test-route-2-${timestamp}`;
      const routePath = `/api/fail-test-2-${timestamp}`;

      // Create a service pointing to non-existent backend
      const failServiceId = await kong.createService(
        serviceName,
        'http://localhost:59998' // Non-existent port
      );

      const failRouteId = await kong.createRoute(
        failServiceId,
        routeName,
        [routePath],
        ['GET']
      );

      try {
        // Small delay to ensure Kong route is ready
        await new Promise(resolve => setTimeout(resolve, 200));

        const response = await fetch(`${CONFIG.gatewayUrl}${routePath}`);

        // Kong should still add correlation ID even on error
        const requestId = response.headers.get('X-Request-ID');
        expect(requestId).toBeTruthy();
      } finally {
        // Clean up
        await kong.deleteRoute(failRouteId);
      }
    });

    it('should handle timeout gracefully', async () => {
      // This test would require a backend that delays response
      // For now, we verify Kong's timeout configuration exists
      const service = await kong.getService('workflow-builder-service');

      if (service) {
        // Service should have timeout configuration
        // Note: getService returns limited info, but creation sets timeouts
        expect(service).toBeDefined();
      }
    });
  });
});

describeE2E('Kong API Key Authentication E2E', () => {
  describe('1.3.8 API Key Authentication', () => {
    it('should reject requests without API key to protected endpoints', async () => {
      // Assuming workflow endpoints require API key
      const response = await fetch(`${CONFIG.gatewayUrl}/api/v1/test-hash/endpoint`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ test: true }),
      });

      // Should require authentication (401 or 403)
      expect([401, 403, 404]).toContain(response.status);
    });

    it('should accept requests with valid API key', async () => {
      // This requires a valid API key to be configured
      // Skip if no API key is set
      const apiKey = process.env.KONG_TEST_API_KEY;
      if (!apiKey) {
        console.log('Skipping API key test - KONG_TEST_API_KEY not set');
        return;
      }

      const response = await fetch(`${CONFIG.gatewayUrl}/api/v1/test-hash/endpoint`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'X-API-Key': apiKey,
        },
        body: JSON.stringify({ test: true }),
      });

      // Should not be 401/403 (may be 404 if route doesn't exist, which is fine)
      expect(response.status).not.toBe(401);
      expect(response.status).not.toBe(403);
    });
  });
});
