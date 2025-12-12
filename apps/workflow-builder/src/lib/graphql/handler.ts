/**
 * GraphQL Request Handler
 * Handles GraphQL requests and routes them to Temporal workflows
 */

import { graphql, GraphQLSchema } from 'graphql';
import type { SupabaseClient } from '@supabase/supabase-js';
import type { Database } from '@/types/database';
import { buildGraphQLSchema } from './schema-builder';
import {
  getWorkflowForQuery,
  getWorkflowForMutation,
  mapGraphQLInputToWorkflow,
  getResolverConfig,
} from './resolver-mapper';

export interface GraphQLRequest {
  query: string;
  variables?: Record<string, any>;
  operationName?: string;
}

export interface GraphQLResponse {
  data?: any;
  errors?: Array<{
    message: string;
    locations?: Array<{ line: number; column: number }>;
    path?: (string | number)[];
  }>;
}

/**
 * Create resolvers that execute Temporal workflows
 */
async function createResolvers(
  schema: GraphQLSchema,
  serviceInterfaceId: string,
  supabase: SupabaseClient<Database>,
  executeWorkflow: (workflowId: string, input: Record<string, any>) => Promise<any>
) {
  const resolverConfig = await getResolverConfig(serviceInterfaceId, supabase);
  
  if (!resolverConfig) {
    return {};
  }

  const resolvers: Record<string, any> = {
    Query: {},
    Mutation: {},
  };

  // Create query resolvers
  for (const query of resolverConfig.queries) {
    resolvers.Query[query.queryName] = async (_: any, args: Record<string, any>) => {
      const workflowInput = mapGraphQLInputToWorkflow(args, query.inputMapping);
      const result = await executeWorkflow(query.workflowId, workflowInput);
      return result;
    };
  }

  // Create mutation resolvers
  for (const mutation of resolverConfig.mutations) {
    resolvers.Mutation[mutation.mutationName] = async (_: any, args: Record<string, any>) => {
      const workflowInput = mapGraphQLInputToWorkflow(args, mutation.inputMapping);
      const result = await executeWorkflow(mutation.workflowId, workflowInput);
      return result;
    };
  }

  return resolvers;
}

/**
 * Handle a GraphQL request
 */
export async function handleGraphQLRequest(
  request: GraphQLRequest,
  serviceInterfaceId: string,
  schemaSDL: string,
  supabase: SupabaseClient<Database>,
  executeWorkflow: (workflowId: string, input: Record<string, any>) => Promise<any>
): Promise<GraphQLResponse> {
  try {
    // Build schema from SDL
    const schema = buildGraphQLSchema(schemaSDL);

    // Create resolvers (async function)
    const resolvers = await createResolvers(schema, serviceInterfaceId, supabase, executeWorkflow);

    // Execute GraphQL query
    const result = await graphql({
      schema,
      source: request.query,
      variableValues: request.variables,
      rootValue: resolvers,
    });

    return {
      data: result.data,
      errors: result.errors?.map(e => ({
        message: e.message,
        locations: e.locations?.map(l => ({ line: l.line, column: l.column })),
        path: e.path,
      })),
    };
  } catch (error) {
    return {
      errors: [{
        message: error instanceof Error ? error.message : 'Unknown error',
      }],
    };
  }
}

/**
 * Get GraphQL schema SDL from service interface
 */
export async function getGraphQLSchemaSDL(
  serviceInterfaceId: string,
  supabase: SupabaseClient<Database>
): Promise<string | null> {
  const { data: serviceInterface, error } = await supabase
    .from('service_interfaces')
    .select('graphql_schema')
    .eq('id', serviceInterfaceId)
    .single();

  if (error || !serviceInterface || !serviceInterface.graphql_schema) {
    return null;
  }

  const schema = serviceInterface.graphql_schema as any;
  return schema.sdl || null;
}

