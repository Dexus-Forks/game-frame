// Where the site's pages come from, and the URL each one gets.
//
// The site has no copies of the documentation. VitePress reads the Markdown files where they
// are in the repository (srcDir is the repository root), and these functions decide which
// files are pages and what they are called on the site:
//
//   README.md                         -> /overview
//   docs/README.md                    -> /guide
//   docs/<folder>/README.md           -> /<folder>/
//   docs/<path>.md                    -> /<path, lower case>
//   examples/<name>/README.md         -> /examples/<name>
//   crates/<name>/README.md           -> /crates/<name>
//   site/index.md                     -> /            (the home page, the one page written for the site)
//   site/skills/[skill].md            -> /skills/<skill>, one per .claude/skills/<skill>/SKILL.md
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

export const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..')
export const GITHUB = 'https://github.com/tanepiper/game-frame'

/** Repository paths (relative to ROOT) that are not pages, for VitePress's srcExclude. */
export const EXCLUDE = [
  '.git/**', '.venv/**', '**/node_modules/**', 'analysis/**', 'derived/**', 'target/**',
  'templates/**', 'tools/**', 'tests/**', 'scripts/**', 'labels/**',
  'AGENTS.md', 'CLAUDE.md', 'agents/**', 'site/README.md', 'site/.vitepress/**',
  'examples/*/build/**', 'crates/target/**', 'crates/*/THIRD_PARTY.md',
]

/** The site path (with .md) for a repository Markdown file. */
export function rewrite(id: string): string {
  if (id === 'README.md') return 'overview.md'
  if (id === 'site/index.md') return 'index.md'
  if (id.startsWith('site/skills/')) return id.slice('site/'.length)
  if (id === 'docs/README.md') return 'guide.md'
  let m = id.match(/^crates\/([^/]+)\/README\.md$/)
  if (m) return `crates/${m[1]}.md`
  m = id.match(/^examples\/([^/]+)\/README\.md$/)
  if (m) return `examples/${m[1]}.md`
  m = id.match(/^docs\/(.*)$/)
  if (m) {
    const rest = m[1].replace(/(^|\/)README\.md$/, '$1index.md')
    return rest.toLowerCase()
  }
  return id
}

/** The site URL for a repository path, or null when it is not a page. */
export function pageUrl(repoPath: string): string | null {
  const skill = repoPath.match(/^\.claude\/skills\/([^/]+)(\/SKILL\.md)?\/?$/)
  if (skill) return `/skills/${skill[1]}`
  let p = repoPath.replace(/\/$/, '')
  if (!p.endsWith('.md')) {
    if (fs.existsSync(path.join(ROOT, p, 'README.md'))) p = `${p}/README.md`
    else return null
  }
  if (!isPage(p)) return null
  return '/' + rewrite(p).replace(/(^|\/)index\.md$/, '$1').replace(/\.md$/, '')
}

function isPage(p: string): boolean {
  if (!fs.existsSync(path.join(ROOT, p))) return false
  return p === 'README.md' || /^docs\/.+\.md$/.test(p) || /^examples\/[^/]+\/README\.md$/.test(p)
    || /^crates\/[^/]+\/README\.md$/.test(p)
}

/** A GitHub URL for a repository path that is not a page (a tool, a script, a folder). */
export function githubUrl(repoPath: string): string {
  const p = repoPath.replace(/\/$/, '')
  const full = path.join(ROOT, p)
  const kind = fs.existsSync(full) && fs.statSync(full).isDirectory() ? 'tree' : 'blob'
  return `${GITHUB}/${kind}/main/${p}`
}

/** The skills, in the order of the README's skills table. */
export function skills(): { name: string, description: string, file: string }[] {
  const dir = path.join(ROOT, '.claude/skills')
  const readme = fs.readFileSync(path.join(ROOT, 'README.md'), 'utf8')
  const order = [...readme.matchAll(/\.claude\/skills\/([\w-]+)\/SKILL\.md/g)].map(m => m[1])
  const names = fs.readdirSync(dir).filter(n => fs.existsSync(path.join(dir, n, 'SKILL.md')))
  names.sort((a, b) => rank(order, a) - rank(order, b) || a.localeCompare(b))
  return names.map(name => {
    const file = path.join(dir, name, 'SKILL.md')
    const text = fs.readFileSync(file, 'utf8')
    const description = text.match(/^description:\s*(.*)$/m)?.[1] ?? ''
    return { name, description, file }
  })
}

function rank(order: string[], name: string): number {
  const i = order.indexOf(name)
  return i < 0 ? order.length : i
}

/** The first heading of a Markdown file. */
export function title(repoPath: string): string {
  const text = fs.readFileSync(path.join(ROOT, repoPath), 'utf8')
  return text.match(/^# (.+)$/m)?.[1].trim() ?? path.basename(repoPath, '.md')
}
