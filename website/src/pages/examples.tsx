import {useState, useMemo, type ReactNode} from 'react';
import Layout from '@theme/Layout';
import SEOHead from '@site/src/components/shared/SEOHead';
import FilterBar from '@site/src/components/marketing/FilterBar';
import ExampleCard from '@site/src/components/marketing/ExampleCard';
import Button from '@site/src/components/shared/Button';
import {
  examples,
  categories,
  difficultyLabels,
  type ExampleCategory,
  type DifficultyLevel,
} from '@site/src/data/examples';
import styles from './examples.module.css';

export default function Examples(): ReactNode {
  const [selectedCategory, setSelectedCategory] = useState<string | null>(null);
  const [selectedDifficulty, setSelectedDifficulty] = useState<string | null>(null);

  // Calculate category counts
  const categoryCounts = useMemo(() => {
    const counts: Record<string, number> = {};
    examples.forEach((example) => {
      counts[example.category] = (counts[example.category] || 0) + 1;
    });
    return counts;
  }, []);

  // Calculate difficulty counts
  const difficultyCounts = useMemo(() => {
    const counts: Record<string, number> = {};
    examples.forEach((example) => {
      counts[example.difficulty] = (counts[example.difficulty] || 0) + 1;
    });
    return counts;
  }, []);

  // Filter examples based on selected filters
  const filteredExamples = useMemo(() => {
    return examples.filter((example) => {
      const categoryMatch = !selectedCategory || example.category === selectedCategory;
      const difficultyMatch = !selectedDifficulty || example.difficulty === selectedDifficulty;
      return categoryMatch && difficultyMatch;
    });
  }, [selectedCategory, selectedDifficulty]);

  // Prepare filter options
  const categoryOptions = Object.entries(categories).map(([key, {label}]) => ({
    value: key,
    label,
    count: categoryCounts[key] || 0,
  }));

  const difficultyOptions = Object.entries(difficultyLabels).map(([key, label]) => ({
    value: key,
    label,
    count: difficultyCounts[key] || 0,
  }));

  return (
    <Layout
      title="Examples"
      description="Explore real-world examples and tutorials for building with Radium">
      <SEOHead
        title="Examples"
        description="Learn by example with practical tutorials for building autonomous agent workflows using Radium - from beginner to advanced, including workflows, integrations, and automation"
        keywords={[
          'radium examples',
          'agent workflow examples',
          'radium tutorials',
          'code review automation',
          'data pipeline examples',
          'CI/CD agent examples',
          'multi-agent workflow tutorial',
          'agent integration examples',
        ]}
      />
      <main className={styles.examplesPage}>
        <section className={styles.hero}>
          <div className="container">
            <h1 className={styles.heroTitle}>Learn by Example</h1>
            <p className={styles.heroSubtitle}>
              Practical examples and tutorials to help you build autonomous agent workflows with
              Radium. Filter by category and difficulty to find exactly what you need.
            </p>
          </div>
        </section>

        <section className={styles.content}>
          <div className="container">
            <FilterBar
              categories={categoryOptions}
              difficulties={difficultyOptions}
              selectedCategory={selectedCategory}
              selectedDifficulty={selectedDifficulty}
              onCategoryChange={setSelectedCategory}
              onDifficultyChange={setSelectedDifficulty}
            />

            <div className={styles.resultsHeader}>
              <p className={styles.resultsCount}>
                Showing {filteredExamples.length} of {examples.length} examples
              </p>
            </div>

            {filteredExamples.length > 0 ? (
              <div className={styles.examplesGrid}>
                {filteredExamples.map((example) => (
                  <ExampleCard key={example.id} {...example} />
                ))}
              </div>
            ) : (
              <div className={styles.noResults}>
                <p>No examples match your filters. Try adjusting your selection.</p>
                <Button
                  variant="secondary"
                  onClick={() => {
                    setSelectedCategory(null);
                    setSelectedDifficulty(null);
                  }}>
                  Clear Filters
                </Button>
              </div>
            )}
          </div>
        </section>

        <section className={styles.ctaSection}>
          <div className="container">
            <div className="marketing-cta">
              <h2 className="marketing-cta__heading">Ready to Build?</h2>
              <p className="marketing-cta__description">
                Install Radium and start building your own autonomous agent workflows. Check out our
                comprehensive documentation for detailed guides and API references.
              </p>
              <div className="marketing-cta__buttons">
                <Button variant="primary" size="lg" href="/docs/getting-started/installation">
                  Get Started
                </Button>
                <Button variant="secondary" size="lg" href="/docs/user-guide/user-guide-overview">
                  Read the Docs
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
