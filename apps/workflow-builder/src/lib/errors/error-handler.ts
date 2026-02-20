/**
 * Error Handler Middleware
 * Converts errors to appropriate tRPC errors with user-friendly messages
 */

import { TRPCError } from '@trpc/server';
import {
  WorkflowError,
  ValidationError,
  CompilationError,
  ExecutionError,
  TimeoutError,
  NetworkError,
  DatabaseError,
  AuthorizationError,
  NotFoundError,
  ConflictError,
  RateLimitError,
  toWorkflowError,
} from './workflow-errors';

/**
 * Error logger interface
 */
export interface ErrorLogger {
  error(message: string, context?: Record<string, any>): void;
  warn(message: string, context?: Record<string, any>): void;
}

/**
 * Console-based error logger (default)
 */
export const consoleLogger: ErrorLogger = {
  error: (message: string, context?: Record<string, any>) => {
    console.error(`[ERROR] ${message}`, context || '');
  },
  warn: (message: string, context?: Record<string, any>) => {
    console.warn(`[WARN] ${message}`, context || '');
  },
};

/**
 * Global error logger (can be configured)
 */
let errorLogger: ErrorLogger = consoleLogger;

/**
 * Configure the error logger
 */
export function configureErrorLogger(logger: ErrorLogger): void {
  errorLogger = logger;
}

/**
 * Convert WorkflowError to TRPCError
 */
export function workflowErrorToTRPC(error: WorkflowError): TRPCError {
  // Map status codes to tRPC codes
  const codeMap: Record<number, TRPCError['code']> = {
    400: 'BAD_REQUEST',
    401: 'UNAUTHORIZED',
    403: 'FORBIDDEN',
    404: 'NOT_FOUND',
    408: 'TIMEOUT',
    409: 'CONFLICT',
    429: 'TOO_MANY_REQUESTS',
    500: 'INTERNAL_SERVER_ERROR',
    503: 'INTERNAL_SERVER_ERROR',
  };

  const trpcCode = codeMap[error.statusCode] || 'INTERNAL_SERVER_ERROR';

  return new TRPCError({
    code: trpcCode,
    message: error.userMessage,
    cause: {
      code: error.code,
      suggestions: error.recoverySuggestions,
      details: error.details,
    },
  });
}

/**
 * Handle errors in tRPC procedures
 * Logs errors server-side and returns user-friendly errors
 */
export function handleError(error: unknown): never {
  // Convert to WorkflowError if needed
  const workflowError = toWorkflowError(error);

  // Log error server-side (never expose to user)
  if (workflowError instanceof DatabaseError) {
    // Database errors should never expose details to user
    errorLogger.error('Database error occurred', workflowError.toLogFormat());
  } else if (workflowError.statusCode >= 500) {
    // Internal server errors
    errorLogger.error('Internal server error', workflowError.toLogFormat());
  } else if (workflowError instanceof ValidationError) {
    // Validation errors are user errors (log as warning)
    errorLogger.warn('Validation error', workflowError.toLogFormat());
  } else {
    // Other errors
    errorLogger.error('Error occurred', workflowError.toLogFormat());
  }

  // Convert to tRPC error and throw
  throw workflowErrorToTRPC(workflowError);
}

/**
 * Wrap async operations with error handling
 */
export async function withErrorHandling<T>(
  operation: () => Promise<T>,
  context?: string
): Promise<T> {
  try {
    return await operation();
  } catch (error) {
    if (context) {
      errorLogger.error(`Error in ${context}`, { error });
    }
    handleError(error);
  }
}

/**
 * Wrap database operations with error handling
 */
export async function withDatabaseErrorHandling<T>(
  operation: () => Promise<{ data: T | null; error: any }>,
  operationName: string,
  table?: string
): Promise<T> {
  try {
    const { data, error } = await operation();

    if (error) {
      throw new DatabaseError({
        message: `Database operation failed: ${error.message}`,
        operation: operationName,
        table,
        originalError: error,
      });
    }

    if (data === null) {
      throw new NotFoundError({
        message: 'Resource not found',
        resourceType: table || 'record',
      });
    }

    return data;
  } catch (error) {
    handleError(error);
  }
}

/**
 * Wrap network operations with error handling
 */
export async function withNetworkErrorHandling<T>(
  operation: () => Promise<Response>,
  url: string
): Promise<T> {
  try {
    const response = await operation();

    if (!response.ok) {
      throw new NetworkError({
        message: response.statusText || 'Network request failed',
        url,
        statusCode: response.status,
        recoverySuggestions: [
          'Check your internet connection',
          'Verify the service is available',
          'Try again in a moment',
        ],
      });
    }

    return await response.json();
  } catch (error) {
    if (error instanceof NetworkError) {
      handleError(error);
    }

    // Network connectivity error
    throw new NetworkError({
      message: 'Failed to connect to service',
      url,
      recoverySuggestions: [
        'Check your internet connection',
        'Verify the service URL is correct',
        'Try again in a moment',
      ],
      originalError: error instanceof Error ? error : undefined,
    });
  }
}

/**
 * Validate and handle Supabase responses
 */
export function validateSupabaseResponse<T>(
  result: { data: T | null; error: any },
  operation: string,
  table?: string
): T {
  if (result.error) {
    throw new DatabaseError({
      message: `${operation} failed: ${result.error.message}`,
      operation,
      table,
      originalError: result.error,
    });
  }

  if (result.data === null) {
    throw new NotFoundError({
      message: `${operation} returned no data`,
      resourceType: table || 'record',
    });
  }

  return result.data;
}

/**
 * Check authorization and throw error if unauthorized
 */
export function requireAuthorization(
  condition: boolean,
  resourceType: string,
  resourceId?: string,
  requiredPermission?: string
): void {
  if (!condition) {
    throw new AuthorizationError({
      message: `Not authorized to access ${resourceType}`,
      resourceType,
      resourceId,
      requiredPermission,
    });
  }
}

/**
 * Validate input and throw if invalid
 */
export function validateInput(
  condition: boolean,
  field: string,
  message: string,
  invalidValue?: any,
  recoverySuggestions?: string[]
): void {
  if (!condition) {
    throw new ValidationError({
      message,
      field,
      invalidValue,
      recoverySuggestions,
    });
  }
}

/**
 * Create a retry wrapper with timeout
 */
export async function withRetry<T>(
  operation: () => Promise<T>,
  options: {
    maxAttempts?: number;
    timeoutMs?: number;
    retryDelayMs?: number;
    operationName?: string;
  } = {}
): Promise<T> {
  const maxAttempts = options.maxAttempts || 3;
  const timeoutMs = options.timeoutMs || 30000;
  const retryDelayMs = options.retryDelayMs || 1000;
  const operationName = options.operationName || 'operation';

  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    try {
      // Create timeout promise
      const timeoutPromise = new Promise<never>((_, reject) => {
        setTimeout(() => {
          reject(
            new TimeoutError({
              message: `${operationName} timed out after ${timeoutMs}ms`,
              timeoutMs,
              operation: operationName,
            })
          );
        }, timeoutMs);
      });

      // Race between operation and timeout
      return await Promise.race([operation(), timeoutPromise]);
    } catch (error) {
      if (attempt === maxAttempts) {
        // Last attempt failed
        handleError(error);
      }

      // Wait before retry
      if (attempt < maxAttempts) {
        await new Promise((resolve) => setTimeout(resolve, retryDelayMs * attempt));
      }
    }
  }

  // Should never reach here
  throw new Error('Retry logic error');
}

/**
 * Error boundary for async operations
 */
export class ErrorBoundary {
  private errors: WorkflowError[] = [];
  private warnings: string[] = [];

  /**
   * Execute operation and catch errors
   */
  async try<T>(operation: () => Promise<T>): Promise<T | null> {
    try {
      return await operation();
    } catch (error) {
      this.errors.push(toWorkflowError(error));
      return null;
    }
  }

  /**
   * Add a warning
   */
  warn(message: string): void {
    this.warnings.push(message);
  }

  /**
   * Check if there are any errors
   */
  hasErrors(): boolean {
    return this.errors.length > 0;
  }

  /**
   * Get all errors
   */
  getErrors(): WorkflowError[] {
    return this.errors;
  }

  /**
   * Get all warnings
   */
  getWarnings(): string[] {
    return this.warnings;
  }

  /**
   * Throw if there are errors
   */
  throwIfErrors(): void {
    if (this.errors.length > 0) {
      const firstError = this.errors[0];
      throw firstError;
    }
  }

  /**
   * Get summary of errors and warnings
   */
  getSummary() {
    return {
      errorCount: this.errors.length,
      errors: this.errors.map((e) => e.toJSON()),
      warningCount: this.warnings.length,
      warnings: this.warnings,
    };
  }
}
