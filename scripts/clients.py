#!/usr/bin/env python3
"""The files each coding agent needs in a project made with GameFrame.

Usage (new-project.sh and update-project.sh call it; it also runs on its own):

    scripts/clients.py write DEST [--agent LIST] [--plugin] [--dry-run]
    scripts/clients.py detect DEST
    scripts/clients.py list

LIST names the coding agents (clients) to set the project up for, separated by commas:
claude, codex, gemini, copilot, cursor, opencode, or all (the default for write).

write puts each chosen client's files into DEST:
- the skill links: .claude/skills for Claude Code, .agents/skills for the others. In a plugin
  project (--plugin) Claude Code gets the skills from the plugin, so the other clients get a copy
  of the skills in skills/ and only the .agents/skills link;
- the agents' copies: .codex/agents, .gemini/agents, .github/agents, .opencode/agents. Cursor
  reads .claude/agents in a copied project. A plugin project gets a copy in .cursor/agents,
  since agents in .claude/agents would stand beside the plugin's own in Claude Code;
- the gate hooks: .codex/hooks.json, the hooks in .gemini/settings.json, .cursor/hooks.json,
  .github/hooks/gameframe.json. In a plugin project they run the gates through ./gameframe gate;
- the MCP servers and the rules file: .codex/config.toml, .gemini/settings.json,
  .cursor/mcp.json, .vscode/mcp.json, opencode.json.
For each file: missing, it is added; the same as what GameFrame writes now, it is left alone; a
file that GameFrame's history holds exactly (an older copy, or a copied project's hooks), it is
replaced; anything else was changed by the project, and is kept and listed. A plugin project's
hooks and .gemini/settings.json are rewritten, not copied, so a later change there is kept and
listed too: compare it with GameFrame's.
Claude Code's own files (.claude/settings.json, CLAUDE.md, .mcp.json, .claude/agents) stay with
new-project.sh and update-project.sh.

detect prints the clients DEST already has files for. list prints the client names.
It never removes a client's files: delete them by hand to drop a client. Standard library only.
"""
import json
import os
import re
import shutil
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CLIENTS = ('claude', 'codex', 'gemini', 'copilot', 'cursor', 'opencode')
# The generated agent copies of each client (gameframe agents --emit).
AGENT_DIRS = {'codex': '.codex/agents', 'gemini': '.gemini/agents', 'copilot': '.github/agents',
              'opencode': '.opencode/agents'}
# Files copied as they are in GameFrame, except that a plugin project's hooks run ./gameframe.
CONFIGS = {
    'codex': ('.codex/hooks.json', '.codex/config.toml'),
    'copilot': ('.github/hooks/gameframe.json', '.vscode/mcp.json'),
    'cursor': ('.cursor/hooks.json', '.cursor/mcp.json'),
    'opencode': ('opencode.json',),
}
# A file that shows a project is set up for a client.
MARKERS = {
    'claude': ('.claude/settings.json',), 'codex': ('.codex/agents', '.codex/hooks.json'),
    'gemini': ('.gemini/agents', '.gemini/settings.json'), 'copilot': ('.github/agents', '.github/hooks/gameframe.json'),
    'cursor': ('.cursor/hooks.json', '.cursor/agents'), 'opencode': ('.opencode/agents', 'opencode.json'),
}


def parse(text):
    """'codex,gemini' or 'all' -> the client names, in CLIENTS order."""
    names = {n.strip().lower() for n in ('all' if text is None else text).split(',') if n.strip()}
    if not names:
        raise SystemExit(f'--agent needs a list: {", ".join(CLIENTS)} or all')
    if 'all' in names:
        return list(CLIENTS)
    unknown = sorted(names - set(CLIENTS))
    if unknown:
        raise SystemExit(f'unknown agent {", ".join(unknown)}: choose from {", ".join(CLIENTS)} or all')
    return [c for c in CLIENTS if c in names]


def detect(dest):
    return [c for c in CLIENTS if any(os.path.exists(os.path.join(dest, m)) for m in MARKERS[c])]


def via_launcher(text):
    """Run the gates through the project's ./gameframe launcher instead of scripts/hooks/gate.py."""
    text = re.sub(r'python3? \\"(.*?)scripts/hooks/gate\.py\\"', r'\\"\1gameframe\\" gate', text)
    text = re.sub(r'"python scripts/hooks/gate\.py ', '"bash ./gameframe gate ', text)
    return re.sub(r'python3? scripts/hooks/gate\.py ', './gameframe gate ', text)


def gemini_settings(plugin):
    """.gemini/settings.json: the MCP servers and AGENTS.md, plus the gate hooks."""
    cfg = json.load(open(os.path.join(ROOT, '.gemini', 'settings.json')))
    cfg.update(json.load(open(os.path.join(ROOT, 'hooks', 'gemini-settings.json'))))
    text = json.dumps(cfg, indent=2) + '\n'
    return via_launcher(text) if plugin else text


def git_blob(path):
    return subprocess.run(['git', 'hash-object', path], capture_output=True, text=True).stdout.strip()


def shipped(blob):
    """True when GameFrame's history holds this exact file: an older copy, safe to replace."""
    return bool(blob) and subprocess.run(['git', '-C', ROOT, 'cat-file', '-e', blob], capture_output=True).returncode == 0


class Writer:
    def __init__(self, dest, dry):
        self.dest, self.dry = dest, dry
        self.added, self.replaced, self.kept, self.same = [], [], [], 0

    def put(self, rel, text):
        """Write rel with text; replace a file GameFrame shipped; keep the project's own edits."""
        path = os.path.join(self.dest, rel)
        if os.path.isfile(path):
            if open(path, encoding='utf-8', errors='replace').read() == text:
                self.same += 1
                return
            if not shipped(git_blob(path)):
                self.kept.append(f'{rel} (compare it with GameFrame\'s)')
                return
            self.replaced.append(rel)
        else:
            self.added.append(rel)
        if not self.dry:
            os.makedirs(os.path.dirname(path) or '.', exist_ok=True)
            with open(path, 'w', encoding='utf-8') as f:
                f.write(text)

    def copy(self, rel, src_rel=None, plugin=False):
        text = open(os.path.join(ROOT, src_rel or rel), encoding='utf-8').read()
        self.put(rel, via_launcher(text) if plugin and 'hooks' in rel else text)

    def tree(self, src_rel, dst_rel):
        for base, _, files in os.walk(os.path.join(ROOT, src_rel)):
            for name in sorted(files):
                if name == '__pycache__' or name.endswith('.pyc'):
                    continue
                rel = os.path.relpath(os.path.join(base, name), ROOT)
                self.copy(os.path.join(dst_rel, os.path.relpath(rel, src_rel)), rel)

    def link(self, rel):
        """rel -> ../skills; a copy of skills/ where links cannot be made."""
        path, target = os.path.join(self.dest, rel), os.path.join(self.dest, 'skills')
        if os.path.islink(path) or os.path.isdir(path):
            return
        self.added.append(rel)
        if self.dry:
            return
        os.makedirs(os.path.dirname(path), exist_ok=True)
        try:
            os.symlink(os.path.join('..', 'skills'), path, target_is_directory=True)
        except OSError:
            shutil.copytree(target, path)


def write(dest, clients, plugin=False, dry=False):
    w = Writer(os.path.abspath(dest), dry)
    others = [c for c in clients if c != 'claude']
    if plugin and others:
        w.tree('skills', 'skills')
    if 'claude' in clients and not plugin:
        w.link('.claude/skills')
    if others:
        w.link('.agents/skills')
    for c in others:
        if c in AGENT_DIRS:
            for name in sorted(os.listdir(os.path.join(ROOT, AGENT_DIRS[c]))):
                if name.startswith('re-'):
                    w.copy(f'{AGENT_DIRS[c]}/{name}')
        elif c == 'cursor' and plugin:
            for name in sorted(os.listdir(os.path.join(ROOT, 'agents'))):
                w.copy(f'.cursor/agents/{name}', f'agents/{name}')
        for rel in CONFIGS.get(c, ()):
            w.copy(rel, plugin=plugin)
        if c == 'gemini':
            w.put('.gemini/settings.json', gemini_settings(plugin))
    return w


def main(argv):
    if not argv or argv[0] in ('-h', '--help') or argv[0] not in ('write', 'detect', 'list'):
        print(__doc__.strip())
        return 0 if argv and argv[0] in ('-h', '--help') else 2
    if argv[0] == 'list':
        print(' '.join(CLIENTS))
        return 0
    if len(argv) < 2:
        print(__doc__.strip())
        return 2
    dest = argv[1]
    if argv[0] == 'detect':
        print(','.join(detect(dest)))
        return 0
    opts, agent = argv[2:], None
    if '--agent' in opts:
        i = opts.index('--agent')
        agent = opts[i + 1] if i + 1 < len(opts) else ''
    w = write(dest, parse(agent), plugin='--plugin' in opts, dry='--dry-run' in opts)
    verb = 'would be' if w.dry else 'were'
    print(f'Agent files: {len(w.added)} {verb} added, {len(w.replaced)} {verb} replaced, {w.same} were already current.')
    for f in w.kept:
        print(f'  kept {f}')
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
