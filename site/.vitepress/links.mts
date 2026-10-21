// Links written for GitHub, made to work on the site.
//
// The documentation links between files with relative paths, so it reads correctly on GitHub
// and in an editor. On the site, each link is resolved against the file it is written in:
// a link to a page becomes that page's URL, a link to a skill becomes /skills/<name>, and a
// link to anything else (a tool, a script, a folder) goes to the file on GitHub.
//
// Skill pages are rendered from a dynamic route, not from their own file, so their loader
// rewrites relative links to "/@repo/<path>" first; those are handled here too.
import fs from 'node:fs'
import path from 'node:path'
import type MarkdownIt from 'markdown-it'
import { ROOT, githubUrl, pageUrl } from './repo.mts'

export const REPO_PREFIX = '/@repo/'

export function repoLinks(md: MarkdownIt) {
  md.core.ruler.after('inline', 'repo-links', state => {
    const env = state.env as { path?: string, realPath?: string }
    const source = path.relative(ROOT, env.realPath ?? env.path ?? '')
    for (const block of state.tokens) {
      for (const token of block.children ?? []) {
        if (token.type !== 'link_open') continue
        const href = token.attrGet('href')
        if (!href) continue
        const fixed = resolve(source, href)
        if (fixed) token.attrSet('href', fixed)
      }
    }
  })
}

function resolve(source: string, href: string): string | null {
  let target: string
  if (href.startsWith(REPO_PREFIX)) target = href.slice(REPO_PREFIX.length)
  else if (/^([a-z]+:|\/|#)/i.test(href)) return null
  else target = path.posix.normalize(path.posix.join(path.posix.dirname(source), href))
  const [file, hash] = target.split('#')
  if (!fs.existsSync(path.join(ROOT, file))) {
    throw new Error(`Dead link in ${source}: ${href} (no file ${file})`)
  }
  const page = pageUrl(file)
  const anchor = hash ? `#${hash}` : ''
  return page ? page + anchor : githubUrl(file) + anchor
}
