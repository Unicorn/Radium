/**
 * Error Handling System
 * Centralized error handling, validation, and logging
 */

// Error classes
export {
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
  isValidationError,
  isCompilationError,
  isExecutionError,
  isTimeoutError,
  isNetworkError,
  isDatabaseError,
} from './workflow-errors';

// Error handler middleware
export {
  handleError,
  workflowErrorToTRPC,
  withErrorHandling,
  withDatabaseErrorHandling,
  withNetworkErrorHandling,
  validateSupabaseResponse,
  requireAuthorization,
  validateInput,
  withRetry,
  ErrorBoundary as ErrorBoundaryClass,
  configureErrorLogger,
  consoleLogger,
  type ErrorLogger,
} from './error-handler';

// Error logging
export {
  ErrorSeverity,
  ConsoleErrorLogger,
  DatabaseErrorLogger,
  CompositeErrorLogger,
  setGlobalErrorLogger,
  getGlobalErrorLogger,
  logger,
  calculateErrorMetrics,
  type IErrorLogger,
  type ErrorLogEntry,
  type ErrorMetrics,
} from './error-logger';
