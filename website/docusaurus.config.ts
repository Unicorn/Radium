import {themes as prismThemes} from 'prism-react-renderer';
import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';

// This runs in Node.js - Don't use client-side code here (browser APIs, JSX...)

const config: Config = {
  title: 'Radium',
  tagline: 'Next-generation agentic orchestration platform',
  favicon: 'img/logo-apple.png',

  // Future flags, see https://docusaurus.io/docs/api/docusaurus-config#future
  future: {
    v4: true, // Improve compatibility with the upcoming Docusaurus v4
  },

  // Set the production url of your site here
  url: 'https://radium.love',
  // Set the /<baseUrl>/ pathname under which your site is served
  // For custom domains, use '/' as baseUrl
  baseUrl: '/',

  // GitHub pages deployment config.
  // If you aren't using GitHub pages, you don't need these.
  organizationName: 'Unicorn', // Usually your GitHub org/user name.
  projectName: 'Radium', // Usually your repo name.

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
          editUrl: 'https://github.com/Unicorn/Radium/tree/main/website/docs/',
        },
        blog: false,
        theme: {
          customCss: ['./src/css/custom.css', './src/css/marketing.css', './src/css/radium-theme.css'],
        },
      } satisfies Preset.Options,
    ],
  ],

  themeConfig: {
    image: 'img/radium-social-card.png',
    colorMode: {
      defaultMode: 'dark',
      respectPrefersColorScheme: false,
    },
    navbar: {
      logo: {
        alt: 'Radium Logo',
        src: 'img/logo-apple.png',
        width: 32,
        height: 32,
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
          to: '/use-cases',
          label: 'Use Cases',
          position: 'left',
        },
        {
          to: '/examples',
          label: 'Examples',
          position: 'left',
        },
        {
          to: '/community',
          label: 'Community',
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
          href: 'https://github.com/Unicorn/Radium',
          label: 'GitHub',
          position: 'right',
        },
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Product',
          items: [
            {
              label: 'Features',
              to: '/features',
            },
            {
              label: 'Use Cases',
              to: '/use-cases',
            },
            {
              label: 'Examples',
              to: '/examples',
            },
          ],
        },
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
              label: 'API Reference',
              to: '/docs/api/radium_core',
            },
          ],
        },
        {
          title: 'Community',
          items: [
            {
              label: 'GitHub',
              href: 'https://github.com/Unicorn/Radium',
            },
            {
              label: 'Community Page',
              to: '/community',
            },
            {
              label: 'Discussions',
              href: 'https://github.com/Unicorn/Radium/discussions',
            },
            {
              label: 'Issues',
              href: 'https://github.com/Unicorn/Radium/issues',
            },
          ],
        },
        {
          title: 'Resources',
          items: [
            {
              label: 'CLI Reference',
              to: '/docs/cli/README',
            },
            {
              label: 'Developer Guide',
              to: '/docs/developer-guide/developer-guide-overview',
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
