/**
 * Projects Router
 * 
 * Main router that re-exports all project sub-routers for backward compatibility.
 * Split into sub-routers for better organization:
 * - projects-core.ts: Core CRUD operations (list, get, create, update, archive, unarchive)
 * - project-settings.ts: Settings and statistics (stats)
 * - project-workers.ts: Worker management (workerHealth, startWorker, stopWorker)
 */

import { createTRPCRouter } from '../trpc';
import { projectsCoreRouter } from './projects-core';
import { projectSettingsRouter } from './project-settings';
import { projectWorkersRouter } from './project-workers';

// Re-export all procedures from sub-routers for backward compatibility
// Using manual assignment to preserve TypeScript types
export const projectsRouter = createTRPCRouter({
  // Core operations
  list: projectsCoreRouter._def.procedures.list,
  get: projectsCoreRouter._def.procedures.get,
  create: projectsCoreRouter._def.procedures.create,
  update: projectsCoreRouter._def.procedures.update,
  archive: projectsCoreRouter._def.procedures.archive,
  unarchive: projectsCoreRouter._def.procedures.unarchive,
  
  // Settings operations
  stats: projectSettingsRouter._def.procedures.stats,
  
  // Worker operations
  workerHealth: projectWorkersRouter._def.procedures.workerHealth,
  startWorker: projectWorkersRouter._def.procedures.startWorker,
  stopWorker: projectWorkersRouter._def.procedures.stopWorker,
});
