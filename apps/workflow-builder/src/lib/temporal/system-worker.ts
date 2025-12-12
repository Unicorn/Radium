/**
 * System Worker Manager
 * 
 * Manages a dedicated worker for system workflows (agent tester, etc.)
 * System workflows are loaded directly from source, not from database
 */

import { Worker, NativeConnection } from '@temporalio/worker';
import { bundleWorkflowCode } from '@temporalio/worker';
import * as path from 'path';
import * as fs from 'fs';
import * as os from 'os';

let systemWorker: Worker | null = null;
let systemWorkerRunning = false;

/**
 * Start the system worker
 * Loads system workflows from source files
 */
export async function startSystemWorker(): Promise<Worker> {
  if (systemWorker && systemWorkerRunning) {
    console.log('✅ System worker already running');
    return systemWorker;
  }
  
  console.log('🔧 Starting system worker...');
  
  try {
    // 1. Create temporary directory for bundled code
    const bundleDir = path.join(os.tmpdir(), 'workflow-builder', 'system');
    if (!fs.existsSync(bundleDir)) {
      fs.mkdirSync(bundleDir, { recursive: true });
    }
    
    // 2. Write system workflow code
    const workflowPath = path.join(bundleDir, 'workflows.ts');
    const activitiesPath = path.join(bundleDir, 'activities.ts');
    
    // Import and write workflow code
    // Note: In production, these would be loaded from compiled files
    // For now, we'll write them directly
    const workflowCode = `
import { agentTesterWorkflow } from '@/lib/agent-tester/workflow';

export { agentTesterWorkflow };
`;
    
    const activitiesCode = `
import * as activities from '@/lib/agent-tester/activities';

export { activities };
`;
    
    // For now, we need to use a different approach since we can't import at runtime
    // We'll need to compile the workflows first or use a different bundling strategy
    // This is a placeholder - the actual implementation would need to handle
    // TypeScript compilation and bundling properly
    
    fs.writeFileSync(workflowPath, workflowCode);
    fs.writeFileSync(activitiesPath, activitiesCode);
    
    // 3. Bundle workflow code
    console.log('🔨 Bundling system workflow code...');
    const { code: workflowBundle } = await bundleWorkflowCode({
      workflowsPath: workflowPath,
    });
    
    const bundlePath = path.join(bundleDir, 'workflow-bundle.js');
    fs.writeFileSync(bundlePath, workflowBundle);
    
    console.log('✅ System workflow bundle created');
    
    // 4. Load activities
    // Note: Activities need to be available at runtime
    // We'll need to ensure they're compiled and accessible
    const activities = await import('@/lib/agent-tester/activities');
    
    // 5. Create Temporal connection
    const connection = await NativeConnection.connect({
      address: process.env.TEMPORAL_ADDRESS || 'localhost:7233',
    });
    
    console.log('🔗 Connected to Temporal for system worker');
    
    // 6. Create worker
    systemWorker = await Worker.create({
      connection,
      namespace: process.env.TEMPORAL_NAMESPACE || 'default',
      taskQueue: 'system-workflows-queue',
      workflowBundle: {
        codePath: bundlePath,
      },
      activities: {
        ...activities,
      },
      maxConcurrentActivityTaskExecutions: 10,
      maxConcurrentWorkflowTaskExecutions: 10,
    });
    
    // 7. Start worker (non-blocking)
    systemWorker.run().catch((error) => {
      console.error('❌ System worker error:', error);
      systemWorkerRunning = false;
      systemWorker = null;
    });
    
    systemWorkerRunning = true;
    console.log('✅ System worker started on queue: system-workflows-queue');
    
    return systemWorker;
  } catch (error) {
    console.error('❌ Failed to start system worker:', error);
    systemWorker = null;
    systemWorkerRunning = false;
    throw error;
  }
}

/**
 * Stop the system worker
 */
export async function stopSystemWorker(): Promise<void> {
  if (systemWorker) {
    await systemWorker.shutdown();
    systemWorker = null;
    systemWorkerRunning = false;
    console.log('✅ System worker stopped');
  }
}

/**
 * Check if system worker is running
 */
export function isSystemWorkerRunning(): boolean {
  return systemWorkerRunning && systemWorker !== null;
}

