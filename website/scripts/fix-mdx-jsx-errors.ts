#!/usr/bin/env bun

/**
 * fix-mdx-jsx-errors.ts
 *
 * Fixes MDX/JSX compilation errors by escaping angle brackets that MDX
 * interprets as tags but are actually text content.
 */

import { readdirSync, readFileSync, writeFileSync, statSync } from 'fs';
import { join } from 'path';

const DOCS_DIR = './docs';

let filesFixed = 0;
let totalReplacements = 0;

function fixMDXInFile(filePath: string): boolean {
  try {
    let content = readFileSync(filePath, 'utf-8');
    const originalContent = content;
    let replacements = 0;

    // Fix angle brackets followed by numbers (e.g., <100ms, <1>, <7>)
    // Replace with HTML entities
    content = content.replace(/([^`])<(\d)/g, (match, before, digit) => {
      replacements++;
      return `${before}&lt;${digit}`;
    });

    // Fix at start of line
    content = content.replace(/^<(\d)/gm, (match, digit) => {
      replacements++;
      return `&lt;${digit}`;
    });

    // Fix specific tags that should be code
    const patterns = [
      { from: /<source>/g, to: '`<source>`' },
      { from: /<Client>/g, to: '`<Client>`' },
      { from: /Arc<Mutex<>>/g, to: '`Arc<Mutex<>>`' },
    ];

    for (const { from, to } of patterns) {
      const before = content;
      content = content.replace(from, to);
      if (before !== content) {
        replacements++;
      }
    }

    if (content !== originalContent) {
      writeFileSync(filePath, content, 'utf-8');
      filesFixed++;
      totalReplacements += replacements;
      console.log(`✓ Fixed ${replacements} issues in: ${filePath.replace(DOCS_DIR + '/', '')}`);
      return true;
    }

    return false;
  } catch (error) {
    console.error(`✗ Error processing ${filePath}:`, error);
    return false;
  }
}

function processDirectory(dir: string) {
  const entries = readdirSync(dir);

  for (const entry of entries) {
    const fullPath = join(dir, entry);
    const stat = statSync(fullPath);

    if (stat.isDirectory()) {
      processDirectory(fullPath);
    } else if (entry.endsWith('.md')) {
      fixMDXInFile(fullPath);
    }
  }
}

console.log('Fixing MDX/JSX compilation errors...\n');
processDirectory(DOCS_DIR);
console.log(`\n✓ Fixed ${filesFixed} files (${totalReplacements} total replacements)`);
