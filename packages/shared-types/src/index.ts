/**
 * Shared TypeScript type definitions for Radium.
 * 
 * This package exports all type definitions that match the Rust proto definitions,
 * enabling type-safe communication between the frontend and backend.
 */

// Agent types
export * from './types/agent';

// Workflow types
export * from './types/workflow';

// Task types
export * from './types/task';

// Orchestrator types
export * from './types/orchestrator';

// Common types
export interface PingRequest {
	message: string;
}

export interface PingResponse {
	message: string;
}

