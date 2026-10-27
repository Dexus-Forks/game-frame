// The skills, the agents and the tools, as pages.
//
// Each comes from its own file, read when the site builds, and nothing is copied:
//
//   skills/<name>/SKILL.md   -> /skills/<name>   (the skill, then its agents and tools)
//   agents/<name>.md                 -> /agents/<name>   (the agent's brief, with its skills)
//   tools/..., scripts/...           -> /tools/<name>    (the tool's own help: its docstring or
//                                                         header comment, read by ./gameframe)
//   (generated)                      -> /skills/, /agents/, /tools/   (one index each)
//
// The links between them go both ways: a skill lists the agents that preload it and the tools
// it names, an agent links its skills, and a tool lists the skills and agents that name it.
import { execFileSync } from 'node:child_process'
import fs from 'node:fs'
import path from 'node:path'
import { ROOT, githubUrl, skills } from './repo.mts'

export type Agent = { name: string, description: string, skills: string[], file: string, body: string }
export type Tool = { name: string, path: string, kind: string, group: string, summary: string, runs: boolean, doc: string }

// The folders of the toolkit, in the order of the method, as ./gameframe names them. A folder
// missing here still gets a group, after these, under its own name (toolGroups()).
const GROUPS: Record<string, string> = {
  disc: 'Discs and disk images', pe: 'Windows PE', dos: '16-bit DOS', triage: 'Triage',
  emu: 'Emulators', formats: 'Formats', live: 'The running game', verify: 'Verify',
  ghidra: 'Ghidra scripts', rea: 'REA', llm: 'Models', site: 'Sites', install: 'Installing builds',
  scripts: 'Scripts',
}

/** Every group that has tools, as [folder, title]: the known ones in order, then any other. */
export function toolGroups(): [string, string][] {
  const seen = new Set(tools().map(t => t.group))
  const known = Object.entries(GROUPS).filter(([g]) => seen.has(g))
  return [...known, ...[...seen].filter(g => !(g in GROUPS)).sort().map(g => [g, g] as [string, string])]
}

function stripFrontMatter(text: string): string {
  return text.replace(/^---\n[\s\S]*?\n---\n+/, '')
}

let agentCache: Agent[] | undefined
/** The agents in agents/, in the order of re-start's tables: the stages, then the tasks. */
export function agents(): Agent[] {
  if (agentCache) return agentCache
  const dir = path.join(ROOT, 'agents')
  const start = fs.readFileSync(path.join(ROOT, 'skills/re-start/SKILL.md'), 'utf8')
  const order = [...start.matchAll(/^\| `(re-[\w-]+)` \|/gm)].map(m => m[1])
  const rank = (n: string) => (order.indexOf(n) + 1 || order.length + 1)
  agentCache = fs.readdirSync(dir).filter(f => f.endsWith('.md')).map(f => {
    const file = path.join(dir, f)
    const text = fs.readFileSync(file, 'utf8')
    const front = text.match(/^---\n([\s\S]*?)\n---/)?.[1] ?? ''
    const name = front.match(/^name:\s*(.+)$/m)?.[1].trim() ?? path.basename(f, '.md')
    const description = (front.match(/^description:\s*(.+)$/m)?.[1] ?? '').trim().replace(/^"(.*)"$/, '$1')
    const list = front.match(/^skills:\s*\n((?:\s+-\s+.+\n?)+)/m)?.[1] ?? ''
    return { name, description, skills: [...list.matchAll(/-\s+([\w:-]+)/g)].map(m => m[1].split(':').pop()!),
      file, body: stripFrontMatter(text) }
  }).sort((a, b) => rank(a.name) - rank(b.name) || a.name.localeCompare(b.name))
  return agentCache
}

let toolCache: Tool[] | undefined
/** Every tool and script with its own help, from ./gameframe (standard-library Python). */
export function tools(): Tool[] {
  if (toolCache) return toolCache
  const run = (...args: string[]) => execFileSync('python3', [path.join(ROOT, 'tools/gameframe'), ...args],
    { cwd: ROOT, encoding: 'utf8' })
  const list = JSON.parse(run('tools', '--json')) as Omit<Tool, 'doc'>[]
  toolCache = list.map(t => ({ ...t, doc: (JSON.parse(run('tool', t.name, '--json')) as { doc: string }).doc }))
  return toolCache
}

/** The page of a tool, by its repository path (a file, or a package folder). */
export function toolUrl(repoPath: string): string | null {
  const p = repoPath.replace(/\/$/, '')
  const t = tools().find(x => x.path === p || (p.startsWith(`${x.path}/`) && !x.path.includes('.')))
  return t ? `/tools/${t.name}` : null
}

/** The skills and agents whose text names a tool, by its path or its file name. */
function namedBy(tool: Tool): { skills: string[], agents: string[] } {
  const file = path.basename(tool.path)
  const re = new RegExp(`(^|[^\\w-])(${escape(tool.path)}|${escape(file)})(?![\\w-])`)
  const skillNames = skills().filter(s => re.test(fs.readFileSync(s.file, 'utf8'))).map(s => s.name)
  const agentNames = agents().filter(a => re.test(a.body)).map(a => a.name)
  return { skills: skillNames, agents: agentNames }
}

function escape(s: string): string {
  return s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
}

/** The tools a text names, by path or file name, in the toolkit's order. */
function toolsNamedIn(text: string): Tool[] {
  return tools().filter(t => {
    const file = path.basename(t.path)
    return new RegExp(`(^|[^\\w-])(${escape(t.path)}|${escape(file)})(?![\\w-])`).test(text)
  })
}

/** Markdown added to the end of a skill's page: the agents that preload it, the tools it names. */
export function skillExtras(name: string, text: string): string {
  const users = agents().filter(a => a.skills.includes(name))
  const named = toolsNamedIn(text)
  const out = ['', '## Agents and tools', '']
  out.push(users.length
    ? `Agents that preload this skill: ${users.map(a => `[${a.name}](/agents/${a.name})`).join(', ')}.`
    : 'No agent preloads this skill. The main session reads it.')
  out.push('')
  if (named.length) {
    out.push('Tools this skill names:', '')
    for (const t of named) out.push(`- [${t.name}](/tools/${t.name}): ${esc(t.summary)}`)
  }
  return out.join('\n') + '\n'
}

/** Markdown for an agent's page. */
export function agentPage(a: Agent, relink: (body: string, file: string) => string): string {
  const head = [
    `> **Agent \`${a.name}\`.** ${esc(a.description)}`, '',
    `Preloaded skills: ${a.skills.map(s => `[${s}](/skills/${s})`).join(', ') || 'none'}.`, '',
    'Claude Code runs it as a subagent from `.claude/agents/` in a project, or from the GameFrame'
    + ' plugin as `gameframe:' + a.name + '`. Codex, Gemini CLI and Copilot read the copies that'
    + ' `gameframe agents --emit` writes ([Agents in every client](/working/agents#agents-in-every-client)).'
    + ' [Agent teams](/teams/) shows how the agents hand work'
    + ` to each other. [Source on GitHub](${githubUrl(path.relative(ROOT, a.file))}).`, '',
  ].join('\n')
  const body = relink(a.body, a.file)
  return body.replace(/^(# .*\n)/m, `$1\n${head}\n`)
}

/** Markdown for a tool's page: its own help, unchanged, with where it lives and who names it. */
export function toolPage(t: Tool): string {
  const by = namedBy(t)
  const how = t.kind === 'program' ? `Run it with \`./gameframe run ${t.name} ARGS\` or \`${t.path.endsWith('.py') ? 'python3 ' : ''}${t.path}\`.`
    : t.kind === 'script' ? `Run it as \`${t.path}\`.`
    : t.kind === 'ghidra' ? 'A Ghidra script: `scripts/ghidra-export.sh` and `scripts/ghidra-run.sh` run it headless.'
    : 'A library: other tools and your own scripts import it.'
  const lines = [
    `# ${t.name}`, '',
    `> **Tool \`${t.path}\`** (${t.kind}, ${GROUPS[t.group] ?? t.group}). ${esc(t.summary)}`, '',
    `${how} [Source on GitHub](${githubUrl(t.path)}).`, '',
    '## Its own help', '',
    'This is the help text in the tool itself, as `./gameframe tool ' + t.name + '` prints it.', '',
    '```text', t.doc.trim(), '```', '',
  ]
  if (by.skills.length || by.agents.length) {
    lines.push('## Named by', '')
    if (by.skills.length) lines.push(`Skills: ${by.skills.map(s => `[${s}](/skills/${s})`).join(', ')}.`, '')
    if (by.agents.length) lines.push(`Agents: ${by.agents.map(a => `[${a}](/agents/${a})`).join(', ')}.`, '')
  }
  return lines.join('\n')
}

/** Markdown for the three index pages. */
export function indexPage(section: string): string {
  if (section === 'skills') {
    const lines = ['# Skills', '',
      'One skill for each step of the method. A skill is a Markdown file of instructions that a coding'
      + ' agent loads when the step comes up. Each page below is the skill itself, followed by the'
      + ' agents that preload it and the tools it names.', '',
      '| Skill | What it does |', '|---|---|']
    for (const s of skills()) lines.push(`| [${s.name}](/skills/${s.name}) | ${cell(s.description)} |`)
    return lines.join('\n') + '\n'
  }
  if (section === 'agents') {
    const lines = ['# Agents', '',
      'Each agent does one stage of the method or one task, with its skills preloaded, and ends with a'
      + ' hand-off that names the next agent. [Agent teams](/teams/) shows them working together.', '',
      '| Agent | Skills | What it does |', '|---|---|---|']
    for (const a of agents()) {
      lines.push(`| [${a.name}](/agents/${a.name}) | ${a.skills.map(s => `[${s}](/skills/${s})`).join(', ')} | ${cell(a.description)} |`)
    }
    return lines.join('\n') + '\n'
  }
  const lines = ['# Tools', '',
    'Every tool and script in GameFrame, with its own help. The same list is in the terminal with'
    + ' `./gameframe tools`, and one tool\'s help with `./gameframe tool NAME`.', '']
  for (const [group, title] of toolGroups()) {
    const list = tools().filter(t => t.group === group)
    if (!list.length) continue
    lines.push(`## ${title}`, '', '| Tool | Kind | What it does |', '|---|---|---|')
    for (const t of list) lines.push(`| [${t.name}](/tools/${t.name}) | ${t.kind} | ${cell(t.summary)} |`)
    lines.push('')
  }
  return lines.join('\n')
}

/** Text from a file, safe in Markdown that Vue also compiles: angle brackets outside code
 * spans become entities (inside a code span they show as written). */
function esc(text: string): string {
  return text.split(/(`[^`]*`)/).map((part, i) => i % 2 ? part
    : part.replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/\{\{/g, '&#123;&#123;')).join('')
}

function cell(text: string): string {
  return esc(text).replace(/\|/g, '\\|')
}

export const SECTIONS = ['skills', 'agents', 'tools']
