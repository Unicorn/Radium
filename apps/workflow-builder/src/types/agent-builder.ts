/**
 * Type definitions for agent creation and testing system
 */

export interface Message {
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: Date;
}

export interface AgentTestSession {
  id: string;
  agentPromptId: string;
  workflowId: string;
  status: 'active' | 'completed' | 'timeout' | 'cancelled';
  messageCount: number;
}

export interface ConversationState {
  messages: Message[];
  isActive: boolean;
}

export interface BuilderSession {
  sessionId: string;
  userId: string;
  messages: Message[];
  generatedPrompt?: string;
  metadata: {
    name?: string;
    displayName?: string;
    description?: string;
    capabilities?: string[];
    tags?: string[];
  };
}

export interface AgentTestResult {
  conversation: Message[];
  endedAt: Date;
}

