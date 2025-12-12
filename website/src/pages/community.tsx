import type {ReactNode} from 'react';
import Layout from '@theme/Layout';
import Button from '@site/src/components/shared/Button';
import Card from '@site/src/components/shared/Card';
import styles from './community.module.css';

const involvementPathways = [
  {
    title: 'Contribute Code',
    icon: '💻',
    description: 'Fix bugs, add features, or improve performance. Check out our good first issues.',
    links: [
      {label: 'Good First Issues', href: 'https://github.com/clay-curry/RAD/labels/good%20first%20issue'},
      {label: 'Contributing Guide', href: 'https://github.com/clay-curry/RAD/blob/main/CONTRIBUTING.md'},
    ],
  },
  {
    title: 'Join Discussions',
    icon: '💬',
    description: 'Share ideas, ask questions, and help others in the community.',
    links: [
      {label: 'GitHub Discussions', href: 'https://github.com/clay-curry/RAD/discussions'},
      {label: 'Report Issues', href: 'https://github.com/clay-curry/RAD/issues'},
    ],
  },
  {
    title: 'Share Extensions',
    icon: '🔌',
    description: 'Build and share custom agents, personas, and workflows with the community.',
    links: [
      {label: 'Extension Guide', href: '/docs/developer-guide/extension-system'},
      {label: 'Example Extensions', href: '/examples'},
    ],
  },
  {
    title: 'Improve Docs',
    icon: '📚',
    description: 'Help make our documentation better for everyone.',
    links: [
      {label: 'Documentation Issues', href: 'https://github.com/clay-curry/RAD/labels/documentation'},
      {label: 'Edit on GitHub', href: 'https://github.com/clay-curry/RAD/tree/main/docs'},
    ],
  },
];

const roadmapItems = [
  {
    quarter: 'Q1 2025',
    items: [
      'Enhanced multi-agent DAG execution',
      'Improved vibe check system',
      'Desktop app beta release',
    ],
  },
  {
    quarter: 'Q2 2025',
    items: [
      'Extension marketplace',
      'Advanced cost tracking',
      'Multi-cloud support expansion',
    ],
  },
  {
    quarter: 'Q3 2025',
    items: [
      'Visual workflow builder',
      'Team collaboration features',
      'Enterprise authentication',
    ],
  },
];

export default function Community(): ReactNode {
  return (
    <Layout
      title="Community"
      description="Join the Radium community of developers building autonomous agent workflows">
      <main className={styles.communityPage}>
        <section className={styles.hero}>
          <div className="container">
            <h1 className={styles.heroTitle}>Built by Developers, for Developers</h1>
            <p className={styles.heroSubtitle}>
              Radium is an open-source project driven by a passionate community of developers,
              contributors, and users building the future of autonomous agent orchestration.
            </p>
          </div>
        </section>

        <section className={styles.stats}>
          <div className="container">
            <div className={styles.statsGrid}>
              <div className={styles.statCard}>
                <div className={styles.statIcon}>⭐</div>
                <div className={styles.statNumber}>
                  <a
                    href="https://github.com/clay-curry/RAD"
                    target="_blank"
                    rel="noopener noreferrer"
                    className={styles.statLink}>
                    Star on GitHub →
                  </a>
                </div>
                <div className={styles.statLabel}>GitHub Repository</div>
              </div>
              <div className={styles.statCard}>
                <div className={styles.statIcon}>🤝</div>
                <div className={styles.statNumber}>
                  <a
                    href="https://github.com/clay-curry/RAD/graphs/contributors"
                    target="_blank"
                    rel="noopener noreferrer"
                    className={styles.statLink}>
                    View Contributors →
                  </a>
                </div>
                <div className={styles.statLabel}>Contributors</div>
              </div>
              <div className={styles.statCard}>
                <div className={styles.statIcon}>🚀</div>
                <div className={styles.statNumber}>
                  <a
                    href="https://github.com/clay-curry/RAD/releases"
                    target="_blank"
                    rel="noopener noreferrer"
                    className={styles.statLink}>
                    Latest Release →
                  </a>
                </div>
                <div className={styles.statLabel}>Releases</div>
              </div>
            </div>
          </div>
        </section>

        <section className={styles.involvement}>
          <div className="container">
            <h2 className={styles.sectionTitle}>Get Involved</h2>
            <p className={styles.sectionSubtitle}>
              There are many ways to contribute to Radium, regardless of your experience level.
            </p>
            <div className={styles.involvementGrid}>
              {involvementPathways.map((pathway) => (
                <Card key={pathway.title} className={styles.pathwayCard}>
                  <div className={styles.pathwayIcon}>{pathway.icon}</div>
                  <h3 className={styles.pathwayTitle}>{pathway.title}</h3>
                  <p className={styles.pathwayDescription}>{pathway.description}</p>
                  <div className={styles.pathwayLinks}>
                    {pathway.links.map((link) => (
                      <a
                        key={link.label}
                        href={link.href}
                        className={styles.pathwayLink}
                        target={link.href.startsWith('http') ? '_blank' : undefined}
                        rel={link.href.startsWith('http') ? 'noopener noreferrer' : undefined}>
                        {link.label} →
                      </a>
                    ))}
                  </div>
                </Card>
              ))}
            </div>
          </div>
        </section>

        <section className={styles.roadmap}>
          <div className="container">
            <h2 className={styles.sectionTitle}>Roadmap</h2>
            <p className={styles.sectionSubtitle}>
              Here's what we're working on next. Want to contribute? Check out our GitHub issues.
            </p>
            <div className={styles.roadmapGrid}>
              {roadmapItems.map((item) => (
                <div key={item.quarter} className={styles.roadmapCard}>
                  <h3 className={styles.roadmapQuarter}>{item.quarter}</h3>
                  <ul className={styles.roadmapList}>
                    {item.items.map((feature) => (
                      <li key={feature} className={styles.roadmapItem}>
                        {feature}
                      </li>
                    ))}
                  </ul>
                </div>
              ))}
            </div>
          </div>
        </section>

        <section className={styles.channels}>
          <div className="container">
            <h2 className={styles.sectionTitle}>Communication Channels</h2>
            <p className={styles.sectionSubtitle}>
              Connect with the Radium community and stay updated on the latest developments.
            </p>
            <div className={styles.channelsGrid}>
              <Card className={styles.channelCard}>
                <h3 className={styles.channelTitle}>GitHub</h3>
                <p className={styles.channelDescription}>
                  View source code, report issues, and contribute to development.
                </p>
                <Button
                  variant="tertiary"
                  size="sm"
                  href="https://github.com/clay-curry/RAD"
                  target="_blank"
                  rel="noopener noreferrer">
                  Visit Repository
                </Button>
              </Card>
              <Card className={styles.channelCard}>
                <h3 className={styles.channelTitle}>Discussions</h3>
                <p className={styles.channelDescription}>
                  Ask questions, share ideas, and help other community members.
                </p>
                <Button
                  variant="tertiary"
                  size="sm"
                  href="https://github.com/clay-curry/RAD/discussions"
                  target="_blank"
                  rel="noopener noreferrer">
                  Join Discussions
                </Button>
              </Card>
              <Card className={styles.channelCard}>
                <h3 className={styles.channelTitle}>Issues</h3>
                <p className={styles.channelDescription}>
                  Report bugs, request features, or help triage existing issues.
                </p>
                <Button
                  variant="tertiary"
                  size="sm"
                  href="https://github.com/clay-curry/RAD/issues"
                  target="_blank"
                  rel="noopener noreferrer">
                  View Issues
                </Button>
              </Card>
            </div>
          </div>
        </section>

        <section className={styles.conduct}>
          <div className="container">
            <div className={styles.conductCard}>
              <h2 className={styles.conductTitle}>Code of Conduct</h2>
              <p className={styles.conductDescription}>
                We are committed to providing a welcoming and inclusive environment for everyone.
                Please read and follow our Code of Conduct when participating in the Radium
                community.
              </p>
              <Button
                variant="secondary"
                href="https://github.com/clay-curry/RAD/blob/main/CODE_OF_CONDUCT.md"
                target="_blank"
                rel="noopener noreferrer">
                Read Code of Conduct
              </Button>
            </div>
          </div>
        </section>

        <section className={styles.ctaSection}>
          <div className="container">
            <div className="marketing-cta">
              <h2 className="marketing-cta__heading">Start Building with Radium</h2>
              <p className="marketing-cta__description">
                Install Radium and join the community building the next generation of autonomous
                agent systems.
              </p>
              <div className="marketing-cta__buttons">
                <Button variant="primary" size="lg" href="/docs/getting-started/installation">
                  Get Started
                </Button>
                <Button variant="tertiary" size="lg" href="https://github.com/clay-curry/RAD">
                  GitHub
                </Button>
              </div>
            </div>
          </div>
        </section>
      </main>
    </Layout>
  );
}
