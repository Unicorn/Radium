import type {ReactNode} from 'react';
import clsx from 'clsx';
import styles from './styles.module.css';

export interface CardProps {
  children: ReactNode;
  className?: string;
  title?: string;
  hoverable?: boolean;
  onClick?: () => void;
}

export default function Card({
  children,
  className,
  title,
  hoverable = false,
  onClick,
}: CardProps): ReactNode {
  const classes = clsx(
    styles.card,
    hoverable && styles.hoverable,
    onClick && styles.clickable,
    className
  );

  return (
    <div className={classes} onClick={onClick}>
      {title && <h3 className={styles.cardTitle}>{title}</h3>}
      <div className={styles.cardContent}>{children}</div>
    </div>
  );
}
