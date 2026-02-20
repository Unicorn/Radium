/**
 * Tamagui configuration
 */

import { createTamagui } from '@tamagui/core';
import { config as defaultConfig } from '@tamagui/config/v3';

const config = createTamagui(defaultConfig);

export type Conf = typeof config;

declare module '@tamagui/core' {
  interface TamaguiCustomConfig extends Conf {}
}

export default config;

