/**
 * Component Builder Performance Tests
 *
 * Benchmarks key operations to ensure acceptable performance.
 *
 * Run with: npx vitest run src/lib/component-builder/__tests__/performance.test.ts
 */

import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import * as fs from 'fs/promises';
import * as path from 'path';
import { fileURLToPath } from 'url';
import {
  ComponentStorage,
  FilesystemStorageBackend,
  DatabaseStorageBackend,
  type StoredComponent,
} from '../storage';
import { ComponentTemplateLibrary } from '../templates/library';
import { MigrationRecordProcessor } from '../knowledge-base/processor';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Performance thresholds (in milliseconds)
const THRESHOLDS = {
  storageSave: 50,
  storageGet: 10,
  storageList: 100,
  storageSearch: 50,
  templateSearch: 20,
  templateGet: 5,
  knowledgeProcess: 500,
};

// Sample component for testing
function createTestComponent(id: string): StoredComponent {
  return {
    id,
    name: `Test Component ${id}`,
    description: `A test component for performance benchmarking (${id})`,
    category: 'utilities',
    temporalType: 'activity',
    version: '1.0.0',
    artifacts: {
      rustSchema: `use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ${id.replace(/-/g, '_')}Input {
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ${id.replace(/-/g, '_')}Output {
    pub result: String,
}`,
      typescriptCode: `export interface ${id}Input {
  value: string;
}

export interface ${id}Output {
  result: string;
}

export async function execute${id}(input: ${id}Input): Promise<${id}Output> {
  return { result: input.value };
}`,
      testCases: `#[test]
fn test_${id.replace(/-/g, '_')}() {
    assert!(true);
}`,
      migrationRecord: `component:
  name: ${id}
  version: 1.0.0
`,
    },
    schema: {
      inputs: [{ name: 'value', type: 'String', required: true, description: 'Input value' }],
      outputs: [{ name: 'result', type: 'String', required: true, description: 'Output result' }],
      validationRules: [],
      connectionRules: { allowedSources: ['*'], allowedTargets: ['*'] },
    },
    metadata: {
      tags: ['test', 'performance', 'benchmark'],
      usageCount: 0,
      isMarketplace: false,
    },
    createdAt: new Date(),
    updatedAt: new Date(),
    createdBy: 'performance-test',
    status: 'published',
  };
}

// Utility to measure execution time
async function measureTime<T>(
  fn: () => Promise<T>
): Promise<{ result: T; duration: number }> {
  const start = performance.now();
  const result = await fn();
  const duration = performance.now() - start;
  return { result, duration };
}

// Utility to run multiple iterations and get average
async function benchmark<T>(
  name: string,
  fn: () => Promise<T>,
  iterations: number = 10
): Promise<{ avg: number; min: number; max: number; results: T[] }> {
  const durations: number[] = [];
  const results: T[] = [];

  for (let i = 0; i < iterations; i++) {
    const { result, duration } = await measureTime(fn);
    durations.push(duration);
    results.push(result);
  }

  const avg = durations.reduce((a, b) => a + b, 0) / durations.length;
  const min = Math.min(...durations);
  const max = Math.max(...durations);

  console.log(
    `  ${name}: avg=${avg.toFixed(2)}ms, min=${min.toFixed(2)}ms, max=${max.toFixed(2)}ms`
  );

  return { avg, min, max, results };
}

describe('Storage Performance', () => {
  let testDir: string;
  let filesystemBackend: FilesystemStorageBackend;
  let databaseBackend: DatabaseStorageBackend;
  let fsStorage: ComponentStorage;
  let dbStorage: ComponentStorage;

  beforeAll(async () => {
    testDir = path.join(__dirname, 'perf-test-storage');
    await fs.mkdir(testDir, { recursive: true });

    filesystemBackend = new FilesystemStorageBackend({ baseDir: testDir });
    databaseBackend = new DatabaseStorageBackend();

    fsStorage = new ComponentStorage({
      type: 'custom',
      customBackend: filesystemBackend,
    });
    dbStorage = new ComponentStorage({
      type: 'custom',
      customBackend: databaseBackend,
    });

    await fsStorage.initialize();
    await dbStorage.initialize();
  });

  afterAll(async () => {
    try {
      await fs.rm(testDir, { recursive: true });
    } catch {
      // Ignore cleanup errors
    }
  });

  describe('Filesystem Backend', () => {
    it('should save components within threshold', async () => {
      console.log('\n--- Filesystem Save Performance ---');
      const component = createTestComponent('perf-fs-save');

      const { avg } = await benchmark(
        'save',
        async () => fsStorage.save({ ...component, id: `perf-fs-${Date.now()}` }),
        20
      );

      expect(avg).toBeLessThan(THRESHOLDS.storageSave);
    });

    it('should retrieve components within threshold', async () => {
      console.log('\n--- Filesystem Get Performance ---');
      const component = createTestComponent('perf-fs-get');
      await fsStorage.save(component);

      const { avg } = await benchmark(
        'get',
        async () => fsStorage.get('perf-fs-get'),
        50
      );

      expect(avg).toBeLessThan(THRESHOLDS.storageGet);
    });

    it('should list components within threshold', async () => {
      console.log('\n--- Filesystem List Performance ---');
      // Add some components for listing
      for (let i = 0; i < 20; i++) {
        await fsStorage.save(createTestComponent(`perf-fs-list-${i}`));
      }

      const { avg } = await benchmark(
        'list',
        async () => fsStorage.list({ limit: 50 }),
        20
      );

      expect(avg).toBeLessThan(THRESHOLDS.storageList);
    });

    it('should search components within threshold', async () => {
      console.log('\n--- Filesystem Search Performance ---');

      const { avg } = await benchmark(
        'search',
        async () => fsStorage.search('performance'),
        20
      );

      expect(avg).toBeLessThan(THRESHOLDS.storageSearch);
    });
  });

  describe('Database Backend (In-Memory)', () => {
    it('should save components within threshold', async () => {
      console.log('\n--- Database Save Performance ---');
      const component = createTestComponent('perf-db-save');

      const { avg } = await benchmark(
        'save',
        async () => dbStorage.save({ ...component, id: `perf-db-${Date.now()}` }),
        50
      );

      // In-memory should be faster
      expect(avg).toBeLessThan(THRESHOLDS.storageSave / 2);
    });

    it('should retrieve components within threshold', async () => {
      console.log('\n--- Database Get Performance ---');
      const component = createTestComponent('perf-db-get');
      await dbStorage.save(component);

      const { avg } = await benchmark(
        'get',
        async () => dbStorage.get('perf-db-get'),
        100
      );

      // In-memory should be very fast
      expect(avg).toBeLessThan(THRESHOLDS.storageGet / 2);
    });

    it('should list components within threshold', async () => {
      console.log('\n--- Database List Performance ---');
      for (let i = 0; i < 50; i++) {
        await dbStorage.save(createTestComponent(`perf-db-list-${i}`));
      }

      const { avg } = await benchmark(
        'list',
        async () => dbStorage.list({ limit: 100 }),
        50
      );

      // In-memory should be faster
      expect(avg).toBeLessThan(THRESHOLDS.storageList / 2);
    });

    it('should search components within threshold', async () => {
      console.log('\n--- Database Search Performance ---');

      const { avg } = await benchmark(
        'search',
        async () => dbStorage.search('performance'),
        50
      );

      expect(avg).toBeLessThan(THRESHOLDS.storageSearch / 2);
    });
  });
});

describe('Template Library Performance', () => {
  let library: ComponentTemplateLibrary;

  beforeAll(() => {
    library = new ComponentTemplateLibrary();
  });

  it('should search templates within threshold', async () => {
    console.log('\n--- Template Search Performance ---');

    const { avg } = await benchmark(
      'search',
      async () => library.search('email'),
      100
    );

    expect(avg).toBeLessThan(THRESHOLDS.templateSearch);
  });

  it('should get template by ID within threshold', async () => {
    console.log('\n--- Template Get Performance ---');

    const { avg } = await benchmark(
      'get',
      async () => library.get('email-sender'),
      100
    );

    expect(avg).toBeLessThan(THRESHOLDS.templateGet);
  });

  it('should list templates by category within threshold', async () => {
    console.log('\n--- Template List By Category Performance ---');

    const { avg } = await benchmark(
      'getByCategory',
      async () => library.getByCategory('communication'),
      100
    );

    expect(avg).toBeLessThan(THRESHOLDS.templateSearch);
  });

  it('should get all templates within threshold', async () => {
    console.log('\n--- Template GetAll Performance ---');

    const { avg } = await benchmark(
      'getAll',
      async () => library.getAll(),
      100
    );

    expect(avg).toBeLessThan(THRESHOLDS.templateGet);
  });
});

describe('Knowledge Base Processing Performance', () => {
  it('should process migration records within threshold', async () => {
    console.log('\n--- Knowledge Base Processing Performance ---');

    const recordsDir = path.resolve(
      __dirname,
      '../../../../../../crates/radium-workflow/component-records'
    );

    // Check if records exist
    try {
      await fs.access(recordsDir);
    } catch {
      console.log('  Skipping: component-records directory not found');
      return;
    }

    const processor = new MigrationRecordProcessor({ recordsDir });

    const { avg, results } = await benchmark(
      'processAll',
      async () => processor.processAll(),
      5
    );

    console.log(`  Processed ${results[0]?.length ?? 0} records`);

    expect(avg).toBeLessThan(THRESHOLDS.knowledgeProcess);
  });
});

describe('Concurrent Operations Performance', () => {
  let storage: ComponentStorage;

  beforeAll(async () => {
    const backend = new DatabaseStorageBackend();
    storage = new ComponentStorage({
      type: 'custom',
      customBackend: backend,
    });
    await storage.initialize();
  });

  it('should handle concurrent saves efficiently', async () => {
    console.log('\n--- Concurrent Saves Performance ---');

    const concurrency = 10;
    const iterations = 5;

    const { avg } = await benchmark(
      `${concurrency} concurrent saves`,
      async () => {
        const promises = Array.from({ length: concurrency }, (_, i) =>
          storage.save(createTestComponent(`concurrent-${Date.now()}-${i}`))
        );
        return Promise.all(promises);
      },
      iterations
    );

    // Should complete all concurrent saves in reasonable time
    expect(avg).toBeLessThan(THRESHOLDS.storageSave * concurrency);
  });

  it('should handle concurrent reads efficiently', async () => {
    console.log('\n--- Concurrent Reads Performance ---');

    // Setup: save a component to read
    const component = createTestComponent('concurrent-read-target');
    await storage.save(component);

    const concurrency = 50;
    const iterations = 10;

    const { avg } = await benchmark(
      `${concurrency} concurrent reads`,
      async () => {
        const promises = Array.from({ length: concurrency }, () =>
          storage.get('concurrent-read-target')
        );
        return Promise.all(promises);
      },
      iterations
    );

    // Concurrent reads should still be fast
    expect(avg).toBeLessThan(THRESHOLDS.storageGet * 5);
  });

  it('should handle mixed read/write workload', async () => {
    console.log('\n--- Mixed Workload Performance ---');

    const iterations = 5;

    const { avg } = await benchmark(
      'mixed workload (5 writes, 10 reads, 2 searches)',
      async () => {
        const timestamp = Date.now();
        const writes = Array.from({ length: 5 }, (_, i) =>
          storage.save(createTestComponent(`mixed-${timestamp}-${i}`))
        );
        const reads = Array.from({ length: 10 }, () =>
          storage.list({ limit: 10 })
        );
        const searches = [
          storage.search('mixed'),
          storage.search('test'),
        ];

        return Promise.all([...writes, ...reads, ...searches]);
      },
      iterations
    );

    // Mixed workload should complete reasonably
    expect(avg).toBeLessThan(200);
  });
});

describe('Performance Summary', () => {
  it('should log performance summary', () => {
    console.log('\n');
    console.log('='.repeat(60));
    console.log('PERFORMANCE TEST SUMMARY');
    console.log('='.repeat(60));
    console.log('\nThresholds (ms):');
    Object.entries(THRESHOLDS).forEach(([key, value]) => {
      console.log(`  ${key}: ${value}ms`);
    });
    console.log('\nAll performance tests completed.');
    console.log('='.repeat(60));
  });
});
