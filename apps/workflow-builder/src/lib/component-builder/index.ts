/**
 * Component Builder Module
 *
 * AI-powered system for creating new workflow components through
 * conversational interaction. Uses migration records as training data
 * to understand component patterns and generate production-ready code.
 */

// Re-export all submodules
export * from './knowledge-base';
export * from './agent';

// Convenience imports
import { initializeKnowledgeBase } from './knowledge-base';
import { ComponentBuilderAgent, BuilderSessionManager } from './agent';
import type { BuilderAgentOptions, AgentResponse } from './agent';

/** Global session manager instance */
let globalSessionManager: BuilderSessionManager | null = null;

/**
 * Initialize the Component Builder system
 *
 * This should be called once at application startup to load the
 * knowledge base and prepare the session manager.
 */
export async function initializeComponentBuilder(
  options?: BuilderAgentOptions
): Promise<BuilderSessionManager> {
  // Initialize knowledge base
  const { retrieval } = await initializeKnowledgeBase();

  // Create session manager
  globalSessionManager = new BuilderSessionManager(retrieval, options);

  console.log('Component Builder initialized');
  return globalSessionManager;
}

/**
 * Get the global session manager
 *
 * @throws Error if Component Builder hasn't been initialized
 */
export function getSessionManager(): BuilderSessionManager {
  if (!globalSessionManager) {
    throw new Error(
      'Component Builder not initialized. Call initializeComponentBuilder() first.'
    );
  }
  return globalSessionManager;
}

/**
 * Quick helper to chat with a new or existing session
 */
export async function chat(
  message: string,
  conversationId?: string
): Promise<AgentResponse & { conversationId: string }> {
  const manager = getSessionManager();

  let agent: ComponentBuilderAgent;
  if (conversationId) {
    const existing = manager.getSession(conversationId);
    if (!existing) {
      throw new Error(`Session not found: ${conversationId}`);
    }
    agent = existing;
  } else {
    agent = manager.createSession();
  }

  const response = await agent.chat(message);
  return {
    ...response,
    conversationId: agent.getConversationId(),
  };
}

/**
 * Component Builder configuration
 */
export interface ComponentBuilderConfig {
  /** Anthropic API key */
  apiKey?: string;

  /** Model to use */
  model?: string;

  /** Path to component records directory */
  recordsPath?: string;

  /** Enable debug logging */
  debug?: boolean;
}

/**
 * Status of the Component Builder system
 */
export interface ComponentBuilderStatus {
  /** Whether the system is initialized */
  initialized: boolean;

  /** Number of active sessions */
  activeSessions: number;

  /** Knowledge base statistics */
  knowledgeBase: {
    componentCount: number;
    patternCount: number;
    decisionCount: number;
  } | null;
}

/**
 * Get the current status of the Component Builder
 */
export function getStatus(): ComponentBuilderStatus {
  if (!globalSessionManager) {
    return {
      initialized: false,
      activeSessions: 0,
      knowledgeBase: null,
    };
  }

  return {
    initialized: true,
    activeSessions: globalSessionManager.sessionCount,
    knowledgeBase: {
      componentCount: 0, // Would need to track this
      patternCount: 0,
      decisionCount: 0,
    },
  };
}
