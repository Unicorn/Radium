/**
 * Custom Error Classes for Workflow System
 * Provides specific, actionable error messages with recovery suggestions
 */

/**
 * Base error class for all workflow-related errors
 */
export class WorkflowError extends Error {
  public readonly code: string;
  public readonly statusCode: number;
  public readonly userMessage: string;
  public readonly recoverySuggestions: string[];
  public readonly details?: Record<string, any>;
  public readonly originalError?: Error;

  constructor(params: {
    message: string;
    code: string;
    statusCode?: number;
    userMessage: string;
    recoverySuggestions?: string[];
    details?: Record<string, any>;
    originalError?: Error;
  }) {
    super(params.message);
    this.name = this.constructor.name;
    this.code = params.code;
    this.statusCode = params.statusCode || 500;
    this.userMessage = params.userMessage;
    this.recoverySuggestions = params.recoverySuggestions || [];
    this.details = params.details;
    this.originalError = params.originalError;

    // Maintains proper stack trace for where error was thrown
    if (Error.captureStackTrace) {
      Error.captureStackTrace(this, this.constructor);
    }
  }

  /**
   * Format error for API response (user-facing)
   */
  toJSON() {
    return {
      error: this.userMessage,
      code: this.code,
      suggestions: this.recoverySuggestions,
      details: this.details,
    };
  }

  /**
   * Format error for logging (includes full details)
   */
  toLogFormat() {
    return {
      name: this.name,
      message: this.message,
      code: this.code,
      statusCode: this.statusCode,
      userMessage: this.userMessage,
      recoverySuggestions: this.recoverySuggestions,
      details: this.details,
      stack: this.stack,
      originalError: this.originalError
        ? {
            message: this.originalError.message,
            stack: this.originalError.stack,
          }
        : undefined,
    };
  }
}

/**
 * Validation errors - when user input is invalid
 */
export class ValidationError extends WorkflowError {
  public readonly field?: string;
  public readonly invalidValue?: any;

  constructor(params: {
    message: string;
    field?: string;
    invalidValue?: any;
    recoverySuggestions?: string[];
    details?: Record<string, any>;
  }) {
    super({
      message: params.message,
      code: 'VALIDATION_ERROR',
      statusCode: 400,
      userMessage: params.field
        ? `Invalid value for ${params.field}: ${params.message}`
        : params.message,
      recoverySuggestions: params.recoverySuggestions || [
        'Check the input values and try again',
        'Ensure all required fields are filled',
      ],
      details: {
        ...params.details,
        field: params.field,
        invalidValue: params.invalidValue,
      },
    });
    this.field = params.field;
    this.invalidValue = params.invalidValue;
  }
}

/**
 * Compilation errors - when workflow cannot be compiled
 */
export class CompilationError extends WorkflowError {
  public readonly nodeId?: string;
  public readonly compilationPhase?: 'validation' | 'codegen' | 'optimization';

  constructor(params: {
    message: string;
    nodeId?: string;
    compilationPhase?: 'validation' | 'codegen' | 'optimization';
    recoverySuggestions?: string[];
    details?: Record<string, any>;
  }) {
    const nodeSuffix = params.nodeId ? ` in node ${params.nodeId}` : '';
    super({
      message: params.message,
      code: 'COMPILATION_ERROR',
      statusCode: 400,
      userMessage: `Compilation failed${nodeSuffix}: ${params.message}`,
      recoverySuggestions: params.recoverySuggestions || [
        'Check that all nodes are properly configured',
        'Ensure all required connections are made',
        'Verify that referenced components exist',
      ],
      details: {
        ...params.details,
        nodeId: params.nodeId,
        compilationPhase: params.compilationPhase,
      },
    });
    this.nodeId = params.nodeId;
    this.compilationPhase = params.compilationPhase;
  }
}

/**
 * Execution errors - when workflow execution fails
 */
export class ExecutionError extends WorkflowError {
  public readonly workflowId: string;
  public readonly executionId?: string;
  public readonly failedStep?: string;

  constructor(params: {
    message: string;
    workflowId: string;
    executionId?: string;
    failedStep?: string;
    recoverySuggestions?: string[];
    details?: Record<string, any>;
    originalError?: Error;
  }) {
    const stepSuffix = params.failedStep ? ` at step "${params.failedStep}"` : '';
    super({
      message: params.message,
      code: 'EXECUTION_ERROR',
      statusCode: 500,
      userMessage: `Workflow execution failed${stepSuffix}: ${params.message}`,
      recoverySuggestions: params.recoverySuggestions || [
        'Check the execution logs for more details',
        'Verify that all activities are properly deployed',
        'Ensure Temporal worker is running',
      ],
      details: {
        ...params.details,
        workflowId: params.workflowId,
        executionId: params.executionId,
        failedStep: params.failedStep,
      },
      originalError: params.originalError,
    });
    this.workflowId = params.workflowId;
    this.executionId = params.executionId;
    this.failedStep = params.failedStep;
  }
}

/**
 * Timeout errors - when operation exceeds time limit
 */
export class TimeoutError extends WorkflowError {
  public readonly timeoutMs: number;
  public readonly operation: string;

  constructor(params: {
    message: string;
    timeoutMs: number;
    operation: string;
    recoverySuggestions?: string[];
    details?: Record<string, any>;
  }) {
    super({
      message: params.message,
      code: 'TIMEOUT_ERROR',
      statusCode: 408,
      userMessage: `Operation "${params.operation}" timed out after ${params.timeoutMs}ms`,
      recoverySuggestions: params.recoverySuggestions || [
        `Increase the timeout setting (currently ${params.timeoutMs}ms)`,
        'Optimize the workflow to complete faster',
        'Check if external services are responding slowly',
      ],
      details: {
        ...params.details,
        timeoutMs: params.timeoutMs,
        operation: params.operation,
      },
    });
    this.timeoutMs = params.timeoutMs;
    this.operation = params.operation;
  }
}

/**
 * Network errors - when external service calls fail
 */
export class NetworkError extends WorkflowError {
  public readonly url?: string;
  public readonly statusCode?: number;

  constructor(params: {
    message: string;
    url?: string;
    statusCode?: number;
    recoverySuggestions?: string[];
    details?: Record<string, any>;
    originalError?: Error;
  }) {
    super({
      message: params.message,
      code: 'NETWORK_ERROR',
      statusCode: params.statusCode || 503,
      userMessage: params.url
        ? `Network request to ${params.url} failed: ${params.message}`
        : `Network request failed: ${params.message}`,
      recoverySuggestions: params.recoverySuggestions || [
        'Check your internet connection',
        'Verify the service is available',
        'Click retry to try again',
      ],
      details: {
        ...params.details,
        url: params.url,
        httpStatusCode: params.statusCode,
      },
      originalError: params.originalError,
    });
    this.url = params.url;
  }
}

/**
 * Database errors - when database operations fail
 */
export class DatabaseError extends WorkflowError {
  public readonly operation: string;
  public readonly table?: string;

  constructor(params: {
    message: string;
    operation: string;
    table?: string;
    details?: Record<string, any>;
    originalError?: Error;
  }) {
    super({
      message: params.message,
      code: 'DATABASE_ERROR',
      statusCode: 500,
      userMessage: 'A database error occurred. Please try again later.',
      recoverySuggestions: [
        'Wait a moment and try again',
        'If the problem persists, contact support',
      ],
      details: {
        ...params.details,
        operation: params.operation,
        table: params.table,
      },
      originalError: params.originalError,
    });
    this.operation = params.operation;
    this.table = params.table;
  }
}

/**
 * Authorization errors - when user lacks permissions
 */
export class AuthorizationError extends WorkflowError {
  public readonly resourceType: string;
  public readonly resourceId?: string;
  public readonly requiredPermission?: string;

  constructor(params: {
    message: string;
    resourceType: string;
    resourceId?: string;
    requiredPermission?: string;
    details?: Record<string, any>;
  }) {
    super({
      message: params.message,
      code: 'AUTHORIZATION_ERROR',
      statusCode: 403,
      userMessage: `You don't have permission to access this ${params.resourceType}`,
      recoverySuggestions: [
        'Verify you have the correct permissions',
        'Contact the resource owner for access',
      ],
      details: {
        ...params.details,
        resourceType: params.resourceType,
        resourceId: params.resourceId,
        requiredPermission: params.requiredPermission,
      },
    });
    this.resourceType = params.resourceType;
    this.resourceId = params.resourceId;
    this.requiredPermission = params.requiredPermission;
  }
}

/**
 * Resource not found errors
 */
export class NotFoundError extends WorkflowError {
  public readonly resourceType: string;
  public readonly resourceId?: string;

  constructor(params: {
    message: string;
    resourceType: string;
    resourceId?: string;
    recoverySuggestions?: string[];
    details?: Record<string, any>;
  }) {
    super({
      message: params.message,
      code: 'NOT_FOUND',
      statusCode: 404,
      userMessage: params.resourceId
        ? `${params.resourceType} with ID ${params.resourceId} not found`
        : `${params.resourceType} not found`,
      recoverySuggestions: params.recoverySuggestions || [
        'Check that the ID is correct',
        'Verify the resource exists',
        'You may not have permission to view this resource',
      ],
      details: {
        ...params.details,
        resourceType: params.resourceType,
        resourceId: params.resourceId,
      },
    });
    this.resourceType = params.resourceType;
    this.resourceId = params.resourceId;
  }
}

/**
 * Conflict errors - when operation conflicts with current state
 */
export class ConflictError extends WorkflowError {
  public readonly conflictType: string;

  constructor(params: {
    message: string;
    conflictType: string;
    recoverySuggestions?: string[];
    details?: Record<string, any>;
  }) {
    super({
      message: params.message,
      code: 'CONFLICT',
      statusCode: 409,
      userMessage: params.message,
      recoverySuggestions: params.recoverySuggestions || [
        'Refresh the page and try again',
        'Check for conflicting resources',
      ],
      details: {
        ...params.details,
        conflictType: params.conflictType,
      },
    });
    this.conflictType = params.conflictType;
  }
}

/**
 * Rate limit errors
 */
export class RateLimitError extends WorkflowError {
  public readonly retryAfterMs?: number;

  constructor(params: {
    message: string;
    retryAfterMs?: number;
    details?: Record<string, any>;
  }) {
    const retryMessage = params.retryAfterMs
      ? ` Please try again in ${Math.ceil(params.retryAfterMs / 1000)} seconds.`
      : ' Please try again later.';

    super({
      message: params.message,
      code: 'RATE_LIMIT_EXCEEDED',
      statusCode: 429,
      userMessage: `Too many requests.${retryMessage}`,
      recoverySuggestions: [
        'Wait a moment before trying again',
        'Reduce the frequency of requests',
      ],
      details: {
        ...params.details,
        retryAfterMs: params.retryAfterMs,
      },
    });
    this.retryAfterMs = params.retryAfterMs;
  }
}

/**
 * Helper function to convert unknown errors to WorkflowError
 */
export function toWorkflowError(error: unknown): WorkflowError {
  if (error instanceof WorkflowError) {
    return error;
  }

  if (error instanceof Error) {
    return new WorkflowError({
      message: error.message,
      code: 'UNKNOWN_ERROR',
      statusCode: 500,
      userMessage: 'An unexpected error occurred. Please try again.',
      recoverySuggestions: ['Refresh the page', 'Try again later'],
      originalError: error,
    });
  }

  return new WorkflowError({
    message: String(error),
    code: 'UNKNOWN_ERROR',
    statusCode: 500,
    userMessage: 'An unexpected error occurred. Please try again.',
    recoverySuggestions: ['Refresh the page', 'Try again later'],
  });
}

/**
 * Error type guards
 */
export function isValidationError(error: unknown): error is ValidationError {
  return error instanceof ValidationError;
}

export function isCompilationError(error: unknown): error is CompilationError {
  return error instanceof CompilationError;
}

export function isExecutionError(error: unknown): error is ExecutionError {
  return error instanceof ExecutionError;
}

export function isTimeoutError(error: unknown): error is TimeoutError {
  return error instanceof TimeoutError;
}

export function isNetworkError(error: unknown): error is NetworkError {
  return error instanceof NetworkError;
}

export function isDatabaseError(error: unknown): error is DatabaseError {
  return error instanceof DatabaseError;
}
