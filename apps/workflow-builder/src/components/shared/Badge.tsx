/**
 * Badge Component - Simple badge wrapper for Tamagui
 */

'use client';

import { styled, Stack } from 'tamagui';

export const Badge = styled(Stack, {
  paddingHorizontal: '$2',
  paddingVertical: '$1',
  borderRadius: '$2',
  backgroundColor: '$gray5',
  alignItems: 'center',
  justifyContent: 'center',
  
  variants: {
    size: {
      1: {
        paddingHorizontal: '$1.5',
        paddingVertical: '$0.5',
      },
      2: {
        paddingHorizontal: '$2',
        paddingVertical: '$1',
      },
      3: {
        paddingHorizontal: '$3',
        paddingVertical: '$1.5',
      },
    },
  } as const,
  
  defaultVariants: {
    size: 2,
  },
});

