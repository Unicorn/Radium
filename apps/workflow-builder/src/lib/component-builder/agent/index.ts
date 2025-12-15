/**
 * Component Builder Agent Module
 *
 * AI-powered agent for conversational component design and generation.
 */

export * from './types';
export * from './builder-agent';

import { ComponentBuilderAgent } from './builder-agent';
import { KnowledgeRetrieval } from '../knowledge-base/retrieval';
import type { BuilderAgentOptions } from './types';

/**
 * Create a Component Builder Agent with default configuration
 */
export function createBuilderAgent(
  knowledge: KnowledgeRetrieval,
  options?: BuilderAgentOptions
): ComponentBuilderAgent {
  return new ComponentBuilderAgent(knowledge, options);
}

/**
 * Session manager for multiple concurrent builder conversations
 */
export class BuilderSessionManager {
  private sessions: Map<string, ComponentBuilderAgent>;
  private knowledge: KnowledgeRetrieval;
  private options: BuilderAgentOptions;

  constructor(knowledge: KnowledgeRetrieval, options: BuilderAgentOptions = {}) {
    this.sessions = new Map();
    this.knowledge = knowledge;
    this.options = options;
  }

  /**
   * Create a new builder session
   */
  createSession(): ComponentBuilderAgent {
    const agent = new ComponentBuilderAgent(this.knowledge, this.options);
    this.sessions.set(agent.getConversationId(), agent);
    return agent;
  }

  /**
   * Get an existing session by ID
   */
  getSession(conversationId: string): ComponentBuilderAgent | undefined {
    return this.sessions.get(conversationId);
  }

  /**
   * Delete a session
   */
  deleteSession(conversationId: string): boolean {
    return this.sessions.delete(conversationId);
  }

  /**
   * Get all active session IDs
   */
  getActiveSessions(): string[] {
    return Array.from(this.sessions.keys());
  }

  /**
   * Clear all sessions
   */
  clearAll(): void {
    this.sessions.clear();
  }

  /**
   * Get session count
   */
  get sessionCount(): number {
    return this.sessions.size;
  }
}
