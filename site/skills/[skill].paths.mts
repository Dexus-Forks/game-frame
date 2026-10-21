// One page per skill, made from .claude/skills/<skill>/SKILL.md when the site builds.
// The front matter becomes a short lead; relative links are marked for links.mts to resolve.
import fs from 'node:fs'
import path from 'node:path'
import { ROOT, skills } from '../.vitepress/repo.mts'
import { REPO_PREFIX } from '../.vitepress/links.mts'

export default {
  watch: ['../../.claude/skills/*/SKILL.md'],
  paths() {
    return skills().map(({ name, description, file }) => {
      const dir = path.relative(ROOT, path.dirname(file))
      const body = fs.readFileSync(file, 'utf8').replace(/^---\n[\s\S]*?\n---\n/, '')
      const content = body
        .replace(/\]\((?![a-z]+:|\/|#)([^)]+)\)/gi, (_, href) =>
          `](${REPO_PREFIX}${path.posix.normalize(path.posix.join(dir, href))})`)
        .replace(/^(# .*\n)/m, `$1\n> **Skill \`${name}\`.** ${description}\n`)
      return { params: { skill: name }, content }
    })
  },
}
