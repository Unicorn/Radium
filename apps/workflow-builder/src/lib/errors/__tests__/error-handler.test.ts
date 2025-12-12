/**
 * Tests for Error Handler Middleware
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { TRPCError } from '@trpc/server';
import {
  handleError,
  workflowErrorToTRPC,
  withErrorHandling,
  withDatabaseErrorHandling,
  validateSupabaseResponse,
  requireAuthorization,
  validateInput,
  ErrorBoundary,
  configureErrorLogger,
  consoleLogger,
} from '../error-handler';
import {
  WorkflowError,
  ValidationError,
  DatabaseError,
  NotFoundError,
  AuthorizationError,
  TimeoutError,
} from '../workflow-errors';

describe('Error Handler', () => {
  beforeEach(() => {
    // Reset logger
    configureErrorLogger(consoleLogger);
    // Spy on console
    vi.spyOn(console, 'error').mockImplementation(() => {});
    vi.spyOn(console, 'warn').mockImplementation(() => {});
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('workflowErrorToTRPC', () => {
    it('should convert ValidationError to BAD_REQUEST', () => {
      const error = new ValidationError({
        message: 'Invalid input',
        field: 'email',
      });

      const trpcError = workflowErrorToTRPC(error);

      expect(trpcError.code).toBe('BAD_REQUEST');
      expect(trpcError.message).toContain('Invalid value for email');
    });

    it('should convert NotFoundError to NOT_FOUND', () => {
      const error = new NotFoundError({
        message: 'Not found',
        resourceType: 'Workflow',
        resourceId: 'wf-123',
      });

      const trpcError = workflowErrorToTRPC(error);

      expect(trpcError.code).toBe('NOT_FOUND');
      expect(trpcError.message).toContain('Workflow');
    });

    it('should convert AuthorizationError to FORBIDDEN', () => {
      const error = new AuthorizationError({
        message: 'Not authorized',
        resourceType: 'workflow',
      });

      const trpcError = workflowErrorToTRPC(error);

      expect(trpcError.code).toBe('FORBIDDEN');
    });

    it('should convert TimeoutError to TIMEOUT', () => {
      const error = new TimeoutError({
        message: 'Timeout',
        timeoutMs: 30000,
        operation: 'test',
      });

      const trpcError = workflowErrorToTRPC(error);

      expect(trpcError.code).toBe('TIMEOUT');
    });

    it('should include error cause with details', () => {
      const error = new ValidationError({
        message: 'Invalid',
        field: 'test',
        recoverySuggestions: ['Fix it'],
        details: { foo: 'bar' },
      });

      const trpcError = workflowErrorToTRPC(error);

      // TRPCError wraps cause in an Error object
      expect(trpcError.cause).toBeDefined();
      const cause = trpcError.cause as any;
      expect(cause.code).toBe('VALIDATION_ERROR');
      expect(cause.suggestions).toEqual(['Fix it']);
      expect(cause.details).toMatchObject({ foo: 'bar' });
    });
  });

  describe('handleError', () => {
    it('should log and throw tRPC error', () => {
      const error = new ValidationError({
        message: 'Test error',
        field: 'test',
      });

      expect(() => handleError(error)).toThrow(TRPCError);
      expect(console.warn).toHaveBeenCalled();
    });

    it('should log database errors without exposing details', () => {
      const error = new DatabaseError({
        message: 'Connection pool exhausted',
        operation: 'insert',
        table: 'workflows',
      });

      expect(() => handleError(error)).toThrow(TRPCError);
      expect(console.error).toHaveBeenCalledWith(
        expect.stringContaining('Database error'),
        expect.any(Object)
      );

      try {
        handleError(error);
      } catch (e: any) {
        expect(e.message).not.toContain('pool');
        expect(e.message).not.toContain('workflows');
      }
    });

    it('should convert unknown errors to WorkflowError', () => {
      const error = new Error('Regular error');

      expect(() => handleError(error)).toThrow(TRPCError);
    });
  });

  describe('withErrorHandling', () => {
    it('should execute operation successfully', async () => {
      const operation = vi.fn().mockResolvedValue('success');

      const result = await withErrorHandling(operation, 'test operation');

      expect(result).toBe('success');
      expect(operation).toHaveBeenCalled();
    });

    it('should handle errors and convert to tRPC', async () => {
      const operation = vi.fn().mockRejectedValue(new Error('Test error'));

      await expect(withErrorHandling(operation, 'test operation')).rejects.toThrow(
        TRPCError
      );
      expect(console.error).toHaveBeenCalledWith(
        expect.stringContaining('test operation'),
        expect.any(Object)
      );
    });
  });

  describe('withDatabaseErrorHandling', () => {
    it('should return data on success', async () => {
      const operation = vi.fn().mockResolvedValue({
        data: { id: '123', name: 'Test' },
        error: null,
      });

      const result = await withDatabaseErrorHandling(operation, 'select', 'workflows');

      expect(result).toEqual({ id: '123', name: 'Test' });
    });

    it('should throw DatabaseError on error', async () => {
      const operation = vi.fn().mockResolvedValue({
        data: null,
        error: { message: 'Database error' },
      });

      await expect(
        withDatabaseErrorHandling(operation, 'insert', 'workflows')
      ).rejects.toThrow(TRPCError);
    });

    it('should throw NotFoundError when data is null', async () => {
      const operation = vi.fn().mockResolvedValue({
        data: null,
        error: null,
      });

      await expect(
        withDatabaseErrorHandling(operation, 'select', 'workflows')
      ).rejects.toThrow(TRPCError);
    });
  });

  describe('validateSupabaseResponse', () => {
    it('should return data on success', () => {
      const result = validateSupabaseResponse(
        { data: { id: '123' }, error: null },
        'select',
        'workflows'
      );

      expect(result).toEqual({ id: '123' });
    });

    it('should throw DatabaseError on error', () => {
      expect(() =>
        validateSupabaseResponse(
          { data: null, error: { message: 'Error' } },
          'insert',
          'workflows'
        )
      ).toThrow(DatabaseError);
    });

    it('should throw NotFoundError when data is null', () => {
      expect(() =>
        validateSupabaseResponse({ data: null, error: null }, 'select', 'workflows')
      ).toThrow(NotFoundError);
    });
  });

  describe('requireAuthorization', () => {
    it('should not throw when authorized', () => {
      expect(() =>
        requireAuthorization(true, 'workflow', 'wf-123')
      ).not.toThrow();
    });

    it('should throw AuthorizationError when unauthorized', () => {
      expect(() =>
        requireAuthorization(false, 'workflow', 'wf-123', 'edit')
      ).toThrow(AuthorizationError);
    });
  });

  describe('validateInput', () => {
    it('should not throw when valid', () => {
      expect(() =>
        validateInput(true, 'email', 'Email is required')
      ).not.toThrow();
    });

    it('should throw ValidationError when invalid', () => {
      expect(() =>
        validateInput(false, 'email', 'Invalid email', 'not-an-email', [
          'Use valid format',
        ])
      ).toThrow(ValidationError);
    });
  });

  describe('ErrorBoundary', () => {
    it('should collect errors without throwing', async () => {
      const boundary = new ErrorBoundary();

      const result = await boundary.try(async () => {
        throw new Error('Test error');
      });

      expect(result).toBeNull();
      expect(boundary.hasErrors()).toBe(true);
      expect(boundary.getErrors()).toHaveLength(1);
    });

    it('should return result on success', async () => {
      const boundary = new ErrorBoundary();

      const result = await boundary.try(async () => {
        return 'success';
      });

      expect(result).toBe('success');
      expect(boundary.hasErrors()).toBe(false);
    });

    it('should collect multiple errors', async () => {
      const boundary = new ErrorBoundary();

      await boundary.try(async () => {
        throw new Error('Error 1');
      });
      await boundary.try(async () => {
        throw new Error('Error 2');
      });

      expect(boundary.getErrors()).toHaveLength(2);
    });

    it('should collect warnings', () => {
      const boundary = new ErrorBoundary();

      boundary.warn('Warning 1');
      boundary.warn('Warning 2');

      expect(boundary.getWarnings()).toHaveLength(2);
    });

    it('should throw first error when throwIfErrors is called', async () => {
      const boundary = new ErrorBoundary();

      await boundary.try(async () => {
        throw new Error('Test error');
      });

      expect(() => boundary.throwIfErrors()).toThrow(WorkflowError);
    });

    it('should not throw when no errors', () => {
      const boundary = new ErrorBoundary();

      expect(() => boundary.throwIfErrors()).not.toThrow();
    });

    it('should provide summary', async () => {
      const boundary = new ErrorBoundary();

      await boundary.try(async () => {
        throw new ValidationError({ message: 'Error', field: 'test' });
      });
      boundary.warn('Warning');

      const summary = boundary.getSummary();

      expect(summary.errorCount).toBe(1);
      expect(summary.warningCount).toBe(1);
      expect(summary.errors).toHaveLength(1);
      expect(summary.warnings).toHaveLength(1);
    });
  });

  describe('Custom Error Logger', () => {
    it('should use custom logger', () => {
      const customLogger = {
        error: vi.fn(),
        warn: vi.fn(),
      };

      configureErrorLogger(customLogger as any);

      const error = new ValidationError({
        message: 'Test',
        field: 'test',
      });

      try {
        handleError(error);
      } catch (e) {
        // Expected
      }

      expect(customLogger.warn).toHaveBeenCalled();
    });
  });
});
