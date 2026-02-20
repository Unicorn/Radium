/**
 * Agent Tester Signals and Queries
 * 
 * Defines Temporal signals and queries for agent tester workflow
 */

import { defineSignal, defineQuery } from '@temporalio/workflow';
import type { ConversationState, Message } from '@/types/agent-builder';

/**
 * Signal to send a user message to the agent
 */
export const sendMessageSignal = defineSignal<[string]>('sendMessage');

/**
 * Signal to end the test session
 */
export const endTestSignal = defineSignal('endTest');

/**
 * Query to get current conversation state
 */
export const getConversationQuery = defineQuery<ConversationState>('getConversation');

