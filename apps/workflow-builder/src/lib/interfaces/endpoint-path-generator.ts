/**
 * Endpoint Path Generator
 * Generates API endpoint paths from component names
 */

/**
 * Convert camelCase to kebab-case endpoint path
 * Example: "receiveOrder" -> "/receive-order"
 */
export function generateEndpointPath(componentName: string): string {
  if (!componentName) {
    return '/';
  }

  // Convert camelCase to kebab-case
  const kebab = componentName
    .replace(/([a-z])([A-Z])/g, '$1-$2')
    .toLowerCase();
  
  // Ensure it starts with /
  return kebab.startsWith('/') ? kebab : `/${kebab}`;
}

/**
 * Validate endpoint path format
 */
export function validateEndpointPath(path: string): { valid: boolean; error?: string } {
  if (!path) {
    return { valid: false, error: 'Endpoint path is required' };
  }

  if (!path.startsWith('/')) {
    return { valid: false, error: 'Endpoint path must start with /' };
  }

  // Check for valid characters (alphanumeric, hyphens, underscores, slashes)
  if (!/^\/[a-z0-9\-_\/]*$/i.test(path)) {
    return { valid: false, error: 'Endpoint path can only contain letters, numbers, hyphens, underscores, and slashes' };
  }

  // Check for consecutive slashes (except at start)
  if (/\/{2,}/.test(path.substring(1))) {
    return { valid: false, error: 'Endpoint path cannot contain consecutive slashes' };
  }

  return { valid: true };
}

