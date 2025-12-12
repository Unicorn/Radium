import type {ReactNode} from 'react';
import clsx from 'clsx';
import styles from './styles.module.css';

export type BadgeVariant = 'default' | 'primary' | 'success' | 'warning' | 'danger' | 'info';
export type BadgeSize = 'sm' | 'md';

export interface BadgeProps {
  children: ReactNode;
  variant?: BadgeVariant;
  size?: BadgeSize;
  className?: string;
}

export default function Badge({
  children,
  variant = 'default',
  size = 'md',
  className,
}: BadgeProps): ReactNode {
  const classes = clsx(
    styles.badge,
    styles[variant],
    styles[size],
    className
  );

  return <span className={classes}>{children}</span>;
}
