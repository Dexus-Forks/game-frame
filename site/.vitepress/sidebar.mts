// The site's menu, read from the documentation's own structure.
//
// docs/README.md has one "## Section" per menu group, each with a list of links. A link to a
// folder's README.md opens into that folder's pages, in the order of the links in that README.
// A section with several folders shows each as its own group; a folder README whose page list
// is split under "## " headings shows those headings as groups; a subfolder's README linked
// from it opens into its pages too.
// The skills follow, in the order of the README's skills table, then the agents, the tools
// (grouped by folder) and the examples.
import fs from 'node:fs'
import path from 'node:path'
import type { DefaultTheme } from 'vitepress'
import { ROOT, pageUrl, skills, title } from './repo.mts'
import { GROUP_TITLES, agents, tools } from './catalog.mts'

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
    const nest = section.links.length > 1
    for (const { target, text } of section.links) {
      const link = pageUrl(target)!
      const folderReadme = /\/README\.md$/.test(target) && target !== 'docs/README.md'
      if (!folderReadme) {
        items.push({ text, link })
      } else if (nest) {
        // Several folders in one section: each folder is its own collapsible group.
        items.push({ text, link, collapsed: true, items: folderItems(target) })
      } else {
        items.push({ text, link }, ...folderItems(target))
      }
    }
    groups.push({ text: section.name, collapsed: true, items })
  }
  groups.push({
    text: 'Skills',
    collapsed: true,
    items: [{ text: 'All skills', link: '/skills/' }, ...skills().map(s => ({ text: s.name, link: `/skills/${s.name}` }))],
  })
  groups.push({
    text: 'In your browser',
    collapsed: true,
    items: [{ text: 'Overview', link: '/lab/' }, { text: 'File explorer', link: '/lab/files' },
      { text: 'Pixel explorer', link: '/lab/pixels' }, { text: 'Records explorer', link: '/lab/records' }],
  })
  groups.push({
    text: 'Agents',
    collapsed: true,
    items: [{ text: 'All agents', link: '/agents/' }, ...agents().map(a => ({ text: a.name, link: `/agents/${a.name}` }))],
  })
  groups.push({
    text: 'Tools',
    collapsed: true,
    items: [{ text: 'All tools', link: '/tools/' }, ...Object.entries(GROUP_TITLES).map(([group, text]) => ({
      text, collapsed: true,
      items: tools().filter(t => t.group === group).map(t => ({ text: t.name, link: `/tools/${t.name}` })),
    })).filter(g => g.items.length)],
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

/**
 * A folder's pages, from its README. When the README lists its pages under two or more "## "
 * headings, each heading becomes a collapsible group; otherwise the pages are listed in order.
 * A subfolder's README opens into its own pages, one level deep.
 */
function folderItems(readme: string): Item[] {
  const dir = path.dirname(readme)
  const own = (t: string) => path.dirname(t) === dir || /^crates\/[^/]+\/README\.md$/.test(t)
  const sub = (t: string) => path.dirname(path.dirname(t)) === dir && /\/README\.md$/.test(t)
  const item = (c: Link): Item => sub(c.target)
    ? {
        text: c.text,
        link: pageUrl(c.target)!,
        collapsed: true,
        items: links(c.target)
          .filter(s => path.dirname(s.target) === path.dirname(c.target))
          .map(s => ({ text: s.text, link: pageUrl(s.target)! })),
      }
    : { text: c.text, link: pageUrl(c.target)! }
  const grouped = sections(readme, true)
    .map(s => ({ name: s.name, links: s.links.filter(c => own(c.target) || sub(c.target)) }))
    .filter(s => s.links.length > 0)
  if (grouped.length >= 2) {
    // A heading with a single page shows the page itself, not a group of one.
    return grouped.map(g => g.links.length === 1
      ? item(g.links[0])
      : { text: g.name, collapsed: true, items: g.links.map(item) })
  }
  return links(readme).filter(c => own(c.target) || sub(c.target)).map(item)
}

/** The "## " sections of a Markdown file, each with the pages it links to (only from list
 * items that start with a link, when `listOnly` is set). */
function sections(repoPath: string, listOnly = false): { name: string, links: Link[] }[] {
  const text = fs.readFileSync(path.join(ROOT, repoPath), 'utf8')
  const out: { name: string, links: Link[] }[] = []
  for (const block of text.split(/^## /m).slice(1)) {
    const name = block.split('\n', 1)[0].trim()
    const body = listOnly
      ? block.split('\n').filter(l => /^\s*(?:[-*]|\d+\.)\s+\[/.test(l)).join('\n')
      : block
    out.push({ name, links: mdLinks(repoPath, body) })
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
