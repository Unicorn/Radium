/**
 * File Operations Router
 * 
 * tRPC endpoints that wrap file system activities for UI use.
 * These endpoints allow the UI to perform file operations like discovering
 * components, loading workflow files, and searching for files.
 */

import { z } from 'zod';
import { createTRPCRouter, protectedProcedure } from '../trpc';
import { TRPCError } from '@trpc/server';
import {
  findFiles,
  readFile,
  searchFileContent,
  listDirectory,
  type FindFilesInput,
  type ReadFileInput,
  type SearchFileContentInput,
  type ListDirectoryInput,
} from '@/lib/activities/file-system.activities';

export const fileOperationsRouter = createTRPCRouter({
  /**
   * Find files by pattern (for component discovery)
   */
  findFiles: protectedProcedure
    .input(z.object({
      directory: z.string(),
      pattern: z.string().optional(),
      regex: z.string().optional(),
      includeDirs: z.boolean().optional(),
      excludeDirs: z.array(z.string()).optional(),
      maxDepth: z.number().optional(),
    }))
    .query(async ({ input }) => {
      try {
        const result = await findFiles(input as FindFilesInput);
        if (!result.success) {
          throw new TRPCError({
            code: 'INTERNAL_SERVER_ERROR',
            message: result.error || 'Failed to find files',
          });
        }
        return result;
      } catch (error) {
        if (error instanceof TRPCError) throw error;
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error instanceof Error ? error.message : 'Failed to find files',
        });
      }
    }),

  /**
   * Read file content (for loading workflow definitions, component schemas)
   */
  readFile: protectedProcedure
    .input(z.object({
      path: z.string(),
      encoding: z.enum(['utf8', 'base64']).optional(),
    }))
    .query(async ({ input }) => {
      try {
        const result = await readFile(input as ReadFileInput);
        if (!result.success) {
          throw new TRPCError({
            code: 'INTERNAL_SERVER_ERROR',
            message: result.error || 'Failed to read file',
          });
        }
        return result;
      } catch (error) {
        if (error instanceof TRPCError) throw error;
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error instanceof Error ? error.message : 'Failed to read file',
        });
      }
    }),

  /**
   * Search file content (for component discovery, code analysis)
   */
  searchFileContent: protectedProcedure
    .input(z.object({
      directory: z.string(),
      pattern: z.string(), // Regex pattern
      filePattern: z.string().optional(), // Glob pattern for files to search
      caseSensitive: z.boolean().optional(),
      maxResults: z.number().optional(),
    }))
    .query(async ({ input }) => {
      try {
        const result = await searchFileContent({
          directory: input.directory,
          pattern: input.pattern,
          filePattern: input.filePattern,
          caseSensitive: input.caseSensitive,
          maxResults: input.maxResults,
        } as SearchFileContentInput);
        if (!result.success) {
          throw new TRPCError({
            code: 'INTERNAL_SERVER_ERROR',
            message: result.error || 'Failed to search file content',
          });
        }
        return result;
      } catch (error) {
        if (error instanceof TRPCError) throw error;
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error instanceof Error ? error.message : 'Failed to search file content',
        });
      }
    }),

  /**
   * List directory contents (for workspace browsing, component discovery)
   */
  listDirectory: protectedProcedure
    .input(z.object({
      directory: z.string(),
      recursive: z.boolean().optional(),
      includeFiles: z.boolean().optional().default(true),
      includeDirs: z.boolean().optional().default(true),
    }))
    .query(async ({ input }) => {
      try {
        const result = await listDirectory({
          directory: input.directory,
          recursive: input.recursive,
          includeFiles: input.includeFiles,
          includeDirs: input.includeDirs,
        } as ListDirectoryInput);
        if (!result.success) {
          throw new TRPCError({
            code: 'INTERNAL_SERVER_ERROR',
            message: result.error || 'Failed to list directory',
          });
        }
        return result;
      } catch (error) {
        if (error instanceof TRPCError) throw error;
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error instanceof Error ? error.message : 'Failed to list directory',
        });
      }
    }),
});

