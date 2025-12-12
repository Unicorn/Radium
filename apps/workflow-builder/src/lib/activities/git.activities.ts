/**
 * Git Activities
 * 
 * Temporal activities for git operations used by the Compiler component
 * for version control and code management of generated workflows.
 * 
 * @see plans/ui-compiler-activities.md for comprehensive requirements
 * @see plans/package-builder/future/activities/git-activities.md for detailed specifications
 */

import { spawn } from 'child_process';
import * as fs from 'fs/promises';
import * as path from 'path';

// ============================================================================
// Type Definitions
// ============================================================================

export interface GitStatusInput {
  workspacePath: string;
  short?: boolean; // Use --short format
}

export interface GitStatusResult {
  success: boolean;
  isClean: boolean;
  hasChanges: boolean;
  stagedFiles: string[];
  unstagedFiles: string[];
  untrackedFiles: string[];
  stdout: string;
  error?: string;
}

export interface GitDiffInput {
  workspacePath: string;
  commit1?: string; // Default: HEAD
  commit2?: string; // Default: working directory
  filePath?: string; // Specific file
  stat?: boolean; // Only show stats
}

export interface FileStat {
  file: string;
  insertions: number;
  deletions: number;
}

export interface GitDiffResult {
  success: boolean;
  diff: string;
  filesChanged: number;
  insertions: number;
  deletions: number;
  fileStats: FileStat[];
  error?: string;
}

export interface CreateTagInput {
  workspacePath: string;
  tagName: string;
  message?: string;
  commitHash?: string; // Default: HEAD
  annotated?: boolean; // Default: true
}

export interface CreateTagResult {
  success: boolean;
  tagName: string;
  stdout: string;
  stderr?: string;
  error?: string;
}

export interface ListBranchesInput {
  workspacePath: string;
  remote?: boolean;
  merged?: boolean; // Only show merged branches
}

export interface BranchInfo {
  name: string;
  isCurrent: boolean;
  isRemote: boolean;
  lastCommit: string;
}

export interface ListBranchesResult {
  success: boolean;
  branches: BranchInfo[];
  currentBranch?: string;
  error?: string;
}

// ============================================================================
// Helper Functions
// ============================================================================

/**
 * Execute git command and return result
 */
async function executeGitCommand(
  workspacePath: string,
  args: string[],
  timeout: number = 30000
): Promise<{ success: boolean; stdout: string; stderr: string; exitCode: number }> {
  return new Promise((resolve) => {
    let stdout = '';
    let stderr = '';
    let timeoutHandle: NodeJS.Timeout | null = null;

    // Setup timeout
    timeoutHandle = setTimeout(() => {
      resolve({
        success: false,
        stdout,
        stderr: stderr || 'Git command timed out',
        exitCode: -1,
      });
    }, timeout);

    try {
      const process = spawn('git', args, {
        cwd: workspacePath,
        stdio: ['pipe', 'pipe', 'pipe'],
      });

      process.stdout.on('data', (data: Buffer) => {
        stdout += data.toString();
      });

      process.stderr.on('data', (data: Buffer) => {
        stderr += data.toString();
      });

      process.on('close', (code) => {
        if (timeoutHandle) {
          clearTimeout(timeoutHandle);
        }

        resolve({
          success: code === 0,
          stdout: stdout.trim(),
          stderr: stderr.trim(),
          exitCode: code ?? -1,
        });
      });

      process.on('error', (error) => {
        if (timeoutHandle) {
          clearTimeout(timeoutHandle);
        }

        resolve({
          success: false,
          stdout,
          stderr: error.message,
          exitCode: -1,
        });
      });
    } catch (error: any) {
      if (timeoutHandle) {
        clearTimeout(timeoutHandle);
      }

      resolve({
        success: false,
        stdout: '',
        stderr: error.message || 'Failed to spawn git process',
        exitCode: -1,
      });
    }
  });
}

/**
 * Check if directory is a git repository
 */
async function isGitRepository(workspacePath: string): Promise<boolean> {
  try {
    const gitDir = path.join(workspacePath, '.git');
    const stat = await fs.stat(gitDir);
    return stat.isDirectory();
  } catch {
    return false;
  }
}

// ============================================================================
// Git Activities
// ============================================================================

/**
 * Get git status and change information
 */
export async function gitStatus(
  input: GitStatusInput
): Promise<GitStatusResult> {
  const { workspacePath, short = false } = input;

  // Check if it's a git repository
  const isRepo = await isGitRepository(workspacePath);
  if (!isRepo) {
    return {
      success: false,
      isClean: false,
      hasChanges: false,
      stagedFiles: [],
      unstagedFiles: [],
      untrackedFiles: [],
      stdout: '',
      error: 'Not a git repository',
    };
  }

  // Execute git status
  const args = short ? ['status', '--short'] : ['status'];
  const result = await executeGitCommand(workspacePath, args);

  if (!result.success) {
    return {
      success: false,
      isClean: false,
      hasChanges: false,
      stagedFiles: [],
      unstagedFiles: [],
      untrackedFiles: [],
      stdout: result.stdout,
      error: result.stderr || 'Git status failed',
    };
  }

  // Parse status output
  const stagedFiles: string[] = [];
  const unstagedFiles: string[] = [];
  const untrackedFiles: string[] = [];

  if (short) {
    // Parse short format: "XY filename" or "?? filename" for untracked
    // X = staged area, Y = working tree
    // Examples: "M " = staged modified, " M" = unstaged modified, "MM" = both, "??" = untracked
    // Format is exactly: 2 chars, space(s), filename
    const lines = result.stdout.split('\n').filter(line => line.trim());
    for (const line of lines) {
      // Match: 2-char status code, whitespace, then filename (may have leading/trailing spaces)
      const trimmedLine = line.trim();
      const match = trimmedLine.match(/^(.{2})\s+(.+)$/);
      if (match) {
        const [, status, file] = match;
        const stagedStatus = status[0];
        const unstagedStatus = status[1];
        const untracked = status === '??';
        const staged = !untracked && stagedStatus !== ' ' && stagedStatus !== '?';
        const unstaged = !untracked && unstagedStatus !== ' ' && unstagedStatus !== '?';

        if (untracked) {
          untrackedFiles.push(file.trim());
        } else {
          // File can be in both staged and unstaged (e.g., "MM")
          if (staged) {
            stagedFiles.push(file.trim());
          }
          if (unstaged) {
            unstagedFiles.push(file.trim());
          }
        }
      }
    }
  } else {
    // Parse long format
    const lines = result.stdout.split('\n');
    let currentSection: 'staged' | 'unstaged' | 'untracked' | null = null;

    for (const line of lines) {
      const trimmedLine = line.trim();
      
      if (line.includes('Changes to be committed:')) {
        currentSection = 'staged';
      } else if (line.includes('Changes not staged for commit:')) {
        currentSection = 'unstaged';
      } else if (line.includes('Untracked files:')) {
        currentSection = 'untracked';
      } else if (trimmedLine.startsWith('modified:') || trimmedLine.startsWith('new file:') || trimmedLine.startsWith('deleted:')) {
        // Extract filename from "modified:   filename" or "\tmodified:   filename"
        // Pattern: "modified:" followed by spaces/tabs, then filename
        const fileMatch = trimmedLine.match(/(?:modified|new file|deleted):\s+(.+)$/);
        if (fileMatch) {
          const file = fileMatch[1].trim();
          if (currentSection === 'staged') {
            stagedFiles.push(file);
          } else if (currentSection === 'unstaged') {
            unstagedFiles.push(file);
          }
        }
      } else if ((line.startsWith('\t') || line.startsWith('  ')) && currentSection === 'untracked') {
        // Untracked files are listed with tab/space indentation (just the filename)
        const file = trimmedLine;
        if (file && !file.includes('(') && !file.includes('use')) {
          untrackedFiles.push(file);
        }
      }
    }
  }

  const hasChanges = stagedFiles.length > 0 || unstagedFiles.length > 0 || untrackedFiles.length > 0;
  const isClean = !hasChanges && result.stdout.includes('nothing to commit');

  return {
    success: true,
    isClean,
    hasChanges,
    stagedFiles,
    unstagedFiles,
    untrackedFiles,
    stdout: result.stdout,
  };
}

/**
 * Get git diff between commits or working directory
 */
export async function gitDiff(
  input: GitDiffInput
): Promise<GitDiffResult> {
  const { workspacePath, commit1, commit2, filePath, stat = false } = input;

  // Check if it's a git repository
  const isRepo = await isGitRepository(workspacePath);
  if (!isRepo) {
    return {
      success: false,
      diff: '',
      filesChanged: 0,
      insertions: 0,
      deletions: 0,
      fileStats: [],
      error: 'Not a git repository',
    };
  }

  // Build git diff command
  const args: string[] = ['diff'];
  
  if (stat) {
    args.push('--stat');
  }

  if (commit1 && commit2) {
    args.push(commit1, commit2);
  } else if (commit1) {
    args.push(commit1);
  }

  if (filePath) {
    args.push('--', filePath);
  }

  const result = await executeGitCommand(workspacePath, args);

  if (!result.success) {
    return {
      success: false,
      diff: result.stdout,
      filesChanged: 0,
      insertions: 0,
      deletions: 0,
      fileStats: [],
      error: result.stderr || 'Git diff failed',
    };
  }

  // Parse diff stats if stat mode
  let filesChanged = 0;
  let insertions = 0;
  let deletions = 0;
  const fileStats: FileStat[] = [];

  if (stat) {
    // Parse --stat output
    const lines = result.stdout.split('\n').filter(line => line.trim());
    const summaryLine = lines[lines.length - 1];
    
    // Parse file stats
    for (const line of lines) {
      if (line.includes('|')) {
        const match = line.match(/^(.+?)\s+\|\s+(\d+)\s+([+-]+)$/);
        if (match) {
          const [, file, changes, indicators] = match;
          const insertionsCount = (indicators.match(/\+/g) || []).length;
          const deletionsCount = (indicators.match(/-/g) || []).length;
          
          fileStats.push({
            file: file.trim(),
            insertions: insertionsCount,
            deletions: deletionsCount,
          });
          filesChanged++;
          insertions += insertionsCount;
          deletions += deletionsCount;
        }
      }
    }

    // Parse summary line if present
    if (summaryLine && summaryLine.includes('files changed')) {
      const summaryMatch = summaryLine.match(/(\d+)\s+files? changed(?:,\s+(\d+)\s+insertions?\(\+\))?(?:,\s+(\d+)\s+deletions?\(-\))?/);
      if (summaryMatch) {
        filesChanged = parseInt(summaryMatch[1], 10);
        if (summaryMatch[2]) {
          insertions = parseInt(summaryMatch[2], 10);
        }
        if (summaryMatch[3]) {
          deletions = parseInt(summaryMatch[3], 10);
        }
      }
    }
  } else {
    // Parse regular diff for stats
    const diffLines = result.stdout.split('\n');
    let currentFile = '';
    
    for (const line of diffLines) {
      if (line.startsWith('diff --git')) {
        const fileMatch = line.match(/diff --git a\/(.+?)\s+b\/(.+?)$/);
        if (fileMatch) {
          currentFile = fileMatch[2];
          if (!fileStats.find(f => f.file === currentFile)) {
            fileStats.push({
              file: currentFile,
              insertions: 0,
              deletions: 0,
            });
            filesChanged++;
          }
        }
      } else if (line.startsWith('+') && !line.startsWith('+++')) {
        const stat = fileStats.find(f => f.file === currentFile);
        if (stat) {
          stat.insertions++;
          insertions++;
        }
      } else if (line.startsWith('-') && !line.startsWith('---')) {
        const stat = fileStats.find(f => f.file === currentFile);
        if (stat) {
          stat.deletions++;
          deletions++;
        }
      }
    }
  }

  return {
    success: true,
    diff: result.stdout,
    filesChanged,
    insertions,
    deletions,
    fileStats,
  };
}

/**
 * Create a git tag
 */
export async function createTag(
  input: CreateTagInput
): Promise<CreateTagResult> {
  const { workspacePath, tagName, message, commitHash, annotated = true } = input;

  // Check if it's a git repository
  const isRepo = await isGitRepository(workspacePath);
  if (!isRepo) {
    return {
      success: false,
      tagName,
      stdout: '',
      stderr: 'Not a git repository',
      error: 'Not a git repository',
    };
  }

  // Build git tag command
  // Git tag syntax: git tag [-a] [-m message] <tagname> [<commit>]
  // The commit hash must come at the END, after the tag name
  const args: string[] = ['tag'];

  if (annotated) {
    // For annotated tags, always provide a message (use tagName as default if none provided)
    const tagMessage = message || `Tag ${tagName}`;
    args.push('-a');
    args.push('-m', tagMessage);
    args.push(tagName);
    if (commitHash) {
      args.push(commitHash);
    }
  } else {
    args.push(tagName);
    if (commitHash) {
      args.push(commitHash);
    }
  }

  const result = await executeGitCommand(workspacePath, args);

  return {
    success: result.success,
    tagName,
    stdout: result.stdout,
    stderr: result.stderr,
    error: result.success ? undefined : (result.stderr || 'Git tag creation failed'),
  };
}

/**
 * List git branches
 */
export async function listBranches(
  input: ListBranchesInput
): Promise<ListBranchesResult> {
  const { workspacePath, remote = false, merged = false } = input;

  // Check if it's a git repository
  const isRepo = await isGitRepository(workspacePath);
  if (!isRepo) {
    return {
      success: false,
      branches: [],
      error: 'Not a git repository',
    };
  }

  // Get current branch first
  const currentBranchResult = await executeGitCommand(workspacePath, ['branch', '--show-current']);
  const currentBranch = currentBranchResult.success ? currentBranchResult.stdout : undefined;

  // Build git branch command
  const args: string[] = ['branch'];
  
  if (remote) {
    args.push('-r');
  } else {
    args.push('-a'); // Show both local and remote
  }

  if (merged) {
    args.push('--merged');
  }

  // Add verbose flag to get commit info
  args.push('-v');

  const result = await executeGitCommand(workspacePath, args);

  if (!result.success) {
    return {
      success: false,
      branches: [],
      error: result.stderr || 'Git branch listing failed',
    };
  }

  // Parse branch output (format: "* branch-name commit-hash commit-message" or "  branch-name commit-hash commit-message")
  const branches: BranchInfo[] = [];
  const lines = result.stdout.split('\n').filter(line => line.trim());

  for (const line of lines) {
    // Parse format: "* branch-name commit-hash commit-message" or "  branch-name commit-hash commit-message"
    // Or for remote: "  remotes/origin/branch-name commit-hash commit-message"
    // The format is: marker(1 char) + space + branch-name + spaces + commit-hash + spaces + message
    const match = line.match(/^(\*|\s)\s+([^\s]+)\s+([a-f0-9]+)\s+(.+)$/);
    if (match) {
      const [, headMarker, name, commit] = match;
      const isRemote = name.startsWith('remotes/');
      const branchName = isRemote ? name.replace(/^remotes\/[^/]+\//, '') : name.trim();
      const isCurrent = headMarker === '*' || branchName === currentBranch;

      branches.push({
        name: branchName,
        isCurrent,
        isRemote,
        lastCommit: commit.trim(),
      });
    }
  }

  return {
    success: true,
    branches,
    currentBranch,
  };
}

