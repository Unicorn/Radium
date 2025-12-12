/**
 * Sync existing component registries to Supabase database
 */

import { createClient } from '@supabase/supabase-js';
import type { Database } from '@/types/database';

// This will be imported from the existing system once we integrate
interface ActivityMetadata {
  name: string;
  displayName: string;
  description: string;
  activityType: 'standard' | 'cli' | 'agentic';
  version: string;
  tags?: string[];
  capabilities?: string[];
  modelProvider?: string;
  modelName?: string;
  agentPromptId?: string;
}

/**
 * Sync components from existing ActivityRegistry to Supabase
 * This should be run as a one-time migration or periodically
 */
export async function syncActivityRegistryToDatabase(
  activities: ActivityMetadata[],
  supabaseUrl: string,
  supabaseKey: string,
  userId: string
) {
  const supabase = createClient<Database>(supabaseUrl, supabaseKey);

  // Get component type IDs
  const { data: activityType } = await supabase
    .from('component_types')
    .select('id')
    .eq('name', 'activity')
    .single();

  const { data: publicVisibility } = await supabase
    .from('component_visibility')
    .select('id')
    .eq('name', 'public')
    .single();

  if (!activityType || !publicVisibility) {
    throw new Error('Required lookup data not found in database');
  }

  const results = {
    created: 0,
    skipped: 0,
    errors: [] as string[],
  };

  for (const activity of activities) {
    try {
      // Check if already exists
      const { data: existing } = await supabase
        .from('components')
        .select('id')
        .eq('name', activity.name)
        .eq('version', activity.version)
        .eq('created_by', userId)
        .single();

      if (existing) {
        results.skipped++;
        continue;
      }

      // Create component
      const { error } = await supabase
        .from('components')
        .insert({
          name: activity.name,
          display_name: activity.displayName,
          description: activity.description,
          component_type_id: activityType.id,
          version: activity.version,
          created_by: userId,
          visibility_id: publicVisibility.id,
          tags: activity.tags || null,
          capabilities: activity.capabilities || null,
          model_provider: activity.modelProvider || null,
          model_name: activity.modelName || null,
          // Note: agent_prompt_id would need to be looked up/created separately
        });

      if (error) {
        results.errors.push(`${activity.name}: ${error.message}`);
      } else {
        results.created++;
      }
    } catch (err: any) {
      results.errors.push(`${activity.name}: ${err.message}`);
    }
  }

  return results;
}

/**
 * Example usage script
 */
export async function syncExistingComponents() {
  // This would import from existing packages
  // const { ActivityRegistry } = await import('@coordinator/temporal-registry');
  // const activities = ActivityRegistry.list();
  
  const exampleActivities: ActivityMetadata[] = [
    {
      name: 'runBuild',
      displayName: 'Run package build (yarn build)',
      description: 'Executes yarn build in package directory and captures output',
      activityType: 'cli',
      version: '1.0.0',
      tags: ['build', 'cli', 'yarn'],
      capabilities: ['build-package'],
    },
    {
      name: 'runTests',
      displayName: 'Run package tests (yarn test)',
      description: 'Executes yarn test with coverage reporting',
      activityType: 'cli',
      version: '1.0.0',
      tags: ['test', 'cli', 'yarn'],
      capabilities: ['test-package'],
    },
    {
      name: 'spawnFixAgent',
      displayName: 'Fix test failures (claude-sonnet-4-5 | fix-agent-v1.2.0)',
      description: 'AI agent that analyzes and fixes failing tests, linting errors, and build issues',
      activityType: 'agentic',
      version: '1.2.0',
      modelProvider: 'anthropic',
      modelName: 'claude-sonnet-4-5-20250929',
      agentPromptId: 'fix-agent',
      capabilities: ['fix-failing-tests', 'fix-linting-errors', 'fix-build-errors'],
      tags: ['agent', 'fix', 'testing', 'quality'],
    },
  ];

  const result = await syncActivityRegistryToDatabase(
    exampleActivities,
    process.env.NEXT_PUBLIC_SUPABASE_URL!,
    process.env.SUPABASE_SERVICE_KEY!,
    process.env.ADMIN_USER_ID! // Would be passed in
  );

  console.log('Sync complete:', result);
  return result;
}

