// SITE_TITLE: a log of the work and what it found. Made by GameFrame's scripts/new-site.sh; the
// re-site skill says what goes on each page. The project's documents are shown with
// <!--@include: ...-->, so each has one copy, in docs/. The site holds none of the game's files:
// prose, addresses, offsets, constants, and captioned pictures (screenshots of the original and
// of the project's viewer, frames from the port).
import { readdirSync, readFileSync } from 'node:fs'
import { join } from 'node:path'
import { defineConfig } from 'vitepress'

const diaryDir = join(__dirname, '..', 'diary')
const repo = "REPO_URL"

// Newest first, from the file names (YYYY-MM-DD-slug.md) and each entry's title.
function diaryEntries() {
  return readdirSync(diaryDir)
    .filter((f) => /^\d{4}-\d{2}-\d{2}-.+\.md$/.test(f))
    .sort()
    .reverse()
    .map((f) => {
      const text = readFileSync(join(diaryDir, f), 'utf8')
      const title = /^title:\s*(.+)$/m.exec(text)?.[1].replace(/^["']|["']$/g, '') ?? f
      return { text: `${f.slice(0, 10)} · ${title}`, link: `/diary/${f.replace(/\.md$/, '')}` }
    })
}

export default defineConfig({
  title: "SITE_TITLE",
  description: "SITE_DESCRIPTION",
  // GitHub Pages serves the site under /<repository>/ (or /<repository>/<site>/): the workflow sets it.
  base: process.env.SITE_BASE || '/',
  lang: 'en-GB',
  cleanUrls: true,
  lastUpdated: true,
  srcExclude: ['README.md'],
  themeConfig: {
    nav: [
      { text: 'Diary', link: '/diary/' },
      { text: 'Findings', link: '/findings' },
      { text: 'Status', link: '/status' },
    ],
    sidebar: [
      { text: 'Diary', items: diaryEntries() },
      {
        text: 'Project',
        items: [
          { text: 'Findings', link: '/findings' },
          { text: 'Status', link: '/status' },
        ],
      },
    ],
    socialLinks: repo.startsWith('http') ? [{ icon: 'github', link: repo }] : [],
    footer: {
      message: 'Prose CC0 1.0, code Apache 2.0. The original game belongs to its publisher; none of its files are on this site.',
    },
    search: { provider: 'local' },
  },
})
