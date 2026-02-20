/**
 * GraphQL API Route
 * Handles GraphQL requests and routes them to Temporal workflows
 */

import { NextRequest, NextResponse } from 'next/server';
import { handleGraphQLRequest, getGraphQLSchemaSDL } from '@/lib/graphql/handler';
import { createClient } from '@/lib/supabase/server';
import { getTemporalClient } from '@/lib/temporal/connection';

export async function POST(request: NextRequest) {
  try {
    // Get service interface ID from headers or query params
    const serviceInterfaceId = request.headers.get('X-Service-Interface-Id') || 
                                new URL(request.url).searchParams.get('serviceInterfaceId');

    if (!serviceInterfaceId) {
      return NextResponse.json(
        { errors: [{ message: 'Service interface ID is required' }] },
        { status: 400 }
      );
    }

    // Get request body
    const body = await request.json();
    const { query, variables, operationName } = body;

    if (!query) {
      return NextResponse.json(
        { errors: [{ message: 'GraphQL query is required' }] },
        { status: 400 }
      );
    }

    // Get Supabase client
    const supabase = await createClient();

    // Get GraphQL schema SDL
    const schemaSDL = await getGraphQLSchemaSDL(serviceInterfaceId, supabase);

    if (!schemaSDL) {
      return NextResponse.json(
        { errors: [{ message: 'GraphQL schema not found for this service interface' }] },
        { status: 404 }
      );
    }

    // Execute workflow function
    const executeWorkflow = async (workflowId: string, input: Record<string, any>) => {
      const client = await getTemporalClient();
      const handle = client.workflow.getHandle(workflowId);
      
      // For queries, use query; for mutations, use signal
      // This is a simplified version - full implementation would check workflow type
      try {
        const result = await handle.query('getResult', input);
        return result;
      } catch (error) {
        // If query fails, try to start workflow and get result
        // Note: This is a placeholder - actual implementation would need workflow type
        throw new Error(`Failed to execute workflow: ${error instanceof Error ? error.message : 'Unknown error'}`);
      }
    };

    // Handle GraphQL request
    const response = await handleGraphQLRequest(
      { query, variables, operationName },
      serviceInterfaceId,
      schemaSDL,
      supabase,
      executeWorkflow
    );

    return NextResponse.json(response);
  } catch (error) {
    console.error('GraphQL request error:', error);
    return NextResponse.json(
      { errors: [{ message: error instanceof Error ? error.message : 'Internal server error' }] },
      { status: 500 }
    );
  }
}

export async function GET(request: NextRequest) {
  // GraphQL typically uses POST, but support GET for introspection
  return NextResponse.json(
    { message: 'GraphQL endpoint - use POST for queries and mutations' },
    { status: 200 }
  );
}

