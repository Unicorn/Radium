/**
 * Tamagui Provider
 */

'use client';

import { TamaguiProvider as Provider } from 'tamagui';
import config from './config';

export function TamaguiProvider({ children }: { children: React.ReactNode }) {
  return (
    <Provider config={config} defaultTheme="light">
      {children}
    </Provider>
  );
}

