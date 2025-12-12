#!/usr/bin/env node
/**
 * Script to sync existing components to Supabase
 * 
 * Usage:
 *   tsx src/scripts/sync-components.ts
 */

import 'dotenv/config';
import { syncExistingComponents } from '../lib/registry-sync';

async function main() {
  console.log('🔄 Syncing components to database...\n');

  try {
    const result = await syncExistingComponents();

    console.log('\n✅ Sync complete!');
    console.log(`   Created: ${result.created}`);
    console.log(`   Skipped: ${result.skipped}`);
    
    if (result.errors.length > 0) {
      console.log(`\n⚠️  Errors (${result.errors.length}):`);
      result.errors.forEach(err => console.log(`   - ${err}`));
    }
  } catch (error: any) {
    console.error('\n❌ Sync failed:', error.message);
    process.exit(1);
  }
}

main();

