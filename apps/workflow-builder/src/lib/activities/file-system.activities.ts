/**
 * File System Activities
 * 
 * Temporal activities for file system operations including read, write, search, and discovery.
 * These activities are used by both UI and Compiler components.
 * 
 * @see plans/ui-compiler-activities.md for comprehensive requirements
 * @see plans/package-builder/future/activities/file-system.md for detailed specifications
 */

import * as fs from 'fs/promises';
import * as path from 'path';
import { glob } from 'fast-glob';
import type { PathLike } from 'fs';

// ============================================================================
// Type Definitions
// ============================================================================

export interface FindFilesInput {
  directory: string;
  pattern?: string; // Glob pattern (e.g., '**/*.ts')
  regex?: string; // Regex pattern for file names
  includeDirs?: boolean;
  excludeDirs?: string[]; // e.g., ['node_modules', '.git']
  maxDepth?: number;
}

export interface FileInfo {
  path: string;
  relativePath: string;
  size: number;
  modified: Date;
  isDirectory: boolean;
}

export interface FindFilesResult {
  success: boolean;
  files: FileInfo[];
  total: number;
  error?: string;
}

export interface ReadFileInput {
  filePath: string;
  encoding?: BufferEncoding; // Default: 'utf-8'
  baseDir?: string; // For relative paths
}

export interface ReadFileResult {
  success: boolean;
  content: string;
  size: number;
  error?: string;
}

export interface WriteFileInput {
  filePath: string;
  content: string;
  encoding?: BufferEncoding; // Default: 'utf-8'
  baseDir?: string; // For relative paths
  createDir?: boolean; // Create directory if missing
}

export interface WriteFileResult {
  success: boolean;
  path: string;
  size: number;
  error?: string;
}

export interface SearchFileContentInput {
  directory: string;
  pattern: string; // Regex pattern
  filePattern?: string; // File filter (glob)
  caseSensitive?: boolean;
  multiline?: boolean;
  excludeDirs?: string[];
  maxResults?: number; // Limit results
}

export interface SearchMatch {
  file: string;
  line: number;
  column: number;
  match: string;
  context?: string; // Surrounding lines
}

export interface SearchFileContentResult {
  success: boolean;
  matches: SearchMatch[];
  total: number;
  filesSearched: number;
  error?: string;
}

export interface ListDirectoryInput {
  directory: string;
  recursive?: boolean;
  includeHidden?: boolean;
  filter?: (entry: { name: string; isDirectory: boolean }) => boolean;
}

export interface DirectoryEntry {
  name: string;
  path: string;
  isDirectory: boolean;
  size: number;
  modified: Date;
  children?: DirectoryEntry[];
}

export interface ListDirectoryResult {
  success: boolean;
  entries: DirectoryEntry[];
  total: number;
  totalSize: number;
  error?: string;
}

export interface BatchReadFilesInput {
  files: string[];
  baseDir: string;
  encoding?: BufferEncoding; // Default: 'utf-8'
}

export interface FileContent {
  path: string;
  content: string;
  error?: string;
}

export interface BatchReadFilesResult {
  success: boolean;
  files: FileContent[];
  failed: number;
  error?: string;
}

export interface BatchWriteFilesInput {
  operations: Array<{
    path: string;
    content: string;
    operation: 'create' | 'update';
  }>;
  baseDir: string;
  atomic?: boolean; // All succeed or all fail
}

export interface BatchWriteFilesResult {
  success: boolean;
  written: string[];
  failed: Array<{ path: string; error: string }>;
  error?: string;
}

// ============================================================================
// Path Validation and Security
// ============================================================================

/**
 * Validates that a path is safe and within allowed directories
 */
function validatePathSafety(
  filePath: string,
  baseDir?: string
): { safe: boolean; reason?: string; absolutePath: string } {
  const absolutePath = baseDir
    ? path.resolve(baseDir, filePath)
    : path.resolve(filePath);

  // Prevent path traversal
  if (filePath.includes('..')) {
    return {
      safe: false,
      reason: 'Path traversal detected (..)',
      absolutePath,
    };
  }

  // If baseDir is provided, ensure path is within it
  if (baseDir) {
    const baseAbsolute = path.resolve(baseDir);
    if (!absolutePath.startsWith(baseAbsolute)) {
      return {
        safe: false,
        reason: 'Path outside allowed directory',
        absolutePath,
      };
    }
  }

  return { safe: true, absolutePath };
}

// ============================================================================
// File System Activities
// ============================================================================

/**
 * Find files by pattern (glob or regex)
 */
export async function findFiles(
  input: FindFilesInput
): Promise<FindFilesResult> {
  try {
    const { directory, pattern, regex, includeDirs = false, excludeDirs = [], maxDepth } = input;

    // Validate directory exists
    const dirStat = await fs.stat(directory).catch(() => null);
    if (!dirStat || !dirStat.isDirectory()) {
      return {
        success: false,
        files: [],
        total: 0,
        error: `Directory does not exist: ${directory}`,
      };
    }

    // Build glob pattern
    const globPattern = pattern || '**/*';
    const globOptions: any = {
      cwd: directory,
      absolute: true,
      onlyFiles: !includeDirs,
      ignore: excludeDirs.map(dir => `**/${dir}/**`),
    };

    if (maxDepth !== undefined) {
      globOptions.deep = maxDepth;
    }

    // Find files using glob
    let filePaths = await glob(globPattern, globOptions);

    // Apply regex filter if provided
    if (regex) {
      const regexPattern = new RegExp(regex);
      filePaths = filePaths.filter(p => regexPattern.test(path.basename(p)));
    }

    // Get file info
    const files: FileInfo[] = [];
    for (const filePath of filePaths) {
      try {
        const stat = await fs.stat(filePath);
        files.push({
          path: filePath,
          relativePath: path.relative(directory, filePath),
          size: stat.size,
          modified: stat.mtime,
          isDirectory: stat.isDirectory(),
        });
      } catch (error) {
        // Skip files that can't be accessed
        console.warn(`Failed to stat file: ${filePath}`, error);
      }
    }

    return {
      success: true,
      files,
      total: files.length,
    };
  } catch (error: any) {
    return {
      success: false,
      files: [],
      total: 0,
      error: error.message || 'Unknown error finding files',
    };
  }
}

/**
 * Read file content
 */
export async function readFile(
  input: ReadFileInput
): Promise<ReadFileResult> {
  try {
    const { filePath, encoding = 'utf-8', baseDir } = input;

    // Validate path safety
    const validation = validatePathSafety(filePath, baseDir);
    if (!validation.safe) {
      return {
        success: false,
        content: '',
        size: 0,
        error: validation.reason,
      };
    }

    const absolutePath = validation.absolutePath;

    // Read file
    const content = await fs.readFile(absolutePath, encoding);
    const stat = await fs.stat(absolutePath);

    return {
      success: true,
      content,
      size: stat.size,
    };
  } catch (error: any) {
    if (error.code === 'ENOENT') {
      return {
        success: false,
        content: '',
        size: 0,
        error: `File not found: ${input.filePath}`,
      };
    }

    return {
      success: false,
      content: '',
      size: 0,
      error: error.message || 'Unknown error reading file',
    };
  }
}

/**
 * Write file content
 */
export async function writeFile(
  input: WriteFileInput
): Promise<WriteFileResult> {
  try {
    const { filePath, content, encoding = 'utf-8', baseDir, createDir = true } = input;

    // Validate path safety
    const validation = validatePathSafety(filePath, baseDir);
    if (!validation.safe) {
      return {
        success: false,
        path: filePath,
        size: 0,
        error: validation.reason,
      };
    }

    const absolutePath = validation.absolutePath;

    // Create directory if needed
    if (createDir) {
      const dir = path.dirname(absolutePath);
      await fs.mkdir(dir, { recursive: true });
    }

    // Write file
    await fs.writeFile(absolutePath, content, encoding);
    const stat = await fs.stat(absolutePath);

    return {
      success: true,
      path: absolutePath,
      size: stat.size,
    };
  } catch (error: any) {
    return {
      success: false,
      path: input.filePath,
      size: 0,
      error: error.message || 'Unknown error writing file',
    };
  }
}

/**
 * Search file content using regex
 */
export async function searchFileContent(
  input: SearchFileContentInput
): Promise<SearchFileContentResult> {
  try {
    const {
      directory,
      pattern,
      filePattern = '**/*',
      caseSensitive = false,
      multiline = false,
      excludeDirs = ['node_modules', '.git', '.next'],
      maxResults = 1000,
    } = input;

    // Validate directory
    const dirStat = await fs.stat(directory).catch(() => null);
    if (!dirStat || !dirStat.isDirectory()) {
      return {
        success: false,
        matches: [],
        total: 0,
        filesSearched: 0,
        error: `Directory does not exist: ${directory}`,
      };
    }

    // Find files to search
    const findResult = await findFiles({
      directory,
      pattern: filePattern,
      excludeDirs,
      includeDirs: false,
    });

    if (!findResult.success) {
      return {
        success: false,
        matches: [],
        total: 0,
        filesSearched: 0,
        error: findResult.error,
      };
    }

    // Build regex
    const regexFlags = caseSensitive ? 'g' : 'gi';
    const regex = new RegExp(pattern, regexFlags);

    // Search files
    const matches: SearchMatch[] = [];
    let filesSearched = 0;

    for (const file of findResult.files) {
      if (file.isDirectory) continue;
      if (matches.length >= maxResults) break;

      try {
        const content = await fs.readFile(file.path, 'utf-8');
        filesSearched++;

        const lines = content.split('\n');
        lines.forEach((line, lineIndex) => {
          const lineMatches = [...line.matchAll(regex)];
          for (const match of lineMatches) {
            if (matches.length >= maxResults) return;

            const column = match.index ?? 0;
            const contextLines = 2;
            const startLine = Math.max(0, lineIndex - contextLines);
            const endLine = Math.min(lines.length - 1, lineIndex + contextLines);
            const context = lines.slice(startLine, endLine + 1).join('\n');

            matches.push({
              file: file.relativePath,
              line: lineIndex + 1,
              column: column + 1,
              match: match[0],
              context,
            });
          }
        });
      } catch (error) {
        // Skip files that can't be read
        console.warn(`Failed to search file: ${file.path}`, error);
      }
    }

    return {
      success: true,
      matches,
      total: matches.length,
      filesSearched,
    };
  } catch (error: any) {
    return {
      success: false,
      matches: [],
      total: 0,
      filesSearched: 0,
      error: error.message || 'Unknown error searching files',
    };
  }
}

/**
 * List directory contents
 */
export async function listDirectory(
  input: ListDirectoryInput
): Promise<ListDirectoryResult> {
  try {
    const { directory, recursive = false, includeHidden = false, filter } = input;

    // Validate directory
    const dirStat = await fs.stat(directory).catch(() => null);
    if (!dirStat || !dirStat.isDirectory()) {
      return {
        success: false,
        entries: [],
        total: 0,
        totalSize: 0,
        error: `Directory does not exist: ${directory}`,
      };
    }

    const entries: DirectoryEntry[] = [];
    const allEntries: DirectoryEntry[] = []; // Flat list for recursive mode
    let totalSize = 0;

    async function processEntry(entryPath: string, parent?: DirectoryEntry): Promise<void> {
      try {
        const stat = await fs.stat(entryPath);
        const name = path.basename(entryPath);

        // Skip hidden files if not including them
        if (!includeHidden && name.startsWith('.')) {
          return;
        }

        const entry: DirectoryEntry = {
          name,
          path: entryPath,
          isDirectory: stat.isDirectory(),
          size: stat.size,
          modified: stat.mtime,
        };

        // Apply filter if provided
        if (filter && !filter({ name, isDirectory: entry.isDirectory })) {
          return;
        }

        // Process children if recursive and directory
        if (recursive && entry.isDirectory) {
          entry.children = [];
          const children = await fs.readdir(entryPath);
          for (const child of children) {
            await processEntry(path.join(entryPath, child), entry);
          }
        }

        // Add to parent or root
        if (parent) {
          parent.children = parent.children || [];
          parent.children.push(entry);
        } else {
          entries.push(entry);
        }

        // Also add to flat list if recursive
        if (recursive) {
          allEntries.push(entry);
        }

        totalSize += stat.size;
      } catch (error) {
        // Skip entries that can't be accessed
        console.warn(`Failed to process entry: ${entryPath}`, error);
      }
    }

    // Process directory
    const children = await fs.readdir(directory);
    for (const child of children) {
      await processEntry(path.join(directory, child));
    }

    // Return flat list if recursive, otherwise hierarchical
    const resultEntries = recursive ? allEntries : entries;

    return {
      success: true,
      entries: resultEntries,
      total: resultEntries.length,
      totalSize,
    };
  } catch (error: any) {
    return {
      success: false,
      entries: [],
      total: 0,
      totalSize: 0,
      error: error.message || 'Unknown error listing directory',
    };
  }
}

/**
 * Batch read multiple files
 */
export async function batchReadFiles(
  input: BatchReadFilesInput
): Promise<BatchReadFilesResult> {
  try {
    const { files, baseDir, encoding = 'utf-8' } = input;

    const results: FileContent[] = [];
    let failed = 0;

    // Read files in parallel
    const readPromises = files.map(async (filePath) => {
      const readResult = await readFile({
        filePath,
        baseDir,
        encoding,
      });

      if (readResult.success) {
        results.push({
          path: filePath,
          content: readResult.content,
        });
      } else {
        failed++;
        results.push({
          path: filePath,
          content: '',
          error: readResult.error,
        });
      }
    });

    await Promise.all(readPromises);

    return {
      success: failed === 0,
      files: results,
      failed,
    };
  } catch (error: any) {
    return {
      success: false,
      files: [],
      failed: input.files.length,
      error: error.message || 'Unknown error in batch read',
    };
  }
}

/**
 * Batch write multiple files (optionally atomically)
 */
export async function batchWriteFiles(
  input: BatchWriteFilesInput
): Promise<BatchWriteFilesResult> {
  try {
    const { operations, baseDir, atomic = false } = input;

    const written: string[] = [];
    const failed: Array<{ path: string; error: string }> = [];

    // If atomic, write to temp files first, then move
    if (atomic) {
      const tempFiles: string[] = [];
      try {
        // Write all to temp files first
        for (const op of operations) {
          const tempPath = `${op.path}.tmp.${Date.now()}`;
          const writeResult = await writeFile({
            filePath: tempPath,
            content: op.content,
            baseDir,
            createDir: true,
          });

          if (writeResult.success) {
            tempFiles.push(writeResult.path);
          } else {
            throw new Error(`Failed to write temp file: ${writeResult.error}`);
          }
        }

        // All succeeded, move temp files to final locations
        for (let i = 0; i < operations.length; i++) {
          const op = operations[i];
          const tempPath = tempFiles[i];
          const finalPath = baseDir
            ? path.resolve(baseDir, op.path)
            : path.resolve(op.path);

          // Create final directory
          await fs.mkdir(path.dirname(finalPath), { recursive: true });

          // Move temp file to final location
          await fs.rename(tempPath, finalPath);
          written.push(op.path);
        }

        return {
          success: true,
          written,
          failed: [],
        };
      } catch (error: any) {
        // Cleanup temp files on failure
        for (const tempPath of tempFiles) {
          await fs.unlink(tempPath).catch(() => {
            // Ignore cleanup errors
          });
        }

        return {
          success: false,
          written: [],
          failed: operations.map(op => ({
            path: op.path,
            error: error.message || 'Atomic write failed',
          })),
        };
      }
    } else {
      // Non-atomic: write files individually
      for (const op of operations) {
        const writeResult = await writeFile({
          filePath: op.path,
          content: op.content,
          baseDir,
          createDir: true,
        });

        if (writeResult.success) {
          written.push(op.path);
        } else {
          failed.push({
            path: op.path,
            error: writeResult.error || 'Unknown error',
          });
        }
      }

      return {
        success: failed.length === 0,
        written,
        failed,
      };
    }
  } catch (error: any) {
    return {
      success: false,
      written: [],
      failed: input.operations.map(op => ({
        path: op.path,
        error: error.message || 'Unknown error in batch write',
      })),
    };
  }
}

