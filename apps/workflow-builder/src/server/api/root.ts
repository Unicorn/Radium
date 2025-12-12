/**
 * Root tRPC router
 */

import { createTRPCRouter } from './trpc';
import { usersRouter } from './routers/users';
import { componentsRouter } from './routers/components';
import { agentPromptsRouter } from './routers/agent-prompts';
import { taskQueuesRouter } from './routers/task-queues';
import { workflowsRouter } from './routers/workflows';
import { workQueuesRouter } from './routers/work-queues';
import { signalsRouter, queriesRouter } from './routers/signals-queries';
import { compilerRouter } from './routers/compiler';
import { deploymentRouter } from './routers/deployment';
import { executionRouter } from './routers/execution';
import { projectsRouter } from './routers/projects';
import { agentBuilderRouter } from './routers/agent-builder';
import { agentTesterRouter } from './routers/agent-tester';
import { workflowEndpointsRouter } from './routers/workflow-endpoints';
import { connectionsRouter } from './routers/connections';
import { activitiesRouter } from './routers/activities';
import { apiKeysRouter } from './routers/apiKeys';
import { serviceInterfacesRouter } from './routers/serviceInterfaces';
import { publicInterfacesRouter } from './routers/publicInterfaces';
import { connectorsRouter } from './routers/connectors';
import { projectConnectorsRouter } from './routers/projectConnectors';
import { fileOperationsRouter } from './routers/file-operations';
import { notificationsRouter } from './routers/notifications';
import { stateVariablesRouter } from './routers/stateVariables';
import { projectLoggingRouter } from './routers/projectLogging';
import { kongCacheRouter } from './routers/kongCache';
import { stateMonitoringRouter } from './routers/stateMonitoring';

export const appRouter = createTRPCRouter({
  users: usersRouter,
  components: componentsRouter,
  agentPrompts: agentPromptsRouter,
  taskQueues: taskQueuesRouter,
  workflows: workflowsRouter,
  workQueues: workQueuesRouter,
  signals: signalsRouter,
  queries: queriesRouter,
  compiler: compilerRouter,
  deployment: deploymentRouter,
  execution: executionRouter,
  projects: projectsRouter,
  agentBuilder: agentBuilderRouter,
  agentTester: agentTesterRouter,
  workflowEndpoints: workflowEndpointsRouter,
  connections: connectionsRouter,
  activities: activitiesRouter,
  apiKeys: apiKeysRouter,
  serviceInterfaces: serviceInterfacesRouter,
  publicInterfaces: publicInterfacesRouter,
  connectors: connectorsRouter,
  projectConnectors: projectConnectorsRouter,
  fileOperations: fileOperationsRouter,
  notifications: notificationsRouter,
  stateVariables: stateVariablesRouter,
  projectLogging: projectLoggingRouter,
  kongCache: kongCacheRouter,
  stateMonitoring: stateMonitoringRouter,
});

export type AppRouter = typeof appRouter;

