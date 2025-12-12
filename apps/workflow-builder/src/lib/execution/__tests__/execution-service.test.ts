/**
 * ExecutionService Tests
 *
 * These tests verify the ExecutionService class provides correct CRUD operations
 * for workflow executions. We use a mock Supabase client to avoid database dependencies.
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { ExecutionService, type CreateExecutionInput } from '../execution-service';
import type { SupabaseClient } from '@supabase/supabase-js';
import type { Database } from '@/types/database';

// Mock Supabase client
function createMockSupabaseClient() {
  const mockData: any[] = [];

  // Create a chain that always returns itself for fluent API
  const mockChain: any = {
    select: vi.fn(),
    insert: vi.fn(),
    update: vi.fn(),
    delete: vi.fn(),
    eq: vi.fn(),
    order: vi.fn(),
    range: vi.fn(),
    single: vi.fn(),
  };

  // Make each method return the chain itself
  mockChain.select.mockReturnValue(mockChain);
  mockChain.insert.mockReturnValue(mockChain);
  mockChain.update.mockReturnValue(mockChain);
  mockChain.delete.mockReturnValue(mockChain);
  mockChain.eq.mockReturnValue(mockChain);
  mockChain.order.mockReturnValue(mockChain);
  mockChain.range.mockReturnValue(mockChain);

  const mock = {
    from: vi.fn(() => mockChain),
    _mockData: mockData,
    _mockChain: mockChain,
  };

  return mock as any as SupabaseClient<Database>;
}

describe('ExecutionService', () => {
  let service: ExecutionService;
  let mockSupabase: any;

  beforeEach(() => {
    mockSupabase = createMockSupabaseClient();
    service = new ExecutionService(mockSupabase);

    // Reset all mocks between tests
    vi.clearAllMocks();
  });

  describe('createExecution', () => {
    it('should create a new execution record', async () => {
      const input: CreateExecutionInput = {
        workflowId: 'workflow-123',
        temporalWorkflowId: 'wf-abc-123',
        temporalRunId: 'run-xyz-456',
        userId: 'user-1',
        input: { param1: 'value1' },
      };

      const mockExecution = {
        id: 'exec-1',
        workflow_id: input.workflowId,
        temporal_workflow_id: input.temporalWorkflowId,
        temporal_run_id: input.temporalRunId,
        status: 'running',
        input: input.input,
        created_by: input.userId,
        started_at: new Date().toISOString(),
        created_at: new Date().toISOString(),
        completed_at: null,
        output: null,
        error_message: null,
        duration_ms: null,
        activities_executed: null,
      };

      mockSupabase._mockChain.single.mockResolvedValueOnce({
        data: mockExecution,
        error: null,
      });

      const result = await service.createExecution(input);

      expect(result).toEqual(mockExecution);
      expect(mockSupabase.from).toHaveBeenCalledWith('workflow_executions');
      expect(mockSupabase._mockChain.insert).toHaveBeenCalledWith(
        expect.objectContaining({
          workflow_id: input.workflowId,
          temporal_workflow_id: input.temporalWorkflowId,
          temporal_run_id: input.temporalRunId,
          status: 'running',
          created_by: input.userId,
        })
      );
    });

    it('should throw error if creation fails', async () => {
      const input: CreateExecutionInput = {
        workflowId: 'workflow-123',
        temporalWorkflowId: 'wf-abc-123',
        temporalRunId: 'run-xyz-456',
        userId: 'user-1',
      };

      mockSupabase._mockChain.single.mockResolvedValueOnce({
        data: null,
        error: { message: 'Database error' },
      });

      await expect(service.createExecution(input)).rejects.toThrow(
        'Failed to create execution: Database error'
      );
    });
  });

  describe('updateExecution', () => {
    it('should update execution status to completed', async () => {
      const temporalWorkflowId = 'wf-abc-123';
      const completedAt = new Date();
      const mockUpdated = {
        id: 'exec-1',
        status: 'completed',
        completed_at: completedAt.toISOString(),
        duration_ms: 5000,
        output: { result: 'success' },
      };

      mockSupabase._mockChain.single.mockResolvedValueOnce({
        data: mockUpdated,
        error: null,
      });

      const result = await service.updateExecution(temporalWorkflowId, {
        status: 'completed',
        completedAt,
        durationMs: 5000,
        output: { result: 'success' },
      });

      expect(result.status).toBe('completed');
      expect(result.duration_ms).toBe(5000);
      expect(mockSupabase._mockChain.eq).toHaveBeenCalledWith(
        'temporal_workflow_id',
        temporalWorkflowId
      );
    });

    it('should update execution with error status', async () => {
      const temporalWorkflowId = 'wf-abc-123';
      const mockUpdated = {
        id: 'exec-1',
        status: 'failed',
        error_message: 'Activity failed',
        completed_at: new Date().toISOString(),
      };

      mockSupabase._mockChain.single.mockResolvedValueOnce({
        data: mockUpdated,
        error: null,
      });

      const result = await service.updateExecution(temporalWorkflowId, {
        status: 'failed',
        error: 'Activity failed',
        completedAt: new Date(),
      });

      expect(result.status).toBe('failed');
      expect(result.error_message).toBe('Activity failed');
    });
  });

  describe('getExecution', () => {
    it('should retrieve execution by temporal workflow ID', async () => {
      const temporalWorkflowId = 'wf-abc-123';
      const mockExecution = {
        id: 'exec-1',
        temporal_workflow_id: temporalWorkflowId,
        status: 'running',
        workflow: {
          id: 'workflow-1',
          name: 'test-workflow',
          display_name: 'Test Workflow',
          description: 'A test workflow',
        },
      };

      mockSupabase._mockChain.single.mockResolvedValueOnce({
        data: mockExecution,
        error: null,
      });

      const result = await service.getExecution(temporalWorkflowId);

      expect(result).toEqual(mockExecution);
      expect(mockSupabase._mockChain.eq).toHaveBeenCalledWith(
        'temporal_workflow_id',
        temporalWorkflowId
      );
    });

    it('should return null if execution not found', async () => {
      mockSupabase._mockChain.single.mockResolvedValueOnce({
        data: null,
        error: { code: 'PGRST116', message: 'Not found' },
      });

      const result = await service.getExecution('non-existent-id');

      expect(result).toBeNull();
    });

    it('should throw error for database errors', async () => {
      mockSupabase._mockChain.single.mockResolvedValueOnce({
        data: null,
        error: { code: 'SOME_ERROR', message: 'Database error' },
      });

      await expect(service.getExecution('wf-123')).rejects.toThrow(
        'Failed to get execution: Database error'
      );
    });
  });

  describe('listExecutions', () => {
    it('should list executions for a workflow', async () => {
      const workflowId = 'workflow-123';
      const mockExecutions = [
        {
          id: 'exec-1',
          workflow_id: workflowId,
          status: 'completed',
          started_at: '2025-01-19T10:00:00Z',
        },
        {
          id: 'exec-2',
          workflow_id: workflowId,
          status: 'running',
          started_at: '2025-01-19T11:00:00Z',
        },
      ];

      mockSupabase._mockChain.range.mockResolvedValueOnce({
        data: mockExecutions,
        error: null,
      });

      const result = await service.listExecutions(workflowId);

      expect(result).toHaveLength(2);
      expect(result).toEqual(mockExecutions);
      expect(mockSupabase._mockChain.eq).toHaveBeenCalledWith(
        'workflow_id',
        workflowId
      );
    });

    it('should filter executions by status', async () => {
      const workflowId = 'workflow-123';
      const mockExecutions = [
        {
          id: 'exec-1',
          workflow_id: workflowId,
          status: 'completed',
        },
      ];

      // For this test, we need to ensure the promise resolves at the END of the chain
      // The code does: .select().eq(workflow_id).order().range() then conditionally .eq(status)
      // So the final operation that should resolve is the last .eq() call
      const freshMockSupabase = createMockSupabaseClient();
      const freshService = new ExecutionService(freshMockSupabase);

      // Set up the chain so that the last operation (the second eq call for status) resolves
      freshMockSupabase._mockChain.eq
        .mockReturnValueOnce(freshMockSupabase._mockChain) // First eq (workflow_id) returns chain
        .mockResolvedValueOnce({ data: mockExecutions, error: null }); // Second eq (status) resolves

      const result = await freshService.listExecutions(workflowId, { status: 'completed' });

      // Verify both eq calls were made
      expect(freshMockSupabase._mockChain.eq).toHaveBeenNthCalledWith(1, 'workflow_id', workflowId);
      expect(freshMockSupabase._mockChain.eq).toHaveBeenNthCalledWith(2, 'status', 'completed');
      expect(result).toEqual(mockExecutions);
    });

    it('should apply pagination', async () => {
      const workflowId = 'workflow-123';

      mockSupabase._mockChain.range.mockResolvedValueOnce({
        data: [],
        error: null,
      });

      await service.listExecutions(workflowId, {
        limit: 10,
        offset: 20,
      });

      expect(mockSupabase._mockChain.range).toHaveBeenCalledWith(20, 29);
    });
  });

  describe('getStats', () => {
    it('should calculate execution statistics', async () => {
      const workflowId = 'workflow-123';
      const mockExecutions = [
        { status: 'completed', duration_ms: 1000 },
        { status: 'completed', duration_ms: 2000 },
        { status: 'completed', duration_ms: 3000 },
        { status: 'failed', duration_ms: 1500 },
        { status: 'running', duration_ms: null },
      ];

      mockSupabase.from().select().eq = vi.fn().mockResolvedValueOnce({
        data: mockExecutions,
        error: null,
      });

      const stats = await service.getStats(workflowId);

      expect(stats.total).toBe(5);
      expect(stats.completed).toBe(3);
      expect(stats.failed).toBe(1);
      expect(stats.running).toBe(1);
      expect(stats.avgDuration).toBe(1875); // (1000+2000+3000+1500)/4
      expect(stats.minDuration).toBe(1000);
      expect(stats.maxDuration).toBe(3000);
      expect(stats.successRate).toBe(60); // 3/5 * 100
    });

    it('should handle no executions', async () => {
      mockSupabase.from().select().eq = vi.fn().mockResolvedValueOnce({
        data: [],
        error: null,
      });

      const stats = await service.getStats('workflow-123');

      expect(stats.total).toBe(0);
      expect(stats.completed).toBe(0);
      expect(stats.running).toBe(0);
      expect(stats.avgDuration).toBe(0);
      expect(stats.minDuration).toBeNull();
      expect(stats.maxDuration).toBeNull();
      expect(stats.successRate).toBe(0);
    });

    it('should handle executions with no durations', async () => {
      const mockExecutions = [
        { status: 'running', duration_ms: null },
        { status: 'running', duration_ms: null },
      ];

      mockSupabase.from().select().eq = vi.fn().mockResolvedValueOnce({
        data: mockExecutions,
        error: null,
      });

      const stats = await service.getStats('workflow-123');

      expect(stats.total).toBe(2);
      expect(stats.avgDuration).toBe(0);
      expect(stats.minDuration).toBeNull();
      expect(stats.maxDuration).toBeNull();
    });
  });

  describe('markCompleted', () => {
    it('should mark execution as completed with output', async () => {
      const temporalWorkflowId = 'wf-abc-123';
      const output = { result: 'success', data: [1, 2, 3] };
      const durationMs = 5500;

      const mockUpdated = {
        id: 'exec-1',
        status: 'completed',
        output,
        duration_ms: durationMs,
        completed_at: expect.any(String),
      };

      mockSupabase._mockChain.single.mockResolvedValueOnce({
        data: mockUpdated,
        error: null,
      });

      const result = await service.markCompleted(
        temporalWorkflowId,
        output,
        durationMs
      );

      expect(result.status).toBe('completed');
      expect(result.output).toEqual(output);
      expect(result.duration_ms).toBe(durationMs);
    });
  });

  describe('markFailed', () => {
    it('should mark execution as failed with error message', async () => {
      const temporalWorkflowId = 'wf-abc-123';
      const error = 'Network timeout';

      const mockUpdated = {
        id: 'exec-1',
        status: 'failed',
        error_message: error,
        completed_at: expect.any(String),
      };

      mockSupabase._mockChain.single.mockResolvedValueOnce({
        data: mockUpdated,
        error: null,
      });

      const result = await service.markFailed(temporalWorkflowId, error);

      expect(result.status).toBe('failed');
      expect(result.error_message).toBe(error);
    });
  });

  describe('deleteExecution', () => {
    it('should delete an execution by ID', async () => {
      const executionId = 'exec-1';

      mockSupabase._mockChain.eq.mockResolvedValueOnce({
        error: null,
      });

      await service.deleteExecution(executionId);

      expect(mockSupabase.from).toHaveBeenCalledWith('workflow_executions');
      expect(mockSupabase._mockChain.delete).toHaveBeenCalled();
      expect(mockSupabase._mockChain.eq).toHaveBeenCalledWith('id', executionId);
    });

    it('should throw error if deletion fails', async () => {
      mockSupabase._mockChain.eq.mockResolvedValueOnce({
        error: { message: 'Cannot delete' },
      });

      await expect(service.deleteExecution('exec-1')).rejects.toThrow(
        'Failed to delete execution: Cannot delete'
      );
    });
  });

  describe('listExecutionsByUser', () => {
    it('should list executions for a specific user', async () => {
      const userId = 'user-1';
      const mockExecutions = [
        {
          id: 'exec-1',
          created_by: userId,
          status: 'completed',
          workflow: {
            id: 'workflow-1',
            name: 'test-workflow',
            display_name: 'Test Workflow',
            description: null,
          },
        },
        {
          id: 'exec-2',
          created_by: userId,
          status: 'running',
          workflow: {
            id: 'workflow-2',
            name: 'another-workflow',
            display_name: 'Another Workflow',
            description: null,
          },
        },
      ];

      mockSupabase._mockChain.range.mockResolvedValueOnce({
        data: mockExecutions,
        error: null,
      });

      const result = await service.listExecutionsByUser(userId);

      expect(result).toHaveLength(2);
      expect(result[0]).toHaveProperty('workflow');
      expect(mockSupabase._mockChain.eq).toHaveBeenCalledWith('created_by', userId);
    });
  });

  describe('updateExecutionById', () => {
    it('should update execution by database ID', async () => {
      const executionId = 'exec-1';
      const mockUpdated = {
        id: executionId,
        status: 'completed',
        duration_ms: 3000,
      };

      mockSupabase._mockChain.single.mockResolvedValueOnce({
        data: mockUpdated,
        error: null,
      });

      const result = await service.updateExecutionById(executionId, {
        status: 'completed',
        durationMs: 3000,
      });

      expect(result.id).toBe(executionId);
      expect(result.status).toBe('completed');
      expect(mockSupabase._mockChain.eq).toHaveBeenCalledWith('id', executionId);
    });
  });
});
