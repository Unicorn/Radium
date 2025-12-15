/**
 * Component Builder Chat API Route
 *
 * Handles chat messages for the Component Builder Agent.
 */

import { NextRequest, NextResponse } from 'next/server';
import {
  initializeKnowledgeBase,
  KnowledgeRetrieval,
} from '@/lib/component-builder/knowledge-base';
import {
  ComponentBuilderAgent,
  BuilderSessionManager,
} from '@/lib/component-builder/agent';

// Session manager singleton
let sessionManager: BuilderSessionManager | null = null;
let knowledgeBase: KnowledgeRetrieval | null = null;
let initializationPromise: Promise<void> | null = null;

/**
 * Initialize the component builder system
 */
async function ensureInitialized(): Promise<void> {
  if (sessionManager && knowledgeBase) {
    return;
  }

  if (initializationPromise) {
    await initializationPromise;
    return;
  }

  initializationPromise = (async () => {
    try {
      const { retrieval } = await initializeKnowledgeBase();
      knowledgeBase = retrieval;
      sessionManager = new BuilderSessionManager(retrieval, {
        model: 'claude-sonnet-4-20250514',
      });
      console.log('Component Builder API initialized');
    } catch (error) {
      console.error('Failed to initialize Component Builder:', error);
      throw error;
    }
  })();

  await initializationPromise;
}

/** Request body type */
interface ChatRequest {
  message: string;
  conversationId?: string;
}

/** Response type */
interface ChatResponse {
  response: string;
  conversationId: string;
  phase: string;
  phaseChanged: boolean;
  suggestedActions?: string[];
  artifacts?: {
    rustSchema?: string;
    typescript?: string;
    tests?: string;
  };
  error?: string;
}

/**
 * POST /api/component-builder/chat
 *
 * Send a message to the Component Builder Agent
 */
export async function POST(request: NextRequest): Promise<NextResponse<ChatResponse>> {
  try {
    // Ensure system is initialized
    await ensureInitialized();

    if (!sessionManager) {
      return NextResponse.json(
        {
          response: '',
          conversationId: '',
          phase: 'error',
          phaseChanged: false,
          error: 'Component Builder not initialized',
        },
        { status: 500 }
      );
    }

    // Parse request body
    const body = (await request.json()) as ChatRequest;

    if (!body.message || typeof body.message !== 'string') {
      return NextResponse.json(
        {
          response: '',
          conversationId: '',
          phase: 'error',
          phaseChanged: false,
          error: 'Message is required',
        },
        { status: 400 }
      );
    }

    // Get or create session
    let agent: ComponentBuilderAgent;
    if (body.conversationId) {
      const existing = sessionManager.getSession(body.conversationId);
      if (!existing) {
        return NextResponse.json(
          {
            response: '',
            conversationId: body.conversationId,
            phase: 'error',
            phaseChanged: false,
            error: 'Session not found',
          },
          { status: 404 }
        );
      }
      agent = existing;
    } else {
      agent = sessionManager.createSession();
    }

    // Process message
    const result = await agent.chat(body.message);

    // Build response
    const response: ChatResponse = {
      response: result.response,
      conversationId: agent.getConversationId(),
      phase: result.phase,
      phaseChanged: result.phaseChanged,
      suggestedActions: result.suggestedActions,
    };

    // Include artifacts if in reviewing phase
    if (result.state.generatedArtifacts) {
      response.artifacts = {
        rustSchema: result.state.generatedArtifacts.rustSchema,
        typescript: result.state.generatedArtifacts.typescriptCode,
        tests: result.state.generatedArtifacts.testCases,
      };
    }

    return NextResponse.json(response);
  } catch (error) {
    console.error('Component Builder chat error:', error);
    return NextResponse.json(
      {
        response: '',
        conversationId: '',
        phase: 'error',
        phaseChanged: false,
        error: error instanceof Error ? error.message : 'Unknown error',
      },
      { status: 500 }
    );
  }
}

/**
 * GET /api/component-builder/chat
 *
 * Get session information
 */
export async function GET(request: NextRequest): Promise<NextResponse> {
  try {
    await ensureInitialized();

    if (!sessionManager) {
      return NextResponse.json(
        { error: 'Component Builder not initialized' },
        { status: 500 }
      );
    }

    const url = new URL(request.url);
    const conversationId = url.searchParams.get('conversationId');

    if (conversationId) {
      const agent = sessionManager.getSession(conversationId);
      if (!agent) {
        return NextResponse.json(
          { error: 'Session not found' },
          { status: 404 }
        );
      }

      const state = agent.getState();
      return NextResponse.json({
        conversationId,
        phase: state.phase,
        messageCount: state.messages.length,
        hasDesign: state.designDraft !== null,
        hasArtifacts: state.generatedArtifacts !== null,
        createdAt: state.createdAt,
        updatedAt: state.updatedAt,
      });
    }

    // Return overall status
    return NextResponse.json({
      initialized: true,
      activeSessions: sessionManager.sessionCount,
      sessions: sessionManager.getActiveSessions(),
    });
  } catch (error) {
    console.error('Component Builder GET error:', error);
    return NextResponse.json(
      { error: error instanceof Error ? error.message : 'Unknown error' },
      { status: 500 }
    );
  }
}

/**
 * DELETE /api/component-builder/chat
 *
 * Delete a session
 */
export async function DELETE(request: NextRequest): Promise<NextResponse> {
  try {
    await ensureInitialized();

    if (!sessionManager) {
      return NextResponse.json(
        { error: 'Component Builder not initialized' },
        { status: 500 }
      );
    }

    const url = new URL(request.url);
    const conversationId = url.searchParams.get('conversationId');

    if (!conversationId) {
      return NextResponse.json(
        { error: 'conversationId is required' },
        { status: 400 }
      );
    }

    const deleted = sessionManager.deleteSession(conversationId);

    if (!deleted) {
      return NextResponse.json(
        { error: 'Session not found' },
        { status: 404 }
      );
    }

    return NextResponse.json({ success: true, deleted: conversationId });
  } catch (error) {
    console.error('Component Builder DELETE error:', error);
    return NextResponse.json(
      { error: error instanceof Error ? error.message : 'Unknown error' },
      { status: 500 }
    );
  }
}
