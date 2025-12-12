import type {ReactNode} from 'react';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';
import StructuredData from '@site/src/components/shared/StructuredData';
import Hero from '@site/src/components/marketing/Hero';
import StatsBar from '@site/src/components/marketing/StatsBar';
import CodeShowcase from '@site/src/components/marketing/CodeShowcase';
import HomepageFeatures from '@site/src/components/HomepageFeatures';
import Button from '@site/src/components/shared/Button';

import styles from './index.module.css';

export default function Home(): ReactNode {
  const {siteConfig} = useDocusaurusContext();

  return (
    <Layout
      title="Home"
      description="Build autonomous agents that actually work - Next-generation agentic orchestration platform">
      <StructuredData type="Organization" />
      <StructuredData type="SoftwareApplication" />
      <StructuredData type="WebSite" />
      <Hero
        tagline="Open Source AI Orchestration"
        title="Build Autonomous Agents That Actually Work"
        description={
          <>
            Radium is a next-generation orchestration platform that enables you to build,
            deploy, and manage multi-agent workflows with minimal code. Leverage intelligent
            agent selection, policy-driven execution, and seamless model integration.
          </>
        }
        primaryCta={{
          label: 'Get Started',
          href: '/docs/getting-started/installation',
        }}
        secondaryCta={{
          label: 'View Examples',
          href: '/docs/examples',
        }}
        githubUrl="https://github.com/clay-curry/RAD"
        showTerminal={true}
        terminalContent={`$ cargo install radium-cli

$ radium-cli init my-project
✓ Created project structure
✓ Generated agent configs
✓ Set up policy engine

$ radium-cli agents create code-reviewer \\
    --model claude-3-sonnet \\
    --persona expert-reviewer
✓ Agent created successfully

$ radium-cli chat code-reviewer
> Review the authentication module
🤖 Analyzing codebase...
✓ Found 3 security improvements
✓ Suggested 5 optimization opportunities
📊 Generated detailed report`}
      />

      <main>
        <StatsBar
          showGitHubStats={true}
          githubOwner="clay-curry"
          githubRepo="RAD"
          bordered={true}
        />

        <section className={styles.problemSolution}>
          <div className="container">
            <div className={styles.problemSolutionContent}>
              <h2 className="marketing-heading--lg marketing-text--center">
                Every Feature You Need, Nothing You Don't
              </h2>
              <p className="marketing-subheading">
                Traditional agent frameworks force you to write boilerplate, manage complex
                orchestration logic, and deal with vendor lock-in. Radium provides a
                batteries-included platform that handles the complexity so you can focus on
                building intelligent workflows.
              </p>
            </div>
          </div>
        </section>

        <HomepageFeatures />

        <CodeShowcase
          title="From Complexity to Clarity"
          description={
            <>
              Compare traditional multi-agent setup with Radium's declarative approach.
              Radium handles orchestration, error recovery, and agent selection automatically.
            </>
          }
          before={{
            title: 'Without Radium',
            subtitle: 'Manual orchestration, complex error handling',
            badge: { label: '120+ lines', variant: 'danger' },
            code: `// Manual agent orchestration
import { Agent, Task, Workflow } from 'agent-lib';

const codeAgent = new Agent({
  model: 'gpt-4',
  temperature: 0.2,
  maxTokens: 4000,
});

const reviewAgent = new Agent({
  model: 'claude-3-sonnet',
  temperature: 0.1,
  maxTokens: 8000,
});

async function orchestrateWorkflow(task: Task) {
  try {
    // Step 1: Code generation
    const codeResult = await codeAgent.run({
      prompt: task.description,
      context: await loadContext(),
    });

    if (!codeResult.success) {
      throw new Error('Code generation failed');
    }

    // Step 2: Code review
    const reviewResult = await reviewAgent.run({
      prompt: \`Review: \${codeResult.output}\`,
      context: codeResult.context,
    });

    // Step 3: Error recovery
    if (reviewResult.issues.length > 0) {
      const fixedCode = await codeAgent.run({
        prompt: \`Fix: \${reviewResult.issues}\`,
        context: reviewResult.context,
      });
      return fixedCode;
    }

    return reviewResult;
  } catch (error) {
    // Manual fallback logic
    console.error('Workflow failed:', error);
    throw error;
  }
}`,
          }}
          after={{
            title: 'With Radium',
            subtitle: 'Declarative workflow, automatic orchestration',
            badge: { label: '12 lines', variant: 'success' },
            code: `# agents/workflow.toml
[[agents]]
id = "code-generator"
model = "gpt-4"
persona = "expert-developer"

[[agents]]
id = "code-reviewer"
model = "claude-3-sonnet"
persona = "senior-reviewer"

[workflow]
steps = [
  { agent = "code-generator", input = "task.description" },
  { agent = "code-reviewer", input = "previous.output" }
]
policy = "require-approval"
error_recovery = "auto-retry"`,
          }}
        />

        <section className={styles.ctaSection}>
          <div className="container">
            <div className="marketing-cta">
              <h2 className="marketing-cta__heading">Ready to Get Started?</h2>
              <p className="marketing-cta__description">
                Install Radium in minutes and start building autonomous agent workflows today.
                Join our growing community of developers building the future of AI orchestration.
              </p>
              <div className="marketing-cta__buttons">
                <Button variant="primary" size="lg" href="/docs/getting-started/installation">
                  Install Radium
                </Button>
                <Button variant="secondary" size="lg" href="/docs/examples">
                  View Examples
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
