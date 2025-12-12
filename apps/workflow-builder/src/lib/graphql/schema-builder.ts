/**
 * GraphQL Schema Builder
 * Builds and validates GraphQL schemas from SDL strings
 */

import { buildSchema, GraphQLSchema, GraphQLError, validateSchema } from 'graphql';

export interface GraphQLSchemaConfig {
  sdl: string;
  queries?: Array<{
    name: string;
    workflowId: string;
    inputMapping?: Record<string, string>;
  }>;
  mutations?: Array<{
    name: string;
    workflowId: string;
    inputMapping?: Record<string, string>;
  }>;
}

/**
 * Build a GraphQL schema from SDL
 */
export function buildGraphQLSchema(sdl: string): GraphQLSchema {
  try {
    const schema = buildSchema(sdl);
    
    // Validate the schema
    const errors = validateSchema(schema);
    if (errors.length > 0) {
      throw new Error(`GraphQL schema validation failed: ${errors.map(e => e.message).join(', ')}`);
    }
    
    return schema;
  } catch (error) {
    if (error instanceof GraphQLError) {
      throw new Error(`GraphQL schema error: ${error.message}`);
    }
    throw error;
  }
}

/**
 * Validate GraphQL SDL string
 */
export function validateGraphQLSDL(sdl: string): { valid: boolean; errors?: string[] } {
  try {
    const schema = buildSchema(sdl);
    const errors = validateSchema(schema);
    
    if (errors.length > 0) {
      return {
        valid: false,
        errors: errors.map(e => e.message),
      };
    }
    
    return { valid: true };
  } catch (error) {
    return {
      valid: false,
      errors: [error instanceof Error ? error.message : 'Unknown error'],
    };
  }
}

/**
 * Extract query and mutation names from SDL
 */
export function extractOperationNames(sdl: string): {
  queries: string[];
  mutations: string[];
} {
  const queries: string[] = [];
  const mutations: string[] = [];
  
  // Simple regex-based extraction (for basic schemas)
  // For production, use a proper GraphQL parser
  const queryMatch = sdl.match(/type\s+Query\s*\{([^}]+)\}/s);
  if (queryMatch) {
    const queryFields = queryMatch[1];
    // Match field definitions - handle both with and without arguments
    // Pattern: fieldName or fieldName(args): ReturnType (including arrays and non-null)
    const fieldPattern = /(\w+)\s*(?:\([^)]*\))?\s*:\s*[^\s,}]+/g;
    let fieldMatch;
    while ((fieldMatch = fieldPattern.exec(queryFields)) !== null) {
      queries.push(fieldMatch[1]);
    }
  }
  
  const mutationMatch = sdl.match(/type\s+Mutation\s*\{([^}]+)\}/s);
  if (mutationMatch) {
    const mutationFields = mutationMatch[1];
    // Match field definitions - handle both with and without arguments
    // Pattern: fieldName or fieldName(args): ReturnType (including arrays and non-null)
    const fieldPattern = /(\w+)\s*(?:\([^)]*\))?\s*:\s*[^\s,}]+/g;
    let fieldMatch;
    while ((fieldMatch = fieldPattern.exec(mutationFields)) !== null) {
      mutations.push(fieldMatch[1]);
    }
  }
  
  return { queries, mutations };
}

