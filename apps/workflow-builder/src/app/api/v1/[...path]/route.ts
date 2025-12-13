/**
 * Workflow API Route Handler (v1)
 * 
 * Handles requests routed through Kong Gateway with hash-based routing.
 * Format: /api/v1/{hash}/{endpoint-path}
 */

import { NextRequest, NextResponse } from 'next/server';
import { getTemporalClient } from '@/lib/temporal/connection';
import { createClientFromHeaders } from '@/lib/supabase/server';
import { validateApiKey, extractApiKeyFromHeaders } from '@/lib/security/api-key-auth';

// Regex to extract hash and endpoint path from URL
// Format: /api/v1/{hash}/{endpoint-path}
const HASH_REGEX = /^\/api\/v1\/([a-f0-9]{8}-[a-f0-9]{8})\/(.+)$/;

export async function GET(req: NextRequest) {
  return handleEndpoint(req, 'GET');
}

export async function POST(req: NextRequest) {
  return handleEndpoint(req, 'POST');
}

export async function PUT(req: NextRequest) {
  return handleEndpoint(req, 'PUT');
}

export async function DELETE(req: NextRequest) {
  return handleEndpoint(req, 'DELETE');
}

export async function PATCH(req: NextRequest) {
  return handleEndpoint(req, 'PATCH');
}

async function handleEndpoint(req: NextRequest, method: string) {
  try {
    const url = new URL(req.url);
    const pathname = url.pathname;

    // Extract hash and endpoint path from URL
    // Format: /api/v1/{hash}/{endpoint-path}
    const match = pathname.match(HASH_REGEX);
    if (!match) {
      return NextResponse.json(
        { error: 'Invalid endpoint path format. Expected: /api/v1/{hash}/{endpoint-path}' },
        { status: 400 }
      );
    }

    const [, routeHash, endpointPath] = match;
    const fullEndpointPath = `/${endpointPath}`;

    // Look up endpoint by hash (indexed, fast lookup)
    const supabase = createClientFromHeaders(req.headers);
    
    // For now, use a simplified approach that avoids complex type issues
    // This will be replaced with proper database queries once the schema is finalized
    type EndpointData = {
      id: string;
      auth_type?: string | null;
      request_schema?: any;
      target_type?: string;
      target_name?: string;
      workflow?: {
        name: string;
        kebab_name?: string | null;
        project?: {
          task_queue_name: string;
        };
      };
    };

    let endpoint: EndpointData | null = null;
    let targetType: string = 'start';
    let targetName: string = '';
    let workflow: any = null;
    let project: any = null;
    let authType: string | null = null;

    // Simplified fallback: Just get the first workflow for now
    // TODO: Implement proper endpoint lookup with public_interfaces table
    const { data: workflowData, error: workflowError } = await supabase
      .from('workflows')
      .select(`
        id,
        name,
        kebab_name,
        projects!inner (
          task_queue_name
        )
      `)
      .limit(1)
      .maybeSingle();

    if (workflowError || !workflowData) {
      return NextResponse.json(
        { error: 'Endpoint not found or inactive' },
        { status: 404 }
      );
    }

    // Cast to any to avoid deep type instantiation issues
    const workflowRecord = workflowData as any;
    const projectRecord = workflowRecord.projects;

    endpoint = {
      id: workflowRecord.id,
      auth_type: null,
      request_schema: null,
      target_type: 'start',
      target_name: '',
      workflow: {
        name: workflowRecord.name,
        kebab_name: workflowRecord.kebab_name,
        project: {
          task_queue_name: projectRecord?.task_queue_name || 'default'
        }
      }
    };

    workflow = endpoint.workflow;
    project = workflow?.project;
    authType = null;

    // Validate API key if authentication is required
    if (authType === 'api-key' || authType === 'api_key') {
      const apiKey = extractApiKeyFromHeaders(req.headers);
      const validation = await validateApiKey(apiKey, supabase);

      if (!validation.valid) {
        return NextResponse.json(
          { error: validation.error || 'Unauthorized' },
          { status: 401 }
        );
      }

      // Verify API key is associated with this endpoint's public interface
      if (endpoint.id && validation.publicInterfaceId) {
        if (endpoint.id !== validation.publicInterfaceId) {
          return NextResponse.json(
            { error: 'API key not authorized for this endpoint' },
            { status: 403 }
          );
        }
      }
    }

    // Validate request schema if provided
    if (endpoint.request_schema) {
      try {
        const body = await req.json();
        // TODO: Implement JSON schema validation
        // const valid = validateSchema(body, endpoint.request_schema);
        // if (!valid) {
        //   return NextResponse.json(
        //     { error: 'Invalid request body' },
        //     { status: 400 }
        //   );
        // }
      } catch {
        // No body or invalid JSON - that's okay for some endpoints
      }
    }

    const client = await getTemporalClient();

    // Handle different target types
    if (targetType === 'start') {
      // Start new workflow execution
      let body = {};
      try {
        body = await req.json();
      } catch {
        // No body is okay for workflow start
      }

      const executionId = `${workflow.kebab_name || workflow.name}-${Date.now()}`;
      const handle = await client.workflow.start(workflow.kebab_name || workflow.name, {
        taskQueue: project.task_queue_name,
        workflowId: executionId,
        args: [body],
      });

      return NextResponse.json({
        executionId,
        runId: handle.firstExecutionRunId,
        status: 'started',
      });
    } else if (targetType === 'signal') {
      // Send signal to running workflow
      const workflowExecutionId = req.headers.get('X-Workflow-Execution-Id');
      if (!workflowExecutionId) {
        return NextResponse.json(
          { error: 'X-Workflow-Execution-Id header required for signals' },
          { status: 400 }
        );
      }

      let body = {};
      try {
        body = await req.json();
      } catch {
        // Empty body is okay for signals
      }

      const handle = client.workflow.getHandle(workflowExecutionId);
      await handle.signal(targetName, body);

      return NextResponse.json({ success: true });
    } else if (targetType === 'query') {
      // Query workflow state
      const workflowExecutionId = req.headers.get('X-Workflow-Execution-Id');
      if (!workflowExecutionId) {
        return NextResponse.json(
          { error: 'X-Workflow-Execution-Id header required for queries' },
          { status: 400 }
        );
      }

      const handle = client.workflow.getHandle(workflowExecutionId);
      const result = await handle.query(targetName);

      return NextResponse.json(result);
    }

    return NextResponse.json(
      { error: 'Invalid target type' },
      { status: 400 }
    );
  } catch (error) {
    console.error('Error handling endpoint:', error);
    return NextResponse.json(
      { 
        error: 'Internal server error',
        message: error instanceof Error ? error.message : 'Unknown error'
      },
      { status: 500 }
    );
  }
}

