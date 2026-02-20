#!/usr/bin/env bun

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

    // Fix double backticks around <source> and similar patterns
    // This fixes cases where previous attempts created double backticks
    content = content.replace(/``<([^>]+)>``/g, (match, inner) => {
      replacements++;
      return `\`<${inner}>\``;
    });

    // Fix standalone <source>, <Client>, <server>, etc. that aren't already in code blocks
    const standalonePatterns = [
      'source',
      'Client',
      'server',
      'input',
      'output',
      'name',
      'id',
      'path',
      'url',
      'file',
      'value',
      'key',
    ];

    for (const pattern of standalonePatterns) {
      // Match <pattern> that's not already in backticks or code blocks
      const regex = new RegExp(`(?<![\`])(<${pattern}>)(?![\`])`, 'g');
      const before = content;
      content = content.replace(regex, (match) => {
        replacements++;
        return `\`${match}\``;
      });
    }

    // Fix angle brackets followed by numbers (e.g., <100ms, <1>, <7>)
    // Only replace if not already in backticks
    content = content.replace(/([^`\n])<(\d+)/g, (match, before, digit) => {
      // Don't replace if this looks like it's in a code block
      if (before === '`' || before === ' ' && match.includes('```')) {
        return match;
      }
      replacements++;
      return `${before}&lt;${digit}`;
    });

    // Fix at start of line
    content = content.replace(/^<(\d+)/gm, (match, digit) => {
      replacements++;
      return `&lt;${digit}`;
    });

    // Fix Rust type patterns like Arc<Mutex<>> that aren't in code blocks
    // First, let's find lines that have these patterns but aren't in code blocks
    const lines = content.split('\n');
    const fixedLines = lines.map(line => {
      // Skip if line is in a code block (starts with ``` or has 4+ spaces)
      if (line.trim().startsWith('```') || line.startsWith('    ')) {
        return line;
      }

      // Fix Arc<Mutex<>> and similar Rust patterns
      if (line.includes('Arc<') && !line.includes('`Arc<')) {
        replacements++;
        return line.replace(/Arc<([^>]+)>/g, '`Arc<$1>`');
      }

      return line;
    });
    content = fixedLines.join('\n');

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

function processDirectory(dirPath: string): void {
  const items = readdirSync(dirPath);

  for (const item of items) {
    const fullPath = join(dirPath, item);
    const stat = statSync(fullPath);

    if (stat.isDirectory()) {
      processDirectory(fullPath);
    } else if (item.endsWith('.md')) {
      fixMDXInFile(fullPath);
    }
  }
}

console.log('🔧 Fixing MDX compilation errors...\n');
processDirectory(DOCS_DIR);
console.log(`\n✅ Fixed ${totalReplacements} issues across ${filesFixed} files`);
