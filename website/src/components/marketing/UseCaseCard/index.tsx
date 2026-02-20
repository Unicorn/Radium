import {useState, type ReactNode} from 'react';
import type {UseCase} from '@site/src/data/useCases';
import Badge from '@site/src/components/shared/Badge';
import Button from '@site/src/components/shared/Button';
import styles from './styles.module.css';

export interface UseCaseCardProps extends UseCase {
  className?: string;
}

const categoryVariant = {
  development: 'primary' as const,
  data: 'info' as const,
  enterprise: 'warning' as const,
  devops: 'success' as const,
  security: 'danger' as const,
};

export default function UseCaseCard({
  title,
  category,
  problem,
  solution,
  techStack,
  results,
  codeSnippet,
  tutorialLink,
  exampleLink,
  className,
}: UseCaseCardProps): ReactNode {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    if (codeSnippet) {
      await navigator.clipboard.writeText(codeSnippet);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  return (
    <div className={`${styles.useCaseCard} ${className || ''}`}>
      <div className={styles.cardHeader}>
        <Badge variant={categoryVariant[category]} size="sm">
          {category}
        </Badge>
        <h3 className={styles.cardTitle}>{title}</h3>
      </div>

      <div className={styles.cardBody}>
        <div className={styles.section}>
          <h4 className={styles.sectionTitle}>Problem</h4>
          <p className={styles.sectionContent}>{problem}</p>
        </div>

        <div className={styles.section}>
          <h4 className={styles.sectionTitle}>Solution</h4>
          <p className={styles.sectionContent}>{solution}</p>
        </div>

        <div className={styles.section}>
          <h4 className={styles.sectionTitle}>Results</h4>
          <ul className={styles.resultsList}>
            {results.map((result, idx) => (
              <li key={idx} className={styles.resultItem}>
                {result}
              </li>
            ))}
          </ul>
        </div>

        {techStack && techStack.length > 0 && (
          <div className={styles.techStack}>
            {techStack.map((tech) => (
              <span key={tech} className={styles.techBadge}>
                {tech}
              </span>
            ))}
          </div>
        )}

        {codeSnippet && (
          <div className={styles.codeSection}>
            <div className={styles.codeHeader}>
              <span className={styles.codeLabel}>Configuration</span>
              <button
                className={styles.copyButton}
                onClick={handleCopy}
                aria-label="Copy code to clipboard">
                {copied ? '✓ Copied!' : 'Copy'}
              </button>
            </div>
            <pre className={styles.codeBlock}>
              <code>{codeSnippet}</code>
            </pre>
          </div>
        )}
      </div>

      <div className={styles.cardFooter}>
        {tutorialLink && (
          <Button variant="primary" size="sm" href={tutorialLink}>
            Read Tutorial →
          </Button>
        )}
        {exampleLink && (
          <Button variant="tertiary" size="sm" href={exampleLink}>
            View Example
          </Button>
        )}
      </div>
    </div>
  );
}
