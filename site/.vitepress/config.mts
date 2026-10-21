import { createRequire } from 'node:module'
import path from 'node:path'
import { defineConfig } from 'vitepress'
import { EXCLUDE, GITHUB, rewrite } from './repo.mts'
import { repoLinks } from './links.mts'
import { sidebar } from './sidebar.mts'

// The pages are the repository's own Markdown files, read where they are: see repo.mts.
// They live outside site/, where the server build cannot find vue/server-renderer by itself.
const require = createRequire(import.meta.url)
const vueDir = path.dirname(require.resolve('vue/package.json'))

export default defineConfig({
  title: 'Reverse engineering games',
  description: 'A method, skills and tools for AI agents and people who reverse engineer games and rebuild them exactly.',
  // The workflow sets SITE_BASE to the Pages path (/reverse-engineer-games/); locally it is /.
  base: process.env.SITE_BASE || '/',
  srcDir: '..',
  srcExclude: EXCLUDE,
  rewrites: rewrite,
  cleanUrls: true,
  lastUpdated: true,
  vite: {
    resolve: {
      alias: [
        { find: /^vue$/, replacement: path.join(vueDir, 'dist/vue.runtime.esm-bundler.js') },
        { find: /^vue\/server-renderer$/, replacement: path.join(vueDir, 'server-renderer/index.mjs') },
      ],
    },
  },
  markdown: {
    config: md => md.use(repoLinks),
  },
  themeConfig: {
    nav: [
      { text: 'Start', link: '/overview' },
      { text: 'Games', link: '/games/' },
      { text: 'Observe', link: '/observe/' },
      { text: 'Skills', link: '/skills/re-start' },
      { text: 'Case studies', link: '/case-studies/' },
    ],
    sidebar: sidebar(),
    outline: { level: [2, 3] },
    search: { provider: 'local' },
    socialLinks: [{ icon: 'github', link: GITHUB }],
    editLink: {
      // Runs in the browser, so it cannot use GITHUB from repo.mts. Skill pages come from
      // the dynamic route; their source is the SKILL.md.
      pattern: ({ filePath, params }) => 'https://github.com/tanepiper/reverse-engineer-games/edit/main/'
        + (params?.skill ? `.claude/skills/${params.skill}/SKILL.md` : filePath),
      text: 'Edit this page on GitHub',
    },
    footer: {
      message: 'MIT OR Apache-2.0. No game material: bring your own copy.',
    },
  },
})
