/**
 * AST Helper Utilities
 * Utilities for generating TypeScript code blocks
 */

import type { ImportStatement } from '../types';

/**
 * Generate indentation string
 */
export function indent(level: number): string {
  return '  '.repeat(level);
}

/**
 * Convert string to camelCase
 */
export function toCamelCase(str: string): string {
  return str
    .replace(/[^a-zA-Z0-9]+(.)/g, (_, chr) => chr.toUpperCase())
    .replace(/^[A-Z]/, chr => chr.toLowerCase());
}

/**
 * Convert string to PascalCase
 */
export function toPascalCase(str: string): string {
  return str
    .replace(/[^a-zA-Z0-9]+(.)/g, (_, chr) => chr.toUpperCase())
    .replace(/^[a-z]/, chr => chr.toUpperCase());
}

/**
 * Convert string to kebab-case
 */
export function toKebabCase(str: string): string {
  return str
    .replace(/([a-z])([A-Z])/g, '$1-$2')
    .replace(/[\s_]+/g, '-')
    .toLowerCase();
}

/**
 * Generate a safe variable name from a node ID
 */
export function generateVariableName(nodeId: string, prefix = 'result'): string {
  const safeName = nodeId.replace(/[^a-zA-Z0-9]/g, '_');
  return `${prefix}_${safeName}`;
}

/**
 * Format an import statement
 */
export function formatImport(imp: ImportStatement): string {
  switch (imp.type) {
    case 'default':
      return `import ${imp.items[0]} from '${imp.module}';`;

    case 'namespace':
      return `import * as ${imp.items[0]} from '${imp.module}';`;

    case 'named':
    default:
      if (imp.items.length === 0) {
        return `import '${imp.module}';`;
      }
      return `import { ${imp.items.join(', ')} } from '${imp.module}';`;
  }
}

/**
 * Deduplicate and merge import statements
 */
export function mergeImports(imports: string[]): string[] {
  const importMap = new Map<string, Set<string>>();
  const typeImportMap = new Map<string, Set<string>>();
  const namespaceImports = new Map<string, string>();

  for (const imp of imports) {
    // Match namespace imports (import * as foo from 'bar')
    const namespaceMatch = imp.match(/import\s+(?:(type)\s+)?\*\s+as\s+(\w+)\s+from\s+['"]([^'"]+)['"]/);
    if (namespaceMatch) {
      const [, isType, name, module] = namespaceMatch;
      const key = `${isType ? 'type:' : ''}${module}`;
      namespaceImports.set(key, imp);
      continue;
    }

    // Match named imports
    const match = imp.match(/import\s+(?:(type)\s+)?\{([^}]+)\}\s+from\s+['"]([^'"]+)['"]/);
    if (match) {
      const [, isType, items, module] = match;
      const targetMap = isType ? typeImportMap : importMap;

      if (!targetMap.has(module)) {
        targetMap.set(module, new Set());
      }

      const itemSet = targetMap.get(module)!;
      items.split(',').forEach(item => itemSet.add(item.trim()));
    }
  }

  const result: string[] = [];

  // Add namespace imports first
  result.push(...Array.from(namespaceImports.values()));

  // Add type imports
  for (const [module, items] of typeImportMap) {
    result.push(`import type { ${Array.from(items).join(', ')} } from '${module}';`);
  }

  // Add regular imports
  for (const [module, items] of importMap) {
    result.push(`import { ${Array.from(items).join(', ')} } from '${module}';`);
  }

  return result;
}

/**
 * Generate a TypeScript comment block
 */
export function generateComment(text: string, indentLevel = 0): string {
  const indentStr = indent(indentLevel);
  return `${indentStr}// ${text}`;
}

/**
 * Generate a JSDoc comment block
 */
export function generateJSDoc(text: string, indentLevel = 0): string {
  const indentStr = indent(indentLevel);
  const lines = text.split('\n');

  if (lines.length === 1) {
    return `${indentStr}/** ${text} */`;
  }

  return `${indentStr}/**\n${lines.map(line => `${indentStr} * ${line}`).join('\n')}\n${indentStr} */`;
}

/**
 * Parse Temporal duration string to milliseconds
 */
export function parseDuration(duration: string): number {
  const match = duration.match(/^(\d+)\s*(ms|s|m|h|d)$/i);
  if (!match) {
    throw new Error(`Invalid duration format: ${duration}`);
  }

  const value = parseInt(match[1], 10);
  const unit = match[2].toLowerCase();

  switch (unit) {
    case 'ms':
      return value;
    case 's':
      return value * 1000;
    case 'm':
      return value * 60 * 1000;
    case 'h':
      return value * 60 * 60 * 1000;
    case 'd':
      return value * 24 * 60 * 60 * 1000;
    default:
      throw new Error(`Unknown duration unit: ${unit}`);
  }
}

/**
 * Format a value as TypeScript code
 */
export function formatValue(value: any): string {
  if (value === null) {
    return 'null';
  }

  if (value === undefined) {
    return 'undefined';
  }

  if (typeof value === 'string') {
    return JSON.stringify(value);
  }

  if (typeof value === 'number' || typeof value === 'boolean') {
    return String(value);
  }

  if (Array.isArray(value)) {
    return `[${value.map(v => formatValue(v)).join(', ')}]`;
  }

  if (typeof value === 'object') {
    const entries = Object.entries(value).map(([k, v]) => `${k}: ${formatValue(v)}`);
    return `{ ${entries.join(', ')} }`;
  }

  return String(value);
}

/**
 * Generate function signature
 */
export function generateFunctionSignature(
  name: string,
  params: Array<{ name: string; type: string; optional?: boolean }>,
  returnType: string,
  isAsync = false
): string {
  const asyncKeyword = isAsync ? 'async ' : '';
  const paramList = params
    .map(p => `${p.name}${p.optional ? '?' : ''}: ${p.type}`)
    .join(', ');

  return `${asyncKeyword}function ${name}(${paramList}): ${returnType}`;
}

/**
 * Generate arrow function
 */
export function generateArrowFunction(
  params: Array<{ name: string; type?: string }>,
  body: string,
  isAsync = false
): string {
  const asyncKeyword = isAsync ? 'async ' : '';
  const paramList = params
    .map(p => p.type ? `${p.name}: ${p.type}` : p.name)
    .join(', ');

  return `${asyncKeyword}(${paramList}) => ${body}`;
}

/**
 * Wrap code in a try-catch block
 */
export function wrapInTryCatch(
  code: string,
  errorHandler: string,
  indentLevel = 0
): string {
  const indentStr = indent(indentLevel);

  return `${indentStr}try {
${code}
${indentStr}} catch (error) {
${indentStr}  ${errorHandler}
${indentStr}}`;
}

/**
 * Generate object literal code
 */
export function generateObject(
  properties: Record<string, any>,
  indentLevel = 0
): string {
  const indentStr = indent(indentLevel);
  const nextIndent = indent(indentLevel + 1);

  if (Object.keys(properties).length === 0) {
    return '{}';
  }

  const entries = Object.entries(properties)
    .map(([key, value]) => `${nextIndent}${key}: ${formatValue(value)}`)
    .join(',\n');

  return `{\n${entries}\n${indentStr}}`;
}

/**
 * Sanitize identifier to be a valid TypeScript identifier
 */
export function sanitizeIdentifier(identifier: string): string {
  // Replace invalid characters with underscore
  let sanitized = identifier.replace(/[^a-zA-Z0-9_$]/g, '_');

  // Ensure it doesn't start with a number
  if (/^[0-9]/.test(sanitized)) {
    sanitized = '_' + sanitized;
  }

  // Ensure it's not a reserved word
  const reservedWords = [
    'break', 'case', 'catch', 'class', 'const', 'continue', 'debugger',
    'default', 'delete', 'do', 'else', 'enum', 'export', 'extends',
    'false', 'finally', 'for', 'function', 'if', 'import', 'in',
    'instanceof', 'new', 'null', 'return', 'super', 'switch', 'this',
    'throw', 'true', 'try', 'typeof', 'var', 'void', 'while', 'with',
    'yield', 'let', 'static', 'implements', 'interface', 'package',
    'private', 'protected', 'public', 'async', 'await',
  ];

  if (reservedWords.includes(sanitized)) {
    sanitized = '_' + sanitized;
  }

  return sanitized;
}
