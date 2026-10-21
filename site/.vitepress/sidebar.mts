// The site's menu, read from the documentation's own structure.
//
// docs/README.md has one "## Section" per menu group, each with a list of links. A link to a
// folder's README.md opens into that folder's pages, in the order of the links in that README,
// and a subfolder's README linked from it opens into its pages too.
// The skills follow, in the order of the README's skills table, then the examples.
import fs from 'node:fs'
import path from 'node:path'
import type { DefaultTheme } from 'vitepress'
import { ROOT, pageUrl, skills, title } from './repo.mts'

type Item = DefaultTheme.SidebarItem

export function sidebar(): Item[] {
  const groups: Item[] = [{
    text: 'Start',
    items: [
      { text: 'Overview', link: '/overview' },
      { text: 'Documentation map', link: '/guide' },
    ],
  }]
  for (const section of sections('docs/README.md')) {
    const items: Item[] = []
    for (const { target, text } of section.links) {
      items.push({ text, link: pageUrl(target)! })
      const folderReadme = /\/README\.md$/.test(target) && target !== 'docs/README.md'
      if (!folderReadme) continue
      for (const child of links(target)) {
        if (path.dirname(child.target) === path.dirname(target)) {
          items.push({ text: child.text, link: pageUrl(child.target)! })
        } else if (path.dirname(path.dirname(child.target)) === path.dirname(target)
          && /\/README\.md$/.test(child.target)) {
          // A subfolder's README opens into its own pages, one level deep.
          const sub = links(child.target)
            .filter(c => path.dirname(c.target) === path.dirname(child.target))
            .map(c => ({ text: c.text, link: pageUrl(c.target)! }))
          items.push({ text: child.text, link: pageUrl(child.target)!, collapsed: true, items: sub })
        }
      }
    }
    groups.push({ text: section.name, collapsed: true, items })
  }
  groups.push({
    text: 'Skills',
    collapsed: true,
    items: skills().map(s => ({ text: s.name, link: `/skills/${s.name}` })),
  })
  const examples = fs.readdirSync(path.join(ROOT, 'examples'))
    .filter(n => fs.existsSync(path.join(ROOT, 'examples', n, 'README.md')))
    .map(n => `examples/${n}/README.md`)
  groups.push({
    text: 'Examples',
    collapsed: true,
    items: examples.map(e => ({ text: title(e), link: pageUrl(e)! })),
  })
  return groups
}

type Link = { target: string, text: string }

/** The "## " sections of a Markdown file, each with the pages it links to. */
function sections(repoPath: string): { name: string, links: Link[] }[] {
  const text = fs.readFileSync(path.join(ROOT, repoPath), 'utf8')
  const out: { name: string, links: Link[] }[] = []
  for (const block of text.split(/^## /m).slice(1)) {
    const name = block.split('\n', 1)[0].trim()
    out.push({ name, links: mdLinks(repoPath, block) })
  }
  return out
}

/** The pages a file links to, in order, each once, named by the text of its first link. */
function links(repoPath: string): Link[] {
  return mdLinks(repoPath, fs.readFileSync(path.join(ROOT, repoPath), 'utf8'))
}

function mdLinks(repoPath: string, text: string): Link[] {
  const seen = new Map<string, string>()
  for (const m of text.matchAll(/\[([^\]]+)\]\(([^)#\s]+\.md)(#[^)]*)?\)/g)) {
    if (/^[a-z]+:/.test(m[2]) || m[3]) continue
    const target = path.posix.normalize(path.posix.join(path.posix.dirname(repoPath), m[2]))
    if (pageUrl(target) && !seen.has(target)) seen.set(target, m[1].replace(/\*\*/g, ''))
  }
  return [...seen].map(([target, text]) => ({ target, text }))
}
