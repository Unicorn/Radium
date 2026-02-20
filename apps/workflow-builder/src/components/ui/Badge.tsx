'use client';

import { styled, Text, XStack } from 'tamagui';

/**
 * Badge component - simple badge UI element
 * Tamagui doesn't export a Badge component, so we create our own
 */
export const Badge = styled(XStack, {
  name: 'Badge',
  px: '$2',
  py: '$1',
  borderRadius: '$2',
  backgroundColor: '$gray5',
  alignItems: 'center',
  justifyContent: 'center',

  variants: {
    size: {
      1: {
        px: '$1.5',
        py: '$0.5',
      },
      2: {
        px: '$2',
        py: '$1',
      },
      3: {
        px: '$3',
        py: '$1.5',
      },
    },
  } as const,

  defaultVariants: {
    size: 2,
  },
});

export const BadgeText = styled(Text, {
  name: 'BadgeText',
  fontSize: '$2',
  fontWeight: '600',
  color: '$gray12',

  variants: {
    size: {
      1: {
        fontSize: '$1',
      },
      2: {
        fontSize: '$2',
      },
      3: {
        fontSize: '$3',
      },
    },
  } as const,

  defaultVariants: {
    size: 2,
  },
});
