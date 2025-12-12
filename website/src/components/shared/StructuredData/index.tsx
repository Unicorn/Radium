import Head from '@docusaurus/Head';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import type {ReactNode} from 'react';

export interface StructuredDataProps {
  type: 'Organization' | 'SoftwareApplication' | 'WebSite';
  data?: Record<string, any>;
}

export default function StructuredData({type, data}: StructuredDataProps): ReactNode {
  const {siteConfig} = useDocusaurusContext();

  const getStructuredData = () => {
    const baseUrl = `${siteConfig.url}${siteConfig.baseUrl}`;

    switch (type) {
      case 'Organization':
        return {
          '@context': 'https://schema.org',
          '@type': 'Organization',
          name: 'Radium Project',
          description: siteConfig.tagline,
          url: baseUrl,
          logo: `${baseUrl}img/logo.png`,
          sameAs: [
            'https://github.com/clay-curry/RAD',
          ],
          contactPoint: {
            '@type': 'ContactPoint',
            contactType: 'Community Support',
            url: 'https://github.com/clay-curry/RAD/discussions',
          },
          ...data,
        };

      case 'SoftwareApplication':
        return {
          '@context': 'https://schema.org',
          '@type': 'SoftwareApplication',
          name: 'Radium',
          applicationCategory: 'DeveloperApplication',
          description: 'Next-generation agentic orchestration platform for building autonomous multi-agent workflows',
          operatingSystem: 'Linux, macOS, Windows',
          offers: {
            '@type': 'Offer',
            price: '0',
            priceCurrency: 'USD',
          },
          downloadUrl: 'https://github.com/clay-curry/RAD',
          softwareVersion: 'Latest',
          applicationSubCategory: 'AI Agent Orchestration',
          keywords: 'autonomous agents, multi-agent workflows, agent orchestration, AI orchestration, vibe check, policy engine',
          author: {
            '@type': 'Organization',
            name: 'Radium Project',
            url: baseUrl,
          },
          ...data,
        };

      case 'WebSite':
        return {
          '@context': 'https://schema.org',
          '@type': 'WebSite',
          name: siteConfig.title,
          description: siteConfig.tagline,
          url: baseUrl,
          potentialAction: {
            '@type': 'SearchAction',
            target: {
              '@type': 'EntryPoint',
              urlTemplate: `${baseUrl}search?q={search_term_string}`,
            },
            'query-input': 'required name=search_term_string',
          },
          ...data,
        };

      default:
        return null;
    }
  };

  const structuredData = getStructuredData();

  if (!structuredData) {
    return null;
  }

  return (
    <Head>
      <script type="application/ld+json">{JSON.stringify(structuredData)}</script>
    </Head>
  );
}
