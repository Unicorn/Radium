/**
 * Project Worker Manager
 * 
 * Manages Temporal workers on a per-project basis.
 * Each project gets one worker that handles all workflows in that project.
 * Workers dynamically load compiled code from the database.
 */

import { Worker, NativeConnection } from '@temporalio/worker';
import { createClient } from '@supabase/supabase-js';
import { loadWorkflowsForProject } from '../workflow-compiler/storage';
import type { Database } from '@/types/database';
import { bundleWorkflowCode } from '@temporalio/worker';
import * as path from 'path';
import * as fs from 'fs';
import * as os from 'os';

// Server-side Supabase client
function getStorageClient() {
  return createClient<Database>(
    process.env.NEXT_PUBLIC_SUPABASE_URL!,
    process.env.SUPABASE_SERVICE_ROLE_KEY!
  );
}

interface WorkerInfo {
  worker: Worker;
  projectId: string;
  workerId: string;
  heartbeatInterval: NodeJS.Timeout;
  bundlePath: string;
}

export class ProjectWorkerManager {
  private workers: Map<string, WorkerInfo> = new Map(); // projectId -> WorkerInfo
  
  /**
   * Start a worker for a project
   * Creates/reuses worker, loads workflows from database
   */
  async startWorkerForProject(projectId: string): Promise<Worker> {
    // Check if worker already running
    const existing = this.workers.get(projectId);
    if (existing) {
      console.log(`✅ Worker for project ${projectId} already running`);
      return existing.worker;
    }
    
    const supabase = getStorageClient();
    
    try {
      // 1. Get project details
      const { data: project, error: projectError } = await supabase
        .from('projects')
        .select('*')
        .eq('id', projectId)
        .single();
      
      if (projectError || !project) {
        throw new Error(`Project ${projectId} not found: ${projectError?.message}`);
      }
      
      console.log(`🔧 Starting worker for project "${project.name}" (${projectId})`);
      console.log(`📋 Task queue: ${project.task_queue_name}`);
      
      // 2. Load all active workflows for this project
      const workflows = await loadWorkflowsForProject(projectId);
      
      if (workflows.length === 0) {
        console.warn(`⚠️  No compiled workflows found for project ${projectId}`);
      } else {
        console.log(`📦 Loading ${workflows.length} workflow(s) from database`);
      }
      
      // 3. Create temporary directory for bundled code
      const bundleDir = path.join(os.tmpdir(), 'workflow-builder', projectId);
      if (!fs.existsSync(bundleDir)) {
        fs.mkdirSync(bundleDir, { recursive: true });
      }
      
      // 4. Write workflow code to temporary files for bundling
      const workflowPath = path.join(bundleDir, 'workflows.ts');
      const activitiesPath = path.join(bundleDir, 'activities.ts');
      
      // Combine all workflow code
      const allWorkflowCode = workflows
        .map(w => (w as any).compiled_code.workflow_code)
        .join('\n\n');
      
      const allActivitiesCode = workflows
        .map(w => (w as any).compiled_code.activities_code)
        .join('\n\n');
      
      fs.writeFileSync(workflowPath, allWorkflowCode);
      fs.writeFileSync(activitiesPath, allActivitiesCode);
      
      console.log(`📝 Wrote workflow code to ${workflowPath}`);
      
      // 5. Bundle workflow code using Temporal's bundler
      console.log(`🔨 Bundling workflow code...`);
      const { code: workflowBundle } = await bundleWorkflowCode({
        workflowsPath: workflowPath,
      });
      
      const bundlePath = path.join(bundleDir, 'workflow-bundle.js');
      fs.writeFileSync(bundlePath, workflowBundle);
      
      console.log(`✅ Workflow bundle created at ${bundlePath}`);
      
      // 6. Load activities dynamically
      const activities = await this.loadActivities(activitiesPath);
      
      // 7. Create Temporal connection
      const connection = await NativeConnection.connect({
        address: process.env.TEMPORAL_ADDRESS || 'localhost:7233',
      });
      
      console.log(`🔗 Connected to Temporal at ${process.env.TEMPORAL_ADDRESS || 'localhost:7233'}`);

      // 8. Create worker (concurrency limits configurable via environment variables)
      const maxConcurrentActivities = parseInt(
        process.env.MAX_CONCURRENT_ACTIVITY_EXECUTIONS || '10',
        10
      );
      const maxConcurrentWorkflowTasks = parseInt(
        process.env.MAX_CONCURRENT_WORKFLOW_EXECUTIONS || '10',
        10
      );

      const worker = await Worker.create({
        connection,
        namespace: process.env.TEMPORAL_NAMESPACE || 'default',
        taskQueue: project.task_queue_name,
        workflowBundle: {
          codePath: bundlePath,
        },
        activities,
        maxConcurrentActivityTaskExecutions: maxConcurrentActivities,
        maxConcurrentWorkflowTaskExecutions: maxConcurrentWorkflowTasks,
      });
      
      // 9. Generate worker ID
      const workerId = `worker-${projectId}-${Date.now()}`;
      
      // 10. Register worker in database
      await this.registerWorker(projectId, workerId, project.task_queue_name);
      
      // 11. Start worker (non-blocking)
      worker.run().catch(async (error) => {
        console.error(`❌ Worker error for project ${projectId}:`, error);
        await this.handleWorkerError(projectId, workerId, error);
      });
      
      // 12. Start heartbeat
      const heartbeatInterval = this.startHeartbeat(projectId, workerId);
      
      // 13. Store worker info
      this.workers.set(projectId, {
        worker,
        projectId,
        workerId,
        heartbeatInterval,
        bundlePath: bundleDir,
      });
      
      console.log(`✅ Worker started for project "${project.name}"`);
      console.log(`   Worker ID: ${workerId}`);
      console.log(`   Task Queue: ${project.task_queue_name}`);
      console.log(`   Workflows: ${workflows.length}`);
      console.log(`   Max Concurrent Activities: ${maxConcurrentActivities}`);
      console.log(`   Max Concurrent Workflow Tasks: ${maxConcurrentWorkflowTasks}`);
      
      return worker;
    } catch (error) {
      console.error(`❌ Failed to start worker for project ${projectId}:`, error);
      
      // Update database with failure
      await supabase
        .from('workflow_workers')
        .update({
          status: 'failed',
          metadata: { 
            error: error instanceof Error ? error.message : 'Unknown error',
            stack: error instanceof Error ? error.stack : undefined,
          },
          stopped_at: new Date().toISOString(),
        })
        .eq('project_id', projectId)
        .eq('status', 'starting');
      
      throw error;
    }
  }
  
  /**
   * Load activities from compiled code
   */
  private async loadActivities(activitiesPath: string): Promise<Record<string, any>> {
    try {
      // Dynamically import activities
      // Note: This requires the activities code to be valid ES modules
      const activitiesModule = await import(activitiesPath);
      
      console.log(`✅ Loaded ${Object.keys(activitiesModule).length} activities`);
      
      return activitiesModule;
    } catch (error) {
      console.error('❌ Failed to load activities:', error);
      
      // Return empty activities object if loading fails
      // Worker will still start but activities won't be available
      console.warn('⚠️  Worker starting without activities');
      return {};
    }
  }
  
  /**
   * Register worker in database
   */
  private async registerWorker(
    projectId: string,
    workerId: string,
    taskQueueName: string
  ): Promise<void> {
    const supabase = getStorageClient();
    
    const { error } = await supabase
      .from('workflow_workers')
      .insert({
        worker_id: workerId,
        project_id: projectId,
        task_queue_name: taskQueueName,
        status: 'running',
        host: process.env.HOSTNAME || os.hostname(),
        process_id: process.pid.toString(),
        started_at: new Date().toISOString(),
        last_heartbeat: new Date().toISOString(),
      });
    
    if (error) {
      console.error('⚠️  Failed to register worker in database:', error);
      // Don't throw - worker can still function
    } else {
      console.log(`✅ Worker registered in database: ${workerId}`);
    }
  }
  
  /**
   * Start heartbeat for worker
   */
  private startHeartbeat(projectId: string, workerId: string): NodeJS.Timeout {
    const interval = setInterval(async () => {
      const supabase = getStorageClient();
      
      try {
        await supabase
          .from('workflow_workers')
          .update({ 
            last_heartbeat: new Date().toISOString(),
            updated_at: new Date().toISOString(),
          })
          .eq('worker_id', workerId)
          .eq('status', 'running');
      } catch (error) {
        console.error(`⚠️  Heartbeat failed for worker ${workerId}:`, error);
      }
    }, 30000); // Every 30 seconds
    
    console.log(`💓 Heartbeat started for worker (30s interval)`);
    
    return interval;
  }
  
  /**
   * Stop worker for a project
   */
  async stopWorkerForProject(projectId: string): Promise<void> {
    const workerInfo = this.workers.get(projectId);
    if (!workerInfo) {
      console.log(`ℹ️  No worker running for project ${projectId}`);
      return;
    }
    
    console.log(`🛑 Stopping worker for project ${projectId}...`);
    
    try {
      // 1. Clear heartbeat
      clearInterval(workerInfo.heartbeatInterval);
      
      // 2. Shutdown worker gracefully
      await workerInfo.worker.shutdown();
      
      // 3. Clean up bundle files
      if (fs.existsSync(workerInfo.bundlePath)) {
        fs.rmSync(workerInfo.bundlePath, { recursive: true, force: true });
      }
      
      // 4. Update database
      const supabase = getStorageClient();
      await supabase
        .from('workflow_workers')
        .update({
          status: 'stopped',
          stopped_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
        })
        .eq('worker_id', workerInfo.workerId);
      
      // 5. Remove from map
      this.workers.delete(projectId);
      
      console.log(`✅ Worker stopped for project ${projectId}`);
    } catch (error) {
      console.error(`❌ Error stopping worker:`, error);
      throw error;
    }
  }
  
  /**
   * Handle worker error
   */
  private async handleWorkerError(
    projectId: string,
    workerId: string,
    error: Error
  ): Promise<void> {
    console.error(`💥 Worker crashed for project ${projectId}:`, error);
    
    const supabase = getStorageClient();
    
    // Update database
    await supabase
      .from('workflow_workers')
      .update({
        status: 'failed',
        stopped_at: new Date().toISOString(),
        metadata: {
          error: error.message,
          stack: error.stack,
        },
      })
      .eq('worker_id', workerId);
    
    // Clean up
    const workerInfo = this.workers.get(projectId);
    if (workerInfo) {
      clearInterval(workerInfo.heartbeatInterval);
      
      if (fs.existsSync(workerInfo.bundlePath)) {
        fs.rmSync(workerInfo.bundlePath, { recursive: true, force: true });
      }
      
      this.workers.delete(projectId);
    }
  }
  
  /**
   * Restart worker (stop and start)
   */
  async restartWorkerForProject(projectId: string): Promise<Worker> {
    console.log(`🔄 Restarting worker for project ${projectId}...`);
    
    await this.stopWorkerForProject(projectId);
    
    // Wait a bit for cleanup
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    return await this.startWorkerForProject(projectId);
  }
  
  /**
   * Get worker status for a project
   */
  getWorkerStatus(projectId: string): 'running' | 'stopped' {
    return this.workers.has(projectId) ? 'running' : 'stopped';
  }
  
  /**
   * Get all running workers
   */
  getRunningWorkers(): string[] {
    return Array.from(this.workers.keys());
  }
  
  /**
   * Shutdown all workers (for graceful app shutdown)
   */
  async shutdownAll(): Promise<void> {
    console.log(`🛑 Shutting down all workers...`);
    
    const projectIds = Array.from(this.workers.keys());
    
    await Promise.all(
      projectIds.map(id => this.stopWorkerForProject(id))
    );
    
    console.log(`✅ All workers shut down`);
  }
}

// Singleton instance
export const projectWorkerManager = new ProjectWorkerManager();

// Graceful shutdown on process termination
process.on('SIGTERM', async () => {
  console.log('📡 SIGTERM received, shutting down workers...');
  await projectWorkerManager.shutdownAll();
  process.exit(0);
});

process.on('SIGINT', async () => {
  console.log('📡 SIGINT received, shutting down workers...');
  await projectWorkerManager.shutdownAll();
  process.exit(0);
});

