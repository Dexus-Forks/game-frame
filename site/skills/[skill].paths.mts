// One page per skill, made from .claude/skills/<skill>/SKILL.md when the site builds.
// The front matter becomes a short lead with a prompt to copy; relative links are marked for
// links.mts to resolve. The page ends with the agents that preload the skill and the tools it names.
import fs from 'node:fs'
import path from 'node:path'
import { ROOT, skills } from '../.vitepress/repo.mts'
import { REPO_PREFIX } from '../.vitepress/links.mts'
import { skillExtras } from '../.vitepress/catalog.mts'

/** A prompt that asks an agent to use the skill, for the reader to copy. */
function prompt(name: string): string {
  return [
    'Copy this prompt into your coding agent. Replace the part in angle brackets.',
    '',
    '```text',
    `In my GameFrame project, use the ${name} skill for <the task>.`,
    'Read the skill first and follow its steps in order.',
    'Report the evidence it asks for, with its label: observed, measured, inferred or unknown.',
    '```',
    '',
  ].join('\n')
}

export default {
  watch: ['../../.claude/skills/*/SKILL.md'],
  paths() {
    return skills().map(({ name, description, file }) => {
      const dir = path.relative(ROOT, path.dirname(file))
      const body = fs.readFileSync(file, 'utf8').replace(/^---\n[\s\S]*?\n---\n/, '')
      const content = body
        .replace(/\]\((?![a-z]+:|\/|#)([^)]+)\)/gi, (_, href) =>
          `](${REPO_PREFIX}${path.posix.normalize(path.posix.join(dir, href))})`)
        .replace(/^(# .*\n)/m, `$1\n> **Skill \`${name}\`.** ${description}\n\n${prompt(name)}`)
      return { params: { skill: name }, content: content + skillExtras(name, body) }
    })
  },
}
