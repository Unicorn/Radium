import {themes as prismThemes} from 'prism-react-renderer';
import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';

// This runs in Node.js - Don't use client-side code here (browser APIs, JSX...)

const config: Config = {
  title: 'Radium',
  tagline: 'Next-generation agentic orchestration platform',
  favicon: 'img/favicon.ico',

  // Future flags, see https://docusaurus.io/docs/api/docusaurus-config#future
  future: {
    v4: true, // Improve compatibility with the upcoming Docusaurus v4
  },

  // Set the production url of your site here
  url: 'https://clay-curry.github.io',
  // Set the /<baseUrl>/ pathname under which your site is served
  // For GitHub pages deployment, it is often '/<projectName>/'
  baseUrl: '/RAD/',

  // GitHub pages deployment config.
  // If you aren't using GitHub pages, you don't need these.
  organizationName: 'clay-curry', // Usually your GitHub org/user name.
  projectName: 'RAD', // Usually your repo name.

  onBrokenLinks: 'warn', // Set to 'throw' after verifying all links are working

  // Even if you don't use internationalization, you can use this field to set
  // useful metadata like html lang. For example, if your site is Chinese, you
  // may want to replace "en" with "zh-Hans".
  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  presets: [
    [
      'classic',
      {
        docs: {
          sidebarPath: './sidebars.ts',
          // Edit page links point to the docs folder in the main repo
          editUrl: 'https://github.com/clay-curry/RAD/tree/main/docs/',
        },
        blog: false,
        theme: {
          customCss: ['./src/css/custom.css', './src/css/marketing.css'],
        },
      } satisfies Preset.Options,
    ],
  ],

  themeConfig: {
    image: 'img/radium-social-card.png',
    colorMode: {
      respectPrefersColorScheme: true,
    },
    navbar: {
      title: 'Radium',
      logo: {
        alt: 'Radium Logo',
        src: 'img/logo.png',
      },
      items: [
        {
          type: 'docSidebar',
          sidebarId: 'docsSidebar',
          position: 'left',
          label: 'Documentation',
        },
        {
          to: '/features',
          label: 'Features',
          position: 'left',
        },
        {
          to: '/examples',
          label: 'Examples',
          position: 'left',
        },
        {
          to: '/docs/api/radium_core',
          label: 'API Reference',
          position: 'left',
        },
        {
          type: 'docsVersionDropdown',
          position: 'right',
        },
        {
          href: 'https://github.com/clay-curry/RAD',
          label: 'GitHub',
          position: 'right',
        },
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Documentation',
          items: [
            {
              label: 'Getting Started',
              to: '/docs/getting-started/installation',
            },
            {
              label: 'User Guide',
              to: '/docs/user-guide/user-guide-overview',
            },
            {
              label: 'Developer Guide',
              to: '/docs/developer-guide/developer-guide-overview',
            },
          ],
        },
        {
          title: 'Resources',
          items: [
            {
              label: 'Examples',
              to: '/docs/examples/orchestration-workflows',
            },
            {
              label: 'API Reference',
              to: '/docs/api/radium_core',
            },
            {
              label: 'CLI Reference',
              to: '/docs/cli/README',
            },
          ],
        },
        {
          title: 'Community',
          items: [
            {
              label: 'GitHub',
              href: 'https://github.com/clay-curry/RAD',
            },
            {
              label: 'Issues',
              href: 'https://github.com/clay-curry/RAD/issues',
            },
            {
              label: 'Discussions',
              href: 'https://github.com/clay-curry/RAD/discussions',
            },
          ],
        },
      ],
      copyright: `Copyright © ${new Date().getFullYear()} Radium Project. Built with Docusaurus.`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
      additionalLanguages: ['rust', 'toml', 'bash', 'json', 'yaml'],
    },
  } satisfies Preset.ThemeConfig,

  plugins: [
    [
      require.resolve('@easyops-cn/docusaurus-search-local'),
      {
        hashed: true,
        language: ['en'],
        indexDocs: true,
        indexBlog: false,
        indexPages: true,
        docsRouteBasePath: '/docs',
      },
    ],
  ],
};

export default config;
