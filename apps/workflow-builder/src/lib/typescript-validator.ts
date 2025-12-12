/**
 * TypeScript Validator
 * Validates TypeScript code using the TypeScript Compiler API
 */

import ts from 'typescript';

export interface ValidationResult {
  valid: boolean;
  errors: ValidationError[];
  warnings: ValidationWarning[];
}

export interface ValidationError {
  line: number;
  column: number;
  message: string;
  code: number;
}

export interface ValidationWarning {
  line: number;
  column: number;
  message: string;
  code: number;
}

/**
 * Validate TypeScript code with strict mode
 * Ensures code is syntactically valid and type-safe
 */
export function validateTypeScriptCode(
  code: string,
  options: {
    strict?: boolean;
    allowImplicitAny?: boolean;
    allowUnusedLocals?: boolean;
  } = {}
): ValidationResult {
  const errors: ValidationError[] = [];
  const warnings: ValidationWarning[] = [];

  // Default to strict mode
  const compilerOptions: ts.CompilerOptions = {
    strict: options.strict !== false,
    noImplicitAny: !options.allowImplicitAny,
    noUnusedLocals: options.allowUnusedLocals !== false,
    target: ts.ScriptTarget.ES2020,
    module: ts.ModuleKind.ESNext,
    moduleResolution: ts.ModuleResolutionKind.NodeNext,
    esModuleInterop: true,
    skipLibCheck: true,
    lib: ['ES2020'],
    types: ['node'],
  };

  // Create a virtual source file
  const fileName = 'activity.ts';
  const sourceFile = ts.createSourceFile(
    fileName,
    code,
    ts.ScriptTarget.ES2020,
    true
  );

  // Create a program with just this one file
  const host: ts.CompilerHost = {
    getSourceFile: (name) => {
      if (name === fileName) {
        return sourceFile;
      }
      // For lib files and type definitions, return undefined
      // This is a simplified validator that doesn't resolve external types
      return undefined;
    },
    getDefaultLibFileName: () => 'lib.d.ts',
    writeFile: () => {},
    getCurrentDirectory: () => '',
    getCanonicalFileName: (name) => name,
    useCaseSensitiveFileNames: () => true,
    getNewLine: () => '\n',
    fileExists: (name) => name === fileName,
    readFile: (name) => (name === fileName ? code : undefined),
  };

  const program = ts.createProgram([fileName], compilerOptions, host);
  const diagnostics = ts.getPreEmitDiagnostics(program);

  // Process diagnostics
  for (const diagnostic of diagnostics) {
    if (diagnostic.file && diagnostic.start !== undefined) {
      const { line, character } = diagnostic.file.getLineAndCharacterOfPosition(
        diagnostic.start
      );
      
      const message = ts.flattenDiagnosticMessageText(
        diagnostic.messageText,
        '\n'
      );

      const item = {
        line: line + 1,
        column: character + 1,
        message,
        code: diagnostic.code,
      };

      // Categorize as error or warning
      if (diagnostic.category === ts.DiagnosticCategory.Error) {
        errors.push(item);
      } else if (diagnostic.category === ts.DiagnosticCategory.Warning) {
        warnings.push(item);
      }
    } else {
      // Global diagnostic (no specific location)
      const message = ts.flattenDiagnosticMessageText(
        diagnostic.messageText,
        '\n'
      );

      const item = {
        line: 0,
        column: 0,
        message,
        code: diagnostic.code,
      };

      if (diagnostic.category === ts.DiagnosticCategory.Error) {
        errors.push(item);
      } else if (diagnostic.category === ts.DiagnosticCategory.Warning) {
        warnings.push(item);
      }
    }
  }

  return {
    valid: errors.length === 0,
    errors,
    warnings,
  };
}

/**
 * Validate that code exports the expected activity function
 */
export function validateActivityStructure(code: string): {
  valid: boolean;
  error?: string;
} {
  // Basic validation: check for export and async function
  const hasExport = /export\s+(async\s+)?function/g.test(code);
  const hasAsyncFunction = /async\s+function/g.test(code);

  if (!hasExport) {
    return {
      valid: false,
      error: 'Activity must export at least one function',
    };
  }

  if (!hasAsyncFunction) {
    return {
      valid: false,
      error: 'Activity function should be async for proper error handling',
    };
  }

  return { valid: true };
}

/**
 * Extract exported function names from TypeScript code
 */
export function extractExportedFunctions(code: string): string[] {
  const sourceFile = ts.createSourceFile(
    'temp.ts',
    code,
    ts.ScriptTarget.ES2020,
    true
  );

  const exportedFunctions: string[] = [];

  function visit(node: ts.Node) {
    // Check for exported function declarations
    if (ts.isFunctionDeclaration(node)) {
      const hasExportModifier = node.modifiers?.some(
        (m) => m.kind === ts.SyntaxKind.ExportKeyword
      );

      if (hasExportModifier && node.name) {
        exportedFunctions.push(node.name.text);
      }
    }

    // Check for exported const/let with arrow functions
    if (ts.isVariableStatement(node)) {
      const hasExportModifier = node.modifiers?.some(
        (m) => m.kind === ts.SyntaxKind.ExportKeyword
      );

      if (hasExportModifier) {
        node.declarationList.declarations.forEach((decl) => {
          if (
            ts.isIdentifier(decl.name) &&
            decl.initializer &&
            ts.isArrowFunction(decl.initializer)
          ) {
            exportedFunctions.push(decl.name.text);
          }
        });
      }
    }

    ts.forEachChild(node, visit);
  }

  visit(sourceFile);

  return exportedFunctions;
}

