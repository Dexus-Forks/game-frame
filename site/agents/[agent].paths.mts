// One page per agent, made from agents/<agent>.md when the site builds: the agent's brief with
// a short lead that links its preloaded skills. Relative links are marked for links.mts.
import path from 'node:path'
import { ROOT } from '../.vitepress/repo.mts'
import { REPO_PREFIX } from '../.vitepress/links.mts'
import { agentPage, agents } from '../.vitepress/catalog.mts'

function relink(body: string, file: string): string {
  const dir = path.relative(ROOT, path.dirname(file))
  return body.replace(/\]\((?![a-z]+:|\/|#)([^)]+)\)/gi, (_, href) =>
    `](${REPO_PREFIX}${path.posix.normalize(path.posix.join(dir, href))})`)
}

export default {
  watch: ['../../agents/*.md'],
  paths() {
    return agents().map(a => ({ params: { agent: a.name, source: path.relative(ROOT, a.file) }, content: agentPage(a, relink) }))
  },
}
