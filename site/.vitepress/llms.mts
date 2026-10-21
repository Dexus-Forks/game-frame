// Text for AI agents, written next to the HTML when the site builds.
//
//   /<page>.md       a plain Markdown copy of every page, beside its .html
//                    (/games/interaction.md, /games/index.md, /skills/re-start.md)
//   /llms.txt        an index of the pages in the order of the menu (https://llmstxt.org)
//   /llms-full.txt   every page in one file, in the same order
//
// Each copy is the page's own Markdown file: front matter removed, relative links turned into
// links to the site's .md copies (or to GitHub for tools and scripts). Skill pages come from
// .claude/skills/<name>/SKILL.md, with the same short lead as the HTML page. The home page is
// mostly front matter, so its copy is written from the hero and the features.
import fs from 'node:fs'
import path from 'node:path'
import type MarkdownIt from 'markdown-it'
import type { DefaultTheme, SiteConfig } from 'vitepress'
import { ROOT, githubUrl, pageUrl, skills } from './repo.mts'
import { sidebar } from './sidebar.mts'

type Page = { url: string, out: string, title: string, description: string, text: string }

/** The home page's front matter, kept by transformPageData in config.mts. */
export const home: { frontmatter?: Record<string, any> } = {}

export function writeLlmsFiles(config: SiteConfig) {
  const base = config.site.base
  // Full URLs when the workflow says where the site lives; paths from the base otherwise.
  const origin = (process.env.SITE_URL ?? '').replace(/\/$/, '')
  const href = (url: string) => origin + base.replace(/\/$/, '') + mdUrl(url)

  const pages: Page[] = []
  for (const source of config.pages) {
    if (source.startsWith('site/skills/')) continue // the skill route, done below
    const out = config.rewrites.map[source] ?? source
    const url = '/' + out.replace(/(^|\/)index\.md$/, '$1').replace(/\.md$/, '')
    const raw = fs.readFileSync(path.join(ROOT, source), 'utf8')
    const text = source === 'site/index.md' ? homeText(raw, href) : relink(stripFrontMatter(raw), source, href)
    pages.push({ url, out, ...describe(text), text })
  }
  for (const { name, description, file } of skills()) {
    const source = path.relative(ROOT, file)
    const body = relink(stripFrontMatter(fs.readFileSync(file, 'utf8')), source, href)
      .replace(/^(# .*\n)/m, `$1\n> **Skill \`${name}\`.** ${description}\n`)
    pages.push({ url: `/skills/${name}`, out: `skills/${name}.md`, title: describe(body).title, description, text: body })
  }

  for (const page of pages) {
    const file = path.join(config.outDir, page.out)
    fs.mkdirSync(path.dirname(file), { recursive: true })
    fs.writeFileSync(file, page.text)
  }

  // Order the pages like the menu; pages the menu does not list come last.
  const byUrl = new Map(pages.map(p => [p.url, p]))
  const used = new Set<string>(['/'])
  const sections: { name: string, pages: Page[] }[] = []
  for (const group of sidebar()) {
    const list = flatten(group.items ?? []).map(u => byUrl.get(u)).filter((p): p is Page => !!p && !used.has(p.url))
    list.forEach(p => used.add(p.url))
    if (list.length) sections.push({ name: group.text ?? '', pages: list })
  }
  const rest = pages.filter(p => !used.has(p.url)).sort((a, b) => a.url.localeCompare(b.url))
  if (rest.length) sections.push({ name: 'Optional', pages: rest })

  const site = config.site
  const index = [
    `# ${site.title}`, '',
    `> ${site.description}`, '',
    `${site.title} is a generic harness for reverse engineering old games from the copy you own and`
    + ' rebuilding them exactly in new code. It gives a method in steps, one skill per step for AI'
    + ' coding agents, and tested tools around Ghidra, Unicorn and REA. It ships no game material.'
    + ' Every page below is plain Markdown. All of them together are in'
    + ` [llms-full.txt](${origin}${base}llms-full.txt).`, '',
    `- [Home](${href('/')}): ${byUrl.get('/')?.description ?? ''}`, '',
  ]
  for (const s of sections) {
    index.push(`## ${s.name}`, '')
    for (const p of s.pages) index.push(`- [${p.title}](${href(p.url)})${p.description ? `: ${p.description}` : ''}`)
    index.push('')
  }
  fs.writeFileSync(path.join(config.outDir, 'llms.txt'), index.join('\n'))

  const ordered = [byUrl.get('/')!, ...sections.flatMap(s => s.pages)].filter(Boolean)
  const full = ordered.map(p => `<!-- Source: ${href(p.url)} -->\n\n${p.text.trim()}\n`).join('\n---\n\n')
  fs.writeFileSync(path.join(config.outDir, 'llms-full.txt'), `# ${site.title}\n\n> ${site.description}\n\n---\n\n${full}`)
}

/** The .md URL of a page URL: /games/ -> /games/index.md, /dos -> /dos.md. */
export function mdUrl(url: string): string {
  return url.endsWith('/') ? `${url}index.md` : `${url}.md`
}

/** Links in a page to these text files ("/llms.txt", "/games/interaction.md") keep their
 * name and gain the base. VitePress would otherwise send a .md link to the HTML page. */
export function textLinks(base: string) {
  return (md: MarkdownIt) => md.core.ruler.push('llms-links', state => {
    for (const token of state.tokens.flatMap(b => b.children ?? [])) {
      const href = token.type === 'link_open' ? token.attrGet('href') : null
      if (!href || !TEXT_LINK.test(href)) continue
      token.attrSet('href', base + href.slice(1))
      token.attrSet('target', '_self')
    }
  })
}

const TEXT_LINK = /^\/[^#]*\.(txt|md)$/

function flatten(items: DefaultTheme.SidebarItem[]): string[] {
  return items.flatMap(i => [...(i.link ? [i.link] : []), ...flatten(i.items ?? [])])
}

function stripFrontMatter(text: string): string {
  return text.replace(/^---\n[\s\S]*?\n---\n+/, '')
}

/** Relative links, outside code, resolved against the source file. */
function relink(text: string, source: string, href: (url: string) => string): string {
  return text.split(/(^```[\s\S]*?^```)/m).map((part, i) => i % 2 ? part : part.replace(
    /\]\((\/[^)\s]+)\)/g, (whole, link: string) =>
      TEXT_LINK.test(link) ? `](${href('/').replace(/index\.md$/, '')}${link.slice(1)})` : whole,
  ).replace(
    /\]\((?![a-z]+:|\/|#)([^)\s]+)\)/gi, (whole, link: string) => {
      const [file, hash] = path.posix.normalize(path.posix.join(path.posix.dirname(source), link)).split('#')
      if (!fs.existsSync(path.join(ROOT, file))) return whole
      const page = pageUrl(file)
      return `](${page ? href(page) : githubUrl(file)}${hash ? `#${hash}` : ''})`
    })).join('')
}

/** The first heading, and the first sentence of the first paragraph after it (skipping the
 * "not legal advice" line the law pages open with). */
function describe(text: string): { title: string, description: string } {
  const title = text.match(/^# (.+)$/m)?.[1].trim() ?? ''
  const para = text.replace(/^[\s\S]*?^# .*$/m, '').split(/\n\s*\n/)
    .map(b => b.trim()).find(b => b && !/^(#|>|```|\||-|\*|\d+\.|<|!\[)/.test(b) && !/not legal advice/i.test(b)) ?? ''
  const plain = para.replace(/\s+/g, ' ').replace(/\[([^\]]+)\]\([^)]*\)/g, '$1').replace(/[*`]/g, '')
  const sentence = plain.match(/^.+?[.!?](?=\s|$)/)?.[0] ?? plain
  return { title, description: sentence }
}

/** The home page, from its front matter and the Markdown below it. */
function homeText(raw: string, href: (url: string) => string): string {
  const fm = home.frontmatter ?? {}
  const hero = fm.hero ?? {}
  const lines = [`# ${hero.name ?? fm.title ?? 'Home'}`, '']
  if (hero.text) lines.push(`${hero.text}.`, '')
  if (hero.tagline) lines.push(hero.tagline, '')
  for (const f of fm.features ?? []) {
    lines.push(`- **${f.link ? `[${f.title}](${href(f.link)})` : f.title}**: ${f.details}`)
  }
  return [...lines, '', relink(stripFrontMatter(raw), 'site/index.md', href).trim(), ''].join('\n')
}
