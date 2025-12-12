import type {ReactNode} from 'react';
import GitHubStats from '@site/src/components/marketing/GitHubStats';
import styles from './styles.module.css';

export interface Stat {
  value: string | number;
  label: string;
  suffix?: string;
}

export interface StatsBarProps {
  stats?: Stat[];
  githubOwner?: string;
  githubRepo?: string;
  showGitHubStats?: boolean;
  showRelease?: boolean;
  bordered?: boolean;
  className?: string;
}

export default function StatsBar({
  stats,
  githubOwner,
  githubRepo,
  showGitHubStats = false,
  showRelease = false,
  bordered = true,
  className,
}: StatsBarProps): ReactNode {
  // If GitHub stats are requested, use GitHubStats component
  if (showGitHubStats && githubOwner && githubRepo) {
    return (
      <div className={`${styles.statsBar} ${bordered ? styles.bordered : ''} ${className || ''}`}>
        <GitHubStats
          owner={githubOwner}
          repo={githubRepo}
          showRelease={showRelease}
        />
      </div>
    );
  }

  // Otherwise, render custom stats
  if (!stats || stats.length === 0) {
    return null;
  }

  return (
    <div className={`${styles.statsBar} ${bordered ? styles.bordered : ''} ${className || ''}`}>
      {stats.map((stat, index) => (
        <div key={index} className={styles.stat}>
          <div className={styles.statValue}>
            {stat.value}
            {stat.suffix && <span className={styles.statSuffix}>{stat.suffix}</span>}
          </div>
          <div className={styles.statLabel}>{stat.label}</div>
        </div>
      ))}
    </div>
  );
}
