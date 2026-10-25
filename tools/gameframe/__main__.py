"""gameframe: help and discovery for GameFrame, for people and agents.

Usage: gameframe COMMAND [ARGS]       (or: python3 tools/gameframe COMMAND [ARGS])

Commands:
  help [COMMAND]        this text, or one command's options
  steps                 the method's loop, each step with its question and its skill
  skills                every skill with its one-line description
  skill NAME            print a skill (re-trace, or just trace)
      --section TITLE   only the section whose heading contains TITLE
      --sections        only the headings
  tools                 every tool and script, grouped by folder, with its one-line summary
  tool NAME             the tool's own help (its docstring or header comment), unchanged
  run NAME [ARGS]       run a tool or script: Python tools with GameFrame's .venv
  find WORD             which skills and tools mention WORD, line by line
      --max N           lines shown per skill or tool (default 3)
  advise TEXT           known failure modes and context for an error message, a log or a topic
      -                 read the text from stdin (pipe a log into it)
      --signatures      only entries whose error signature matches the text
      --entry ID        print one entry in full
      --list            every entry with its status
      --check           check every entry's fields, facets, links and sections
      --why             say why each entry matched; --max N entries (default 5)
  progress [AGENT]      what running agents have done: each log's last lines, newest first
      --add TEXT        append a line to AGENT's log (derived/progress/AGENT.log)
      --lines N         lines shown per log (default 5); naming AGENT shows its whole log
  serve                 GameFrame's modular MCP server on stdio: discovery, know-how and tool execution
  agents                every agent: its skills, the tools it cannot use, its hand-offs
      --table           the Markdown table that docs/working/agents.md holds
      --emit CLIENT     write the agents in a client's format: codex, gemini, copilot, opencode or all
      --out DIR         where --emit writes (default: the repository root)
  doctor [ARGS]         check the toolchain (scripts/setup.sh; ARGS such as --install pass on)
  mcp --list            show providers enabled in this project and their pinned versions
  mcp --enable NAME     opt this project into external provider(s); refresh client settings
  mcp --disable NAME    remove external provider(s) from this project
  mcp --emit CLIENT     write .mcp.json's servers for codex, gemini, cursor, vscode, opencode or all
      --check           only report which files are not current (exit 1)
  mcp-provider NAME VERSION  launch one enabled provider (used by generated settings)

steps, skills, agents, tools, tool, find and advise take --json, for agents and scripts.

Examples:
  gameframe steps
  gameframe skill re-emulate --section pitfalls
  gameframe tool peinfo
  gameframe run peinfo target/game.exe --imports
  gameframe find vtable
  gameframe advise "A privileged instruction was executed at 004a88cd"
  wine game.exe 2>&1 | tail -50 | gameframe advise -
  gameframe advise directdraw wine

Inputs: the skills (skills/re-*/SKILL.md), the tools (tools/, scripts/) and their own
help, and the know-how entries (docs/knowhow/, GameFrame's and the project's), read from the files. The same command works in GameFrame and in a project made by
scripts/new-project.sh. Standard library only; it needs no .venv itself.

It does not install anything or keep its own copy of the skills or the help: `run` and
`doctor` start the tool or script, which does its own work. The one command that writes files
is `agents --emit`. It writes .codex/agents/*.toml, .gemini/agents/*.md,
.github/agents/*.agent.md and .opencode/agents/*.md from agents/re-*.md
(docs/working/agents.md, "Agents in every client").
"""
import argparse
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import commands  # noqa: E402
import mcp  # noqa: E402
import progress  # noqa: E402
import serve  # noqa: E402
from paths import NAME  # noqa: E402

PASS_ON = {'run', 'doctor'}      # commands whose arguments belong to the tool they start


def parser():
    p = argparse.ArgumentParser(prog=NAME, add_help=False)
    sub = p.add_subparsers(dest='command')
    sub.add_parser('help', help='this text, or one command\'s options').add_argument('topic', nargs='?')
    s = sub.add_parser('steps', help='the method\'s loop')
    s.add_argument('--json', action='store_true')
    s = sub.add_parser('skills', help='every skill')
    s.add_argument('--json', action='store_true')
    s = sub.add_parser('skill', help='print a skill')
    s.add_argument('name', help='re-trace, or just trace')
    s.add_argument('--section', metavar='TITLE', help='only the section whose heading contains TITLE')
    s.add_argument('--sections', action='store_true', help='only the headings')
    s = sub.add_parser('agents', help='every agent and how it is configured')
    s.add_argument('--json', action='store_true')
    s.add_argument('--table', action='store_true')
    s.add_argument('--emit', choices=('codex', 'gemini', 'copilot', 'opencode', 'all'), help='write the agents in a client\'s format')
    s.add_argument('--out', metavar='DIR', help='where --emit writes (default: the repository root)')
    s = sub.add_parser('tools', help='every tool and script')
    s.add_argument('--json', action='store_true')
    s = sub.add_parser('tool', help='one tool\'s own help')
    s.add_argument('name', help='peinfo, decomp_query, ghidra-export or a path')
    s.add_argument('--json', action='store_true')
    s = sub.add_parser('find', help='which skills and tools mention a word')
    s.add_argument('word', help='plain text, any case')
    s.add_argument('--max', type=int, default=3, metavar='N', help='lines shown per skill or tool (default 3)')
    s.add_argument('--json', action='store_true')
    s = sub.add_parser('advise', help='known failure modes and context for some text (docs/working/knowhow.md)')
    s.add_argument('text', nargs='*', help='an error message, a topic, or - to read stdin')
    s.add_argument('--signatures', action='store_true', help='only entries whose error signature matches')
    s.add_argument('--entry', metavar='ID', help='print one entry in full')
    s.add_argument('--list', action='store_true', help='every entry with its status')
    s.add_argument('--check', action='store_true', help='check every entry; exit 1 on a problem')
    s.add_argument('--why', action='store_true', help='say why each entry matched')
    s.add_argument('--max', type=int, default=5, metavar='N', help='entries shown (default 5)')
    s.add_argument('--json', action='store_true')
    s = sub.add_parser('progress', help='what running agents have done (derived/progress/)')
    s.add_argument('name', nargs='?', help='an agent, such as re-porter')
    s.add_argument('--add', metavar='TEXT', help='append a line to the agent\'s log')
    s.add_argument('--lines', type=int, default=5, metavar='N', help='lines shown per log (default 5)')
    s.add_argument('--json', action='store_true')
    sub.add_parser('serve', help='GameFrame\'s MCP server on stdio (see tools/gameframe/serve.py)')
    s = sub.add_parser('mcp', help='manage project MCP providers and client settings (see tools/gameframe/mcp.py)')
    action = s.add_mutually_exclusive_group(required=True)
    action.add_argument('--emit', choices=(*mcp.CLIENTS, 'all'), metavar='CLIENT',
                        help=f'{", ".join(mcp.CLIENTS)} or all')
    action.add_argument('--enable', nargs='+', metavar='PROVIDER', help='opt into external MCP provider(s)')
    action.add_argument('--disable', nargs='+', metavar='PROVIDER', help='remove external MCP provider(s)')
    action.add_argument('--list', action='store_true', help='list provider availability and versions')
    s.add_argument('--check', action='store_true', help='write nothing; exit 1 when a file is not current')
    s.add_argument('--stdout', action='store_true', help='print the files instead of writing them')
    s = sub.add_parser('mcp-provider', help='start an opt-in provider through the project environment')
    s.add_argument('provider')
    s.add_argument('version')
    return p, sub


def main(argv):
    if not argv or argv[0] in ('-h', '--help'):
        print(__doc__.rstrip())
        return 0
    if argv[0] == 'run':
        if len(argv) < 2:
            print(f'usage: {NAME} run NAME [ARGS]', file=sys.stderr)
            return 2
        return commands.run(argv[1], argv[2:])
    if argv[0] == 'doctor':
        return commands.doctor(argv[1:])
    p, sub = parser()
    args = p.parse_args(argv)
    if args.command == 'help':
        if args.topic in sub.choices:
            sub.choices[args.topic].print_help()
        elif args.topic in PASS_ON:
            print(f'{NAME} {args.topic}: see `{NAME} help`; its arguments go to the tool.')
        else:
            print(__doc__.rstrip())
        return 0
    return {'steps': commands.steps, 'skills': commands.list_skills, 'skill': commands.skill,
            'agents': commands.list_agents, 'tools': commands.list_tools, 'tool': commands.tool, 'find': commands.find, 'advise': commands.advise,
            'progress': progress.show, 'mcp': mcp.emit, 'mcp-provider': mcp.run_provider,
            'serve': serve.serve,
            }[args.command](args)


if __name__ == '__main__':
    try:
        sys.exit(main(sys.argv[1:]))
    except BrokenPipeError:            # `gameframe tools | head`
        sys.exit(0)
