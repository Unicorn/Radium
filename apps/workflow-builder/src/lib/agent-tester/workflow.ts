/**
 * Agent Tester Workflow
 * 
 * Temporal workflow for testing agent prompts with human-in-the-loop interaction
 */

import { condition, sleep, setHandler, currentTimeMs } from '@temporalio/workflow';
import { sendMessageSignal, endTestSignal, getConversationQuery } from './signals-queries';
import * as activities from './activities';
import type { Message, AgentTestResult } from '@/types/agent-builder';

export interface AgentTesterWorkflowInput {
  agentPromptId: string;
  userId: string;
  sessionId: string;
  initialMessage?: string;
}

export async function agentTesterWorkflow(
  input: AgentTesterWorkflowInput
): Promise<AgentTestResult> {
  const conversation: Message[] = [];
  let isActive = true;
  let userInput: string | null = input.initialMessage || null;
  let lastActivityTime = currentTimeMs();
  
  // Timeout: 5 minutes of inactivity
  const INACTIVITY_TIMEOUT_MS = 5 * 60 * 1000;
  
  // Signal handlers
  const sendMessageHandler = (message: string) => {
    userInput = message;
    lastActivityTime = currentTimeMs();
  };
  
  const endTestHandler = () => {
    isActive = false;
  };
  
  // Query handler
  const getConversationHandler = (): { messages: Message[]; isActive: boolean } => {
    return {
      messages: conversation,
      isActive,
    };
  };
  
  // Register handlers
  setHandler(sendMessageSignal, sendMessageHandler);
  setHandler(endTestSignal, endTestHandler);
  setHandler(getConversationQuery, getConversationHandler);
  
  // Save initial session
  await activities.saveTestSessionActivity({
    sessionId: input.sessionId,
    agentPromptId: input.agentPromptId,
    userId: input.userId,
    temporalWorkflowId: '', // Will be set by caller
    temporalRunId: '', // Will be set by caller
    conversationHistory: conversation,
    status: 'active',
  });
  
  // Main conversation loop
  while (isActive) {
    // Check for timeout
    const timeSinceLastActivity = currentTimeMs() - lastActivityTime;
    if (timeSinceLastActivity > INACTIVITY_TIMEOUT_MS && conversation.length > 0) {
      isActive = false;
      break;
    }
    
    // Process user input if available
    if (userInput) {
      try {
        // Call agent with conversation history
        const response = await activities.callAgentActivity({
          promptId: input.agentPromptId,
          conversationHistory: conversation,
          userMessage: userInput,
        });
        
        // Add messages to conversation
        const now = new Date();
        conversation.push({
          role: 'user',
          content: userInput,
          timestamp: now,
        });
        
        conversation.push({
          role: 'assistant',
          content: response,
          timestamp: now,
        });
        
        // Update session in database
        await activities.saveTestSessionActivity({
          sessionId: input.sessionId,
          agentPromptId: input.agentPromptId,
          userId: input.userId,
          temporalWorkflowId: '', // Will be updated by caller
          temporalRunId: '', // Will be updated by caller
          conversationHistory: conversation,
          status: 'active',
        });
        
        userInput = null;
        lastActivityTime = currentTimeMs();
      } catch (error) {
        console.error('Error in agent tester workflow:', error);
        
        // Add error message to conversation
        conversation.push({
          role: 'assistant',
          content: `Error: ${error instanceof Error ? error.message : 'Unknown error occurred'}`,
          timestamp: new Date(),
        });
        
        userInput = null;
      }
    }
    
    // Wait for next signal or timeout using condition
    // Wait up to 10 seconds or until we have user input
    try {
      await condition(
        () => userInput !== null || !isActive,
        Math.min(10000, INACTIVITY_TIMEOUT_MS - timeSinceLastActivity)
      );
    } catch (error) {
      // Timeout - check if we should end
      const newTimeSinceLastActivity = currentTimeMs() - lastActivityTime;
      if (newTimeSinceLastActivity > INACTIVITY_TIMEOUT_MS && conversation.length > 0) {
        isActive = false;
        break;
      }
    }
  }
  
  // Save final session state
  const finalStatus = conversation.length === 0 ? 'cancelled' : 'completed';
  
  await activities.saveTestSessionActivity({
    sessionId: input.sessionId,
    agentPromptId: input.agentPromptId,
    userId: input.userId,
    temporalWorkflowId: '', // Will be updated by caller
    temporalRunId: '', // Will be updated by caller
    conversationHistory: conversation,
    status: finalStatus,
  });
  
  return {
    conversation,
    endedAt: new Date(),
  };
}

