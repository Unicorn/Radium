/**
 * Workflow Compiler
 * Generates executable Temporal TypeScript workflow code from JSON definitions
 */

import type { TemporalWorkflow } from '@/types/advanced-patterns';
import type { WorkflowDefinition } from '@/types/workflow';
import { compileWorkflowFromNodes } from './node-compiler';

export interface CompilerOptions {
  packageName?: string;
  outputPath?: string;
  includeComments?: boolean;
  strictMode?: boolean;
}

export interface CompiledWorkflow {
  workflowCode: string;
  activitiesCode: string;
  workerCode: string;
  packageJson: string;
  tsConfig: string;
}

/**
 * Compile a workflow definition into executable TypeScript code
 * Supports both TemporalWorkflow (stages) and WorkflowDefinition (nodes/edges) formats
 */
export function compileWorkflow(
  workflow: TemporalWorkflow | WorkflowDefinition,
  options: CompilerOptions = {}
): CompiledWorkflow {
  // Check if this is a node-based definition
  if ('nodes' in workflow && 'edges' in workflow) {
    return compileNodeBasedWorkflow(workflow as WorkflowDefinition, options);
  }
  
  // Legacy stage-based workflow
  return compileStageBasedWorkflow(workflow as TemporalWorkflow, options);
}

/**
 * Compile node-based workflow definition
 */
function compileNodeBasedWorkflow(
  definition: WorkflowDefinition,
  options: CompilerOptions
): CompiledWorkflow {
  const {
    packageName = 'workflow',
    includeComments = true,
    strictMode = true,
  } = options;

  const workflowCode = compileWorkflowFromNodes(definition, { 
    includeComments,
    workflowName: definition.name,
  });
  const activitiesCode = generateActivitiesFromNodes(definition, includeComments);
  const workerCode = generateWorkerCodeFromNodes(definition, packageName, includeComments);

  return {
    workflowCode,
    activitiesCode,
    workerCode,
    packageJson: generatePackageJson(packageName, 'Workflow'),
    tsConfig: generateTsConfig(strictMode),
  };
}

/**
 * Compile stage-based workflow (legacy)
 */
function compileStageBasedWorkflow(
  workflow: TemporalWorkflow,
  options: CompilerOptions
): CompiledWorkflow {
  const {
    packageName = workflow.kebab_name || (workflow.name?.toLowerCase().replace(/\s+/g, '-')) || 'workflow',
    includeComments = true,
    strictMode = true,
  } = options;

  const displayName = workflow.display_name || workflow.name || workflow.kebab_name || 'Workflow';

  return {
    workflowCode: generateWorkflowCode(workflow, includeComments),
    activitiesCode: generateActivitiesCode(workflow, includeComments),
    workerCode: generateWorkerCode(workflow, packageName, includeComments),
    packageJson: generatePackageJson(packageName, displayName),
    tsConfig: generateTsConfig(strictMode),
  };
}

/**
 * Generate the main workflow TypeScript code
 */
function generateWorkflowCode(workflow: TemporalWorkflow, includeComments: boolean): string {
  const imports = generateImports(workflow);
  const workflowFunction = generateWorkflowFunction(workflow, includeComments);
  const signalHandlers = generateSignalHandlers(workflow, includeComments);
  const queryHandlers = generateQueryHandlers(workflow, includeComments);

  return `${imports}

${signalHandlers}

${queryHandlers}

${workflowFunction}
`;
}

/**
 * Generate imports for the workflow
 */
function generateImports(workflow: TemporalWorkflow): string {
  const imports = [
    `import { proxyActivities, defineSignal, defineQuery, setHandler, condition } from '@temporalio/workflow';`,
  ];

  // Check if we have scheduled workflows
  const hasScheduledWorkflows = workflow.stages.some(s => s.type === 'scheduled-workflow');
  if (hasScheduledWorkflows) {
    imports.push(`import { ChildWorkflowHandle, startChild } from '@temporalio/workflow';`);
  }

  // Import activities interface
  imports.push(`import type * as activities from './activities';`);

  return imports.join('\n');
}

/**
 * Generate signal handlers
 */
function generateSignalHandlers(workflow: TemporalWorkflow, includeComments: boolean): string {
  if (!workflow.signals || workflow.signals.length === 0) {
    return '';
  }

  const handlers = workflow.signals.map(signal => {
    const paramType = signal.parametersSchema 
      ? `{ ${Object.keys(signal.parametersSchema.properties || {}).map(key => `${key}: any`).join(', ')} }`
      : 'any';

    return `${includeComments ? `// Signal: ${signal.description || signal.name}` : ''}
export const ${signal.name} = defineSignal<[${paramType}]>('${signal.name}');`;
  });

  return handlers.join('\n\n');
}

/**
 * Generate query handlers
 */
function generateQueryHandlers(workflow: TemporalWorkflow, includeComments: boolean): string {
  if (!workflow.queries || workflow.queries.length === 0) {
    return '';
  }

  const handlers = workflow.queries.map(query => {
    const returnType = 'any'; // TODO: Parse from returnTypeSchema

    return `${includeComments ? `// Query: ${query.description || query.name}` : ''}
export const ${query.name} = defineQuery<${returnType}>('${query.name}');`;
  });

  return handlers.join('\n\n');
}

/**
 * Generate the main workflow function
 */
function generateWorkflowFunction(workflow: TemporalWorkflow, includeComments: boolean): string {
  const activityProxies = generateActivityProxies(workflow);
  const workflowLogic = generateWorkflowLogic(workflow, includeComments);

  return `${includeComments ? `/**
 * ${workflow.display_name || workflow.name}
 * ${workflow.description || 'Generated Temporal workflow'}
 */` : ''}
export async function ${toCamelCase(workflow.kebab_name || workflow.name)}Workflow(input: any): Promise<any> {
${activityProxies}

${generateStateVariables(workflow)}

${generateSignalHandlerSetup(workflow, includeComments)}

${generateQueryHandlerSetup(workflow, includeComments)}

${workflowLogic}
}`;
}

/**
 * Generate activity proxies
 */
function generateActivityProxies(workflow: TemporalWorkflow): string {
  const activities = workflow.stages.filter(s => s.type === 'activity' || s.type === 'agent');
  
  if (activities.length === 0) {
    return '';
  }

  return `  // Activity proxies
  const { ${activities.map(a => a.metadata?.activityName || toCamelCase(a.id)).join(', ')} } = proxyActivities<typeof activities>({
    startToCloseTimeout: '5 minutes',
    retry: {
      initialInterval: '1s',
      backoffCoefficient: 2,
      maximumAttempts: 3,
    },
  });`;
}

/**
 * Generate state variables
 */
function generateStateVariables(workflow: TemporalWorkflow): string {
  const variables: string[] = [];

  // Work queues
  if (workflow.workQueues && workflow.workQueues.length > 0) {
    workflow.workQueues.forEach(wq => {
      variables.push(`  // Work queue: ${wq.name}`);
      variables.push(`  const ${wq.name}: any[] = [];`);
    });
  }

  return variables.length > 0 ? '\n' + variables.join('\n') + '\n' : '';
}

/**
 * Generate signal handler setup
 */
function generateSignalHandlerSetup(workflow: TemporalWorkflow, includeComments: boolean): string {
  if (!workflow.signals || workflow.signals.length === 0) {
    return '';
  }

  const handlers = workflow.signals.map(signal => {
    // Check if this signal adds to a work queue
    const targetQueue = workflow.workQueues?.find(wq => wq.signalName === signal.name);

    if (targetQueue) {
      return `  ${includeComments ? `// Signal handler: ${signal.name} -> adds to ${targetQueue.name}` : ''}
  setHandler(${signal.name}, (item: any) => {
    ${targetQueue.name}.push(item);
  });`;
    }

    return `  ${includeComments ? `// Signal handler: ${signal.name}` : ''}
  setHandler(${signal.name}, (data: any) => {
    // TODO: Implement signal handler logic
    console.log('Received signal ${signal.name}:', data);
  });`;
  });

  return '\n' + handlers.join('\n\n') + '\n';
}

/**
 * Generate query handler setup
 */
function generateQueryHandlerSetup(workflow: TemporalWorkflow, includeComments: boolean): string {
  if (!workflow.queries || workflow.queries.length === 0) {
    return '';
  }

  const handlers = workflow.queries.map(query => {
    // Check if this query returns a work queue status
    const targetQueue = workflow.workQueues?.find(wq => wq.queryName === query.name);

    if (targetQueue) {
      return `  ${includeComments ? `// Query handler: ${query.name} -> returns ${targetQueue.name} status` : ''}
  setHandler(${query.name}, () => ({
    count: ${targetQueue.name}.length,
    items: ${targetQueue.name},
    maxSize: ${targetQueue.maxSize || 'null'},
    isFull: ${targetQueue.maxSize ? `${targetQueue.name}.length >= ${targetQueue.maxSize}` : 'false'},
  }));`;
    }

    return `  ${includeComments ? `// Query handler: ${query.name}` : ''}
  setHandler(${query.name}, () => {
    // TODO: Implement query handler logic
    return null;
  });`;
  });

  return '\n' + handlers.join('\n\n') + '\n';
}

/**
 * Generate activities from node-based definition
 */
function generateActivitiesFromNodes(definition: WorkflowDefinition, includeComments: boolean): string {
  const nodes = definition.nodes || [];
  const activityNodes = nodes.filter(n => n.type === 'activity' || n.type === 'agent');

  if (activityNodes.length === 0) {
    return `${includeComments ? '// No activities defined' : ''}
export async function placeholderActivity(input: any): Promise<any> {
  return { success: true };
}`;
  }

  const activityFunctions = activityNodes.map(node => {
    const activityName = node.data.componentName || toCamelCase(node.id);
    const description = node.data.label || node.id;

    return `${includeComments ? `/**
 * ${description}
 */` : ''}
export async function ${activityName}(input: any): Promise<any> {
  ${includeComments ? '// TODO: Implement activity logic' : ''}
  console.log('Executing ${activityName}', input);
  
  ${node.type === 'agent' ? `// This is an AI agent activity
  // Call your LLM provider here` : ''}
  
  return { success: true, data: input };
}`;
  });

  return activityFunctions.join('\n\n');
}

/**
 * Generate worker code from node-based definition
 */
function generateWorkerCodeFromNodes(
  definition: WorkflowDefinition,
  packageName: string,
  includeComments: boolean
): string {
  return `${includeComments ? `/**
 * Temporal Worker
 * Auto-generated from workflow definition
 */` : ''}
import { Worker } from '@temporalio/worker';
import * as activities from './activities';

async function run() {
  const worker = await Worker.create({
    workflowsPath: require.resolve('./workflows'),
    activities,
    taskQueue: 'default',
  });

  await worker.run();
}

run().catch((err) => {
  console.error(err);
  process.exit(1);
});
`;
}

/**
 * Generate workflow execution logic (legacy stage-based)
 */
function generateWorkflowLogic(workflow: TemporalWorkflow, includeComments: boolean): string {
  const stages = workflow.stages;
  const logic: string[] = [];

  logic.push(`  ${includeComments ? '// Main workflow logic' : ''}`);

  // Handle scheduled workflows (start them first)
  const scheduledWorkflows = stages.filter(s => s.type === 'scheduled-workflow');
  if (scheduledWorkflows.length > 0) {
    logic.push(`\n  ${includeComments ? '// Start scheduled child workflows' : ''}`);
    scheduledWorkflows.forEach(sw => {
      const metadata = sw.metadata as any;
      logic.push(`  const ${toCamelCase(sw.id)}Handle = await startChild(${toCamelCase(metadata.workflowName || sw.id)}Workflow, {
    workflowId: '${sw.id}',
    taskQueue: '${metadata.taskQueue || 'default'}',
    cronSchedule: '${metadata.cronExpression}',
  });`);
    });
  }

  // Process stages in order
  logic.push(`\n  ${includeComments ? '// Execute workflow stages' : ''}`);
  
  stages.filter(s => s.type === 'activity' || s.type === 'agent').forEach((stage, index) => {
    const activityName = stage.metadata?.activityName || toCamelCase(stage.id);
    
    logic.push(`\n  ${includeComments ? `// Stage ${index + 1}: ${stage.id}` : ''}`);
    logic.push(`  const result${index} = await ${activityName}(input);`);
  });

  logic.push(`\n  return { success: true, results: [${stages.filter(s => s.type === 'activity' || s.type === 'agent').map((_, i) => `result${i}`).join(', ')}] };`);

  return logic.join('\n');
}

/**
 * Generate activities TypeScript code
 */
function generateActivitiesCode(workflow: TemporalWorkflow, includeComments: boolean): string {
  const activities = workflow.stages.filter(s => s.type === 'activity' || s.type === 'agent');

  if (activities.length === 0) {
    return `${includeComments ? '// No activities defined' : ''}
export async function placeholderActivity(input: any): Promise<any> {
  return { success: true };
}`;
  }

  const activityFunctions = activities.map(activity => {
    const activityName = activity.metadata?.activityName || toCamelCase(activity.id);
    const description = activity.metadata?.description || activity.id;

    return `${includeComments ? `/**
 * ${description}
 */` : ''}
export async function ${activityName}(input: any): Promise<any> {
  ${includeComments ? '// TODO: Implement activity logic' : ''}
  console.log('Executing ${activityName}', input);
  
  ${activity.type === 'agent' ? `// This is an AI agent activity
  // Call your LLM provider here with prompt ID: ${activity.metadata?.agentPromptId || 'unknown'}` : ''}
  
  return { success: true, data: input };
}`;
  });

  return activityFunctions.join('\n\n');
}

/**
 * Generate worker TypeScript code
 */
function generateWorkerCode(workflow: TemporalWorkflow, packageName: string, includeComments: boolean): string {

  return `${includeComments ? `/**
 * Temporal Worker for ${workflow.display_name || workflow.name}
 * Auto-generated from workflow definition
 */` : ''}
import { Worker } from '@temporalio/worker';
import * as activities from './activities';

async function run() {
  const worker = await Worker.create({
    workflowsPath: require.resolve('./workflows'),
    activities,
    taskQueue: '${workflow.stages[0]?.metadata?.taskQueue || 'default'}',
  });

  await worker.run();
}

run().catch((err) => {
  console.error(err);
  process.exit(1);
});
`;
}

/**
 * Generate package.json
 */
function generatePackageJson(packageName: string, workflowName: string): string {
  return JSON.stringify({
    name: packageName,
    version: '1.0.0',
    description: `Temporal workflow: ${workflowName}`,
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
 * Generate tsconfig.json
 */
function generateTsConfig(strictMode: boolean): string {
  return JSON.stringify({
    compilerOptions: {
      target: 'es2020',
      module: 'commonjs',
      lib: ['es2020'],
      outDir: './dist',
      rootDir: './src',
      strict: strictMode,
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
 * Helper: Convert string to camelCase
 */
function toCamelCase(str: string): string {
  return str
    .replace(/[^a-zA-Z0-9]+(.)/g, (_, chr) => chr.toUpperCase())
    .replace(/^[A-Z]/, chr => chr.toLowerCase());
}

