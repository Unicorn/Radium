/**
 * Type definitions for Lucide React icons
 */
import type { LucideProps } from 'lucide-react';
import type { ForwardRefExoticComponent, RefAttributes } from 'react';

/**
 * Type for Lucide icon components
 * This properly types the ForwardRefExoticComponent that Lucide exports
 */
export type LucideIcon = ForwardRefExoticComponent<
  Omit<LucideProps, 'ref'> & RefAttributes<SVGSVGElement>
>;

/**
 * Simplified icon props for internal component use
 * Maps Lucide's complex props to a simpler interface
 */
export interface IconProps {
  size?: number;
  color?: string;
}
