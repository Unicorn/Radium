import type {ReactNode} from 'react';
import styles from './styles.module.css';

type SupportLevel = 'full' | 'partial' | 'none';

interface ComparisonFeature {
  feature: string;
  radium: SupportLevel;
  langchain: SupportLevel;
  autogpt: SupportLevel;
  crewai: SupportLevel;
  description?: string;
}

const comparisonData: ComparisonFeature[] = [
  {
    feature: 'Multi-Agent Orchestration',
    radium: 'full',
    langchain: 'partial',
    autogpt: 'partial',
    crewai: 'full',
    description: 'Coordinate multiple agents in complex workflows',
  },
  {
    feature: 'Policy Engine',
    radium: 'full',
    langchain: 'none',
    autogpt: 'none',
    crewai: 'none',
    description: 'Fine-grained control over agent behavior',
  },
  {
    feature: 'Self-Hosted Models',
    radium: 'full',
    langchain: 'full',
    autogpt: 'partial',
    crewai: 'partial',
    description: 'Run models locally with Ollama',
  },
  {
    feature: 'DAG Workflows',
    radium: 'full',
    langchain: 'partial',
    autogpt: 'none',
    crewai: 'partial',
    description: 'Define complex dependencies between tasks',
  },
  {
    feature: 'Cost Tracking',
    radium: 'full',
    langchain: 'partial',
    autogpt: 'none',
    crewai: 'none',
    description: 'Built-in cost monitoring and budgets',
  },
  {
    feature: 'TOML Configuration',
    radium: 'full',
    langchain: 'none',
    autogpt: 'none',
    crewai: 'partial',
    description: 'Declarative, code-free agent setup',
  },
  {
    feature: 'Multiple Interfaces',
    radium: 'full',
    langchain: 'none',
    autogpt: 'none',
    crewai: 'none',
    description: 'CLI, TUI, and Desktop app included',
  },
  {
    feature: 'Vibe Check Oversight',
    radium: 'full',
    langchain: 'none',
    autogpt: 'none',
    crewai: 'none',
    description: 'Metacognitive monitoring of agent behavior',
  },
];

const SupportIcon = ({level}: {level: SupportLevel}): ReactNode => {
  switch (level) {
    case 'full':
      return <span className={`${styles.icon} ${styles.iconFull}`}>✓</span>;
    case 'partial':
      return <span className={`${styles.icon} ${styles.iconPartial}`}>~</span>;
    case 'none':
      return <span className={`${styles.icon} ${styles.iconNone}`}>×</span>;
  }
};

export default function ComparisonTable(): ReactNode {
  return (
    <div className={styles.comparisonContainer}>
      <h2 className={styles.title}>How Radium Compares</h2>
      <p className={styles.subtitle}>
        See how Radium stacks up against popular agent frameworks
      </p>

      <div className={styles.tableWrapper}>
        <table className={styles.comparisonTable}>
          <thead>
            <tr>
              <th className={styles.headerFeature}>Feature</th>
              <th className={`${styles.headerProduct} ${styles.headerRadium}`}>
                <div className={styles.productName}>Radium</div>
              </th>
              <th className={styles.headerProduct}>
                <div className={styles.productName}>LangChain</div>
              </th>
              <th className={styles.headerProduct}>
                <div className={styles.productName}>AutoGPT</div>
              </th>
              <th className={styles.headerProduct}>
                <div className={styles.productName}>CrewAI</div>
              </th>
            </tr>
          </thead>
          <tbody>
            {comparisonData.map((row, idx) => (
              <tr key={idx}>
                <td className={styles.cellFeature}>
                  <div className={styles.featureName}>{row.feature}</div>
                  {row.description && (
                    <div className={styles.featureDescription}>{row.description}</div>
                  )}
                </td>
                <td className={`${styles.cellSupport} ${styles.cellRadium}`}>
                  <SupportIcon level={row.radium} />
                </td>
                <td className={styles.cellSupport}>
                  <SupportIcon level={row.langchain} />
                </td>
                <td className={styles.cellSupport}>
                  <SupportIcon level={row.autogpt} />
                </td>
                <td className={styles.cellSupport}>
                  <SupportIcon level={row.crewai} />
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <div className={styles.legend}>
        <div className={styles.legendItem}>
          <SupportIcon level="full" />
          <span>Full Support</span>
        </div>
        <div className={styles.legendItem}>
          <SupportIcon level="partial" />
          <span>Partial Support</span>
        </div>
        <div className={styles.legendItem}>
          <SupportIcon level="none" />
          <span>Not Supported</span>
        </div>
      </div>
    </div>
  );
}
