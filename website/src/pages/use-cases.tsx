import {useState, useMemo, type ReactNode} from 'react';
import Layout from '@theme/Layout';
import SEOHead from '@site/src/components/shared/SEOHead';
import FilterBar from '@site/src/components/marketing/FilterBar';
import UseCaseCard from '@site/src/components/marketing/UseCaseCard';
import Button from '@site/src/components/shared/Button';
import {useCases, categories, type UseCaseCategory} from '@site/src/data/useCases';
import styles from './use-cases.module.css';

export default function UseCases(): ReactNode {
  const [selectedCategory, setSelectedCategory] = useState<string | null>(null);

  // Calculate category counts
  const categoryCounts = useMemo(() => {
    const counts: Record<string, number> = {};
    useCases.forEach((useCase) => {
      counts[useCase.category] = (counts[useCase.category] || 0) + 1;
    });
    return counts;
  }, []);

  // Filter use cases based on selected category
  const filteredUseCases = useMemo(() => {
    return useCases.filter((useCase) => {
      return !selectedCategory || useCase.category === selectedCategory;
    });
  }, [selectedCategory]);

  // Prepare category filter options
  const categoryOptions = Object.entries(categories).map(([key, {label}]) => ({
    value: key,
    label,
    count: categoryCounts[key] || 0,
  }));

  return (
    <Layout
      title="Use Cases"
      description="Real-world applications and success stories using Radium for autonomous agent orchestration">
      <SEOHead
        title="Use Cases"
        description="Discover real-world applications using Radium for autonomous agent orchestration - CI/CD automation, ETL pipelines, cloud cost optimization, security auditing, and more"
        keywords={[
          'radium use cases',
          'agent automation examples',
          'CI/CD automation',
          'ETL pipeline orchestration',
          'cloud cost optimization',
          'security automation',
          'infrastructure automation',
          'development automation',
        ]}
      />
      <main className={styles.useCasesPage}>
        <section className={styles.hero}>
          <div className="container">
            <h1 className={styles.heroTitle}>Real-World Use Cases</h1>
            <p className={styles.heroSubtitle}>
              Discover how teams are using Radium to automate complex workflows, reduce operational
              costs, and accelerate development cycles across industries.
            </p>
          </div>
        </section>

        <section className={styles.content}>
          <div className="container">
            <FilterBar
              categories={categoryOptions}
              difficulties={[]}
              selectedCategory={selectedCategory}
              selectedDifficulty={null}
              onCategoryChange={setSelectedCategory}
              onDifficultyChange={() => {}}
            />

            <div className={styles.resultsHeader}>
              <p className={styles.resultsCount}>
                Showing {filteredUseCases.length} of {useCases.length} use cases
              </p>
            </div>

            {filteredUseCases.length > 0 ? (
              <div className={styles.useCasesGrid}>
                {filteredUseCases.map((useCase) => (
                  <UseCaseCard key={useCase.id} {...useCase} />
                ))}
              </div>
            ) : (
              <div className={styles.noResults}>
                <p>No use cases match your filters. Try adjusting your selection.</p>
                <Button
                  variant="secondary"
                  onClick={() => {
                    setSelectedCategory(null);
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
              <h2 className="marketing-cta__heading">Build Your Own Use Case</h2>
              <p className="marketing-cta__description">
                Start building your own autonomous agent workflows with Radium. Follow our
                comprehensive guides and examples to implement solutions tailored to your needs.
              </p>
              <div className="marketing-cta__buttons">
                <Button variant="primary" size="lg" href="/docs/getting-started/installation">
                  Get Started
                </Button>
                <Button variant="secondary" size="lg" href="/examples">
                  View Examples
                </Button>
                <Button variant="tertiary" size="lg" href="/docs/user-guide/user-guide-overview">
                  Read the Docs
                </Button>
              </div>
            </div>
          </div>
        </section>
      </main>
    </Layout>
  );
}
