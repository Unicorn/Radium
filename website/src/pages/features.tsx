import type {ReactNode} from 'react';
import Layout from '@theme/Layout';
import SEOHead from '@site/src/components/shared/SEOHead';
import TabInterface from '@site/src/components/marketing/TabInterface';
import FeatureCard from '@site/src/components/marketing/FeatureCard';
import ComparisonTable from '@site/src/components/marketing/ComparisonTable';
import Button from '@site/src/components/shared/Button';
import {featureCategories} from '@site/src/data/features';
import styles from './features.module.css';

export default function Features(): ReactNode {
  const tabs = featureCategories.map((category) => ({
    id: category.id,
    label: category.title,
    content: (
      <div className={styles.categoryContent}>
        <p className={styles.categoryDescription}>{category.description}</p>
        <div className={styles.featureGrid}>
          {category.features.map((feature) => (
            <FeatureCard key={feature.id} {...feature} />
          ))}
        </div>
      </div>
    ),
  }));

  return (
    <Layout
      title="Features"
      description="Explore Radium's powerful features for building autonomous agent workflows">
      <SEOHead
        title="Features"
        description="Explore Radium's comprehensive features for building autonomous agent workflows - intelligent orchestration, multi-agent DAGs, policy engine, vibe check system, and more"
        keywords={[
          'radium features',
          'autonomous agents',
          'agent orchestration',
          'multi-agent workflows',
          'AI agent platform',
          'agent policy engine',
          'vibe check',
          'metacognitive oversight',
        ]}
      />
      <main className={styles.featuresPage}>
        <section className={styles.hero}>
          <div className="container">
            <h1 className={styles.heroTitle}>Every Feature You Need</h1>
            <p className={styles.heroSubtitle}>
              Radium provides a comprehensive platform for building, deploying, and managing
              multi-agent workflows. From intelligent orchestration to fine-grained security
              controls, every feature is designed for production use.
            </p>
          </div>
        </section>

        <section className={styles.tabsSection}>
          <div className="container">
            <TabInterface tabs={tabs} defaultTab="orchestration" />
          </div>
        </section>

        <section className={styles.comparisonSection}>
          <div className="container">
            <ComparisonTable />
          </div>
        </section>

        <section className={styles.ctaSection}>
          <div className="container">
            <div className="marketing-cta">
              <h2 className="marketing-cta__heading">Ready to Explore?</h2>
              <p className="marketing-cta__description">
                Try Radium's features yourself or explore our comprehensive documentation to learn
                more about building autonomous agent workflows.
              </p>
              <div className="marketing-cta__buttons">
                <Button variant="primary" size="lg" href="/docs/getting-started/installation">
                  Get Started
                </Button>
                <Button variant="secondary" size="lg" href="/docs/user-guide/user-guide-overview">
                  Read the Docs
                </Button>
                <Button variant="tertiary" size="lg" href="/docs/examples">
                  View Examples
                </Button>
              </div>
            </div>
          </div>
        </section>
      </main>
    </Layout>
  );
}
