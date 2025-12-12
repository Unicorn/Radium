import type {ReactNode} from 'react';
import clsx from 'clsx';
import Heading from '@theme/Heading';
import styles from './styles.module.css';

type FeatureItem = {
  title: string;
  Svg: React.ComponentType<React.ComponentProps<'svg'>>;
  description: ReactNode;
};

const FeatureList: FeatureItem[] = [
  {
    title: 'Autonomous Orchestration',
    Svg: require('@site/static/img/undraw_docusaurus_mountain.svg').default,
    description: (
      <>
        YOLO mode enables fully autonomous execution from high-level goals to
        complete implementations. Intelligent agent selection, resource allocation,
        and error recovery with multi-agent coordination.
      </>
    ),
  },
  {
    title: 'Multi-Agent Workflows',
    Svg: require('@site/static/img/undraw_docusaurus_tree.svg').default,
    description: (
      <>
        Define complex DAG workflows with behavior orchestration, checkpoint
        recovery, and policy-driven execution. Specialized agents collaborate
        to tackle multi-step tasks with full observability.
      </>
    ),
  },
  {
    title: 'Provider Agnostic',
    Svg: require('@site/static/img/undraw_docusaurus_react.svg').default,
    description: (
      <>
        Seamlessly switch between Claude, OpenAI, Gemini, and self-hosted models.
        Built-in cost tracking, quota management, and intelligent fallback chains
        keep your workflows running efficiently.
      </>
    ),
  },
];

function Feature({title, Svg, description}: FeatureItem) {
  return (
    <div className={clsx('col col--4')}>
      <div className="text--center">
        <Svg className={styles.featureSvg} role="img" />
      </div>
      <div className="text--center padding-horiz--md">
        <Heading as="h3">{title}</Heading>
        <p>{description}</p>
      </div>
    </div>
  );
}

export default function HomepageFeatures(): ReactNode {
  return (
    <section className={styles.features}>
      <div className="container">
        <div className="row">
          {FeatureList.map((props, idx) => (
            <Feature key={idx} {...props} />
          ))}
        </div>
      </div>
    </section>
  );
}
