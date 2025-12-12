/**
 * Kong Compiler Patterns Tests
 * Tests for Kong component compiler patterns
 */

import { describe, it, expect } from 'vitest';
import { KongLoggingPattern } from '../kong-logging';
import { KongCachePattern } from '../kong-cache';
import { KongCorsPattern } from '../kong-cors';
import { GraphQLGatewayPattern } from '../graphql-gateway';
import { MCPServerPattern } from '../mcp-server';
import type { WorkflowNode, GeneratorContext } from '../../types';
import type { WorkflowDefinition } from '../../types';

/**
 * Create a test context for pattern testing
 */
function createTestContext(): GeneratorContext {
  const workflow: WorkflowDefinition = {
    id: 'test-workflow',
    name: 'TestWorkflow',
    nodes: [],
    edges: [],
    variables: [],
    settings: {},
  };

  return {
    nodes: workflow.nodes,
    edges: workflow.edges,
    variables: workflow.variables,
    settings: workflow.settings,
    nodeMap: new Map(),
    edgeMap: new Map(),
    visitedNodes: new Set(),
    resultVars: new Map(),
    proxyMap: new Map(),
    currentIndent: 0,
    workflowName: workflow.name,
  };
}

describe('KongLoggingPattern', () => {
  describe('detect', () => {
    it('should detect kong-logging nodes', () => {
      const context = createTestContext();
      const node: WorkflowNode = {
        id: 'logging-1',
        type: 'kong-logging',
        data: { label: 'Kong Logging' },
        position: { x: 0, y: 0 },
      };

      expect(KongLoggingPattern.detect(node, context)).toBe(true);
    });

    it('should not detect other node types', () => {
      const context = createTestContext();
      const node: WorkflowNode = {
        id: 'activity-1',
        type: 'activity',
        data: { label: 'Activity' },
        position: { x: 0, y: 0 },
      };

      expect(KongLoggingPattern.detect(node, context)).toBe(false);
    });
  });

  describe('generate', () => {
    it('should generate logging configuration code', () => {
      const context = createTestContext();
      const node: WorkflowNode = {
        id: 'logging-1',
        type: 'kong-logging',
        data: {
          label: 'Project Logging',
          componentName: 'project-logging',
          config: {
            connectorName: 'http-log-connector',
            connectorId: 'conn-123',
            enabledEndpoints: ['endpoint-1', 'endpoint-2'],
          },
        },
        position: { x: 0, y: 0 },
      };

      const result = KongLoggingPattern.generate(node, context);

      expect(result.code).toContain('Kong Logging Configuration');
      expect(result.code).toContain('Project Logging');
      expect(result.code).toContain('http-log-connector');
      expect(result.code).toContain('Enabled endpoints: 2');
      expect(result.code).toContain('Project-level logging component');
      expect(result.code).toContain('kong-logging-config');
      expect(result.resultVar).toBeDefined();
    });

    it('should handle missing connector configuration', () => {
      const context = createTestContext();
      const node: WorkflowNode = {
        id: 'logging-1',
        type: 'kong-logging',
        data: {
          label: 'Logging',
          config: {},
        },
        position: { x: 0, y: 0 },
      };

      const result = KongLoggingPattern.generate(node, context);

      expect(result.code).toContain('Not configured');
      expect(result.code).toContain('Enabled endpoints: 0');
    });
  });
});

describe('KongCachePattern', () => {
  describe('detect', () => {
    it('should detect kong-cache nodes', () => {
      const context = createTestContext();
      const node: WorkflowNode = {
        id: 'cache-1',
        type: 'kong-cache',
        data: { label: 'Kong Cache' },
        position: { x: 0, y: 0 },
      };

      expect(KongCachePattern.detect(node, context)).toBe(true);
    });
  });

  describe('generate', () => {
    it('should generate cache configuration code', () => {
      const context = createTestContext();
      const node: WorkflowNode = {
        id: 'cache-1',
        type: 'kong-cache',
        data: {
          label: 'API Cache',
          componentName: 'api-cache',
          config: {
            cacheKey: 'cache-abc123',
            connectorName: 'redis-connector',
            connectorId: 'conn-456',
            ttlSeconds: 7200,
            cacheKeyStrategy: 'query-string',
            contentTypes: ['application/json'],
            responseCodes: ['200', '201'],
            isSaved: true,
          },
        },
        position: { x: 0, y: 0 },
      };

      const result = KongCachePattern.generate(node, context);

      expect(result.code).toContain('Kong Cache Configuration');
      expect(result.code).toContain('API Cache');
      expect(result.code).toContain('cache-abc123');
      expect(result.code).toContain('(immutable)');
      expect(result.code).toContain('redis-connector');
      expect(result.code).toContain('TTL: 7200 seconds');
      expect(result.code).toContain('Cache Key Strategy: query-string');
      expect(result.code).toContain('kong-cache-config');
      expect(result.resultVar).toBeDefined();
    });

    it('should handle cache key deletion marker', () => {
      const context = createTestContext();
      const node: WorkflowNode = {
        id: 'cache-1',
        type: 'kong-cache',
        data: {
          label: 'Cache',
          config: {
            cacheKey: 'cache-xyz',
            markedForDeletion: true,
          },
        },
        position: { x: 0, y: 0 },
      };

      const result = KongCachePattern.generate(node, context);

      expect(result.code).toContain('marked for deletion');
      expect(result.code).toContain('removed from Redis');
    });

    it('should show editable status for unsaved cache keys', () => {
      const context = createTestContext();
      const node: WorkflowNode = {
        id: 'cache-1',
        type: 'kong-cache',
        data: {
          label: 'Cache',
          config: {
            cacheKey: 'cache-new',
            isSaved: false,
          },
        },
        position: { x: 0, y: 0 },
      };

      const result = KongCachePattern.generate(node, context);

      expect(result.code).toContain('(editable until save)');
    });
  });
});

describe('KongCorsPattern', () => {
  describe('detect', () => {
    it('should detect kong-cors nodes', () => {
      const context = createTestContext();
      const node: WorkflowNode = {
        id: 'cors-1',
        type: 'kong-cors',
        data: { label: 'Kong CORS' },
        position: { x: 0, y: 0 },
      };

      expect(KongCorsPattern.detect(node, context)).toBe(true);
    });
  });

  describe('generate', () => {
    it('should generate CORS configuration code', () => {
      const context = createTestContext();
      const node: WorkflowNode = {
        id: 'cors-1',
        type: 'kong-cors',
        data: {
          label: 'CORS Config',
          componentName: 'cors-config',
          config: {
            allowedOrigins: ['https://example.com', 'https://app.example.com'],
            allowedMethods: ['GET', 'POST', 'PUT'],
            allowedHeaders: ['Content-Type', 'Authorization'],
            exposedHeaders: ['X-Total-Count'],
            allowCredentials: true,
            maxAge: 86400,
            preflightContinue: false,
          },
        },
        position: { x: 0, y: 0 },
      };

      const result = KongCorsPattern.generate(node, context);

      expect(result.code).toContain('Kong CORS Configuration');
      expect(result.code).toContain('CORS Config');
      expect(result.code).toContain('https://example.com');
      expect(result.code).toContain('GET, POST, PUT');
      expect(result.code).toContain('Content-Type, Authorization');
      expect(result.code).toContain('X-Total-Count');
      expect(result.code).toContain('Allow Credentials: true');
      expect(result.code).toContain('Max Age: 86400 seconds');
      expect(result.code).toContain('kong-cors-config');
      expect(result.resultVar).toBeDefined();
    });

    it('should use default values when config is missing', () => {
      const context = createTestContext();
      const node: WorkflowNode = {
        id: 'cors-1',
        type: 'kong-cors',
        data: {
          label: 'CORS',
          config: {},
        },
        position: { x: 0, y: 0 },
      };

      const result = KongCorsPattern.generate(node, context);

      expect(result.code).toContain('*'); // Default origin
      expect(result.code).toContain('GET, POST, PUT, PATCH, DELETE, OPTIONS'); // Default methods
      expect(result.code).toContain('Allow Credentials: false');
    });
  });
});

describe('GraphQLGatewayPattern', () => {
  describe('detect', () => {
    it('should detect graphql-gateway nodes', () => {
      const context = createTestContext();
      const node: WorkflowNode = {
        id: 'graphql-1',
        type: 'graphql-gateway',
        data: { label: 'GraphQL Gateway' },
        position: { x: 0, y: 0 },
      };

      expect(GraphQLGatewayPattern.detect(node, context)).toBe(true);
    });
  });

  describe('generate', () => {
    it('should generate GraphQL gateway configuration code', () => {
      const context = createTestContext();
      const node: WorkflowNode = {
        id: 'graphql-1',
        type: 'graphql-gateway',
        data: {
          label: 'GraphQL API',
          componentName: 'graphql-api',
          config: {
            endpointPath: '/api/graphql',
            schema: 'type Query { users: [User] }',
            queries: [
              { name: 'getUsers', workflowId: 'wf-123' },
              { name: 'getPosts', workflowId: 'wf-456' },
            ],
            mutations: [
              { name: 'createUser', workflowId: 'wf-789' },
            ],
          },
        },
        position: { x: 0, y: 0 },
      };

      const result = GraphQLGatewayPattern.generate(node, context);

      expect(result.code).toContain('GraphQL Gateway');
      expect(result.code).toContain('GraphQL API');
      expect(result.code).toContain('/api/graphql');
      expect(result.code).toContain('type Query { users: [User] }');
      expect(result.code).toContain('Queries (2):');
      expect(result.code).toContain('Mutations (1):');
      expect(result.code).toContain('getUsers -> Workflow: wf-123');
      expect(result.code).toContain('createUser -> Workflow: wf-789');
      expect(result.code).toContain('graphql-gateway');
      expect(result.imports).toContain("import { GraphQLSchema, GraphQLObjectType, GraphQLString } from 'graphql';");
      expect(result.resultVar).toBeDefined();
    });

    it('should handle missing schema and queries', () => {
      const context = createTestContext();
      const node: WorkflowNode = {
        id: 'graphql-1',
        type: 'graphql-gateway',
        data: {
          label: 'GraphQL',
          config: {
            endpointPath: '/graphql',
          },
        },
        position: { x: 0, y: 0 },
      };

      const result = GraphQLGatewayPattern.generate(node, context);

      expect(result.code).toContain('GraphQL Schema: Not configured');
      // When there are no queries/mutations, the pattern doesn't output those lines
      // but the result object contains the counts
      expect(result.code).toContain('queries: 0, mutations: 0');
    });
  });
});

describe('MCPServerPattern', () => {
  describe('detect', () => {
    it('should detect mcp-server nodes', () => {
      const context = createTestContext();
      const node: WorkflowNode = {
        id: 'mcp-1',
        type: 'mcp-server',
        data: { label: 'MCP Server' },
        position: { x: 0, y: 0 },
      };

      expect(MCPServerPattern.detect(node, context)).toBe(true);
    });
  });

  describe('generate', () => {
    it('should generate MCP server configuration code', () => {
      const context = createTestContext();
      const node: WorkflowNode = {
        id: 'mcp-1',
        type: 'mcp-server',
        data: {
          label: 'MCP API Server',
          componentName: 'mcp-api',
          config: {
            serverName: 'my-mcp-server',
            version: '1.2.0',
            endpointPath: '/api/mcp',
            resources: [
              { name: 'resource1', uri: 'resource://1', workflowId: 'wf-1' },
              { name: 'resource2', uri: 'resource://2', workflowId: 'wf-2' },
            ],
            tools: [
              {
                name: 'tool1',
                workflowId: 'wf-3',
                inputSchema: { type: 'object', properties: { name: { type: 'string' } } },
              },
            ],
          },
        },
        position: { x: 0, y: 0 },
      };

      const result = MCPServerPattern.generate(node, context);

      expect(result.code).toContain('MCP Server');
      expect(result.code).toContain('MCP API Server');
      expect(result.code).toContain('my-mcp-server');
      expect(result.code).toContain('1.2.0');
      expect(result.code).toContain('/api/mcp');
      expect(result.code).toContain('Resources (2):');
      expect(result.code).toContain('Tools (1):');
      expect(result.code).toContain('resource1 -> Workflow: wf-1');
      expect(result.code).toContain('tool1 -> Workflow: wf-3');
      expect(result.code).toContain('mcp-server');
      expect(result.resultVar).toBeDefined();
    });

    it('should handle missing resources and tools', () => {
      const context = createTestContext();
      const node: WorkflowNode = {
        id: 'mcp-1',
        type: 'mcp-server',
        data: {
          label: 'MCP',
          config: {
            serverName: 'server',
            version: '1.0.0',
            endpointPath: '/mcp',
          },
        },
        position: { x: 0, y: 0 },
      };

      const result = MCPServerPattern.generate(node, context);

      // When there are no resources/tools, the pattern doesn't output those lines
      // but the result object contains the counts
      expect(result.code).toContain('MCP Server: MCP');
      expect(result.code).toContain('Server Name: server');
      expect(result.code).toContain('resources: 0, tools: 0');
    });
  });
});

