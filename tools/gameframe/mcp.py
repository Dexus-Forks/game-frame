"""GameFrame's MCP servers for every agent, written from the one source, .mcp.json.

Usage: gameframe mcp --emit CLIENT [--check | --stdout]

  CLIENT    codex, gemini, cursor, vscode, opencode, or all
  --check   write nothing; exit 1 and name each file that differs from what would be written
  --stdout  print each file instead of writing it

Input: `.mcp.json` at the root (Claude Code's format, `mcpServers`). Outputs, one per client:

  codex     .codex/config.toml     [mcp_servers.NAME]; Codex reads it in trusted projects only
  gemini    .gemini/settings.json  mcpServers, and context.fileName = ["AGENTS.md"]
  cursor    .cursor/mcp.json       mcpServers, type "stdio"
  vscode    .vscode/mcp.json       servers, type "stdio" (GitHub Copilot in VS Code)
  opencode  opencode.json          mcp, type "local", command as one list

`.mcp.json` writes `${VAR:-default}`. Claude Code and Gemini CLI expand it as it is. The others
do not, so a value that is only a variable (`"JAVA_HOME": "${JAVA_HOME:-...}"`) becomes:

  codex     env_vars = ["JAVA_HOME"]: Codex passes the variable on when it is set
  cursor    "${env:JAVA_HOME}"        vscode  "${env:JAVA_HOME}"        opencode  "{env:JAVA_HOME}"

The default is dropped, so set the variables before starting the agent (`source
scripts/env.sh` loads .env and finds Ghidra and Java 21). A `command` keeps only its default
(`dosbox-x-mcp-server`), since a client cannot start an empty command: put the server on PATH.

Each JSON file keeps its other keys; the TOML file keeps everything outside its marked block.
It does not install or start any server, and does not change .mcp.json. Standard library only.
"""
import json
import re
import sys

from paths import NAME, ROOT, rel

VAR = re.compile(r'\$\{(\w+)(?::-([^}]*))?\}')
BEGIN, END = '# BEGIN gameframe mcp (gameframe mcp --emit codex)', '# END gameframe mcp'
CLIENTS = ('codex', 'gemini', 'cursor', 'vscode', 'opencode')


def servers():
    return json.loads((ROOT / '.mcp.json').read_text(encoding='utf-8'))['mcpServers']


def _only_var(value):
    """The variable's name when `value` is nothing but `${NAME}` or `${NAME:-default}`."""
    m = VAR.fullmatch(value)
    return m.group(1) if m else None


def _defaults(value):
    """`value` with each `${NAME:-default}` replaced by its default."""
    return VAR.sub(lambda m: m.group(2) or '', value)


def _refs(value, form):
    """`value` with each variable written in the client's own form, defaults dropped."""
    return VAR.sub(lambda m: form.format(m.group(1)), value)


def _plain(s, form):
    """A server for clients without defaults: command and args take defaults, env takes `form`."""
    out = {'command': _defaults(s['command']), 'args': [_defaults(a) for a in s.get('args', [])]}
    if s.get('env'):
        out['env'] = {k: _refs(v, form) for k, v in s['env'].items()}
    return out


def _json_file(path, update):
    """The JSON at `path` (or {}) with `update` applied, as text."""
    old = json.loads(path.read_text(encoding='utf-8')) if path.exists() else {}
    update(old)
    return json.dumps(old, indent=2) + '\n'


def _toml_str(s):
    return json.dumps(s)            # a JSON string is a valid TOML basic string


def codex(path):
    lines = [BEGIN]
    for name, s in servers().items():
        p = _plain(s, '{}')
        forward = [k for k, v in s.get('env', {}).items() if _only_var(v) == k]
        env = {k: _defaults(v) for k, v in s.get('env', {}).items() if k not in forward}
        lines += ['', f'[mcp_servers.{name}]', f'command = {_toml_str(p["command"])}',
                  f'args = [{", ".join(map(_toml_str, p["args"]))}]']
        if forward:
            lines.append(f'env_vars = [{", ".join(map(_toml_str, forward))}]')
        if env:
            lines.append('env = { ' + ', '.join(f'{k} = {_toml_str(v)}' for k, v in env.items()) + ' }')
    lines.append(END)
    block = '\n'.join(lines) + '\n'
    old = path.read_text(encoding='utf-8') if path.exists() else ''
    if BEGIN in old and END in old:
        head, rest = old.split(BEGIN, 1)
        return head + block + rest.split(END, 1)[1].lstrip('\n')
    return old + ('\n' if old and not old.endswith('\n\n') else '') + block if old else block


def gemini(path):
    def update(d):
        d['mcpServers'] = servers()
        d.setdefault('context', {})['fileName'] = ['AGENTS.md']
    return _json_file(path, update)


def cursor(path):
    def update(d):
        d['mcpServers'] = {n: {'type': 'stdio', **_plain(s, '${{env:{}}}')} for n, s in servers().items()}
    return _json_file(path, update)


def vscode(path):
    def update(d):
        d['servers'] = {n: {'type': 'stdio', **_plain(s, '${{env:{}}}')} for n, s in servers().items()}
    return _json_file(path, update)


def opencode(path):
    def update(d):
        d.setdefault('$schema', 'https://opencode.ai/config.json')
        mcp = d.setdefault('mcp', {})
        for n, s in servers().items():
            p = _plain(s, '{{env:{}}}')
            mcp[n] = {'type': 'local', 'command': [p['command'], *p['args']], 'enabled': True}
            if 'env' in p:
                mcp[n]['environment'] = p['env']
    return _json_file(path, update)


FILES = {'codex': ('.codex/config.toml', codex), 'gemini': ('.gemini/settings.json', gemini),
         'cursor': ('.cursor/mcp.json', cursor), 'vscode': ('.vscode/mcp.json', vscode),
         'opencode': ('opencode.json', opencode)}


def emit(args):
    chosen = CLIENTS if args.emit == 'all' else (args.emit,)
    stale = []
    for client in chosen:
        name, make = FILES[client]
        path = ROOT / name
        text = make(path)
        if args.stdout:
            print(f'# {name}\n{text}')
        elif args.check:
            if not path.exists() or path.read_text(encoding='utf-8') != text:
                stale.append(name)
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text, encoding='utf-8')
            print(f'wrote {rel(path)}')
    if stale:
        print(f'{NAME} mcp: not current with .mcp.json: {", ".join(stale)}. '
              f'Run `{NAME} mcp --emit all`.', file=sys.stderr)
        return 1
    return 0
