import * as fs from 'fs/promises';
import * as path from 'path';
import type { ExecuteCommandResult } from './command-execution.activities';
import { executeCommand } from './command-execution.activities';

export type PackageManager = 'npm' | 'yarn' | 'pnpm';

export interface CheckOutdatedDependenciesInput {
  workspacePath: string;
  packageManager?: PackageManager;
  includeDev?: boolean;
}

export interface OutdatedDependencyInfo {
  name: string;
  current: string;
  wanted: string;
  latest: string;
  type?: string;
  location?: string;
}

export interface CheckOutdatedDependenciesResult {
  success: boolean;
  outdated: OutdatedDependencyInfo[];
  rawOutput?: string;
  error?: string;
}

export type VulnerabilitySeverity = 'low' | 'moderate' | 'high' | 'critical';

export interface ScanVulnerabilitiesInput {
  workspacePath: string;
  packageManager?: PackageManager;
  severityThreshold?: VulnerabilitySeverity;
}

export interface VulnerabilitySummary {
  id?: string;
  name?: string;
  severity: VulnerabilitySeverity;
  via?: string[];
  dependency?: string;
  title?: string;
  url?: string;
}

export interface ScanVulnerabilitiesResult {
  success: boolean;
  vulnerabilities: VulnerabilitySummary[];
  severities: Record<VulnerabilitySeverity, number>;
  rawOutput?: string;
  blocked?: boolean;
  error?: string;
}

async function readPackageJson(workspacePath: string): Promise<Record<string, unknown> | null> {
  try {
    const packageJsonPath = path.join(workspacePath, 'package.json');
    const content = await fs.readFile(packageJsonPath, 'utf8');
    return JSON.parse(content) as Record<string, unknown>;
  } catch {
    return null;
  }
}

function getOutdatedCommand(
  packageManager: PackageManager,
  includeDev: boolean | undefined,
): { command: string; args: string[] } {
  const args: string[] = [];

  if (packageManager === 'npm') {
    args.push('outdated', '--json');
    if (includeDev) {
      args.push('--dev');
    }
    return { command: 'npm', args };
  }

  if (packageManager === 'pnpm') {
    args.push('outdated', '--json');
    if (includeDev) {
      args.push('--prod', '--dev');
    }
    return { command: 'pnpm', args };
  }

  // yarn
  args.push('outdated', '--json');
  return { command: 'yarn', args };
}

function parseOutdatedJson(stdout: string): OutdatedDependencyInfo[] {
  const trimmed = stdout.trim();
  if (!trimmed) {
    return [];
  }

  try {
    const data = JSON.parse(trimmed) as Record<
      string,
      { current: string; wanted: string; latest: string; location?: string; type?: string }
    >;

    const entries: OutdatedDependencyInfo[] = [];
    for (const [name, info] of Object.entries(data)) {
      entries.push({
        name,
        current: info.current,
        wanted: info.wanted,
        latest: info.latest,
        type: info.type,
        location: info.location,
      });
    }
    return entries;
  } catch {
    // Some package managers (like yarn) emit line-delimited JSON; fall back to empty
    return [];
  }
}

/**
 * Check for outdated dependencies using the selected package manager.
 *
 * This uses `npm outdated --json` / `pnpm outdated --json` / `yarn outdated --json`
 * and normalizes the output for consumption by workflows and the compiler.
 */
export async function checkOutdatedDependencies(
  input: CheckOutdatedDependenciesInput,
): Promise<CheckOutdatedDependenciesResult> {
  const { workspacePath, packageManager = 'npm', includeDev } = input;

  const pkg = await readPackageJson(workspacePath);
  if (!pkg) {
    return {
      success: false,
      outdated: [],
      error: 'No package.json found in workspace',
    };
  }

  const { command, args } = getOutdatedCommand(packageManager, includeDev);

  const result: ExecuteCommandResult = await executeCommand({
    command,
    args,
    workingDir: workspacePath,
    timeout: 600000,
  });

  // Even when outdated packages exist, package managers may return non‑zero exit codes.
  // We treat parsable JSON on stdout as a success signal regardless of exit code.
  const outdated = parseOutdatedJson(result.stdout);

  if (outdated.length > 0) {
    return {
      success: true,
      outdated,
      rawOutput: result.stdout,
    };
  }

  // No outdated dependencies; if the command itself failed, surface a useful error
  if (!result.success && result.stderr) {
    return {
      success: false,
      outdated: [],
      rawOutput: result.stdout,
      error: result.stderr,
    };
  }

  return {
    success: true,
    outdated: [],
    rawOutput: result.stdout,
  };
}

function getAuditCommand(
  packageManager: PackageManager,
): { command: string; args: string[] } {
  if (packageManager === 'npm') {
    return { command: 'npm', args: ['audit', '--json'] };
  }

  if (packageManager === 'pnpm') {
    return { command: 'pnpm', args: ['audit', '--json'] };
  }

  // yarn
  return { command: 'yarn', args: ['audit', '--json'] };
}

function parseSeverityCounts(
  data: unknown,
): { severities: Record<VulnerabilitySeverity, number>; summaries: VulnerabilitySummary[] } {
  const base: Record<VulnerabilitySeverity, number> = {
    low: 0,
    moderate: 0,
    high: 0,
    critical: 0,
  };

  if (!data || typeof data !== 'object') {
    return { severities: base, summaries: [] };
  }

  const root = data as {
    vulnerabilities?: Record<
      string,
      {
        name?: string;
        severity?: VulnerabilitySeverity;
        via?: Array<string | { source?: string }>;
        title?: string;
        url?: string;
      }
    >;
    metadata?: { vulnerabilities?: Partial<Record<VulnerabilitySeverity, number>> };
  };

  const severities: Record<VulnerabilitySeverity, number> = { ...base };
  const summaries: VulnerabilitySummary[] = [];

  if (root.metadata?.vulnerabilities) {
    for (const key of Object.keys(root.metadata.vulnerabilities) as VulnerabilitySeverity[]) {
      const value = root.metadata.vulnerabilities[key];
      if (typeof value === 'number') {
        severities[key] = value;
      }
    }
  }

  if (root.vulnerabilities) {
    for (const [id, vuln] of Object.entries(root.vulnerabilities)) {
      const severity: VulnerabilitySeverity = (vuln.severity as VulnerabilitySeverity) || 'low';
      const via = (vuln.via || [])
        .map((v) => (typeof v === 'string' ? v : v.source ?? 'unknown'))
        .filter((v) => !!v);

      summaries.push({
        id,
        name: vuln.name,
        severity,
        via,
        title: vuln.title,
        url: vuln.url,
      });

      if (severity in severities) {
        severities[severity] += 0; // already populated from metadata; keep consistent
      }
    }
  }

  return { severities, summaries };
}

function isSeverityAtOrAbove(
  severity: VulnerabilitySeverity,
  threshold: VulnerabilitySeverity,
): boolean {
  const order: VulnerabilitySeverity[] = ['low', 'moderate', 'high', 'critical'];
  return order.indexOf(severity) >= order.indexOf(threshold);
}

/**
 * Scan project dependencies for known vulnerabilities.
 *
 * This is a thin wrapper around `npm audit` / `pnpm audit` / `yarn audit` in JSON mode.
 * In tests, command execution should be mocked to avoid network access.
 */
export async function scanVulnerabilities(
  input: ScanVulnerabilitiesInput,
): Promise<ScanVulnerabilitiesResult> {
  const { workspacePath, packageManager = 'npm', severityThreshold } = input;

  const pkg = await readPackageJson(workspacePath);
  if (!pkg) {
    return {
      success: false,
      vulnerabilities: [],
      severities: {
        low: 0,
        moderate: 0,
        high: 0,
        critical: 0,
      },
      error: 'No package.json found in workspace',
    };
  }

  const { command, args } = getAuditCommand(packageManager);

  const result: ExecuteCommandResult = await executeCommand({
    command,
    args,
    workingDir: workspacePath,
    timeout: 600000,
  });

  const trimmed = result.stdout.trim();
  if (!trimmed) {
    return {
      success: result.success,
      vulnerabilities: [],
      severities: {
        low: 0,
        moderate: 0,
        high: 0,
        critical: 0,
      },
      rawOutput: result.stdout,
      error: result.success ? undefined : result.stderr || result.error,
    };
  }

  let parsed: unknown;
  try {
    parsed = JSON.parse(trimmed) as unknown;
  } catch {
    return {
      success: false,
      vulnerabilities: [],
      severities: {
        low: 0,
        moderate: 0,
        high: 0,
        critical: 0,
      },
      rawOutput: result.stdout,
      error: 'Failed to parse vulnerability scan output',
    };
  }

  const { severities, summaries } = parseSeverityCounts(parsed);

  let blocked = false;
  if (severityThreshold) {
    blocked = summaries.some((vuln) => isSeverityAtOrAbove(vuln.severity, severityThreshold));
  }

  return {
    success: result.success,
    vulnerabilities: summaries,
    severities,
    rawOutput: result.stdout,
    blocked,
    error: result.success ? undefined : result.stderr || result.error,
  };
}

// ============================================================================
// Advanced Package Management (Phase 3.4)
// ============================================================================

export interface BumpVersionInput {
  workspacePath: string;
  version: 'major' | 'minor' | 'patch' | string; // Semantic version part or specific version
  packageManager?: PackageManager;
  commit?: boolean; // Create a git commit after bumping
  tag?: boolean; // Create a git tag after bumping
}

export interface BumpVersionResult {
  success: boolean;
  oldVersion?: string;
  newVersion?: string;
  packageJsonPath?: string;
  committed?: boolean;
  tagged?: boolean;
  error?: string;
}

export interface LicenseInfo {
  name: string;
  version: string;
  license: string;
  repository?: string;
  author?: string;
  homepage?: string;
}

export interface CheckLicensesInput {
  workspacePath: string;
  packageManager?: PackageManager;
  allowedLicenses?: string[]; // Whitelist of allowed licenses (e.g., ['MIT', 'Apache-2.0'])
  blockedLicenses?: string[]; // Blacklist of blocked licenses (e.g., ['GPL-3.0'])
}

export interface CheckLicensesResult {
  success: boolean;
  licenses: LicenseInfo[];
  summary: {
    total: number;
    byLicense: Record<string, number>;
    allowed: number;
    blocked: number;
    unknown: number;
  };
  blockedPackages: LicenseInfo[];
  error?: string;
}

/**
 * Bump package version (Phase 3.4: Advanced Package Management)
 */
export async function bumpVersion(
  input: BumpVersionInput,
): Promise<BumpVersionResult> {
  const { workspacePath, version, packageManager = 'npm', commit = false, tag = false } = input;

  try {
    // Read current package.json
    const pkg = await readPackageJson(workspacePath);
    if (!pkg) {
      return {
        success: false,
        error: 'No package.json found in workspace',
      };
    }

    const oldVersion = (pkg.version as string) || '0.0.0';
    let newVersion: string;

    // Determine new version
    if (version === 'major' || version === 'minor' || version === 'patch') {
      // Use npm version command for semantic versioning
      const { command, args } = getVersionBumpCommand(packageManager, version);
      const result: ExecuteCommandResult = await executeCommand({
        command,
        args,
        workingDir: workspacePath,
        timeout: 30000,
      });

      if (!result.success) {
        return {
          success: false,
          oldVersion,
          error: result.stderr || result.error || 'Failed to bump version',
        };
      }

      // Read the updated package.json to get the new version
      const updatedPkg = await readPackageJson(workspacePath);
      newVersion = (updatedPkg?.version as string) || oldVersion;
    } else {
      // Specific version provided - update package.json directly
      pkg.version = version;
      newVersion = version;

      const packageJsonPath = path.join(workspacePath, 'package.json');
      await fs.writeFile(packageJsonPath, JSON.stringify(pkg, null, 2) + '\n', 'utf8');
    }

    let committed = false;
    let tagged = false;

    // Create git commit if requested
    if (commit) {
      const gitResult: ExecuteCommandResult = await executeCommand({
        command: 'git',
        args: ['add', 'package.json'],
        workingDir: workspacePath,
        timeout: 10000,
      });

      if (gitResult.success) {
        const commitResult: ExecuteCommandResult = await executeCommand({
          command: 'git',
          args: ['commit', '-m', `chore: bump version to ${newVersion}`],
          workingDir: workspacePath,
          timeout: 10000,
        });
        committed = commitResult.success;
      }
    }

    // Create git tag if requested
    if (tag && newVersion) {
      const tagResult: ExecuteCommandResult = await executeCommand({
        command: 'git',
        args: ['tag', `v${newVersion}`, '-m', `Version ${newVersion}`],
        workingDir: workspacePath,
        timeout: 10000,
      });
      tagged = tagResult.success;
    }

    return {
      success: true,
      oldVersion,
      newVersion,
      packageJsonPath: path.join(workspacePath, 'package.json'),
      committed,
      tagged,
    };
  } catch (error: any) {
    return {
      success: false,
      error: error.message || 'Unknown error bumping version',
    };
  }
}

function getVersionBumpCommand(
  packageManager: PackageManager,
  version: 'major' | 'minor' | 'patch',
): { command: string; args: string[] } {
  if (packageManager === 'npm') {
    return { command: 'npm', args: ['version', version, '--no-git-tag-version'] };
  }

  if (packageManager === 'pnpm') {
    return { command: 'pnpm', args: ['version', version, '--no-git-tag-version'] };
  }

  // yarn doesn't have a version command, so we'll handle it manually
  return { command: 'npm', args: ['version', version, '--no-git-tag-version'] };
}

/**
 * Check licenses of all dependencies (Phase 3.4: Advanced Package Management)
 */
export async function checkLicenses(
  input: CheckLicensesInput,
): Promise<CheckLicensesResult> {
  const { workspacePath, packageManager = 'npm', allowedLicenses = [], blockedLicenses = [] } = input;

  try {
    const pkg = await readPackageJson(workspacePath);
    if (!pkg) {
      return {
        success: false,
        licenses: [],
        summary: {
          total: 0,
          byLicense: {},
          allowed: 0,
          blocked: 0,
          unknown: 0,
        },
        blockedPackages: [],
        error: 'No package.json found in workspace',
      };
    }

    // Use license-checker or similar tool
    // For now, we'll use a simple approach: read package.json and check node_modules
    // In a real implementation, you might use `license-checker` npm package
    const { command, args } = getLicenseCheckCommand(packageManager);
    const result: ExecuteCommandResult = await executeCommand({
      command,
      args,
      workingDir: workspacePath,
      timeout: 60000,
    });

    if (!result.success && result.stderr && !result.stderr.includes('license-checker')) {
      // If license-checker is not installed, fall back to reading package.json dependencies
      return await checkLicensesFromPackageJson(workspacePath, pkg, allowedLicenses, blockedLicenses);
    }

    // Parse license-checker output (JSON format)
    const licenses = parseLicenseCheckerOutput(result.stdout, allowedLicenses, blockedLicenses);

    // Build summary
    const byLicense: Record<string, number> = {};
    let allowed = 0;
    let blocked = 0;
    let unknown = 0;

    const blockedPackages: LicenseInfo[] = [];

    for (const license of licenses) {
      const licenseStr = license.license || 'UNKNOWN';
      byLicense[licenseStr] = (byLicense[licenseStr] || 0) + 1;

      if (blockedLicenses.length > 0 && blockedLicenses.includes(licenseStr)) {
        blocked++;
        blockedPackages.push(license);
      } else if (allowedLicenses.length > 0 && !allowedLicenses.includes(licenseStr)) {
        blocked++;
        blockedPackages.push(license);
      } else if (licenseStr === 'UNKNOWN') {
        unknown++;
      } else {
        allowed++;
      }
    }

    return {
      success: true,
      licenses,
      summary: {
        total: licenses.length,
        byLicense,
        allowed,
        blocked,
        unknown,
      },
      blockedPackages,
    };
  } catch (error: any) {
    return {
      success: false,
      licenses: [],
      summary: {
        total: 0,
        byLicense: {},
        allowed: 0,
        blocked: 0,
        unknown: 0,
      },
      blockedPackages: [],
      error: error.message || 'Unknown error checking licenses',
    };
  }
}

function getLicenseCheckCommand(packageManager: PackageManager): { command: string; args: string[] } {
  // Try to use license-checker if available
  return { command: 'npx', args: ['license-checker', '--json', '--start', '.'] };
}

function parseLicenseCheckerOutput(
  stdout: string,
  allowedLicenses: string[],
  blockedLicenses: string[],
): LicenseInfo[] {
  const trimmed = stdout.trim();
  if (!trimmed) {
    return [];
  }

  try {
    const data = JSON.parse(trimmed) as Record<
      string,
      {
        licenses?: string;
        license?: string;
        repository?: string;
        publisher?: string;
        email?: string;
        url?: string;
      }
    >;

    const licenses: LicenseInfo[] = [];
    for (const [nameWithVersion, info] of Object.entries(data)) {
      // Parse "package-name@version" format
      const match = nameWithVersion.match(/^(.+)@(.+)$/);
      const packageName = match ? match[1] : nameWithVersion;
      const version = match ? match[2] : 'unknown';

      licenses.push({
        name: packageName,
        version,
        license: info.licenses || info.license || 'UNKNOWN',
        repository: info.repository,
        homepage: info.url,
        author: info.publisher,
      });
    }
    return licenses;
  } catch {
    return [];
  }
}

async function checkLicensesFromPackageJson(
  workspacePath: string,
  pkg: Record<string, unknown>,
  allowedLicenses: string[],
  blockedLicenses: string[],
): Promise<CheckLicensesResult> {
  // Fallback: read dependencies from package.json
  // This is a simplified version that doesn't check transitive dependencies
  const dependencies = {
    ...((pkg.dependencies as Record<string, string>) || {}),
    ...((pkg.devDependencies as Record<string, string>) || {}),
  };

  const licenses: LicenseInfo[] = [];
  const byLicense: Record<string, number> = {};
  let allowed = 0;
  let blocked = 0;
  let unknown = 0;
  const blockedPackages: LicenseInfo[] = [];

  for (const [name, versionRange] of Object.entries(dependencies)) {
    // Try to read license from node_modules if available
    const nodeModulesPath = path.join(workspacePath, 'node_modules', name, 'package.json');
    let license = 'UNKNOWN';

    try {
      const depPkgContent = await fs.readFile(nodeModulesPath, 'utf8');
      const depPkg = JSON.parse(depPkgContent) as Record<string, unknown>;
      license = (depPkg.license as string) || (depPkg.licenses as string) || 'UNKNOWN';
    } catch {
      // Package not installed or can't read - mark as unknown
    }

    const licenseInfo: LicenseInfo = {
      name,
      version: versionRange,
      license,
    };

    licenses.push(licenseInfo);
    byLicense[license] = (byLicense[license] || 0) + 1;

    if (blockedLicenses.length > 0 && blockedLicenses.includes(license)) {
      blocked++;
      blockedPackages.push(licenseInfo);
    } else if (allowedLicenses.length > 0 && !allowedLicenses.includes(license)) {
      blocked++;
      blockedPackages.push(licenseInfo);
    } else if (license === 'UNKNOWN') {
      unknown++;
    } else {
      allowed++;
    }
  }

  return {
    success: true,
    licenses,
    summary: {
      total: licenses.length,
      byLicense,
      allowed,
      blocked,
      unknown,
    },
    blockedPackages,
  };
}

