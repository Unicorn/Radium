/**
 * Workflow Compiler Kong Service Registration
 *
 * Registers the Rust workflow compiler service with Kong.
 */

import { KongClient } from './client';
import { getKongConfig, isKongEnabled } from './config';

/** Configuration for workflow compiler service */
export interface WorkflowCompilerConfig {
  /** Base URL of the workflow compiler service */
  serviceUrl: string;
  /** Rate limit per minute for compile endpoints */
  compileRateLimit?: number;
  /** Rate limit per minute for full compile endpoints */
  fullCompileRateLimit?: number;
}

/** Default configuration */
const DEFAULT_CONFIG: Required<WorkflowCompilerConfig> = {
  serviceUrl: 'http://workflow-compiler:3020',
  compileRateLimit: 30,
  fullCompileRateLimit: 10,
};

/** Result of service registration */
export interface WorkflowCompilerRegistration {
  serviceId: string;
  routes: {
    compile: string;
    validate: string;
    generate: string;
    verify: string;
    fullCompile: string;
    schema: string;
    version: string;
    health: string;
  };
}

/**
 * Register the workflow compiler service with Kong
 */
export async function registerWorkflowCompilerService(
  config: Partial<WorkflowCompilerConfig> = {}
): Promise<WorkflowCompilerRegistration> {
  if (!isKongEnabled()) {
    throw new Error('Kong is not enabled. Cannot register workflow compiler service.');
  }

  const fullConfig = { ...DEFAULT_CONFIG, ...config };
  const kong = new KongClient();

  // Verify Kong is accessible
  const isHealthy = await kong.healthCheck();
  if (!isHealthy) {
    throw new Error('Kong Admin API is not accessible');
  }

  // Create or get the service
  const serviceId = await kong.createService(
    'workflow-compiler',
    fullConfig.serviceUrl
  );

  // Create routes
  const routes: WorkflowCompilerRegistration['routes'] = {
    compile: '',
    validate: '',
    generate: '',
    verify: '',
    fullCompile: '',
    schema: '',
    version: '',
    health: '',
  };

  // Main compile endpoint
  const existingCompile = await kong.getRoute('workflow-compiler-compile');
  if (existingCompile) {
    routes.compile = existingCompile.id;
  } else {
    routes.compile = await kong.createRoute(
      serviceId,
      'workflow-compiler-compile',
      ['/api/v1/compile'],
      ['POST']
    );
    // Add rate limiting
    await kong.enablePlugin(routes.compile, 'rate-limiting', {
      minute: fullConfig.compileRateLimit,
      hour: fullConfig.compileRateLimit * 60,
      policy: 'local',
    });
  }

  // Validate endpoint
  const existingValidate = await kong.getRoute('workflow-compiler-validate');
  if (existingValidate) {
    routes.validate = existingValidate.id;
  } else {
    routes.validate = await kong.createRoute(
      serviceId,
      'workflow-compiler-validate',
      ['/api/v1/compile/validate'],
      ['POST']
    );
  }

  // Generate endpoint
  const existingGenerate = await kong.getRoute('workflow-compiler-generate');
  if (existingGenerate) {
    routes.generate = existingGenerate.id;
  } else {
    routes.generate = await kong.createRoute(
      serviceId,
      'workflow-compiler-generate',
      ['/api/v1/compile/generate'],
      ['POST']
    );
  }

  // Verify endpoint
  const existingVerify = await kong.getRoute('workflow-compiler-verify');
  if (existingVerify) {
    routes.verify = existingVerify.id;
  } else {
    routes.verify = await kong.createRoute(
      serviceId,
      'workflow-compiler-verify',
      ['/api/v1/compile/verify'],
      ['POST']
    );
  }

  // Full compile endpoint
  const existingFull = await kong.getRoute('workflow-compiler-full');
  if (existingFull) {
    routes.fullCompile = existingFull.id;
  } else {
    routes.fullCompile = await kong.createRoute(
      serviceId,
      'workflow-compiler-full',
      ['/api/v1/compile/full'],
      ['POST']
    );
    // Add stricter rate limiting for full compile
    await kong.enablePlugin(routes.fullCompile, 'rate-limiting', {
      minute: fullConfig.fullCompileRateLimit,
      hour: fullConfig.fullCompileRateLimit * 10,
      policy: 'local',
    });
  }

  // Schema endpoint
  const existingSchema = await kong.getRoute('workflow-compiler-schema');
  if (existingSchema) {
    routes.schema = existingSchema.id;
  } else {
    routes.schema = await kong.createRoute(
      serviceId,
      'workflow-compiler-schema',
      ['/api/v1/schema'],
      ['GET']
    );
  }

  // Version endpoint
  const existingVersion = await kong.getRoute('workflow-compiler-version');
  if (existingVersion) {
    routes.version = existingVersion.id;
  } else {
    routes.version = await kong.createRoute(
      serviceId,
      'workflow-compiler-version',
      ['/api/v1/version'],
      ['GET']
    );
  }

  // Health endpoint
  const existingHealth = await kong.getRoute('workflow-compiler-health');
  if (existingHealth) {
    routes.health = existingHealth.id;
  } else {
    routes.health = await kong.createRoute(
      serviceId,
      'workflow-compiler-health',
      ['/health', '/healthz'],
      ['GET']
    );
  }

  console.log('Registered workflow compiler service with Kong');

  return {
    serviceId,
    routes,
  };
}

/**
 * Get the base URL for the workflow compiler through Kong
 */
export function getWorkflowCompilerUrl(): string {
  const kongConfig = getKongConfig();
  return `${kongConfig.gatewayUrl}/api/v1/compile`;
}

/**
 * Client interface for calling the workflow compiler service
 */
export interface WorkflowCompilerClient {
  /** Compile a workflow (validate + generate) */
  compile(workflow: unknown, options?: CompileOptions): Promise<CompileResponse>;
  /** Validate a workflow definition */
  validate(workflow: unknown): Promise<ValidationResponse>;
  /** Generate TypeScript code (skip verification) */
  generate(workflow: unknown, options?: CompileOptions): Promise<GenerateResponse>;
  /** Verify generated code */
  verify(code: GeneratedCode): Promise<VerificationResponse>;
  /** Full compilation pipeline (validate + generate + verify) */
  fullCompile(workflow: unknown, options?: CompileOptions): Promise<FullCompileResponse>;
}

export interface CompileOptions {
  workflowName?: string;
  defaultTimeout?: string;
  includeComments?: boolean;
  strictMode?: boolean;
  verify?: boolean;
  outputDir?: string;
}

export interface GeneratedCode {
  workflow: string;
  activities: string;
  worker: string;
  packageJson: string;
  tsconfig: string;
}

export interface ValidationResponse {
  success: boolean;
  data: {
    valid: boolean;
    errors: Array<{ type: string; details?: unknown }>;
    warnings: Array<{ type: string; details?: unknown }>;
  };
}

export interface GenerateResponse {
  success: boolean;
  data: GeneratedCode;
}

export interface VerificationResponse {
  success: boolean;
  data: {
    success: boolean;
    typescript: {
      success: boolean;
      errors: Array<{
        file: string;
        line: number;
        column: number;
        code: string;
        message: string;
      }>;
    };
    eslint: {
      success: boolean;
      issues: Array<{
        file: string;
        line: number;
        column: number;
        ruleId?: string;
        severity: number;
        message: string;
      }>;
      errorCount: number;
      warningCount: number;
    };
  };
}

export interface CompileResponse {
  validation: ValidationResponse['data'];
  code?: GeneratedCode;
  verification?: VerificationResponse['data'];
}

export interface FullCompileResponse {
  validation: ValidationResponse['data'];
  code: GeneratedCode;
  verification: VerificationResponse['data'];
}

/**
 * Create a workflow compiler client
 */
export function createWorkflowCompilerClient(
  baseUrl?: string
): WorkflowCompilerClient {
  const url = baseUrl || getWorkflowCompilerUrl();

  return {
    async compile(workflow: unknown, options?: CompileOptions): Promise<CompileResponse> {
      const response = await fetch(url, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ workflow, options }),
      });

      if (!response.ok) {
        const error = await response.text();
        throw new Error(`Compilation failed: ${response.status} ${error}`);
      }

      return response.json();
    },

    async validate(workflow: unknown): Promise<ValidationResponse> {
      const response = await fetch(`${url}/validate`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ workflow }),
      });

      if (!response.ok) {
        const error = await response.text();
        throw new Error(`Validation failed: ${response.status} ${error}`);
      }

      return response.json();
    },

    async generate(workflow: unknown, options?: CompileOptions): Promise<GenerateResponse> {
      const response = await fetch(`${url}/generate`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ workflow, options }),
      });

      if (!response.ok) {
        const error = await response.text();
        throw new Error(`Code generation failed: ${response.status} ${error}`);
      }

      return response.json();
    },

    async verify(code: GeneratedCode): Promise<VerificationResponse> {
      const response = await fetch(`${url}/verify`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(code),
      });

      if (!response.ok) {
        const error = await response.text();
        throw new Error(`Verification failed: ${response.status} ${error}`);
      }

      return response.json();
    },

    async fullCompile(
      workflow: unknown,
      options?: CompileOptions
    ): Promise<FullCompileResponse> {
      const response = await fetch(`${url}/full`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ workflow, options }),
      });

      if (!response.ok) {
        const error = await response.text();
        throw new Error(`Full compilation failed: ${response.status} ${error}`);
      }

      return response.json();
    },
  };
}
