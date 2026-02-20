import type {ReactNode} from 'react';
import type {Feature} from '@site/src/data/features';
import Button from '@site/src/components/shared/Button';
import styles from './styles.module.css';

export interface FeatureCardProps extends Feature {
  className?: string;
}

export default function FeatureCard({
  title,
  description,
  icon,
  codeExample,
  links,
  className,
}: FeatureCardProps): ReactNode {
  return (
    <div className={`${styles.featureCard} ${className || ''}`}>
      <div className={styles.featureHeader}>
        {icon && <div className={styles.featureIcon}>{icon}</div>}
        <h3 className={styles.featureTitle}>{title}</h3>
      </div>

      <div className={styles.featureDescription}>{description}</div>

      {codeExample && (
        <div className={styles.codeSection}>
          <pre className={styles.codeBlock}>
            <code className={`language-${codeExample.language}`}>{codeExample.code}</code>
          </pre>
        </div>
      )}

      {links && links.length > 0 && (
        <div className={styles.featureLinks}>
          {links.map((link, idx) => (
            <Button
              key={idx}
              variant="tertiary"
              size="sm"
              href={link.href}
              className={styles.featureLink}>
              {link.label} →
            </Button>
          ))}
        </div>
      )}
    </div>
  );
}
