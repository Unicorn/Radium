/**
 * Tests for Interface Component Manager
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import {
  createServiceInterfaceFromComponent,
  getServiceInterfaceForComponent,
  deleteServiceInterfaceForComponent,
} from '../interface-component-manager';
import type { SupabaseClient } from '@supabase/supabase-js';
import type { Database } from '@/types/database';

// Mock Supabase client
const createMockSupabase = () => {
  const mockSupabase = {
    from: vi.fn(),
  } as unknown as SupabaseClient<Database>;

  return mockSupabase;
};

type MockQueryBuilder = {
  select?: (columns?: string) => MockQueryBuilder;
  eq?: (column: string, value: unknown) => MockQueryBuilder;
  single?: () => Promise<{ data: unknown; error: unknown }>;
  insert?: (values: unknown) => MockQueryBuilder;
  update?: (values: unknown) => MockQueryBuilder;
  delete?: () => MockQueryBuilder;
};

describe('Interface Component Manager', () => {
  let mockSupabase: SupabaseClient<Database>;
  let mockFrom: ReturnType<typeof vi.fn<[string], Partial<MockQueryBuilder>>>;

  beforeEach(() => {
    mockSupabase = createMockSupabase();
    mockFrom = vi.fn();
    (mockSupabase.from as unknown as typeof mockFrom) = mockFrom;
  });

  describe('createServiceInterfaceFromComponent', () => {
    it('should create service interface for data-in component', async () => {
      const componentId = 'comp-123';
      const workflowId = 'wf-123';
      const config = {
        endpointPath: '/receive-data',
        httpMethod: 'POST' as const,
        inputSchema: { type: 'object' },
        isPublic: true,
      };

      const mockComponent = {
        id: componentId,
        name: 'receiveData',
        display_name: 'Receive Data',
        description: 'Receives incoming data',
        component_type: { id: 'type-1', name: 'data-in' },
        input_schema: null,
        output_schema: null,
      };

      const mockServiceInterface = {
        id: 'si-123',
        workflow_id: workflowId,
        name: 'receiveData',
        display_name: 'Receive Data',
        interface_type: 'signal',
        callable_name: 'receive-data',
        activity_connection_id: componentId,
        is_public: true,
      };

      // Mock component fetch
      const mockSelect = vi.fn().mockReturnThis();
      const mockEq = vi.fn().mockReturnThis();
      const mockSingle = vi.fn().mockResolvedValue({
        data: mockComponent,
        error: null,
      });

      mockFrom.mockReturnValueOnce({
        select: mockSelect,
        eq: mockEq,
        single: mockSingle,
      });

      // Mock service interface insert
      const mockInsert = vi.fn().mockReturnThis();
      const mockInsertSelect = vi.fn().mockReturnThis();
      const mockInsertSingle = vi.fn().mockResolvedValue({
        data: mockServiceInterface,
        error: null,
      });

      mockFrom.mockReturnValueOnce({
        insert: mockInsert,
        select: mockInsertSelect,
        single: mockInsertSingle,
      });

      const result = await createServiceInterfaceFromComponent(
        componentId,
        workflowId,
        config,
        mockSupabase
      );

      expect(result).toEqual(mockServiceInterface);
      expect(mockFrom).toHaveBeenCalledWith('components');
      expect(mockFrom).toHaveBeenCalledWith('service_interfaces');
      expect(mockInsert).toHaveBeenCalledWith(
        expect.objectContaining({
          workflow_id: workflowId,
          name: 'receiveData',
          interface_type: 'signal',
          is_public: true,
        })
      );
    });

    it('should create service interface for data-out component', async () => {
      const componentId = 'comp-456';
      const workflowId = 'wf-456';
      const config = {
        endpointPath: '/get-data',
        httpMethod: 'GET' as const,
        outputSchema: { type: 'object' },
        isPublic: true,
      };

      const mockComponent = {
        id: componentId,
        name: 'getData',
        display_name: 'Get Data',
        description: 'Provides data',
        component_type: { id: 'type-2', name: 'data-out' },
        input_schema: null,
        output_schema: null,
      };

      const mockServiceInterface = {
        id: 'si-456',
        workflow_id: workflowId,
        name: 'getData',
        display_name: 'Get Data',
        interface_type: 'query',
        callable_name: 'get-data',
        activity_connection_id: componentId,
        is_public: true,
      };

      // Mock component fetch
      const mockSelect = vi.fn().mockReturnThis();
      const mockEq = vi.fn().mockReturnThis();
      const mockSingle = vi.fn().mockResolvedValue({
        data: mockComponent,
        error: null,
      });

      mockFrom.mockReturnValueOnce({
        select: mockSelect,
        eq: mockEq,
        single: mockSingle,
      });

      // Mock service interface insert
      const mockInsert = vi.fn().mockReturnThis();
      const mockInsertSelect = vi.fn().mockReturnThis();
      const mockInsertSingle = vi.fn().mockResolvedValue({
        data: mockServiceInterface,
        error: null,
      });

      mockFrom.mockReturnValueOnce({
        insert: mockInsert,
        select: mockInsertSelect,
        single: mockInsertSingle,
      });

      const result = await createServiceInterfaceFromComponent(
        componentId,
        workflowId,
        config,
        mockSupabase
      );

      expect(result).toEqual(mockServiceInterface);
      expect(result.interface_type).toBe('query');
    });

    it('should throw error for non-interface component type', async () => {
      const componentId = 'comp-789';
      const workflowId = 'wf-789';
      const config = {
        endpointPath: '/test',
        httpMethod: 'POST' as const,
      };

      const mockComponent = {
        id: componentId,
        name: 'testActivity',
        display_name: 'Test Activity',
        component_type: { id: 'type-3', name: 'activity' }, // Not an interface component
        input_schema: null,
        output_schema: null,
      };

      const mockSelect = vi.fn().mockReturnThis();
      const mockEq = vi.fn().mockReturnThis();
      const mockSingle = vi.fn().mockResolvedValue({
        data: mockComponent,
        error: null,
      });

      mockFrom.mockReturnValueOnce({
        select: mockSelect,
        eq: mockEq,
        single: mockSingle,
      });

      await expect(
        createServiceInterfaceFromComponent(componentId, workflowId, config, mockSupabase)
      ).rejects.toThrow('Component type activity is not an interface component');
    });

    it('should throw error when component not found', async () => {
      const componentId = 'comp-not-found';
      const workflowId = 'wf-123';
      const config = {
        endpointPath: '/test',
        httpMethod: 'POST' as const,
      };

      const mockSelect = vi.fn().mockReturnThis();
      const mockEq = vi.fn().mockReturnThis();
      const mockSingle = vi.fn().mockResolvedValue({
        data: null,
        error: { message: 'Not found' },
      });

      mockFrom.mockReturnValueOnce({
        select: mockSelect,
        eq: mockEq,
        single: mockSingle,
      });

      await expect(
        createServiceInterfaceFromComponent(componentId, workflowId, config, mockSupabase)
      ).rejects.toThrow('Component not found');
    });

    it('should update existing service interface on unique constraint violation', async () => {
      const componentId = 'comp-123';
      const workflowId = 'wf-123';
      const config = {
        endpointPath: '/receive-data',
        httpMethod: 'POST' as const,
        isPublic: true,
      };

      const mockComponent = {
        id: componentId,
        name: 'receiveData',
        display_name: 'Receive Data',
        description: 'Receives incoming data',
        component_type: { id: 'type-1', name: 'data-in' },
        input_schema: null,
        output_schema: null,
      };

      const existingInterface = {
        id: 'si-existing',
        workflow_id: workflowId,
        name: 'receiveData',
      };

      const updatedInterface = {
        id: 'si-existing',
        workflow_id: workflowId,
        name: 'receiveData',
        display_name: 'Receive Data',
        interface_type: 'signal',
        is_public: true,
      };

      // Mock component fetch
      const mockSelect = vi.fn().mockReturnThis();
      const mockEq = vi.fn().mockReturnThis();
      const mockSingle = vi.fn().mockResolvedValue({
        data: mockComponent,
        error: null,
      });

      mockFrom.mockReturnValueOnce({
        select: mockSelect,
        eq: mockEq,
        single: mockSingle,
      });

      // Mock service interface insert (fails with unique constraint)
      const mockInsert = vi.fn().mockReturnThis();
      const mockInsertSelect = vi.fn().mockReturnThis();
      const mockInsertSingle = vi.fn().mockResolvedValue({
        data: null,
        error: { code: '23505', message: 'Unique constraint violation' },
      });

      // Mock existing interface fetch
      const mockExistingSelect = vi.fn().mockReturnThis();
      const mockExistingEq = vi.fn().mockReturnThis();
      const mockExistingSingle = vi.fn().mockResolvedValue({
        data: existingInterface,
        error: null,
      });

      // Mock update
      const mockUpdate = vi.fn().mockReturnThis();
      const mockUpdateEq = vi.fn().mockReturnThis();
      const mockUpdateSelect = vi.fn().mockReturnThis();
      const mockUpdateSingle = vi.fn().mockResolvedValue({
        data: updatedInterface,
        error: null,
      });

      mockFrom
        .mockReturnValueOnce({
          select: mockSelect,
          eq: mockEq,
          single: mockSingle,
        })
        .mockReturnValueOnce({
          insert: mockInsert,
          select: mockInsertSelect,
          single: mockInsertSingle,
        })
        .mockReturnValueOnce({
          select: mockExistingSelect,
          eq: mockExistingEq,
          single: mockExistingSingle,
        })
        .mockReturnValueOnce({
          update: mockUpdate,
          eq: mockUpdateEq,
          select: mockUpdateSelect,
          single: mockUpdateSingle,
        });

      const result = await createServiceInterfaceFromComponent(
        componentId,
        workflowId,
        config,
        mockSupabase
      );

      expect(result).toEqual(updatedInterface);
      expect(mockUpdate).toHaveBeenCalled();
    });
  });

  describe('getServiceInterfaceForComponent', () => {
    it('should return service interface when found', async () => {
      const componentId = 'comp-123';
      const workflowId = 'wf-123';

      const mockServiceInterface = {
        id: 'si-123',
        workflow_id: workflowId,
        activity_connection_id: componentId,
      };

      const mockSelect = vi.fn().mockReturnThis();
      const mockEq = vi.fn().mockReturnThis();
      const mockSingle = vi.fn().mockResolvedValue({
        data: mockServiceInterface,
        error: null,
      });

      mockFrom.mockReturnValueOnce({
        select: mockSelect,
        eq: mockEq,
        single: mockSingle,
      });

      const result = await getServiceInterfaceForComponent(
        componentId,
        workflowId,
        mockSupabase
      );

      expect(result).toEqual(mockServiceInterface);
    });

    it('should return null when service interface not found', async () => {
      const componentId = 'comp-123';
      const workflowId = 'wf-123';

      const mockSelect = vi.fn().mockReturnThis();
      const mockEq = vi.fn().mockReturnThis();
      const mockSingle = vi.fn().mockResolvedValue({
        data: null,
        error: { code: 'PGRST116', message: 'No rows returned' },
      });

      mockFrom.mockReturnValueOnce({
        select: mockSelect,
        eq: mockEq,
        single: mockSingle,
      });

      const result = await getServiceInterfaceForComponent(
        componentId,
        workflowId,
        mockSupabase
      );

      expect(result).toBeNull();
    });
  });

  describe('deleteServiceInterfaceForComponent', () => {
    it('should delete service interface successfully', async () => {
      const componentId = 'comp-123';
      const workflowId = 'wf-123';

      const mockDelete = vi.fn().mockReturnThis();
      const mockEq = vi.fn().mockResolvedValue({
        error: null,
      });

      mockFrom.mockReturnValueOnce({
        delete: mockDelete,
        eq: mockEq,
      });

      await deleteServiceInterfaceForComponent(componentId, workflowId, mockSupabase);

      expect(mockFrom).toHaveBeenCalledWith('service_interfaces');
      expect(mockDelete).toHaveBeenCalled();
    });

    it('should throw error on delete failure', async () => {
      const componentId = 'comp-123';
      const workflowId = 'wf-123';

      const mockDelete = vi.fn().mockReturnThis();
      const mockEq = vi.fn().mockResolvedValue({
        error: { message: 'Delete failed' },
      });

      mockFrom.mockReturnValueOnce({
        delete: mockDelete,
        eq: mockEq,
      });

      await expect(
        deleteServiceInterfaceForComponent(componentId, workflowId, mockSupabase)
      ).rejects.toThrow('Failed to delete service interface');
    });
  });
});

