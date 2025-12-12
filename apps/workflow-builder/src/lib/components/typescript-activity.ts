/**
 * TypeScript Activity
 * 
 * Temporal activity for executing TypeScript code
 * 
 * Note: This is a simplified implementation. In production, you would want
 * to use a proper sandboxed execution environment like vm2 or isolated-vm.
 */

export interface TypeScriptActivityInput {
  code: string;
  input: any;
}

export interface TypeScriptActivityOutput {
  success: boolean;
  output?: any;
  error?: string;
}

/**
 * Execute TypeScript code
 * 
 * WARNING: This is a basic implementation. For production use, you should:
 * 1. Use a proper sandboxed execution environment
 * 2. Implement timeouts
 * 3. Limit resource usage
 * 4. Validate code before execution
 */
export async function typescriptProcessActivity(
  input: TypeScriptActivityInput
): Promise<TypeScriptActivityOutput> {
  try {
    // Basic validation
    if (!input.code || typeof input.code !== 'string') {
      throw new Error('Code must be a non-empty string');
    }

    // For now, we'll use a simple eval approach
    // In production, use vm2 or isolated-vm for sandboxing
    // This is a placeholder implementation
    
    // Extract the process function from the code
    // The code should export a function named 'process'
    const processFunction = new Function(
      'input',
      `
      ${input.code}
      if (typeof process !== 'function') {
        throw new Error('Code must export a function named "process"');
      }
      return process(input);
      `
    );

    const output = await processFunction(input.input);

    return {
      success: true,
      output,
    };
  } catch (error: any) {
    return {
      success: false,
      error: error.message || 'Unknown error',
    };
  }
}

