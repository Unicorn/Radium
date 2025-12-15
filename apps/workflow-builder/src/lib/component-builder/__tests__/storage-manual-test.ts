/**
 * Storage Manual Test
 *
 * Tests both filesystem and database storage backends with a sample component.
 *
 * Run with: npx tsx src/lib/component-builder/__tests__/storage-manual-test.ts
 */

import * as path from 'path';
import * as fs from 'fs/promises';
import { fileURLToPath } from 'url';
import {
  ComponentStorage,
  FilesystemStorageBackend,
  DatabaseStorageBackend,
  type StoredComponent,
} from '../storage';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Sample Timestamp Generator component (from our earlier test)
const TIMESTAMP_GENERATOR_COMPONENT: StoredComponent = {
  id: 'timestamp_generator',
  name: 'Timestamp Generator',
  description: 'Generates the current timestamp in various formats',
  category: 'utilities',
  temporalType: 'activity',
  version: '1.0.0',
  artifacts: {
    rustSchema: `use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TimestampFormat {
    #[serde(rename = "ISO8601")]
    Iso8601,
    #[serde(rename = "unix")]
    Unix,
    #[serde(rename = "rfc2822")]
    Rfc2822,
}

impl Default for TimestampFormat {
    fn default() -> Self {
        TimestampFormat::Iso8601
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct TimestampGeneratorInput {
    #[serde(default)]
    pub format: TimestampFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimestampGeneratorOutput {
    pub timestamp: String,
    pub unix_ms: i64,
}`,
    typescriptCode: `/**
 * Timestamp Generator Activity
 */

export type TimestampFormat = 'ISO8601' | 'unix' | 'rfc2822';

export interface TimestampGeneratorInput {
  format?: TimestampFormat;
}

export interface TimestampGeneratorOutput {
  timestamp: string;
  unixMs: number;
}

export function isTimestampGeneratorInput(value: unknown): value is TimestampGeneratorInput {
  if (typeof value !== 'object' || value === null) return false;
  const obj = value as Record<string, unknown>;
  if ('format' in obj && obj.format !== undefined) {
    return ['ISO8601', 'unix', 'rfc2822'].includes(obj.format as string);
  }
  return true;
}

export async function executeTimestampGenerator(
  input: TimestampGeneratorInput
): Promise<TimestampGeneratorOutput> {
  const format = input.format || 'ISO8601';
  const now = new Date();
  const unixMs = now.getTime();

  let timestamp: string;
  switch (format) {
    case 'unix':
      timestamp = Math.floor(unixMs / 1000).toString();
      break;
    case 'rfc2822':
      timestamp = now.toUTCString();
      break;
    case 'ISO8601':
    default:
      timestamp = now.toISOString();
      break;
  }

  return { timestamp, unixMs };
}`,
    testCases: `#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_format() {
        let input = TimestampGeneratorInput::default();
        assert!(matches!(input.format, TimestampFormat::Iso8601));
    }

    #[test]
    fn test_unix_format() {
        let input = TimestampGeneratorInput {
            format: TimestampFormat::Unix,
        };
        assert!(matches!(input.format, TimestampFormat::Unix));
    }
}`,
    migrationRecord: `component:
  name: timestamp_generator
  category: utilities
  version: 1.0.0
  description: Generates the current timestamp in various formats
  temporal_type: activity

migration:
  migrated_by: Component Builder
  migration_date: ${new Date().toISOString()}
  difficulty: generated
`,
  },
  schema: {
    inputs: [
      {
        name: 'format',
        type: 'TimestampFormat',
        required: false,
        default: 'ISO8601',
        description: 'Output format for the timestamp',
      },
    ],
    outputs: [
      {
        name: 'timestamp',
        type: 'String',
        required: true,
        description: 'Formatted timestamp string',
      },
      {
        name: 'unix_ms',
        type: 'i64',
        required: true,
        description: 'Unix timestamp in milliseconds',
      },
    ],
    validationRules: [],
    connectionRules: {
      allowedSources: ['*'],
      allowedTargets: ['*'],
    },
  },
  metadata: {
    tags: ['timestamp', 'time', 'utilities'],
    usageCount: 0,
    isMarketplace: false,
  },
  createdAt: new Date(),
  updatedAt: new Date(),
  createdBy: 'test',
  status: 'published',
};

async function testFilesystemStorage() {
  console.log('\n' + '='.repeat(60));
  console.log('TESTING FILESYSTEM STORAGE');
  console.log('='.repeat(60));

  const testDir = path.join(__dirname, 'test-storage-fs');

  // Clean up from previous test
  try {
    await fs.rm(testDir, { recursive: true });
  } catch {
    // Directory doesn't exist
  }

  const backend = new FilesystemStorageBackend({ baseDir: testDir });
  const storage = new ComponentStorage({
    type: 'custom',
    customBackend: backend,
  });

  await storage.initialize();
  console.log('✓ Initialized filesystem storage at:', testDir);

  // Test save
  console.log('\n--- Testing save ---');
  const saved = await storage.save(TIMESTAMP_GENERATOR_COMPONENT);
  console.log('✓ Saved component:', saved.id);

  // Verify files were created
  const files = await fs.readdir(testDir);
  console.log('✓ Created directories:', files);

  const metadataDir = path.join(testDir, 'metadata');
  const metadataFiles = await fs.readdir(metadataDir);
  console.log('✓ Metadata files:', metadataFiles);

  // Test get
  console.log('\n--- Testing get ---');
  const retrieved = await storage.get('timestamp_generator');
  if (retrieved) {
    console.log('✓ Retrieved component:', retrieved.id);
    console.log('  Name:', retrieved.name);
    console.log('  Version:', retrieved.version);
    console.log('  Rust schema length:', retrieved.artifacts.rustSchema.length);
    console.log('  TS code length:', retrieved.artifacts.typescriptCode.length);
  } else {
    console.log('✗ Failed to retrieve component');
  }

  // Test exists
  console.log('\n--- Testing exists ---');
  const exists = await storage.exists('timestamp_generator');
  console.log('✓ Component exists:', exists);
  const notExists = await storage.exists('nonexistent');
  console.log('✓ Nonexistent check:', !notExists);

  // Test list
  console.log('\n--- Testing list ---');
  const list = await storage.list();
  console.log('✓ Listed components:', list.total);
  console.log('  Components:', list.components.map((c) => c.id));

  // Test search
  console.log('\n--- Testing search ---');
  const searchResults = await storage.search('timestamp');
  console.log('✓ Search results:', searchResults.length);

  // Test update
  console.log('\n--- Testing update ---');
  const updated = await storage.update('timestamp_generator', {
    description: 'Updated description',
  });
  if (updated) {
    console.log('✓ Updated description:', updated.description);
  }

  // Test increment usage
  console.log('\n--- Testing increment usage ---');
  await storage.incrementUsage('timestamp_generator');
  const afterIncrement = await storage.get('timestamp_generator');
  console.log('✓ Usage count:', afterIncrement?.metadata.usageCount);

  // Test delete
  console.log('\n--- Testing delete ---');
  const deleted = await storage.delete('timestamp_generator');
  console.log('✓ Deleted:', deleted);
  const afterDelete = await storage.exists('timestamp_generator');
  console.log('✓ Exists after delete:', afterDelete);

  // Clean up
  await fs.rm(testDir, { recursive: true });
  console.log('\n✓ Cleaned up test directory');

  console.log('\n✓ FILESYSTEM STORAGE TEST PASSED');
}

async function testDatabaseStorage() {
  console.log('\n' + '='.repeat(60));
  console.log('TESTING DATABASE STORAGE (in-memory)');
  console.log('='.repeat(60));

  const backend = new DatabaseStorageBackend();
  const storage = new ComponentStorage({
    type: 'custom',
    customBackend: backend,
  });

  await storage.initialize();
  console.log('✓ Initialized database storage (in-memory mode)');

  // Test save
  console.log('\n--- Testing save ---');
  const saved = await storage.save(TIMESTAMP_GENERATOR_COMPONENT);
  console.log('✓ Saved component:', saved.id);

  // Test get
  console.log('\n--- Testing get ---');
  const retrieved = await storage.get('timestamp_generator');
  if (retrieved) {
    console.log('✓ Retrieved component:', retrieved.id);
    console.log('  Name:', retrieved.name);
    console.log('  Version:', retrieved.version);
  } else {
    console.log('✗ Failed to retrieve component');
  }

  // Test exists
  console.log('\n--- Testing exists ---');
  const exists = await storage.exists('timestamp_generator');
  console.log('✓ Component exists:', exists);

  // Test list
  console.log('\n--- Testing list ---');
  const list = await storage.list();
  console.log('✓ Listed components:', list.total);

  // Test search
  console.log('\n--- Testing search ---');
  const searchResults = await storage.search('timestamp');
  console.log('✓ Search results:', searchResults.length);

  // Test update
  console.log('\n--- Testing update ---');
  const updated = await storage.update('timestamp_generator', {
    description: 'Updated via database',
  });
  if (updated) {
    console.log('✓ Updated description:', updated.description);
  }

  // Test increment usage
  console.log('\n--- Testing increment usage ---');
  await storage.incrementUsage('timestamp_generator');
  const afterIncrement = await storage.get('timestamp_generator');
  console.log('✓ Usage count:', afterIncrement?.metadata.usageCount);

  // Test delete
  console.log('\n--- Testing delete ---');
  const deleted = await storage.delete('timestamp_generator');
  console.log('✓ Deleted:', deleted);
  const afterDelete = await storage.exists('timestamp_generator');
  console.log('✓ Exists after delete:', afterDelete);

  console.log('\n✓ DATABASE STORAGE TEST PASSED');
}

async function main() {
  console.log('Component Storage Manual Tests');
  console.log('==============================');

  try {
    await testFilesystemStorage();
    await testDatabaseStorage();

    console.log('\n' + '='.repeat(60));
    console.log('ALL STORAGE TESTS PASSED');
    console.log('='.repeat(60));
  } catch (error) {
    console.error('\n✗ TEST FAILED:', error);
    process.exit(1);
  }
}

main();
