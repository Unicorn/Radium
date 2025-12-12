/**
 * Command Execution Activities
 * 
 * Temporal activities for safe command execution with timeout handling,
 * resource monitoring, and process management.
 * These activities are used by the Compiler component for validation.
 * 
 * @see plans/ui-compiler-activities.md for comprehensive requirements
 * @see plans/package-builder/future/activities/command-line.md for detailed specifications
 */

import { spawn } from 'child_process';
import * as fs from 'fs/promises';
import * as path from 'path';
import { randomUUID } from 'crypto';
import type { ChildProcess } from 'child_process';

// ============================================================================
// Type Definitions
// ============================================================================

export interface ExecuteCommandInput {
  command: string;
  args?: string[];
  workingDir: string;
  timeout?: number; // milliseconds, default: 10 minutes
  env?: Record<string, string>;
  stdin?: string;
  resourceLimits?: {
    maxCpuPercent?: number;
    maxMemoryMB?: number;
  };
  captureOutput?: boolean; // Default: true
}

export interface ExecuteCommandResult {
  success: boolean;
  exitCode: number;
  stdout: string;
  stderr: string;
  duration: number;
  pid?: number;
  resourceUsage?: {
    cpuPercent: number;
    memoryMB: number;
  };
  error?: string;
  commandId: string;
}

export interface BuildCommandInput {
  command: 'npm' | 'yarn' | 'pnpm';
  subcommand: 'install' | 'build' | 'run';
  args?: string[];
  workingDir: string;
  timeout?: number; // Default: 15 minutes for builds
}

export interface BuildError {
  file: string;
  line: number;
  message: string;
  type: 'error' | 'warning';
}

export interface BuildCommandResult extends ExecuteCommandResult {
  buildErrors?: BuildError[];
  buildWarnings?: number;
  buildTime?: number;
}

export interface TestCommandInput {
  command: 'npm' | 'yarn' | 'pnpm' | 'jest' | 'vitest';
  testPattern?: string;
  workingDir: string;
  timeout?: number; // Default: 30 minutes for full test suites
  coverage?: boolean;
  watch?: boolean; // Default: false for CI
}

export interface TestResults {
  passed: number;
  failed: number;
  skipped: number;
  total: number;
  duration: number;
}

export interface Coverage {
  lines: number;
  statements: number;
  functions: number;
  branches: number;
}

export interface TestFailure {
  test: string;
  file: string;
  error: string;
}

export interface TestCommandResult extends ExecuteCommandResult {
  testResults?: TestResults;
  coverage?: Coverage;
  failures?: TestFailure[];
}

export interface LintCommandInput {
  command: 'npm' | 'yarn' | 'pnpm' | 'eslint';
  files?: string[];
  workingDir: string;
  fix?: boolean; // Auto-fix issues
  timeout?: number; // Default: 5 minutes
  format?: 'json' | 'stylish' | 'compact';
}

export interface LintIssue {
  file: string;
  line: number;
  column: number;
  rule: string;
  severity: 'error' | 'warning';
  message: string;
  fixable: boolean;
}

export interface LintResults {
  errors: number;
  warnings: number;
  fixable: number;
}

export interface LintCommandResult extends ExecuteCommandResult {
  lintResults?: LintResults;
  issues?: LintIssue[];
}

export interface CommandLogEntry {
  commandId: string;
  workflowRunId?: string;
  stepName?: string;
  timestamp: string;
  command: string;
  args: string[];
  workingDir: string;
  duration: number;
  success: boolean;
  exitCode: number;
  resourceUsage?: {
    cpuPercent: number;
    memoryMB: number;
  };
  outputSize: number; // bytes
  errorType?: string;
  retryCount?: number;
}

// ============================================================================
// Command Validation and Security
// ============================================================================

/**
 * Allowed commands whitelist for security
 */
const ALLOWED_COMMANDS = new Set([
  'npm',
  'yarn',
  'pnpm',
  'npx',
  'node',
  'tsc',
  'eslint',
  'jest',
  'vitest',
  'echo',
  'cat',
  'ls',
  'pwd',
  'git',
  'sleep', // For testing timeouts
]);

/**
 * Dangerous commands that should be blocked
 */
const DANGEROUS_COMMANDS = new Set([
  'rm',
  'rmdir',
  'del',
  'format',
  'dd',
  'mkfs',
  'fdisk',
]);

/**
 * Validates command is safe to execute
 */
export function validateCommand(command: string): { allowed: boolean; reason?: string } {
  const baseCommand = command.split(' ')[0].split('/').pop() || '';

  if (DANGEROUS_COMMANDS.has(baseCommand)) {
    return {
      allowed: false,
      reason: `Dangerous command blocked: ${baseCommand}`,
    };
  }

  if (!ALLOWED_COMMANDS.has(baseCommand)) {
    return {
      allowed: false,
      reason: `Command not in whitelist: ${baseCommand}`,
    };
  }

  return { allowed: true };
}

// ============================================================================
// Process Management
// ============================================================================

/**
 * Kill process and all children
 */
async function killProcessTree(pid: number, signal: NodeJS.Signals = 'SIGTERM'): Promise<void> {
  try {
    process.kill(pid, signal);
    
    // Wait a bit, then force kill if still running
    await new Promise(resolve => setTimeout(resolve, 5000));
    
    try {
      process.kill(pid, 0); // Check if still exists
      process.kill(pid, 'SIGKILL'); // Force kill
    } catch {
      // Process already dead
    }
  } catch (error) {
    // Process might not exist
    console.warn(`Failed to kill process ${pid}:`, error);
  }
}

/**
 * Get process resource usage (simplified - would use pidusage in production)
 */
async function getResourceUsage(pid: number): Promise<{ cpuPercent: number; memoryMB: number }> {
  // Simplified implementation - in production would use pidusage library
  // For now, return placeholder values
  return {
    cpuPercent: 0,
    memoryMB: 0,
  };
}

// ============================================================================
// Command Execution Activities
// ============================================================================

/**
 * Execute a shell command safely with timeout and resource monitoring
 */
export async function executeCommand(
  input: ExecuteCommandInput
): Promise<ExecuteCommandResult> {
  const commandId = randomUUID();
  const startTime = Date.now();
  const {
    command,
    args = [],
    workingDir,
    timeout = 600000, // 10 minutes default
    env = {},
    stdin,
    resourceLimits,
    captureOutput = true,
  } = input;

  // Validate command
  const validation = validateCommand(command);
  if (!validation.allowed) {
    return {
      success: false,
      exitCode: -1,
      stdout: '',
      stderr: validation.reason || 'Command validation failed',
      duration: Date.now() - startTime,
      commandId,
      error: validation.reason,
    };
  }

  // Validate working directory exists
  try {
    const dirStat = await fs.stat(workingDir);
    if (!dirStat.isDirectory()) {
      return {
        success: false,
        exitCode: -1,
        stdout: '',
        stderr: `Working directory is not a directory: ${workingDir}`,
        duration: Date.now() - startTime,
        commandId,
        error: `Invalid working directory: ${workingDir}`,
      };
    }
  } catch (error: any) {
    return {
      success: false,
      exitCode: -1,
      stdout: '',
      stderr: `Working directory does not exist: ${workingDir}`,
      duration: Date.now() - startTime,
      commandId,
      error: `Working directory does not exist: ${workingDir}`,
    };
  }

  return new Promise((resolve) => {
    let stdout = '';
    let stderr = '';
    let childProcess: ChildProcess | null = null;
    let timeoutHandle: NodeJS.Timeout | null = null;
    let killed = false;

    // Setup timeout
    timeoutHandle = setTimeout(() => {
      if (childProcess && childProcess.pid) {
        killed = true;
        killProcessTree(childProcess.pid);
        resolve({
          success: false,
          exitCode: -1,
          stdout,
          stderr: stderr || 'Command timed out',
          duration: Date.now() - startTime,
          pid: childProcess.pid,
          commandId,
          error: `Command timed out after ${timeout}ms`,
        });
      }
    }, timeout);

    // Spawn process
    try {
      childProcess = spawn(command, args, {
        cwd: workingDir,
        env: { ...process.env, ...env },
        stdio: captureOutput ? ['pipe', 'pipe', 'pipe'] : 'ignore',
      });

      const pid = childProcess.pid;
      if (!pid) {
        throw new Error('Failed to spawn process');
      }

      // Capture output
      if (captureOutput && childProcess.stdout) {
        childProcess.stdout.on('data', (data: Buffer) => {
          stdout += data.toString();
        });
      }

      if (captureOutput && childProcess.stderr) {
        childProcess.stderr.on('data', (data: Buffer) => {
          stderr += data.toString();
        });
      }

      // Send stdin if provided
      if (stdin && childProcess.stdin) {
        childProcess.stdin.write(stdin);
        childProcess.stdin.end();
      }

      // Handle process completion
      childProcess.on('close', async (code) => {
        if (timeoutHandle) {
          clearTimeout(timeoutHandle);
        }

        if (killed) {
          return; // Already resolved by timeout
        }

        const duration = Date.now() - startTime;
        const resourceUsage = await getResourceUsage(pid);

        resolve({
          success: code === 0,
          exitCode: code ?? -1,
          stdout,
          stderr,
          duration,
          pid,
          resourceUsage,
          commandId,
          error: code !== 0 ? `Command failed with exit code ${code}` : undefined,
        });
      });

      childProcess.on('error', (error) => {
        if (timeoutHandle) {
          clearTimeout(timeoutHandle);
        }

        if (killed) {
          return;
        }

        resolve({
          success: false,
          exitCode: -1,
          stdout,
          stderr: error.message,
          duration: Date.now() - startTime,
          pid: childProcess?.pid,
          commandId,
          error: error.message,
        });
      });
    } catch (error: any) {
      if (timeoutHandle) {
        clearTimeout(timeoutHandle);
      }

      resolve({
        success: false,
        exitCode: -1,
        stdout: '',
        stderr: error.message || 'Failed to spawn process',
        duration: Date.now() - startTime,
        commandId,
        error: error.message || 'Process spawn failed',
      });
    }
  });
}

/**
 * Execute build command with specialized error parsing
 */
export async function runBuildCommand(
  input: BuildCommandInput
): Promise<BuildCommandResult> {
  const {
    command,
    subcommand,
    args = [],
    workingDir,
    timeout = 900000, // 15 minutes default
  } = input;

  // Build command string
  const commandArgs: string[] = [];
  if (command === 'npm' || command === 'yarn' || command === 'pnpm') {
    commandArgs.push(subcommand, ...args);
  } else {
    commandArgs.push(...args);
  }

  const result = await executeCommand({
    command,
    args: commandArgs,
    workingDir,
    timeout,
  });

  // Parse build errors from output (check both stdout and stderr)
  const buildErrors: BuildError[] = [];
  let buildWarnings = 0;

  // Parse TypeScript errors/warnings from output (even on success, warnings may be present)
  const errorRegex = /(.+?)\((\d+),(\d+)\):\s*(error|warning)\s+(.+)/g;
  const output = (result.stderr || result.stdout || '');
  let match;
  while ((match = errorRegex.exec(output)) !== null) {
    const [, file, line, , type, message] = match;
    buildErrors.push({
      file: file.trim(),
      line: parseInt(line, 10),
      message: message.trim(),
      type: type as 'error' | 'warning',
    });
    if (type === 'warning') {
      buildWarnings++;
    }
  }

  return {
    ...result,
    buildErrors: buildErrors.length > 0 ? buildErrors : undefined,
    buildWarnings: buildWarnings > 0 ? buildWarnings : undefined,
    buildTime: result.duration,
  };
}

/**
 * Execute test command with result parsing
 */
export async function runTestCommand(
  input: TestCommandInput
): Promise<TestCommandResult> {
  const {
    command,
    testPattern,
    workingDir,
    timeout = 1800000, // 30 minutes default
    coverage: includeCoverage = false,
    watch = false,
  } = input;

  // Build command args
  const commandArgs: string[] = [];
  
  if (command === 'npm' || command === 'yarn' || command === 'pnpm') {
    commandArgs.push('test');
    if (includeCoverage) {
      commandArgs.push('--coverage');
    }
    if (testPattern) {
      commandArgs.push('--', testPattern);
    }
  } else if (command === 'jest' || command === 'vitest') {
    if (includeCoverage) {
      commandArgs.push('--coverage');
    }
    if (testPattern) {
      commandArgs.push(testPattern);
    }
    if (!watch) {
      commandArgs.push('--run');
    }
  }

  const result = await executeCommand({
    command,
    args: commandArgs,
    workingDir,
    timeout,
  });

  // Parse test results (simplified - would use JSON reporter in production)
  const testResults: TestResults = {
    passed: 0,
    failed: 0,
    skipped: 0,
    total: 0,
    duration: result.duration,
  };

  const coverageData: Coverage = {
    lines: 0,
    statements: 0,
    functions: 0,
    branches: 0,
  };

  const failures: TestFailure[] = [];

  if (result.success) {
    // Parse test output (simplified - would parse JSON in production)
    // Prioritize "Tests:" line over "Test Suites:" line
    const testsLineMatch = result.stdout.match(/Tests:\s+(\d+)\s+passed(?:,\s+(\d+)\s+failed)?(?:,\s+(\d+)\s+skipped)?(?:,\s+(\d+)\s+total)?/i);
    
    if (testsLineMatch) {
      // Parse from "Tests: X passed, Y failed, Z skipped, N total" format
      testResults.passed = parseInt(testsLineMatch[1], 10);
      if (testsLineMatch[2]) testResults.failed = parseInt(testsLineMatch[2], 10);
      if (testsLineMatch[3]) testResults.skipped = parseInt(testsLineMatch[3], 10);
      if (testsLineMatch[4]) testResults.total = parseInt(testsLineMatch[4], 10);
    } else {
      // Fallback to individual matches
      const passedMatch = result.stdout.match(/Tests:\s+(\d+)\s+passed/i);
      const failedMatch = result.stdout.match(/Tests:\s+.*?(\d+)\s+failed/i);
      const skippedMatch = result.stdout.match(/Tests:\s+.*?(\d+)\s+skipped/i);
      const totalMatch = result.stdout.match(/Tests:\s+.*?(\d+)\s+total/i);

      if (passedMatch) testResults.passed = parseInt(passedMatch[1], 10);
      if (failedMatch) testResults.failed = parseInt(failedMatch[1], 10);
      if (skippedMatch) testResults.skipped = parseInt(skippedMatch[1], 10);
      if (totalMatch) testResults.total = parseInt(totalMatch[1], 10);
    }
    
    // If we have passed/failed but no total, calculate it
    if (testResults.total === 0 && (testResults.passed > 0 || testResults.failed > 0)) {
      testResults.total = testResults.passed + testResults.failed + testResults.skipped;
    }

    // Parse coverage (simplified)
    const coverageMatch = result.stdout.match(/Lines\s+:\s+(\d+(?:\.\d+)?)%/);
    if (coverageMatch) {
      coverageData.lines = parseFloat(coverageMatch[1]);
    }
  } else {
    // Extract failures from output
    const failureRegex = /FAIL\s+(.+?)\s+(.+?)\n([\s\S]*?)(?=FAIL|PASS|$)/g;
    let match;
    while ((match = failureRegex.exec(result.stdout || result.stderr)) !== null) {
      failures.push({
        test: match[1].trim(),
        file: match[2].trim(),
        error: match[3].trim(),
      });
    }
  }

  return {
    ...result,
    testResults: testResults.total > 0 ? testResults : undefined,
    coverage: coverageData.lines > 0 ? coverageData : undefined,
    failures: failures.length > 0 ? failures : undefined,
  };
}

/**
 * Execute lint command with issue parsing
 */
export async function runLintCommand(
  input: LintCommandInput
): Promise<LintCommandResult> {
  const {
    command,
    files = [],
    workingDir,
    fix = false,
    timeout = 300000, // 5 minutes default
    format = 'json',
  } = input;

  // Build command args
  const commandArgs: string[] = [];
  
  if (command === 'npm' || command === 'yarn' || command === 'pnpm') {
    commandArgs.push('run', 'lint');
    if (fix) {
      commandArgs.push('--', '--fix');
    }
  } else if (command === 'eslint') {
    if (format === 'json') {
      commandArgs.push('--format', 'json');
    }
    if (fix) {
      commandArgs.push('--fix');
    }
    if (files.length > 0) {
      commandArgs.push(...files);
    } else {
      commandArgs.push('.');
    }
  }

  const result = await executeCommand({
    command,
    args: commandArgs,
    workingDir,
    timeout,
  });

  // Parse lint results
  const lintResults: LintResults = {
    errors: 0,
    warnings: 0,
    fixable: 0,
  };

  const issues: LintIssue[] = [];

  if (format === 'json' && result.stdout) {
    try {
      const lintOutput = JSON.parse(result.stdout);
      if (Array.isArray(lintOutput)) {
        for (const file of lintOutput) {
          if (file.messages && Array.isArray(file.messages)) {
            for (const message of file.messages) {
              const issue: LintIssue = {
                file: file.filePath || '',
                line: message.line || 0,
                column: message.column || 0,
                rule: message.ruleId || '',
                severity: message.severity === 2 ? 'error' : 'warning',
                message: message.message || '',
                fixable: !!message.fix,
              };

              issues.push(issue);
              
              if (issue.severity === 'error') {
                lintResults.errors++;
              } else {
                lintResults.warnings++;
              }

              if (issue.fixable) {
                lintResults.fixable++;
              }
            }
          }
        }
      }
    } catch (error) {
      // Failed to parse JSON, try text parsing
      const errorMatch = result.stdout.match(/(\d+)\s+error/);
      const warningMatch = result.stdout.match(/(\d+)\s+warning/);
      if (errorMatch) lintResults.errors = parseInt(errorMatch[1], 10);
      if (warningMatch) lintResults.warnings = parseInt(warningMatch[1], 10);
    }
  } else {
    // Parse text output
    const errorMatch = (result.stdout || result.stderr).match(/(\d+)\s+error/);
    const warningMatch = (result.stdout || result.stderr).match(/(\d+)\s+warning/);
    if (errorMatch) lintResults.errors = parseInt(errorMatch[1], 10);
    if (warningMatch) lintResults.warnings = parseInt(warningMatch[1], 10);
  }

  return {
    ...result,
    lintResults: lintResults.errors > 0 || lintResults.warnings > 0 ? lintResults : undefined,
    issues: issues.length > 0 ? issues : undefined,
  };
}

/**
 * Log command execution for optimization analysis
 */
export async function logCommandExecution(
  workspacePath: string,
  entry: CommandLogEntry
): Promise<void> {
  try {
    const logPath = path.join(workspacePath, 'command_execution_log.jsonl');
    const line = JSON.stringify({
      ...entry,
      timestamp: entry.timestamp || new Date().toISOString(),
    }) + '\n';
    await fs.appendFile(logPath, line);
  } catch (error) {
    // Non-blocking - log errors but don't fail
    console.warn('Failed to log command execution:', error);
  }
}

