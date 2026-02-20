#!/usr/bin/env node

/**
 * Dead Link Checker for Docusaurus Website
 * 
 * Scans all markdown files in the website/docs directory for dead links:
 * - Internal links (relative paths to other docs)
 * - External links (HTTP/HTTPS URLs)
 * 
 * Usage: npx tsx scripts/check-dead-links.ts [--fix]
 */

import * as fs from 'fs';
import * as path from 'path';
import { promisify } from 'util';

const readdir = promisify(fs.readdir);
const readFile = promisify(fs.readFile);
const stat = promisify(fs.stat);
const writeFile = promisify(fs.writeFile);

interface LinkIssue {
  file: string;
  line: number;
  link: string;
  type: 'internal' | 'external';
  issue: 'broken' | 'missing-anchor' | 'external-error';
  message: string;
}

interface LinkResult {
  issues: LinkIssue[];
  totalLinks: number;
  brokenLinks: number;
}

const DOCS_DIR = path.join(__dirname, '..', 'docs');
const WEBSITE_ROOT = path.join(__dirname, '..');

// Patterns for extracting links from markdown
const MARKDOWN_LINK_PATTERN = /\[([^\]]+)\]\(([^)]+)\)/g;
const HTML_LINK_PATTERN = /<a[^>]+href=["']([^"']+)["'][^>]*>/gi;
const REFERENCE_LINK_PATTERN = /\[([^\]]+)\]:\s*(.+)/g;

// Get all markdown files recursively
async function getAllMarkdownFiles(dir: string): Promise<string[]> {
  const files: string[] = [];
  
  async function walkDir(currentDir: string) {
    const entries = await readdir(currentDir, { withFileTypes: true });
    
    for (const entry of entries) {
      const fullPath = path.join(currentDir, entry.name);
      
      if (entry.isDirectory()) {
        await walkDir(fullPath);
      } else if (entry.isFile() && entry.name.endsWith('.md')) {
        files.push(fullPath);
      }
    }
  }
  
  await walkDir(dir);
  return files;
}

// Extract all links from a markdown file
function extractLinks(content: string): Array<{ text: string; url: string; line: number }> {
  const links: Array<{ text: string; url: string; line: number }> = [];
  const lines = content.split('\n');
  
  lines.forEach((line, index) => {
    // Markdown links [text](url)
    let match;
    while ((match = MARKDOWN_LINK_PATTERN.exec(line)) !== null) {
      links.push({
        text: match[1],
        url: match[2],
        line: index + 1,
      });
    }
    
    // HTML links <a href="url">
    while ((match = HTML_LINK_PATTERN.exec(line)) !== null) {
      links.push({
        text: '',
        url: match[1],
        line: index + 1,
      });
    }
  });
  
  return links;
}

// Resolve internal link path
async function resolveInternalPath(fromFile: string, linkPath: string): Promise<string> {
  // Remove anchor if present
  const [pathPart] = linkPath.split('#');
  
  // Handle absolute paths from root
  if (pathPart.startsWith('/')) {
    return path.join(DOCS_DIR, pathPart.slice(1));
  }
  
  // Handle relative paths
  const fromDir = path.dirname(fromFile);
  const resolved = path.resolve(fromDir, pathPart);
  
  // If it's a directory, try index.md
  // But if the original link ends with /, it's a directory link which is valid in Docusaurus
  if (fs.existsSync(resolved)) {
    const stats = await stat(resolved);
    if (stats.isDirectory()) {
      // Directory links ending with / are valid in Docusaurus (they link to category pages)
      if (linkPath.endsWith('/')) {
        return resolved; // Return directory path as-is for directory links
      }
      return path.join(resolved, 'index.md');
    }
  }
  
  // Try with .md extension if not present
  if (!resolved.endsWith('.md')) {
    const withExt = resolved + '.md';
    if (fs.existsSync(withExt)) {
      return withExt;
    }
  }
  
  return resolved;
}

// Check if internal link is valid
async function checkInternalLink(
  fromFile: string,
  linkPath: string
): Promise<{ valid: boolean; message: string }> {
  try {
    const resolvedPath = await resolveInternalPath(fromFile, linkPath);
    
    // Check if file or directory exists
    if (!fs.existsSync(resolvedPath)) {
      return { valid: false, message: `File not found: ${resolvedPath}` };
    }
    
    const stats = await stat(resolvedPath);
    // Directory links are valid in Docusaurus
    if (stats.isDirectory() && linkPath.endsWith('/')) {
      return { valid: true, message: 'OK (directory link)' };
    }
    if (!stats.isFile()) {
      return { valid: false, message: `Path is not a file: ${resolvedPath}` };
    }
    
    // Check anchor if present
    const [pathPart, anchor] = linkPath.split('#');
    if (anchor) {
      const content = await readFile(resolvedPath, 'utf-8');
      // Check for markdown heading anchors (Docusaurus auto-generates these)
      // Format: # Heading becomes heading
      const headingPattern = new RegExp(`^#+\\s+${anchor.replace(/-/g, '[- ]')}`, 'mi');
      const idPattern = new RegExp(`id:\\s*["']?${anchor}["']?`, 'i');
      
      if (!headingPattern.test(content) && !idPattern.test(content)) {
        // Check for auto-generated anchor format (lowercase, spaces to dashes)
        const normalizedAnchor = anchor.toLowerCase().replace(/\s+/g, '-');
        const normalizedHeadings = content.match(/^#+\s+(.+)$/gm) || [];
        const headingAnchors = normalizedHeadings.map(h => {
          const text = h.replace(/^#+\s+/, '').toLowerCase()
            .replace(/[^\w\s-]/g, '')
            .replace(/\s+/g, '-');
          return text;
        });
        
        if (!headingAnchors.includes(normalizedAnchor)) {
          return { valid: false, message: `Anchor not found: #${anchor}` };
        }
      }
    }
    
    return { valid: true, message: 'OK' };
  } catch (error) {
    return { valid: false, message: `Error: ${error.message}` };
  }
}

// Check if external link is valid (with timeout)
async function checkExternalLink(url: string): Promise<{ valid: boolean; message: string }> {
  // Skip non-HTTP(S) URLs
  if (!url.startsWith('http://') && !url.startsWith('https://')) {
    return { valid: true, message: 'Skipped (not HTTP/HTTPS)' };
  }
  
  try {
    const controller = new AbortController();
    const timeout = setTimeout(() => controller.abort(), 10000); // 10 second timeout
    
    const response = await fetch(url, {
      method: 'HEAD',
      signal: controller.signal,
      redirect: 'follow',
    });
    
    clearTimeout(timeout);
    
    if (response.ok || response.status === 301 || response.status === 302) {
      return { valid: true, message: 'OK' };
    } else {
      return { valid: false, message: `HTTP ${response.status}` };
    }
  } catch (error) {
    if (error.name === 'AbortError') {
      return { valid: false, message: 'Timeout' };
    }
    return { valid: false, message: `Error: ${error.message}` };
  }
}

// Check all links in a file
async function checkFileLinks(filePath: string): Promise<LinkIssue[]> {
  const issues: LinkIssue[] = [];
  const content = await readFile(filePath, 'utf-8');
  const links = extractLinks(content);
  
  for (const link of links) {
    const { url } = link;
    
    // Skip empty links
    if (!url || url.trim() === '') {
      continue;
    }
    
    // Skip mailto: and other non-documentation links
    if (url.startsWith('mailto:') || url.startsWith('tel:') || url.startsWith('javascript:')) {
      continue;
    }
    
    // Check if it's an internal or external link
    if (url.startsWith('http://') || url.startsWith('https://')) {
      // External link
      const result = await checkExternalLink(url);
      if (!result.valid) {
        issues.push({
          file: path.relative(WEBSITE_ROOT, filePath),
          line: link.line,
          link: url,
          type: 'external',
          issue: 'external-error',
          message: result.message,
        });
      }
    } else if (url.startsWith('/api/')) {
      // Skip API links - these are valid static files served at runtime
      continue;
    } else if (!url.startsWith('#')) {
      // Internal link (relative path)
      const result = await checkInternalLink(filePath, url);
      if (!result.valid) {
        issues.push({
          file: path.relative(WEBSITE_ROOT, filePath),
          line: link.line,
          link: url,
          type: 'internal',
          issue: result.message.includes('Anchor') ? 'missing-anchor' : 'broken',
          message: result.message,
        });
      }
    }
    // Skip anchor-only links (#section) as they're handled by Docusaurus
  }
  
  return issues;
}

// Main function
async function main() {
  const args = process.argv.slice(2);
  const shouldFix = args.includes('--fix');
  
  console.log('🔍 Scanning for dead links...\n');
  
  const markdownFiles = await getAllMarkdownFiles(DOCS_DIR);
  console.log(`Found ${markdownFiles.length} markdown files\n`);
  
  const allIssues: LinkIssue[] = [];
  let totalLinks = 0;
  
  for (const file of markdownFiles) {
    const issues = await checkFileLinks(file);
    allIssues.push(...issues);
    
    const content = await readFile(file, 'utf-8');
    const links = extractLinks(content);
    totalLinks += links.length;
    
    if (issues.length > 0) {
      console.log(`❌ ${path.relative(WEBSITE_ROOT, file)}: ${issues.length} issue(s)`);
    }
  }
  
  console.log(`\n📊 Summary:`);
  console.log(`   Total links checked: ${totalLinks}`);
  console.log(`   Broken links found: ${allIssues.length}\n`);
  
  if (allIssues.length === 0) {
    console.log('✅ No dead links found!');
    process.exit(0);
  }
  
  // Group issues by file
  const issuesByFile = new Map<string, LinkIssue[]>();
  for (const issue of allIssues) {
    if (!issuesByFile.has(issue.file)) {
      issuesByFile.set(issue.file, []);
    }
    issuesByFile.get(issue.file)!.push(issue);
  }
  
  // Print detailed report
  console.log('📋 Dead Links Report:\n');
  for (const [file, issues] of issuesByFile.entries()) {
    console.log(`\n📄 ${file}:`);
    for (const issue of issues) {
      console.log(`   Line ${issue.line}: ${issue.link}`);
      console.log(`   └─ ${issue.message}`);
    }
  }
  
  if (shouldFix) {
    console.log('\n🔧 Fixing dead links...\n');
    await fixDeadLinks(issuesByFile);
  } else {
    console.log('\n💡 Run with --fix to automatically remove dead links');
  }
  
  process.exit(allIssues.length > 0 ? 1 : 0);
}

// Fix dead links by removing them
async function fixDeadLinks(issuesByFile: Map<string, LinkIssue[]>) {
  for (const [file, issues] of issuesByFile.entries()) {
    const filePath = path.join(WEBSITE_ROOT, file);
    let content = await readFile(filePath, 'utf-8');
    const lines = content.split('\n');
    
    // Sort issues by line number (descending) to avoid line number shifts
    const sortedIssues = [...issues].sort((a, b) => b.line - a.line);
    
    for (const issue of sortedIssues) {
      const lineIndex = issue.line - 1;
      const line = lines[lineIndex];
      
      if (line) {
        // Remove the link but keep the text if it's a markdown link
        const markdownLinkMatch = line.match(/\[([^\]]+)\]\(([^)]+)\)/);
        if (markdownLinkMatch && markdownLinkMatch[2] === issue.link) {
          // Replace [text](url) with just text
          lines[lineIndex] = line.replace(/\[([^\]]+)\]\([^)]+\)/, '$1');
          console.log(`   Fixed line ${issue.line} in ${file}: Removed link, kept text`);
        } else {
          // Remove the entire line if it's just a link
          if (line.trim().match(/^[-*]\s*\[.*\]\(.*\)$/)) {
            lines.splice(lineIndex, 1);
            console.log(`   Fixed line ${issue.line} in ${file}: Removed link line`);
          } else {
            // Try to remove just the link part
            lines[lineIndex] = line.replace(/\[([^\]]+)\]\([^)]+\)/g, '$1');
            console.log(`   Fixed line ${issue.line} in ${file}: Removed link from line`);
          }
        }
      }
    }
    
    content = lines.join('\n');
    await writeFile(filePath, content, 'utf-8');
  }
  
  console.log('\n✅ Fixed dead links');
}

// Run the script
main().catch((error) => {
  console.error('Error:', error);
  process.exit(1);
});

