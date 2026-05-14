const {themes} = require('prism-react-renderer');

/** @type {import('@docusaurus/types').Config} */
const config = {
  title: 'Fruits Engine',
  tagline: 'Rust game engine documentation',
  url: 'https://are-you-fruits-studio.github.io',
  baseUrl: '/fruits_engine/',
  favicon: 'img/favicon.png',
  organizationName: 'are-you-fruits-studio',
  projectName: 'fruits_engine',
  trailingSlash: false,
  onBrokenLinks: 'throw',
  markdown: {
    hooks: {
      onBrokenMarkdownLinks: 'warn',
    },
  },
  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  presets: [
    [
      'classic',
      {
        docs: {
          routeBasePath: 'docs',
          sidebarPath: require.resolve('./sidebars.js'),
          showLastUpdateTime: true,
        },
        blog: false,
        theme: {
          customCss: require.resolve('./src/css/custom.css'),
        },
      },
    ],
  ],

  plugins: [
    [
      '@docusaurus/plugin-content-docs',
      {
        id: 'api',
        path: 'api-reference',
        routeBasePath: 'api-reference',
        sidebarPath: require.resolve('./api-sidebars.js'),
        showLastUpdateTime: true,
      },
    ],
  ],

  themeConfig:
    /** @type {import('@docusaurus/preset-classic').ThemeConfig} */
    ({
      navbar: {
        title: 'Fruits Engine',
        items: [
          {
            type: 'docSidebar',
            sidebarId: 'docsSidebar',
            position: 'left',
            label: 'Docs',
          },
          {
            type: 'docSidebar',
            sidebarId: 'apiSidebar',
            docsPluginId: 'api',
            position: 'left',
            label: 'API Reference',
          },
          {
            href: 'https://github.com/are-you-fruits-studio/fruits_engine',
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
                label: 'Docs',
                to: '/docs/getting-started',
              },
              {
                label: 'API Reference',
                to: '/api-reference/overview',
              },
            ],
          },
          {
            title: 'Project',
            items: [
              {
                label: 'GitHub',
                href: 'https://github.com/are-you-fruits-studio/fruits_engine',
              },
            ],
          },
        ],
        copyright: `Copyright © ${new Date().getFullYear()} Are You Fruits?.`,
      },
      prism: {
        theme: themes.github,
        darkTheme: themes.dracula,
        additionalLanguages: ['rust', 'toml'],
      },
    }),
};

module.exports = config;
