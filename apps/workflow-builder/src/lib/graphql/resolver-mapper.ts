/**
 * GraphQL Resolver Mapper
 * Maps GraphQL queries and mutations to Temporal workflows
 */

import type { SupabaseClient } from '@supabase/supabase-js';
import type { Database } from '@/types/database';

export interface ResolverMapping {
  queryName: string;
  workflowId: string;
  inputMapping?: Record<string, string>; // Maps GraphQL input fields to workflow input fields
}

export interface MutationMapping {
  mutationName: string;
  workflowId: string;
  inputMapping?: Record<string, string>;
}

export interface GraphQLResolverConfig {
  queries: ResolverMapping[];
  mutations: MutationMapping[];
}

/**
 * Get resolver configuration for a GraphQL gateway component
 */
export async function getResolverConfig(
  serviceInterfaceId: string,
  supabase: SupabaseClient<Database>
): Promise<GraphQLResolverConfig | null> {
  const { data: serviceInterface, error } = await supabase
    .from('service_interfaces')
    .select('graphql_schema')
    .eq('id', serviceInterfaceId)
    .single();

  if (error || !serviceInterface || !serviceInterface.graphql_schema) {
    return null;
  }

  const schema = serviceInterface.graphql_schema as any;

  return {
    queries: schema.queries || [],
    mutations: schema.mutations || [],
  };
}

/**
 * Find workflow ID for a GraphQL query
 */
export async function getWorkflowForQuery(
  queryName: string,
  serviceInterfaceId: string,
  supabase: SupabaseClient<Database>
): Promise<string | null> {
  const config = await getResolverConfig(serviceInterfaceId, supabase);
  
  if (!config) {
    return null;
  }

  const query = config.queries.find(q => q.queryName === queryName);
  return query?.workflowId || null;
}

/**
 * Find workflow ID for a GraphQL mutation
 */
export async function getWorkflowForMutation(
  mutationName: string,
  serviceInterfaceId: string,
  supabase: SupabaseClient<Database>
): Promise<string | null> {
  const config = await getResolverConfig(serviceInterfaceId, supabase);
  
  if (!config) {
    return null;
  }

  const mutation = config.mutations.find(m => m.mutationName === mutationName);
  return mutation?.workflowId || null;
}

/**
 * Map GraphQL input to workflow input using input mapping
 */
export function mapGraphQLInputToWorkflow(
  graphQLInput: Record<string, any>,
  inputMapping?: Record<string, string>
): Record<string, any> {
  if (!inputMapping) {
    return graphQLInput;
  }

  const workflowInput: Record<string, any> = {};
  
  for (const [graphQLField, workflowField] of Object.entries(inputMapping)) {
    if (graphQLInput[graphQLField] !== undefined) {
      workflowInput[workflowField] = graphQLInput[graphQLField];
    }
  }

  return workflowInput;
}

