#!/usr/bin/env bun

/**
 * add-frontmatter.ts
 *
 * Adds Docusaurus frontmatter to all markdown files that don't already have it.
 *
 * Frontmatter format:
 * ---
 * id: filename-without-extension
 * title: Human Readable Title
 * sidebar_label: Short Label
 * ---
 */

import { readdirSync, readFileSync, writeFileSync, statSync } from 'fs';
import { join, relative, basename, extname } from 'path';

interface FrontmatterData {
  id: string;
  title: string;
  sidebar_label: string;
}

// Colors for console output
const colors = {
  reset: '\x1b[0m',
  green: '\x1b[32m',
  blue: '\x1b[34m',
  yellow: '\x1b[33m',
  red: '\x1b[31m',
  cyan: '\x1b[36m',
};

const DOCS_DIR = join(import.meta.dir, '../docs');

/**
 * Check if a file already has frontmatter
 */
function hasFrontmatter(content: string): boolean {
  return content.trimStart().startsWith('---');
}

/**
 * Extract the first heading from markdown content
 */
function extractTitle(content: string): string | null {
  const lines = content.split('\n');

  for (const line of lines) {
    // Skip frontmatter
    if (line.trim() === '---') continue;

    // Look for # Title or ## Title
    const match = line.match(/^#{1,2}\s+(.+)$/);
    if (match) {
      return match[1].trim();
    }
  }

  return null;
}

/**
 * Convert filename to readable title
 * Examples:
 *   getting-started.md -> Getting Started
 *   api-reference.md -> API Reference
 *   mcp-integration.md -> MCP Integration
 */
function filenameToTitle(filename: string): string {
  const name = basename(filename, extname(filename));

  return name
    .split(/[-_]/)
    .map(word => {
      // Keep acronyms uppercase
      if (word.length <= 3 && word.toUpperCase() === word) {
        return word.toUpperCase();
      }
      // Special cases
      if (word.toLowerCase() === 'mcp') return 'MCP';
      if (word.toLowerCase() === 'api') return 'API';
      if (word.toLowerCase() === 'cli') return 'CLI';
      if (word.toLowerCase() === 'tui') return 'TUI';
      if (word.toLowerCase() === 'oauth') return 'OAuth';
      if (word.toLowerCase() === 'json') return 'JSON';
      if (word.toLowerCase() === 'yaml') return 'YAML';
      if (word.toLowerCase() === 'toml') return 'TOML';
      if (word.toLowerCase() === 'http') return 'HTTP';
      if (word.toLowerCase() === 'https') return 'HTTPS';
      if (word.toLowerCase() === 'url') return 'URL';
      if (word.toLowerCase() === 'ui') return 'UI';
      if (word.toLowerCase() === 'ux') return 'UX';
      if (word.toLowerCase() === 'adr') return 'ADR';

      // Capitalize first letter
      return word.charAt(0).toUpperCase() + word.slice(1).toLowerCase();
    })
    .join(' ');
}

/**
 * Generate frontmatter for a file
 */
function generateFrontmatter(filePath: string, content: string): FrontmatterData {
  const filename = basename(filePath, '.md');
  const title = extractTitle(content) || filenameToTitle(filename);

  // Generate a shorter sidebar label if the title is long
  let sidebarLabel = title;
  if (title.length > 25) {
    // Try to shorten by removing common words
    sidebarLabel = title
      .replace(/\bConfiguration\b/g, 'Config')
      .replace(/\bDocumentation\b/g, 'Docs')
      .replace(/\bIntroduction\b/g, 'Intro')
      .replace(/\bReference\b/g, 'Ref');

    // If still too long, truncate
    if (sidebarLabel.length > 30) {
      sidebarLabel = sidebarLabel.substring(0, 27) + '...';
    }
  }

  return {
    id: filename,
    title,
    sidebar_label: sidebarLabel,
  };
}

/**
 * Add frontmatter to a markdown file
 */
function addFrontmatter(filePath: string): boolean {
  try {
    let content = readFileSync(filePath, 'utf-8');

    // Remove existing frontmatter if present
    if (hasFrontmatter(content)) {
      const lines = content.split('\n');
      let endIndex = -1;
      let foundFirstDelimiter = false;

      for (let i = 0; i < lines.length; i++) {
        if (lines[i].trim() === '---') {
          if (foundFirstDelimiter) {
            endIndex = i;
            break;
          }
          foundFirstDelimiter = true;
        }
      }

      if (endIndex > 0) {
        content = lines.slice(endIndex + 1).join('\n').trimStart();
      }
    }

    const frontmatter = generateFrontmatter(filePath, content);

    const newContent = `---
id: "${frontmatter.id}"
title: "${frontmatter.title}"
sidebar_label: "${frontmatter.sidebar_label}"
---

${content}`;

    writeFileSync(filePath, newContent, 'utf-8');
    return true;
  } catch (error) {
    console.error(`${colors.red}Error processing ${filePath}:${colors.reset}`, error);
    return false;
  }
}

/**
 * Recursively process all markdown files in a directory
 */
function processDirectory(dir: string, stats: { processed: number; skipped: number; total: number }) {
  const entries = readdirSync(dir);

  for (const entry of entries) {
    const fullPath = join(dir, entry);
    const stat = statSync(fullPath);

    if (stat.isDirectory()) {
      processDirectory(fullPath, stats);
    } else if (entry.endsWith('.md')) {
      stats.total++;
      const relativePath = relative(DOCS_DIR, fullPath);

      if (addFrontmatter(fullPath)) {
        stats.processed++;
        console.log(`${colors.green}✓${colors.reset} Updated frontmatter: ${colors.cyan}${relativePath}${colors.reset}`);
      } else {
        stats.skipped++;
        console.log(`${colors.red}✗${colors.reset} Failed to process: ${colors.cyan}${relativePath}${colors.reset}`);
      }
    }
  }
}

/**
 * Main execution
 */
function main() {
  console.log(`${colors.blue}=== Adding Frontmatter to Documentation ===${colors.reset}\n`);
  console.log(`Processing directory: ${colors.cyan}${DOCS_DIR}${colors.reset}\n`);

  const stats = {
    processed: 0,
    skipped: 0,
    total: 0,
  };

  processDirectory(DOCS_DIR, stats);

  console.log(`\n${colors.blue}=== Summary ===${colors.reset}`);
  console.log(`${colors.green}Processed: ${stats.processed}${colors.reset}`);
  console.log(`${colors.yellow}Skipped: ${stats.skipped}${colors.reset}`);
  console.log(`${colors.cyan}Total: ${stats.total}${colors.reset}`);

  if (stats.processed > 0) {
    console.log(`\n${colors.green}✓ Frontmatter updated successfully!${colors.reset}`);
    console.log(`\n${colors.yellow}Next steps:${colors.reset}`);
    console.log(`  1. Test the website: ${colors.blue}bun run website${colors.reset}`);
    console.log(`  2. Verify navigation and search functionality`);
  } else if (stats.skipped > 0) {
    console.log(`\n${colors.red}✗ Some files failed to process.${colors.reset}`);
  } else {
    console.log(`\n${colors.yellow}No markdown files found.${colors.reset}`);
  }
}

main();
