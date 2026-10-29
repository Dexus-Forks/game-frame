"""Manage project MCP servers and write each client's configuration.

Usage:
  gameframe mcp --list
  gameframe mcp --enable PROVIDER [PROVIDER ...] [--check | --stdout]
  gameframe mcp --disable PROVIDER [PROVIDER ...] [--check | --stdout]
  gameframe mcp --emit CLIENT [--check | --stdout]
  gameframe mcp-provider PROVIDER PINNED_VERSION

  CLIENT    codex, gemini, cursor, vscode, opencode, or all
  --check   write nothing; exit 1 and name each file that differs from what would be written
  --stdout  print each file instead of writing it

Inputs: the base `.mcp.json`, the project `.mcp.json` when GAMEFRAME_PROJECT is set,
`tools/gameframe/mcp_providers.json`, and existing client settings. `--enable` and `--disable`
change the project's `.mcp.json`, then refresh generated settings for clients already configured
in that project. `--emit` writes the selected files under GAMEFRAME_PROJECT, or GameFrame's root.

Outputs, one per client:

  codex     .codex/config.toml     [mcp_servers.NAME]; Codex reads it in trusted projects only
  gemini    .gemini/settings.json  mcpServers, and context.fileName = ["AGENTS.md"]
  cursor    .cursor/mcp.json       mcpServers, type "stdio"
  vscode    .vscode/mcp.json       servers, type "stdio" (GitHub Copilot in VS Code)
  opencode  opencode.json          mcp, type "local", command as one list

For custom servers with environment placeholders, Claude Code and Gemini CLI expand
`${VAR:-default}`. The other clients get their own variable form:

  codex     env_vars = ["JAVA_HOME"]: Codex passes the variable on when it is set
  cursor    "${env:JAVA_HOME}"        vscode  "${env:JAVA_HOME}"        opencode  "{env:JAVA_HOME}"

Command variables keep only their default in generated client settings. The opt-in provider
launchers avoid this client difference by reading `.env` after the client starts them.

The provider launcher reads the project's ignored `.env`, then replaces itself with the selected
server. It does not install servers or emulator builds, and it does not proxy provider tools.
The config manager keeps unrelated client settings. Standard library only.
"""
import json
import os
import re
import subprocess
import sys
from pathlib import Path

from paths import NAME, ROOT

VAR = re.compile(r'\$\{(\w+)(?::-([^}]*))?\}')
BEGIN, END = '# BEGIN gameframe mcp (gameframe mcp --emit codex)', '# END gameframe mcp'
CLIENTS = ('codex', 'gemini', 'cursor', 'vscode', 'opencode')
PROVIDERS = ROOT / 'tools' / 'gameframe' / 'mcp_providers.json'


def project_root():
    """The game project named by its launcher, or GameFrame when run in this checkout."""
    return Path(os.environ.get('GAMEFRAME_PROJECT', ROOT)).resolve()


def provider_catalog():
    return json.loads(PROVIDERS.read_text(encoding='utf-8'))


def servers(directory=None):
    """Merge GameFrame's core server with the project's own servers."""
    directory = Path(directory or project_root()).resolve()
    base = json.loads((ROOT / '.mcp.json').read_text(encoding='utf-8')).get('mcpServers', {})
    if directory == ROOT:
        return base
    local_path = directory / '.mcp.json'
    if not local_path.is_file():
        return base
    local = json.loads(local_path.read_text(encoding='utf-8')).get('mcpServers', {})
    return {**base, **local}


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


def _merge_managed(old, current):
    """Refresh GameFrame servers while keeping a client's unrelated MCP entries."""
    managed = set(json.loads((ROOT / '.mcp.json').read_text(encoding='utf-8')).get('mcpServers', {}))
    managed.update(provider_catalog())
    for name in managed:
        old.pop(name, None)
    old.update(current)
    return old


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
        d['mcpServers'] = _merge_managed(d.get('mcpServers', {}), servers())
        d.setdefault('context', {})['fileName'] = ['AGENTS.md']
    return _json_file(path, update)


def cursor(path):
    def update(d):
        current = {n: {'type': 'stdio', **_plain(s, '${{env:{}}}')} for n, s in servers().items()}
        d['mcpServers'] = _merge_managed(d.get('mcpServers', {}), current)
    return _json_file(path, update)


def vscode(path):
    def update(d):
        current = {n: {'type': 'stdio', **_plain(s, '${{env:{}}}')} for n, s in servers().items()}
        d['servers'] = _merge_managed(d.get('servers', {}), current)
    return _json_file(path, update)


def opencode(path):
    def update(d):
        d.setdefault('$schema', 'https://opencode.ai/config.json')
        mcp = d.setdefault('mcp', {})
        managed = set(json.loads((ROOT / '.mcp.json').read_text(encoding='utf-8')).get('mcpServers', {}))
        managed.update(provider_catalog())
        for name in managed:
            mcp.pop(name, None)
        for n, s in servers().items():
            p = _plain(s, '{{env:{}}}')
            mcp[n] = {'type': 'local', 'command': [p['command'], *p['args']], 'enabled': True}
            if 'env' in p:
                mcp[n]['environment'] = p['env']
    return _json_file(path, update)


FILES = {'codex': ('.codex/config.toml', codex), 'gemini': ('.gemini/settings.json', gemini),
         'cursor': ('.cursor/mcp.json', cursor), 'vscode': ('.vscode/mcp.json', vscode),
         'opencode': ('opencode.json', opencode)}


def _is_gameframe_file(path, expected=None):
    """True when GameFrame shipped this file, with only managed Copilot tools changed."""
    blob = subprocess.run(['git', 'hash-object', str(path)], capture_output=True, text=True)
    if not blob.returncode and subprocess.run(
            ['git', '-C', str(ROOT), 'cat-file', '-e', blob.stdout.strip()],
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL).returncode == 0:
        return True
    if expected is None:
        return False

    managed = set(provider_catalog())
    base_servers = json.loads((ROOT / '.mcp.json').read_text(encoding='utf-8')).get('mcpServers', {})
    managed.update(base_servers)

    def without_managed_tools(text):
        match = re.search(r'(?m)^tools: (.+)$', text)
        if not match:
            return text
        try:
            names = json.loads(match.group(1))
        except json.JSONDecodeError:
            return text
        names = [name for name in names if not (name.endswith('/*') and name[:-2] in managed)]
        return text[:match.start(1)] + json.dumps(names) + text[match.end(1):]

    try:
        return without_managed_tools(path.read_text(encoding='utf-8')) == without_managed_tools(expected)
    except (OSError, UnicodeDecodeError):
        return False


def _write_current_configs(directory, clients=None):
    """Refresh generated settings and Copilot allowlists, preserving project edits."""
    selected = clients or [name for name, (relpath, _) in FILES.items()
                           if (directory / relpath).is_file()]
    planned = []
    for name in selected:
        relpath, make = FILES[name]
        path = directory / relpath
        text = make(path)
        previous = path.read_text(encoding='utf-8') if path.is_file() else None
        planned.append((path, relpath, text, previous))

    kept_agents = []
    if (directory / '.github' / 'agents').is_dir():
        from clients import render
        for relpath, text in render('copilot').items():
            path = directory / relpath
            if not path.is_file():
                continue
            previous = path.read_text(encoding='utf-8')
            if previous == text:
                continue
            if not _is_gameframe_file(path, text):
                kept_agents.append(relpath)
                continue
            planned.append((path, relpath, text, previous))

    written = []
    try:
        for path, relpath, text, _ in planned:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text, encoding='utf-8')
            written.append(relpath)
    except OSError:
        for path, _, _, previous in planned:
            if previous is None:
                path.unlink(missing_ok=True)
            else:
                path.write_text(previous, encoding='utf-8')
        raise
    return written, [relpath for _, relpath, _, _ in planned if relpath.startswith('.github/agents/')], kept_agents


def _provider_change(args, enable):
    directory = project_root()
    catalog = provider_catalog()
    requested = args.enable if enable else args.disable
    unknown = sorted(set(requested) - set(catalog))
    if unknown:
        action = 'enable' if enable else 'disable'
        print(f"{NAME} mcp: cannot {action} {', '.join(unknown)}. Available providers: "
              f"{', '.join(catalog)}. Use `{NAME} mcp --list`.", file=sys.stderr)
        return 2

    path = directory / '.mcp.json'
    data = json.loads(path.read_text(encoding='utf-8')) if path.is_file() else {}
    local = data.setdefault('mcpServers', {})
    if not isinstance(local, dict):
        print(f'{NAME} mcp: {path} must have an object named mcpServers', file=sys.stderr)
        return 2

    old_config = path.read_text(encoding='utf-8') if path.is_file() else None
    changed = []
    already = []
    for name in requested:
        if enable:
            entry = catalog[name]['server']
            if name in local:
                if local[name] != entry:
                    print(f"{NAME} mcp: {name!r} is already configured in {path} with its own "
                          f"command; left it unchanged. To use GameFrame's launcher and pin, run "
                          f"`{NAME} mcp --disable {name}`, then `{NAME} mcp --enable {name}`.",
                          file=sys.stderr)
                    return 2
                already.append(name)
            else:
                local[name] = entry
                changed.append(name)
        else:
            if name in local:
                del local[name]
                changed.append(name)
            else:
                already.append(name)

    if changed and args.stdout:
        print(f'# {path.name}\n' + json.dumps(data, indent=2))
        return 0
    if changed and args.check:
        print(f"{NAME} mcp: {path} would change: {', '.join(changed)}", file=sys.stderr)
        return 1
    if changed:
        try:
            path.write_text(json.dumps(data, indent=2) + '\n', encoding='utf-8')
            refreshed, refreshed_agents, kept_agents = _write_current_configs(directory)
        except (OSError, ValueError, json.JSONDecodeError) as exc:
            if old_config is None:
                path.unlink(missing_ok=True)
            else:
                path.write_text(old_config, encoding='utf-8')
            print(f'{NAME} mcp: could not update {path} and its client settings: {exc}',
                  file=sys.stderr)
            print('The previous provider and client settings were restored.', file=sys.stderr)
            return 1
        verb = 'Enabled' if enable else 'Disabled'
        versions = ', '.join(f'{name} ({catalog[name]["version"]})' for name in changed)
        print(f'{verb} {versions} in {path}. Refreshed {", ".join(refreshed) or "no generated client files"}.')
        if refreshed_agents:
            print(f'Refreshed Copilot MCP access in {len(refreshed_agents)} generated agent files.')
        if kept_agents:
            print('Kept customized Copilot agent files: ' + ', '.join(kept_agents) +
                  '. To refresh them manually, run `./gameframe agents --emit copilot --out .`.')
        print('Restart the agent session to load the changed MCP servers.')
    elif already:
        state = 'already enabled' if enable else 'already disabled'
        print(f'{", ".join(already)} {state} in {path}.')
    return 0


def list_providers(args):
    active = servers()
    print(f'Project: {project_root()}')
    print('gameframe  enabled (GameFrame toolkit)')
    for name, info in provider_catalog().items():
        state = 'enabled' if name in active else 'off'
        note = ''
        if name in active and active[name] != info['server']:
            # A hand-written entry runs its own command and version, not the pin shown.
            note = ' (own command in .mcp.json, not this pin)'
        print(f'{name:<12} {state:<8} {info["version"]}  {info["description"]}{note}')
    extra = sorted(set(active) - {'gameframe'} - set(provider_catalog()))
    for name in extra:
        print(f'{name:<12} enabled  project-defined provider')
    return 0


def run_provider(args):
    catalog = provider_catalog()
    if args.provider not in catalog:
        print(f"{NAME}: unknown MCP provider {args.provider!r}; use `{NAME} mcp --list`.", file=sys.stderr)
        return 2
    expected = catalog[args.provider]['server']['args'][-1]
    if args.version != expected:
        print(f'{NAME}: provider {args.provider!r} is pinned to {expected}, not {args.version}.',
              file=sys.stderr)
        return 2
    launcher = ROOT / 'scripts' / 'mcp-provider.sh'
    os.execvp('bash', ['bash', str(launcher), args.provider, args.version])


def emit(args):
    if args.enable:
        return _provider_change(args, True)
    if args.disable:
        return _provider_change(args, False)
    if args.list:
        return list_providers(args)

    directory = project_root()
    chosen = CLIENTS if args.emit == 'all' else (args.emit,)
    stale = []
    for client in chosen:
        name, make = FILES[client]
        path = directory / name
        text = make(path)
        if args.stdout:
            print(f'# {name}\n{text}')
        elif args.check:
            if not path.exists() or path.read_text(encoding='utf-8') != text:
                stale.append(name)
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text, encoding='utf-8')
            print(f'wrote {path.relative_to(directory)}')
    if stale:
        print(f'{NAME} mcp: not current with .mcp.json: {", ".join(stale)}. '
              f'Run `{NAME} mcp --emit all` from the project.', file=sys.stderr)
        return 1
    return 0
