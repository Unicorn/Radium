import {useState, type ReactNode} from 'react';
import type {Example} from '@site/src/data/examples';
import Badge from '@site/src/components/shared/Badge';
import Button from '@site/src/components/shared/Button';
import styles from './styles.module.css';

export interface ExampleCardProps extends Example {
  className?: string;
}

const difficultyVariant = {
  beginner: 'success' as const,
  intermediate: 'warning' as const,
  advanced: 'danger' as const,
};

export default function ExampleCard({
  title,
  description,
  category,
  difficulty,
  timeEstimate,
  tags,
  codeSnippet,
  docLink,
  githubLink,
  className,
}: ExampleCardProps): ReactNode {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    if (codeSnippet) {
      await navigator.clipboard.writeText(codeSnippet);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  return (
    <div className={`${styles.exampleCard} ${className || ''}`}>
      <div className={styles.cardHeader}>
        <div className={styles.badges}>
          <Badge variant={difficultyVariant[difficulty]} size="sm">
            {difficulty}
          </Badge>
          <Badge variant="default" size="sm">
            {timeEstimate}
          </Badge>
        </div>
        <h3 className={styles.cardTitle}>{title}</h3>
      </div>

      <div className={styles.cardBody}>
        <p className={styles.description}>{description}</p>

        {tags && tags.length > 0 && (
          <div className={styles.tags}>
            {tags.map((tag) => (
              <span key={tag} className={styles.tag}>
                {tag}
              </span>
            ))}
          </div>
        )}

        {codeSnippet && (
          <div className={styles.codeSection}>
            <div className={styles.codeHeader}>
              <span className={styles.codeLabel}>Example Code</span>
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
        <Button variant="primary" size="sm" href={docLink}>
          View Tutorial →
        </Button>
        {githubLink && (
          <Button variant="tertiary" size="sm" href={githubLink}>
            GitHub
          </Button>
        )}
      </div>
    </div>
  );
}
