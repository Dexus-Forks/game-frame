import { execFileSync } from 'node:child_process'
import { createRequire } from 'node:module'
import path from 'node:path'
import { defineConfig } from 'vitepress'
import { withMermaid } from 'vitepress-plugin-mermaid'
import { EXCLUDE, GITHUB, ROOT, rewrite } from './repo.mts'
import { repoLinks } from './links.mts'
import { home, textLinks, writeLlmsFiles } from './llms.mts'
import { sidebar } from './sidebar.mts'

// The pages are the repository's own Markdown files, read where they are: see repo.mts.
// They live outside site/, where the server build cannot find vue/server-renderer by itself.
const require = createRequire(import.meta.url)
const vueDir = path.dirname(require.resolve('vue/package.json'))
const siteModules = path.resolve(path.dirname(require.resolve('vitepress-plugin-mermaid')), '../..')

/** The time (ms) of the newest commit touching any of these repository paths, or undefined. */
function gitTime(...paths: string[]): number | undefined {
  try {
    const out = execFileSync('git', ['log', '-1', '--format=%at', '--', ...paths], { cwd: ROOT, encoding: 'utf8' }).trim()
    return out ? Number(out) * 1000 : undefined
  } catch {
    return undefined
  }
}

// Every path the site builds pages from, for the home page's date.
const SITE_SOURCES = ['README.md', 'docs', '.claude/skills', 'examples/*/README.md', 'site/index.md']

// The workflow sets SITE_BASE to the Pages path (/game-frame/); locally it is /.
const base = process.env.SITE_BASE || '/'

// withMermaid renders ```mermaid blocks (the agent teams' diagrams), as GitHub does.
export default withMermaid(defineConfig({
  title: 'GameFrame',
  description: 'An agentic framework for reverse engineering games: a method, skills and tools for AI agents and people who rebuild games exactly.',
  base,
  srcDir: '..',
  srcExclude: EXCLUDE,
  rewrites: rewrite,
  cleanUrls: true,
  lastUpdated: true,
  // Skill pages come from one template, so take the date of their SKILL.md. The home page
  // takes the date of the newest change to any page.
  transformPageData(pageData) {
    const skill = pageData.params?.skill
    if (skill) pageData.lastUpdated = gitTime(`.claude/skills/${skill}/SKILL.md`) ?? pageData.lastUpdated
    else if (pageData.frontmatter.layout === 'home') {
      pageData.lastUpdated = gitTime(...SITE_SOURCES)
      home.frontmatter = pageData.frontmatter
    }
  },
  // Each page names its plain Markdown copy, which llms.mts writes after the build.
  transformHead({ pageData, siteConfig }) {
    if (pageData.isNotFound) return
    const href = siteConfig.site.base + pageData.relativePath
    return [['link', { rel: 'alternate', type: 'text/markdown', href }]]
  },
  buildEnd: writeLlmsFiles,
  vite: {
    resolve: {
      alias: [
        { find: /^vue$/, replacement: path.join(vueDir, 'dist/vue.runtime.esm-bundler.js') },
        { find: /^vitepress-plugin-mermaid\/Mermaid\.vue$/, replacement: path.join(siteModules, 'vitepress-plugin-mermaid/dist/Mermaid.vue') },
        { find: /^mermaid$/, replacement: path.join(siteModules, 'mermaid') },
        { find: /^vue\/server-renderer$/, replacement: path.join(vueDir, 'server-renderer/index.mjs') },
      ],
    },
  },
  markdown: {
    config: md => md.use(repoLinks).use(textLinks(base)),
  },
  themeConfig: {
    nav: [
      { text: 'Start', link: '/overview' },
      { text: 'Preservation', link: '/preservation/' },
      { text: 'Games', link: '/games/' },
      {
        text: 'Guides',
        items: [
          { text: 'Engines and middleware', link: '/engines/' },
          { text: 'Hardware of the era', link: '/hardware/' },
          { text: 'Home computers', link: '/computers/' },
          { text: 'Console games', link: '/consoles/' },
          { text: 'Handhelds', link: '/handhelds/' },
          { text: 'Arcade', link: '/arcade/' },
          { text: 'Media and files', link: '/media/' },
          { text: 'Observing the original', link: '/observe/' },
          { text: 'Agent teams', link: '/teams/' },
          { text: 'Working as an agent', link: '/working/agents' },
        ],
      },
      { text: 'Crates', link: '/crates/' },
      {
        text: 'Skills and tools',
        items: [
          { text: 'Skills', link: '/skills/' },
          { text: 'Agents', link: '/agents/' },
          { text: 'Tools', link: '/tools/' },
        ],
      },
      { text: 'Case studies', link: '/case-studies/' },
    ],
    sidebar: sidebar(),
    outline: { level: [2, 3] },
    lastUpdated: { formatOptions: { dateStyle: 'long' } },
    search: { provider: 'local' },
    socialLinks: [{ icon: 'github', link: GITHUB }],
    editLink: {
      // Runs in the browser, so it cannot use GITHUB from repo.mts. Pages made by a dynamic
      // route (skills, agents, tools, the indexes) carry their source file in params.source.
      pattern: ({ filePath, params }) => 'https://github.com/tanepiper/game-frame/edit/main/'
        + (params?.source ?? (params?.skill ? `.claude/skills/${params.skill}/SKILL.md` : filePath)),
      text: 'Edit this page on GitHub',
    },
    footer: {
      message: 'Code: Apache 2.0. Content: CC0 1.0. No game material: bring your own copy.',
    },
  },
}))
