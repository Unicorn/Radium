/**
 * Test Render Utilities
 *
 * Provides custom render functions that wrap components with necessary providers.
 */

import React from 'react';
import { render, RenderOptions, cleanup } from '@testing-library/react';
import { TamaguiProvider, createTamagui } from '@tamagui/core';
import { shorthands } from '@tamagui/shorthands';
import { tokens as defaultTokens, themes as defaultThemes } from '@tamagui/themes';
import { afterEach } from 'vitest';

// Create a minimal Tamagui config for tests
// We use a simplified config to avoid media query issues in jsdom
const tamaguiConfig = createTamagui({
  shorthands,
  tokens: defaultTokens,
  themes: defaultThemes,
  // Disable media queries in tests
  media: {},
});

// Ensure cleanup runs after each test
afterEach(() => {
  cleanup();
});

/**
 * All providers wrapper for tests
 */
function AllProviders({ children }: { children: React.ReactNode }) {
  return (
    <TamaguiProvider config={tamaguiConfig} defaultTheme="light" disableInjectCSS>
      {children}
    </TamaguiProvider>
  );
}

/**
 * Custom render function that wraps components with all necessary providers
 */
function customRender(
  ui: React.ReactElement,
  options?: Omit<RenderOptions, 'wrapper'>
) {
  return render(ui, { wrapper: AllProviders, ...options });
}

// Re-export everything from testing-library
export * from '@testing-library/react';

// Override render with our custom version
export { customRender as render };
