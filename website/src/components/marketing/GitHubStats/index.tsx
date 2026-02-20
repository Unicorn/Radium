import {useState, useEffect, type ReactNode} from 'react';
import styles from './styles.module.css';

interface GitHubRepoData {
  stars: number;
  forks: number;
  openIssues: number;
  watchers: number;
}

interface GitHubContributorsData {
  contributors: number;
}

interface GitHubReleaseData {
  latestRelease: string;
  releaseDate: string;
}

interface GitHubStatsProps {
  owner: string;
  repo: string;
  showRelease?: boolean;
  className?: string;
}

export default function GitHubStats({
  owner,
  repo,
  showRelease = false,
  className,
}: GitHubStatsProps): ReactNode {
  const [repoData, setRepoData] = useState<GitHubRepoData | null>(null);
  const [contributorsData, setContributorsData] = useState<GitHubContributorsData | null>(null);
  const [releaseData, setReleaseData] = useState<GitHubReleaseData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchGitHubData = async () => {
      try {
        setLoading(true);

        // Fetch repository data
        const repoResponse = await fetch(`https://api.github.com/repos/${owner}/${repo}`);
        if (!repoResponse.ok) {
          throw new Error('Failed to fetch repository data');
        }
        const repoJson = await repoResponse.json();

        setRepoData({
          stars: repoJson.stargazers_count || 0,
          forks: repoJson.forks_count || 0,
          openIssues: repoJson.open_issues_count || 0,
          watchers: repoJson.subscribers_count || 0,
        });

        // Fetch contributors count
        const contributorsResponse = await fetch(
          `https://api.github.com/repos/${owner}/${repo}/contributors?per_page=1`
        );
        if (contributorsResponse.ok) {
          const linkHeader = contributorsResponse.headers.get('Link');
          let contributorsCount = 0;

          if (linkHeader) {
            const match = linkHeader.match(/page=(\d+)>; rel="last"/);
            contributorsCount = match ? parseInt(match[1], 10) : 1;
          } else {
            const contributorsJson = await contributorsResponse.json();
            contributorsCount = contributorsJson.length;
          }

          setContributorsData({ contributors: contributorsCount });
        }

        // Fetch latest release if requested
        if (showRelease) {
          const releaseResponse = await fetch(
            `https://api.github.com/repos/${owner}/${repo}/releases/latest`
          );
          if (releaseResponse.ok) {
            const releaseJson = await releaseResponse.json();
            setReleaseData({
              latestRelease: releaseJson.tag_name || 'N/A',
              releaseDate: releaseJson.published_at
                ? new Date(releaseJson.published_at).toLocaleDateString()
                : 'N/A',
            });
          }
        }

        setLoading(false);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to fetch GitHub data');
        setLoading(false);
      }
    };

    fetchGitHubData();
  }, [owner, repo, showRelease]);

  if (loading) {
    return (
      <div className={`${styles.githubStats} ${className || ''}`}>
        <div className={styles.loading}>Loading GitHub stats...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className={`${styles.githubStats} ${className || ''}`}>
        <div className={styles.error}>Unable to load GitHub stats</div>
      </div>
    );
  }

  return (
    <div className={`${styles.githubStats} ${className || ''}`}>
      {repoData && (
        <>
          <div className={styles.stat}>
            <div className={styles.statValue}>{repoData.stars.toLocaleString()}</div>
            <div className={styles.statLabel}>Stars</div>
          </div>
          <div className={styles.stat}>
            <div className={styles.statValue}>{repoData.forks.toLocaleString()}</div>
            <div className={styles.statLabel}>Forks</div>
          </div>
        </>
      )}

      {contributorsData && (
        <div className={styles.stat}>
          <div className={styles.statValue}>
            {contributorsData.contributors.toLocaleString()}
          </div>
          <div className={styles.statLabel}>Contributors</div>
        </div>
      )}

      {repoData && (
        <div className={styles.stat}>
          <div className={styles.statValue}>{repoData.openIssues.toLocaleString()}</div>
          <div className={styles.statLabel}>Open Issues</div>
        </div>
      )}

      {showRelease && releaseData && (
        <div className={styles.stat}>
          <div className={styles.statValue}>{releaseData.latestRelease}</div>
          <div className={styles.statLabel}>Latest Release</div>
        </div>
      )}
    </div>
  );
}
