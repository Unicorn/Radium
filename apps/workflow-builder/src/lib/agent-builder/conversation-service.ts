/**
 * Agent Builder Conversation Service
 * 
 * Manages session state and AI conversations for agent prompt creation.
 * Uses in-memory storage with TTL for session cleanup.
 */

import Anthropic from '@anthropic-ai/sdk';
import type { Message, BuilderSession } from '@/types/agent-builder';

interface SessionStorage {
  session: BuilderSession;
  expiresAt: number;
}

// In-memory session storage (can be replaced with Redis in production)
const sessions = new Map<string, SessionStorage>();

// Session TTL: 30 minutes
const SESSION_TTL_MS = 30 * 60 * 1000;

// Cleanup interval: every 5 minutes
const CLEANUP_INTERVAL_MS = 5 * 60 * 1000;

// Start cleanup interval
setInterval(() => {
  const now = Date.now();
  for (const [sessionId, storage] of sessions.entries()) {
    if (now > storage.expiresAt) {
      sessions.delete(sessionId);
    }
  }
}, CLEANUP_INTERVAL_MS);

/**
 * System prompt for agent builder assistant
 */
const SYSTEM_PROMPT = `You are an AI assistant helping users create agent prompts. Your goal is to:

1. Ask 3-5 clarifying questions about what the agent should do
2. Determine when you have enough information to generate a prompt
3. Generate a complete agent prompt in markdown format
4. Support revision requests from the user

Guidelines:
- Be conversational and helpful
- Ask one question at a time
- After 3-5 questions, determine if you have enough information
- When ready, generate a complete agent prompt in markdown
- The prompt should include:
  * Clear role definition
  * Capabilities and responsibilities
  * Instructions for behavior
  * Output format expectations
- Support revision requests by asking what needs to change

When you're ready to generate the prompt, say: "I have enough information. Here's the agent prompt I've created:" followed by the markdown prompt.

Current conversation state: {metadata}`;

/**
 * Get or create a session
 */
export function getSession(sessionId: string): BuilderSession | null {
  const storage = sessions.get(sessionId);
  if (!storage) {
    return null;
  }
  
  // Check if expired
  if (Date.now() > storage.expiresAt) {
    sessions.delete(sessionId);
    return null;
  }
  
  return storage.session;
}

/**
 * Create a new session
 */
export function createSession(userId: string): BuilderSession {
  const sessionId = `session-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  
  const session: BuilderSession = {
    sessionId,
    userId,
    messages: [
      {
        role: 'system',
        content: SYSTEM_PROMPT.replace('{metadata}', 'No metadata yet'),
        timestamp: new Date(),
      },
      {
        role: 'assistant',
        content: "I'll help you create an agent prompt! Let's start by understanding what you want your agent to do. What should this agent accomplish?",
        timestamp: new Date(),
      },
    ],
    metadata: {},
  };
  
  sessions.set(sessionId, {
    session,
    expiresAt: Date.now() + SESSION_TTL_MS,
  });
  
  return session;
}

/**
 * Send a message and get AI response
 */
export async function sendMessage(
  sessionId: string,
  userMessage: string
): Promise<{ session: BuilderSession; response: string }> {
  const storage = sessions.get(sessionId);
  if (!storage || Date.now() > storage.expiresAt) {
    throw new Error('Session not found or expired');
  }
  
  const session = storage.session;
  
  // Add user message
  const userMsg: Message = {
    role: 'user',
    content: userMessage,
    timestamp: new Date(),
  };
  session.messages.push(userMsg);
  
  // Call AI API
  const apiKey = process.env.ANTHROPIC_API_KEY;
  if (!apiKey) {
    throw new Error('ANTHROPIC_API_KEY not configured');
  }
  
  const client = new Anthropic({ apiKey });
  
  // Build conversation messages (exclude system message, it goes in system param)
  const conversationMessages = session.messages
    .filter(m => m.role !== 'system')
    .map(m => ({
      role: m.role === 'assistant' ? 'assistant' : 'user' as const,
      content: m.content,
    }));
  
  // Get system message
  const systemMessage = session.messages.find(m => m.role === 'system')?.content || SYSTEM_PROMPT;
  
  // Update system message with current metadata
  const metadataStr = JSON.stringify(session.metadata, null, 2);
  const updatedSystemMessage = systemMessage.replace('{metadata}', metadataStr);
  
  try {
    const response = await client.messages.create({
      model: 'claude-sonnet-4-20250514',
      max_tokens: 4096,
      system: updatedSystemMessage,
      messages: conversationMessages as Anthropic.MessageParam[],
    });
    
    // Extract text content
    const textContent = response.content
      .filter((block): block is Anthropic.TextBlock => block.type === 'text')
      .map(block => block.text)
      .join('\n');
    
    // Add assistant response
    const assistantMsg: Message = {
      role: 'assistant',
      content: textContent,
      timestamp: new Date(),
    };
    session.messages.push(assistantMsg);
    
    // Check if prompt was generated (look for markdown code blocks)
    if (textContent.includes('```') && textContent.includes('agent prompt')) {
      // Extract prompt from markdown
      const promptMatch = textContent.match(/```(?:markdown)?\n([\s\S]*?)\n```/);
      if (promptMatch) {
        session.generatedPrompt = promptMatch[1];
      }
    }
    
    // Update expiration
    storage.expiresAt = Date.now() + SESSION_TTL_MS;
    
    return { session, response: textContent };
  } catch (error) {
    console.error('Error calling Anthropic API:', error);
    throw new Error(`Failed to get AI response: ${error instanceof Error ? error.message : 'Unknown error'}`);
  }
}

/**
 * Regenerate prompt with current context
 */
export async function regeneratePrompt(sessionId: string): Promise<string> {
  const storage = sessions.get(sessionId);
  if (!storage || Date.now() > storage.expiresAt) {
    throw new Error('Session not found or expired');
  }
  
  const session = storage.session;
  
  // Add a system message requesting prompt generation
  const regenerateMessage: Message = {
    role: 'user',
    content: 'Please generate the agent prompt now based on our conversation.',
    timestamp: new Date(),
  };
  
  session.messages.push(regenerateMessage);
  
  const result = await sendMessage(sessionId, regenerateMessage.content);
  
  // Extract prompt if generated
  if (result.session.generatedPrompt) {
    return result.session.generatedPrompt;
  }
  
  // If not in markdown, try to extract from response
  return result.response;
}

/**
 * Update session metadata
 */
export function updateMetadata(sessionId: string, metadata: Partial<BuilderSession['metadata']>): BuilderSession {
  const storage = sessions.get(sessionId);
  if (!storage || Date.now() > storage.expiresAt) {
    throw new Error('Session not found or expired');
  }
  
  storage.session.metadata = {
    ...storage.session.metadata,
    ...metadata,
  };
  
  storage.expiresAt = Date.now() + SESSION_TTL_MS;
  
  return storage.session;
}

/**
 * Delete a session
 */
export function deleteSession(sessionId: string): void {
  sessions.delete(sessionId);
}

