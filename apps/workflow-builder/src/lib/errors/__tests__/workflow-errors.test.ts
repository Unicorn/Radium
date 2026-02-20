/**
 * Tests for Workflow Error Classes
 */

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
  isValidationError,
  isCompilationError,
  isExecutionError,
  isTimeoutError,
  isNetworkError,
  isDatabaseError,
} from '../workflow-errors';

describe('WorkflowError', () => {
  it('should create error with all properties', () => {
    const error = new WorkflowError({
      message: 'Test error',
      code: 'TEST_ERROR',
      statusCode: 500,
      userMessage: 'User-friendly message',
      recoverySuggestions: ['Try again'],
      details: { foo: 'bar' },
    });

    expect(error.message).toBe('Test error');
    expect(error.code).toBe('TEST_ERROR');
    expect(error.statusCode).toBe(500);
    expect(error.userMessage).toBe('User-friendly message');
    expect(error.recoverySuggestions).toEqual(['Try again']);
    expect(error.details).toEqual({ foo: 'bar' });
  });

  it('should format for JSON response', () => {
    const error = new WorkflowError({
      message: 'Test error',
      code: 'TEST_ERROR',
      userMessage: 'User-friendly message',
      recoverySuggestions: ['Suggestion 1', 'Suggestion 2'],
      details: { field: 'value' },
    });

    const json = error.toJSON();

    expect(json).toEqual({
      error: 'User-friendly message',
      code: 'TEST_ERROR',
      suggestions: ['Suggestion 1', 'Suggestion 2'],
      details: { field: 'value' },
    });
  });

  it('should format for logging', () => {
    const error = new WorkflowError({
      message: 'Test error',
      code: 'TEST_ERROR',
      statusCode: 500,
      userMessage: 'User-friendly message',
    });

    const logFormat = error.toLogFormat();

    expect(logFormat.name).toBe('WorkflowError');
    expect(logFormat.message).toBe('Test error');
    expect(logFormat.code).toBe('TEST_ERROR');
    expect(logFormat.statusCode).toBe(500);
    expect(logFormat.stack).toBeDefined();
  });

  it('should include original error in log format', () => {
    const original = new Error('Original error');
    const error = new WorkflowError({
      message: 'Wrapped error',
      code: 'TEST_ERROR',
      userMessage: 'User message',
      originalError: original,
    });

    const logFormat = error.toLogFormat();

    expect(logFormat.originalError).toBeDefined();
    expect(logFormat.originalError?.message).toBe('Original error');
  });
});

describe('ValidationError', () => {
  it('should create validation error with field info', () => {
    const error = new ValidationError({
      message: 'Invalid value',
      field: 'email',
      invalidValue: 'not-an-email',
      recoverySuggestions: ['Use valid email format'],
    });

    expect(error.field).toBe('email');
    expect(error.invalidValue).toBe('not-an-email');
    expect(error.code).toBe('VALIDATION_ERROR');
    expect(error.statusCode).toBe(400);
    expect(error.userMessage).toBe('Invalid value for email: Invalid value');
  });

  it('should include field in details', () => {
    const error = new ValidationError({
      message: 'Required field missing',
      field: 'username',
    });

    expect(error.details).toEqual({
      field: 'username',
      invalidValue: undefined,
    });
  });

  it('should work without field', () => {
    const error = new ValidationError({
      message: 'General validation error',
    });

    expect(error.userMessage).toBe('General validation error');
  });
});

describe('CompilationError', () => {
  it('should create compilation error with node info', () => {
    const error = new CompilationError({
      message: 'Missing component',
      nodeId: 'node-123',
      compilationPhase: 'validation',
      recoverySuggestions: ['Select a component'],
    });

    expect(error.nodeId).toBe('node-123');
    expect(error.compilationPhase).toBe('validation');
    expect(error.code).toBe('COMPILATION_ERROR');
    expect(error.userMessage).toBe('Compilation failed in node node-123: Missing component');
  });

  it('should work without node ID', () => {
    const error = new CompilationError({
      message: 'General compilation error',
    });

    expect(error.userMessage).toBe('Compilation failed: General compilation error');
  });

  it('should include default suggestions', () => {
    const error = new CompilationError({
      message: 'Error',
      nodeId: 'node-1',
    });

    expect(error.recoverySuggestions.length).toBeGreaterThan(0);
    expect(error.recoverySuggestions[0]).toContain('Check that all nodes');
  });
});

describe('ExecutionError', () => {
  it('should create execution error with workflow info', () => {
    const error = new ExecutionError({
      message: 'Activity failed',
      workflowId: 'wf-123',
      executionId: 'exec-456',
      failedStep: 'send-email',
      recoverySuggestions: ['Check email service'],
    });

    expect(error.workflowId).toBe('wf-123');
    expect(error.executionId).toBe('exec-456');
    expect(error.failedStep).toBe('send-email');
    expect(error.userMessage).toBe('Workflow execution failed at step "send-email": Activity failed');
  });

  it('should work without failed step', () => {
    const error = new ExecutionError({
      message: 'General execution error',
      workflowId: 'wf-123',
    });

    expect(error.userMessage).toBe('Workflow execution failed: General execution error');
  });
});

describe('TimeoutError', () => {
  it('should create timeout error with operation info', () => {
    const error = new TimeoutError({
      message: 'Operation timed out',
      timeoutMs: 30000,
      operation: 'database-query',
      recoverySuggestions: ['Increase timeout to 60s'],
    });

    expect(error.timeoutMs).toBe(30000);
    expect(error.operation).toBe('database-query');
    expect(error.code).toBe('TIMEOUT_ERROR');
    expect(error.statusCode).toBe(408);
    expect(error.userMessage).toBe('Operation "database-query" timed out after 30000ms');
  });

  it('should suggest increasing timeout', () => {
    const error = new TimeoutError({
      message: 'Timeout',
      timeoutMs: 5000,
      operation: 'api-call',
    });

    const suggestion = error.recoverySuggestions.find((s) =>
      s.includes('Increase the timeout')
    );
    expect(suggestion).toBeDefined();
    expect(suggestion).toContain('5000ms');
  });
});

describe('NetworkError', () => {
  it('should create network error with URL', () => {
    const error = new NetworkError({
      message: 'Connection refused',
      url: 'https://api.example.com',
      statusCode: 503,
    });

    expect(error.url).toBe('https://api.example.com');
    expect(error.statusCode).toBe(503);
    expect(error.userMessage).toContain('https://api.example.com');
  });

  it('should include retry suggestion', () => {
    const error = new NetworkError({
      message: 'Network error',
      url: 'https://api.example.com',
    });

    const hasRetry = error.recoverySuggestions.some((s) => s.includes('retry'));
    expect(hasRetry).toBe(true);
  });
});

describe('DatabaseError', () => {
  it('should create database error without exposing details', () => {
    const error = new DatabaseError({
      message: 'Connection pool exhausted',
      operation: 'insert',
      table: 'workflows',
    });

    expect(error.operation).toBe('insert');
    expect(error.table).toBe('workflows');
    expect(error.code).toBe('DATABASE_ERROR');
    // User message should NOT contain sensitive info
    expect(error.userMessage).toBe('A database error occurred. Please try again later.');
    expect(error.userMessage).not.toContain('pool');
    expect(error.userMessage).not.toContain('workflows');
  });

  it('should include details in log format but not JSON', () => {
    const error = new DatabaseError({
      message: 'Deadlock detected',
      operation: 'update',
      table: 'users',
    });

    const json = error.toJSON();
    const log = error.toLogFormat();

    // JSON (user-facing) should not have sensitive details
    expect(json.error).not.toContain('Deadlock');

    // Log format should have full details
    expect(log.message).toContain('Deadlock');
    expect(log.details?.operation).toBe('update');
    expect(log.details?.table).toBe('users');
  });
});

describe('AuthorizationError', () => {
  it('should create authorization error', () => {
    const error = new AuthorizationError({
      message: 'Not authorized',
      resourceType: 'workflow',
      resourceId: 'wf-123',
      requiredPermission: 'edit',
    });

    expect(error.resourceType).toBe('workflow');
    expect(error.resourceId).toBe('wf-123');
    expect(error.requiredPermission).toBe('edit');
    expect(error.statusCode).toBe(403);
    expect(error.userMessage).toContain('workflow');
  });
});

describe('NotFoundError', () => {
  it('should create not found error with resource info', () => {
    const error = new NotFoundError({
      message: 'Not found',
      resourceType: 'Workflow',
      resourceId: 'wf-123',
    });

    expect(error.resourceType).toBe('Workflow');
    expect(error.resourceId).toBe('wf-123');
    expect(error.statusCode).toBe(404);
    expect(error.userMessage).toBe('Workflow with ID wf-123 not found');
  });

  it('should work without resource ID', () => {
    const error = new NotFoundError({
      message: 'Not found',
      resourceType: 'Component',
    });

    expect(error.userMessage).toBe('Component not found');
  });
});

describe('ConflictError', () => {
  it('should create conflict error', () => {
    const error = new ConflictError({
      message: 'Workflow name already exists',
      conflictType: 'duplicate-name',
    });

    expect(error.conflictType).toBe('duplicate-name');
    expect(error.statusCode).toBe(409);
  });
});

describe('RateLimitError', () => {
  it('should create rate limit error with retry time', () => {
    const error = new RateLimitError({
      message: 'Rate limit exceeded',
      retryAfterMs: 60000,
    });

    expect(error.retryAfterMs).toBe(60000);
    expect(error.statusCode).toBe(429);
    expect(error.userMessage).toContain('60 seconds');
  });

  it('should work without retry time', () => {
    const error = new RateLimitError({
      message: 'Rate limit exceeded',
    });

    expect(error.userMessage).toContain('try again later');
  });
});

describe('toWorkflowError', () => {
  it('should pass through WorkflowError unchanged', () => {
    const original = new ValidationError({
      message: 'Test',
      field: 'test',
    });

    const result = toWorkflowError(original);

    expect(result).toBe(original);
  });

  it('should wrap regular Error', () => {
    const original = new Error('Test error');
    const result = toWorkflowError(original);

    expect(result).toBeInstanceOf(WorkflowError);
    expect(result.code).toBe('UNKNOWN_ERROR');
    expect(result.originalError).toBe(original);
  });

  it('should wrap unknown values', () => {
    const result = toWorkflowError('String error');

    expect(result).toBeInstanceOf(WorkflowError);
    expect(result.message).toBe('String error');
    expect(result.code).toBe('UNKNOWN_ERROR');
  });
});

describe('Type Guards', () => {
  it('should correctly identify ValidationError', () => {
    const validationError = new ValidationError({ message: 'Test', field: 'test' });
    const otherError = new TimeoutError({ message: 'Test', timeoutMs: 1000, operation: 'test' });

    expect(isValidationError(validationError)).toBe(true);
    expect(isValidationError(otherError)).toBe(false);
    expect(isValidationError(new Error())).toBe(false);
  });

  it('should correctly identify CompilationError', () => {
    const compilationError = new CompilationError({ message: 'Test', nodeId: 'test' });
    const otherError = new ValidationError({ message: 'Test', field: 'test' });

    expect(isCompilationError(compilationError)).toBe(true);
    expect(isCompilationError(otherError)).toBe(false);
  });

  it('should correctly identify ExecutionError', () => {
    const executionError = new ExecutionError({ message: 'Test', workflowId: 'wf-1' });
    const otherError = new NetworkError({ message: 'Test', url: 'http://test' });

    expect(isExecutionError(executionError)).toBe(true);
    expect(isExecutionError(otherError)).toBe(false);
  });

  it('should correctly identify TimeoutError', () => {
    const timeoutError = new TimeoutError({ message: 'Test', timeoutMs: 1000, operation: 'test' });
    const otherError = new ValidationError({ message: 'Test', field: 'test' });

    expect(isTimeoutError(timeoutError)).toBe(true);
    expect(isTimeoutError(otherError)).toBe(false);
  });

  it('should correctly identify NetworkError', () => {
    const networkError = new NetworkError({ message: 'Test', url: 'http://test' });
    const otherError = new DatabaseError({ message: 'Test', operation: 'select' });

    expect(isNetworkError(networkError)).toBe(true);
    expect(isNetworkError(otherError)).toBe(false);
  });

  it('should correctly identify DatabaseError', () => {
    const dbError = new DatabaseError({ message: 'Test', operation: 'insert' });
    const otherError = new NetworkError({ message: 'Test', url: 'http://test' });

    expect(isDatabaseError(dbError)).toBe(true);
    expect(isDatabaseError(otherError)).toBe(false);
  });
});

describe('Error Hierarchy', () => {
  it('should all extend WorkflowError', () => {
    const errors = [
      new ValidationError({ message: 'Test', field: 'test' }),
      new CompilationError({ message: 'Test', nodeId: 'test' }),
      new ExecutionError({ message: 'Test', workflowId: 'wf-1' }),
      new TimeoutError({ message: 'Test', timeoutMs: 1000, operation: 'test' }),
      new NetworkError({ message: 'Test', url: 'http://test' }),
      new DatabaseError({ message: 'Test', operation: 'select' }),
      new AuthorizationError({ message: 'Test', resourceType: 'workflow' }),
      new NotFoundError({ message: 'Test', resourceType: 'workflow' }),
      new ConflictError({ message: 'Test', conflictType: 'duplicate' }),
      new RateLimitError({ message: 'Test' }),
    ];

    errors.forEach((error) => {
      expect(error).toBeInstanceOf(WorkflowError);
      expect(error).toBeInstanceOf(Error);
    });
  });

  it('should have proper error names', () => {
    const errors = [
      { error: new ValidationError({ message: 'Test', field: 'test' }), name: 'ValidationError' },
      { error: new CompilationError({ message: 'Test', nodeId: 'test' }), name: 'CompilationError' },
      { error: new ExecutionError({ message: 'Test', workflowId: 'wf-1' }), name: 'ExecutionError' },
      { error: new TimeoutError({ message: 'Test', timeoutMs: 1000, operation: 'test' }), name: 'TimeoutError' },
      { error: new DatabaseError({ message: 'Test', operation: 'select' }), name: 'DatabaseError' },
    ];

    errors.forEach(({ error, name }) => {
      expect(error.name).toBe(name);
    });
  });
});
