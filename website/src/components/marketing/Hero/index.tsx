import type {ReactNode} from 'react';
import Button from '@site/src/components/shared/Button';
import styles from './styles.module.css';

export interface HeroProps {
  title: string;
  tagline?: string;
  description?: ReactNode;
  primaryCta?: {
    label: string;
    href: string;
  };
  secondaryCta?: {
    label: string;
    href: string;
  };
  githubStars?: number;
  githubUrl?: string;
  showTerminal?: boolean;
  terminalContent?: string;
}

export default function Hero({
  title,
  tagline,
  description,
  primaryCta,
  secondaryCta,
  githubStars,
  githubUrl,
  showTerminal = false,
  terminalContent,
}: HeroProps): ReactNode {
  return (
    <section className={styles.hero}>
      <div className={styles.heroContent}>
        <div className={styles.heroText}>
          {tagline && <div className={styles.tagline}>{tagline}</div>}

          <h1 className={styles.title}>{title}</h1>

          {description && (
            <div className={styles.description}>{description}</div>
          )}

          <div className={styles.ctaButtons}>
            {primaryCta && (
              <Button
                variant="primary"
                size="lg"
                href={primaryCta.href}
                className={styles.primaryCta}
              >
                {primaryCta.label}
              </Button>
            )}

            {secondaryCta && (
              <Button
                variant="secondary"
                size="lg"
                href={secondaryCta.href}
              >
                {secondaryCta.label}
              </Button>
            )}

            {githubUrl && (
              <a
                href={githubUrl}
                target="_blank"
                rel="noopener noreferrer"
                className={styles.githubButton}
              >
                <svg
                  className={styles.githubIcon}
                  viewBox="0 0 16 16"
                  fill="currentColor"
                  aria-hidden="true"
                >
                  <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z" />
                </svg>
                <span>Star on GitHub</span>
                {githubStars !== undefined && (
                  <span className={styles.starCount}>
                    {githubStars.toLocaleString()}
                  </span>
                )}
              </a>
            )}
          </div>
        </div>

        {showTerminal && terminalContent && (
          <div className={styles.heroTerminal}>
            <div className={styles.terminalHeader}>
              <div className={styles.terminalButtons}>
                <span className={styles.terminalButton} />
                <span className={styles.terminalButton} />
                <span className={styles.terminalButton} />
              </div>
              <div className={styles.terminalTitle}>terminal</div>
            </div>
            <div className={styles.terminalBody}>
              <pre className={styles.terminalCode}>
                <code>{terminalContent}</code>
              </pre>
            </div>
          </div>
        )}
      </div>
    </section>
  );
}
