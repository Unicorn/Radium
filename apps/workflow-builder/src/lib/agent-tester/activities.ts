/**
 * Agent Tester Activities
 * 
 * Activities for calling agent APIs and managing test sessions
 */

import { createClient } from '@supabase/supabase-js';
import Anthropic from '@anthropic-ai/sdk';
import type { Database } from '@/types/database';
import type { Message } from '@/types/agent-builder';

/**
 * Get Supabase client for activities
 */
function getSupabaseClient() {
  return createClient<Database>(
    process.env.NEXT_PUBLIC_SUPABASE_URL!,
    process.env.SUPABASE_SERVICE_ROLE_KEY!
  );
}

/**
 * Get agent prompt from database
 */
export async function getAgentPromptActivity(promptId: string): Promise<{
  promptContent: string;
  modelProvider?: string;
  modelName?: string;
}> {
  const supabase = getSupabaseClient();
  
  const { data, error } = await supabase
    .from('agent_prompts')
    .select('prompt_content, model_provider, model_name')
    .eq('id', promptId)
    .single();
  
  if (error || !data) {
    throw new Error(`Failed to fetch agent prompt: ${error?.message || 'Not found'}`);
  }
  
  return {
    promptContent: data.prompt_content,
    modelProvider: data.model_provider || undefined,
    modelName: data.model_name || undefined,
  };
}

/**
 * Call agent with conversation history
 * Token-efficient: only sends recent messages if conversation is long
 */
export async function callAgentActivity(input: {
  promptId: string;
  conversationHistory: Message[];
  userMessage: string;
}): Promise<string> {
  const { promptId, conversationHistory, userMessage } = input;
  
  // Get agent prompt
  const promptData = await getAgentPromptActivity(promptId);
  
  // Build conversation messages (token-efficient: only last 20 messages)
  const maxMessages = 20;
  const recentMessages = conversationHistory.slice(-maxMessages);
  
  // Convert to Anthropic format
  const messages: Anthropic.MessageParam[] = recentMessages.map(msg => ({
    role: msg.role === 'user' ? 'user' : 'assistant',
    content: msg.content,
  }));
  
  // Add current user message
  messages.push({
    role: 'user',
    content: userMessage,
  });
  
  // Get API key
  const apiKey = process.env.ANTHROPIC_API_KEY;
  if (!apiKey) {
    throw new Error('ANTHROPIC_API_KEY not configured');
  }
  
  const client = new Anthropic({ apiKey });
  
  // Determine model
  const model = promptData.modelName || 'claude-sonnet-4-20250514';
  
  try {
    const response = await client.messages.create({
      model,
      max_tokens: 4096,
      system: promptData.promptContent,
      messages,
    });
    
    // Extract text content
    const textContent = response.content
      .filter((block): block is Anthropic.TextBlock => block.type === 'text')
      .map(block => block.text)
      .join('\n');
    
    return textContent;
  } catch (error) {
    console.error('Error calling agent:', error);
    throw new Error(`Failed to call agent: ${error instanceof Error ? error.message : 'Unknown error'}`);
  }
}

/**
 * Save test session to database
 */
export async function saveTestSessionActivity(input: {
  sessionId: string;
  agentPromptId: string;
  userId: string;
  temporalWorkflowId: string;
  temporalRunId: string;
  conversationHistory: Message[];
  status: 'active' | 'completed' | 'cancelled' | 'timeout';
}): Promise<void> {
  const supabase = getSupabaseClient();
  
  const { data: existing } = await supabase
    .from('agent_test_sessions')
    .select('id')
    .eq('id', input.sessionId)
    .single();
  
  if (existing) {
    // Update existing session
    await supabase
      .from('agent_test_sessions')
      .update({
        conversation_history: input.conversationHistory as any,
        message_count: input.conversationHistory.length,
        status: input.status,
        completed_at: input.status !== 'active' ? new Date().toISOString() : null,
        updated_at: new Date().toISOString(),
      })
      .eq('id', input.sessionId);
  } else {
    // Create new session
    await supabase
      .from('agent_test_sessions')
      .insert({
        id: input.sessionId,
        agent_prompt_id: input.agentPromptId,
        user_id: input.userId,
        temporal_workflow_id: input.temporalWorkflowId,
        temporal_run_id: input.temporalRunId,
        conversation_history: input.conversationHistory as any,
        message_count: input.conversationHistory.length,
        status: input.status,
      });
  }
}

