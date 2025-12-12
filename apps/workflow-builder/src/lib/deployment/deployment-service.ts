/**
 * Deployment Service
 * Handles deployment of compiled workflows to the worker service
 */

import { writeFile, mkdir, rm } from 'fs/promises';
import { join } from 'path';
import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

export interface DeploymentResult {
  success: boolean;
  workflowId?: string;
  deployedAt?: Date;
  files?: string[];
  error?: string;
}

export interface WorkflowCode {
  workflowCode: string;
  activitiesCode: string;
  workerCode: string;
}

export class DeploymentService {
  private workflowsDir: string;
  private activitiesDir: string;
  private workerApiUrl: string;
  private skipCompilation: boolean;

  constructor(options?: { skipCompilation?: boolean }) {
    this.workflowsDir = process.env.WORKFLOWS_DIR || '/tmp/workflow-builder/workflows';
    this.activitiesDir = process.env.ACTIVITIES_DIR || '/tmp/workflow-builder/activities';
    this.workerApiUrl = process.env.WORKER_API_URL || 'http://localhost:3011';
    this.skipCompilation = options?.skipCompilation || process.env.NODE_ENV === 'test';
  }

  /**
   * Deploy compiled workflow to worker
   */
  async deployWorkflow(
    workflowId: string,
    code: WorkflowCode
  ): Promise<DeploymentResult> {
    try {
      // 1. Create workflow directory
      const workflowDir = join(this.workflowsDir, workflowId);
      await mkdir(workflowDir, { recursive: true });

      // 2. Create src directory for TypeScript files
      const srcDir = join(workflowDir, 'src');
      await mkdir(srcDir, { recursive: true });

      // 3. Write workflow files
      await writeFile(
        join(srcDir, 'workflow.ts'),
        code.workflowCode
      );

      await writeFile(
        join(srcDir, 'activities.ts'),
        code.activitiesCode
      );

      await writeFile(
        join(srcDir, 'worker.ts'),
        code.workerCode
      );

      // 4. Create package.json if needed
      const packageJsonExists = await this.fileExists(join(workflowDir, 'package.json'));
      if (!packageJsonExists) {
        await writeFile(
          join(workflowDir, 'package.json'),
          this.generatePackageJson(workflowId)
        );
      }

      // 5. Create tsconfig.json if needed
      const tsconfigExists = await this.fileExists(join(workflowDir, 'tsconfig.json'));
      if (!tsconfigExists) {
        await writeFile(
          join(workflowDir, 'tsconfig.json'),
          this.generateTsConfig()
        );
      }

      // 6. Compile TypeScript to JavaScript (skip in test mode)
      if (!this.skipCompilation) {
        console.log(`📦 Compiling workflow ${workflowId}...`);

        // Use the TypeScript compiler from the parent project's node_modules
        // This ensures we don't need to install TypeScript in each workflow directory
        const tscPath = join(process.cwd(), 'node_modules', '.bin', 'tsc');
        const { stdout, stderr } = await execAsync(
          `${tscPath} --project ${workflowDir}/tsconfig.json`,
          { cwd: workflowDir }
        );

        if (stderr && !stderr.includes('deprecated') && !stderr.includes('ExperimentalWarning')) {
          console.warn(`⚠️  Compilation warnings for ${workflowId}:`, stderr);
        }

        if (stdout) {
          console.log(`📝 Compilation output:`, stdout);
        }

        // 7. Verify compilation produced files
        const distDir = join(workflowDir, 'dist');
        const compiledFiles = [
          join(distDir, 'workflow.js'),
          join(distDir, 'activities.js'),
          join(distDir, 'worker.js'),
        ];

        for (const file of compiledFiles) {
          const exists = await this.fileExists(file);
          if (!exists) {
            throw new Error(`Compilation failed: ${file} not created`);
          }
        }
      } else {
        console.log(`⏭️  Skipping compilation for ${workflowId} (test mode)`);

        // Create dist directory and mock compiled files for testing
        const distDir = join(workflowDir, 'dist');
        await mkdir(distDir, { recursive: true });

        // Create placeholder compiled files
        await writeFile(join(distDir, 'workflow.js'), '// Compiled workflow\n');
        await writeFile(join(distDir, 'activities.js'), '// Compiled activities\n');
        await writeFile(join(distDir, 'worker.js'), '// Compiled worker\n');
      }

      // 8. Notify worker to reload
      await this.notifyWorkerReload(workflowId);

      return {
        success: true,
        workflowId,
        deployedAt: new Date(),
        files: [
          `${workflowId}/src/workflow.ts`,
          `${workflowId}/dist/workflow.js`,
          `${workflowId}/src/activities.ts`,
          `${workflowId}/dist/activities.js`,
          `${workflowId}/src/worker.ts`,
          `${workflowId}/dist/worker.js`,
        ]
      };
    } catch (error) {
      console.error(`❌ Deployment failed for ${workflowId}:`, error);
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  /**
   * Notify worker to reload workflows
   */
  private async notifyWorkerReload(workflowId: string): Promise<void> {
    try {
      console.log(`📡 Notifying worker to reload ${workflowId}...`);

      const response = await fetch(`${this.workerApiUrl}/workflows/reload`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ workflowId }),
      });

      if (!response.ok) {
        throw new Error(`Worker reload notification failed: ${response.status} ${response.statusText}`);
      }

      const result = await response.json();
      console.log(`✅ Worker reload notification sent:`, result);
    } catch (error) {
      console.warn('⚠️  Failed to notify worker (non-fatal):', error);
      // Non-fatal - worker will pick up on next restart
    }
  }

  /**
   * Undeploy workflow
   */
  async undeployWorkflow(workflowId: string): Promise<void> {
    const workflowDir = join(this.workflowsDir, workflowId);

    try {
      console.log(`🗑️  Undeploying workflow ${workflowId}...`);
      await rm(workflowDir, { recursive: true, force: true });
      console.log(`✅ Workflow ${workflowId} undeployed`);
    } catch (error) {
      console.error(`❌ Failed to undeploy ${workflowId}:`, error);
      throw error;
    }
  }

  /**
   * List deployed workflows
   */
  async listDeployed(): Promise<string[]> {
    try {
      const { stdout } = await execAsync(`ls ${this.workflowsDir}`);
      return stdout.trim().split('\n').filter(Boolean);
    } catch (error) {
      // Directory doesn't exist or is empty
      return [];
    }
  }

  /**
   * Get deployment info for a specific workflow
   */
  async getDeploymentInfo(workflowId: string): Promise<{
    exists: boolean;
    path?: string;
    files?: string[];
  }> {
    const workflowDir = join(this.workflowsDir, workflowId);
    const exists = await this.fileExists(workflowDir);

    if (!exists) {
      return { exists: false };
    }

    try {
      const { stdout } = await execAsync(`find ${workflowDir} -type f`, { cwd: workflowDir });
      const files = stdout.trim().split('\n').filter(Boolean);

      return {
        exists: true,
        path: workflowDir,
        files: files.map(f => f.replace(`${workflowDir}/`, '')),
      };
    } catch (error) {
      return {
        exists: true,
        path: workflowDir,
        files: [],
      };
    }
  }

  /**
   * Check if a file exists
   */
  private async fileExists(path: string): Promise<boolean> {
    try {
      const { stdout } = await execAsync(`test -e "${path}" && echo "exists" || echo "not_found"`);
      return stdout.trim() === 'exists';
    } catch {
      return false;
    }
  }

  /**
   * Generate package.json for workflow
   */
  private generatePackageJson(workflowId: string): string {
    return JSON.stringify({
      name: `workflow-${workflowId}`,
      version: '1.0.0',
      description: `Temporal workflow: ${workflowId}`,
      main: 'dist/worker.js',
      scripts: {
        build: 'tsc',
        'start.watch': 'nodemon dist/worker.js',
        start: 'node dist/worker.js',
      },
      dependencies: {
        '@temporalio/worker': '^1.10.0',
        '@temporalio/workflow': '^1.10.0',
        '@temporalio/activity': '^1.10.0',
      },
      devDependencies: {
        '@types/node': '^20.0.0',
        nodemon: '^3.0.0',
        typescript: '^5.0.0',
      },
    }, null, 2);
  }

  /**
   * Generate tsconfig.json for workflow
   */
  private generateTsConfig(): string {
    return JSON.stringify({
      compilerOptions: {
        target: 'es2020',
        module: 'commonjs',
        lib: ['es2020'],
        outDir: './dist',
        rootDir: './src',
        strict: true,
        esModuleInterop: true,
        skipLibCheck: true,
        forceConsistentCasingInFileNames: true,
        resolveJsonModule: true,
        declaration: true,
      },
      include: ['src/**/*'],
      exclude: ['node_modules', 'dist'],
    }, null, 2);
  }

  /**
   * Validate workflow code before deployment
   */
  validateWorkflowCode(code: WorkflowCode): { valid: boolean; errors: string[] } {
    const errors: string[] = [];

    if (!code.workflowCode || code.workflowCode.trim().length === 0) {
      errors.push('Workflow code is empty');
    }

    if (!code.activitiesCode || code.activitiesCode.trim().length === 0) {
      errors.push('Activities code is empty');
    }

    if (!code.workerCode || code.workerCode.trim().length === 0) {
      errors.push('Worker code is empty');
    }

    // Check for required imports in workflow code
    if (code.workflowCode && !code.workflowCode.includes('@temporalio/workflow')) {
      errors.push('Workflow code missing @temporalio/workflow import');
    }

    // Check for async workflow function
    if (code.workflowCode && !code.workflowCode.includes('async function')) {
      errors.push('Workflow code missing async function');
    }

    return {
      valid: errors.length === 0,
      errors,
    };
  }
}
