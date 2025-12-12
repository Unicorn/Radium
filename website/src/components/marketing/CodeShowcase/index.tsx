import type {ReactNode} from 'react';
import Badge from '@site/src/components/shared/Badge';
import styles from './styles.module.css';

export interface CodePanel {
  title: string;
  subtitle?: string;
  code: string;
  language?: string;
  badge?: {
    label: string;
    variant?: 'danger' | 'success' | 'warning' | 'info';
  };
}

export interface CodeShowcaseProps {
  before: CodePanel;
  after: CodePanel;
  title?: string;
  description?: ReactNode;
  className?: string;
}

export default function CodeShowcase({
  before,
  after,
  title,
  description,
  className,
}: CodeShowcaseProps): ReactNode {
  return (
    <section className={`${styles.codeShowcase} ${className || ''}`}>
      {title && <h2 className={styles.title}>{title}</h2>}
      {description && <div className={styles.description}>{description}</div>}

      <div className={styles.panels}>
        <CodePanelComponent {...before} />
        <CodePanelComponent {...after} />
      </div>
    </section>
  );
}

function CodePanelComponent({
  title,
  subtitle,
  code,
  language = 'typescript',
  badge,
}: CodePanel): ReactNode {
  return (
    <div className={styles.panel}>
      <div className={styles.panelHeader}>
        <div className={styles.panelHeaderLeft}>
          <h3 className={styles.panelTitle}>{title}</h3>
          {subtitle && <span className={styles.panelSubtitle}>{subtitle}</span>}
        </div>
        {badge && (
          <Badge variant={badge.variant || 'default'} size="sm">
            {badge.label}
          </Badge>
        )}
      </div>
      <div className={styles.panelBody}>
        <pre className={styles.codeBlock}>
          <code className={`language-${language}`}>{code}</code>
        </pre>
      </div>
    </div>
  );
}
