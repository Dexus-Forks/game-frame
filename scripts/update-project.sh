#!/usr/bin/env bash
# Bring a project made with GameFrame up to GameFrame's current version.
#
#   scripts/update-project.sh DIR [--plugin] [--dry-run]
#
# A project normally gets GameFrame's skills, agents, gates and MCP servers from its Claude Code
# plugin, and runs the toolkit through its own small `gameframe` launcher (new-project.sh makes
# such projects). For those, this script only refreshes the launcher and the plugin settings.
#
# An older project, or one made with new-project.sh --copy, holds a copy of GameFrame's toolkit,
# skills, agents, gates and guides. For each file GameFrame provides:
#   - missing in the project: added;
#   - the same as GameFrame's: left alone;
#   - an older version of GameFrame's file (its exact contents are in GameFrame's git history):
#     replaced with the current one;
#   - changed in the project (contents GameFrame never had): kept, and listed at the end, so a
#     project's own edits are never lost.
# The skills live in skills/, with the links .claude/skills and .agents/skills to it (a copy where
# links cannot be made). An older project's skills in .claude/skills move to skills/ first.
# --plugin moves such a project to the plugin: every copied file that is GameFrame's (current or
# older) is removed, the project's changed copies are kept and listed, `gameframe` becomes the
# launcher, and .claude/settings.json enables the plugin in place of the local gates. It then
# lists the project's own files that still name a removed tool path.
# The project's own files (AGENTS.md, README.md, docs/FINDINGS.md, docs/CHECKLIST.md, labels/,
# its code) are never touched. With --dry-run nothing is written. The project's git history shows
# every change; commit it like any other.
set -euo pipefail
. "$(dirname "$0")/lib.sh"
[[ $# -ge 1 ]] || { sed -n '2,26p' "$0"; exit 2; }
dir="$1"; shift; dry=""; plugin=""
for a in "$@"; do case "$a" in --dry-run) dry=--dry-run ;; --plugin) plugin=--plugin ;; *) echo "unknown option $a" >&2; exit 2 ;; esac; done
[[ -d "$dir/.git" || -f "$dir/.git" ]] || { echo "$dir is not a git repository" >&2; exit 1; }
"${PYTHON:-python3}" - "$ROOT" "$dir" "$dry" "$plugin" <<'PY'
import json, os, re, shutil, subprocess, sys
root, dest = os.path.abspath(sys.argv[1]), os.path.abspath(sys.argv[2])
dry, to_plugin = sys.argv[3] == '--dry-run', sys.argv[4] == '--plugin'
launcher = os.path.join(root, 'templates', 'project', 'gameframe')
verb = 'would be' if dry else 'were'

def blob(path):
    return subprocess.run(['git', '-C', root, 'hash-object', path], capture_output=True, text=True).stdout.strip()

def known(h):
    return subprocess.run(['git', '-C', root, 'cat-file', '-e', h], capture_output=True).returncode == 0

def tracked(sub):
    out = subprocess.run(['git', '-C', root, 'ls-files', '-z', sub], capture_output=True, text=True).stdout
    return [f for f in out.split('\0') if f]

# (path in GameFrame, path in the project)
pairs = []
for sub in ('tools', 'scripts'):
    pairs += [(f, f) for f in tracked(sub)]
pairs += [(f, f) for f in tracked('skills') if f.split('/')[1].startswith('re-')]
pairs += [(f, '.claude/' + f) for f in tracked('agents')]
for sub in ('.codex/agents', '.gemini/agents', '.github/agents', '.opencode/agents'):
    pairs += [(f, f) for f in tracked(sub) if f.split('/')[-1].startswith('re-')]
pairs += [(f, f) for f in ('gameframe', 'requirements.txt', 'requirements-live.txt', '.mcp.json')]
for f in ('ABI', 'GAME-PATTERNS', 'DOS', 'LEGAL', 'OUTPUTS', 'SETUP', 'COMPILED-CODE', 'TYPES'):
    pairs.append((f'docs/{f}.md', f'docs/{f}.md'))
for d in ('games', 'observe', 'engines', 'hardware', 'consoles', 'handhelds', 'computers', 'arcade', 'media',
          'working', 'teams', 'case-studies', 'preservation', 'assets'):
    pairs += [(f, f) for f in tracked(f'docs/{d}')]

SKILL_LINKS = ('.claude/skills', '.agents/skills')

def move_skills():
    """Move an older project's skills from .claude/skills (a folder) to skills/."""
    old = os.path.join(dest, '.claude', 'skills')
    if os.path.islink(old) or not os.path.isdir(old) or os.path.isdir(os.path.join(dest, 'skills')):
        return
    if not dry:
        shutil.move(old, os.path.join(dest, 'skills'))

def link_skills():
    """.claude/skills and .agents/skills point to skills/; a copy where links cannot be made."""
    target = os.path.join(dest, 'skills')
    for link in SKILL_LINKS:
        p = os.path.join(dest, link)
        if dry or os.path.islink(p) or not os.path.isdir(target):
            continue
        if os.path.isdir(p):
            if not set(os.listdir(p)) <= set(os.listdir(target)):
                print(f'{link} holds skills that skills/ does not: move them to skills/, then run this again.')
                continue
            shutil.rmtree(p)
        os.makedirs(os.path.dirname(p), exist_ok=True)
        try:
            os.symlink(os.path.join('..', 'skills'), p, target_is_directory=True)
        except OSError:
            shutil.copytree(target, p)

def unlink_skills():
    """Remove the links once skills/ is gone."""
    if dry or os.path.isdir(os.path.join(dest, 'skills')):
        return
    for link in SKILL_LINKS:
        p = os.path.join(dest, link)
        if os.path.islink(p):
            os.remove(p)
        elif os.path.isdir(p):
            shutil.rmtree(p)
    if os.path.isdir(os.path.join(dest, '.agents')) and not os.listdir(os.path.join(dest, '.agents')):
        os.rmdir(os.path.join(dest, '.agents'))

def is_plugin_project():
    d = os.path.join(dest, 'gameframe')
    return os.path.exists(d) and blob(d) in (blob(launcher),) or (
        os.path.exists(d) and known(blob(d)) and 'GAMEFRAME_PROJECT' in open(d, errors='replace').read())

def plugin_settings():
    """Enable the plugin in .claude/settings.json and drop the local gates."""
    path = os.path.join(dest, '.claude', 'settings.json')
    template = json.load(open(os.path.join(root, 'templates', 'project', 'settings.json')))
    if not os.path.exists(path):
        cfg = template
    else:
        cfg = json.load(open(path))
        for event, groups in list(cfg.get('hooks', {}).items()):
            keep = []
            for g in groups:
                g['hooks'] = [h for h in g.get('hooks', []) if 'scripts/hooks/gate.py' not in json.dumps(h)]
                if g['hooks']:
                    keep.append(g)
            if keep:
                cfg['hooks'][event] = keep
            else:
                del cfg['hooks'][event]
        if 'hooks' in cfg and not cfg['hooks']:
            del cfg['hooks']
        cfg.setdefault('extraKnownMarketplaces', {}).update(template['extraKnownMarketplaces'])
        cfg.setdefault('enabledPlugins', {}).update(template['enabledPlugins'])
        perms = cfg.setdefault('permissions', {})
        for kind in ('allow', 'ask'):
            rules = [r for r in perms.get(kind, []) if not re.match(r'Bash\((scripts/|\.venv/bin/python tools/)', r)]
            for r in template['permissions'][kind]:
                if r not in rules:
                    rules.append(r)
            perms[kind] = rules
    if not dry:
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, 'w') as f:
            json.dump(cfg, f, indent=2)
            f.write('\n')

def put_launcher():
    d = os.path.join(dest, 'gameframe')
    state = 'current' if os.path.exists(d) and blob(d) == blob(launcher) else 'written'
    if not dry and state == 'written':
        shutil.copy2(launcher, d)
    return state

if is_plugin_project() and not to_plugin:
    state = put_launcher()
    plugin_settings()
    print(f'Plugin project: the gameframe launcher is {state}; .claude/settings.json enables the plugin.')
    sys.exit(0)

added, replaced, kept, same = [], [], [], 0
move_skills()
if to_plugin:
    removed = []
    for src, dst in pairs:
        s, d = os.path.join(root, src), os.path.join(dest, dst)
        if not os.path.isfile(d):
            continue
        if (os.path.isfile(s) and blob(d) == blob(s)) or known(blob(d)):
            removed.append(dst)
            if not dry:
                os.remove(d)
        else:
            kept.append(dst)
    if not dry:
        for top in ('tools', 'scripts', 'skills', '.claude/agents', 'docs', '.codex', '.gemini', '.github', '.opencode'):
            for base, dirs, files in os.walk(os.path.join(dest, top), topdown=False):
                if not os.listdir(base):
                    os.rmdir(base)
    unlink_skills()
    put_launcher()
    plugin_settings()
    print(f'{len(removed)} copied GameFrame files {verb} removed; gameframe {verb} replaced by the launcher; '
          '.claude/settings.json enables the plugin.')
    if kept:
        print(f'Kept, because the project changed them ({len(kept)}):')
        for f in sorted(kept):
            print(f'  {f}')
    gone = {dst for _, dst in pairs if dst.startswith(('tools/', 'scripts/'))} - set(kept)
    pattern = re.compile(r'(?<![\w/.-])((?:tools|scripts)/[\w./-]+\.(?:py|sh|java))')
    mentions = {}
    out = subprocess.run(['git', '-C', dest, 'ls-files', '-z'], capture_output=True, text=True).stdout
    for f in out.split('\0'):
        p = os.path.join(dest, f)
        if not f or f in gone or f in removed or not os.path.isfile(p) or os.path.getsize(p) > 2_000_000:
            continue
        try:
            text = open(p, encoding='utf-8').read()
        except (UnicodeDecodeError, OSError):
            continue
        lines = [l for l in text.splitlines() if not re.search(r'Generated by|reverse-engineer-games|GameFrame', l)]
        hits = sorted({m for l in lines for m in pattern.findall(l) if m in gone})
        if hits:
            mentions[f] = hits
    if mentions:
        print(f'Project files that name a removed tool ({len(mentions)}). Run it as ./gameframe run NAME,')
        print('or ./gameframe python FILE for scripts that import the toolkit:')
        for f, hits in sorted(mentions.items())[:40]:
            print(f'  {f}: {", ".join(hits[:4])}' + (' ...' if len(hits) > 4 else ''))
    sys.exit(0)

for src, dst in pairs:
    s, d = os.path.join(root, src), os.path.join(dest, dst)
    if not os.path.isfile(s):
        continue
    if not os.path.exists(d):
        added.append(dst)
    elif blob(d) == blob(s):
        same += 1
        continue
    elif known(blob(d)):
        replaced.append(dst)
    else:
        kept.append(dst)
        continue
    if not dry:
        os.makedirs(os.path.dirname(d) or '.', exist_ok=True)
        shutil.copy2(s, d)

link_skills()
settings = os.path.join(dest, '.claude', 'settings.json')
if os.path.exists(settings):
    kept.append('.claude/settings.json (compare it with GameFrame\'s: the gates are in "hooks")')
else:
    added.append('.claude/settings.json')
    if not dry:
        os.makedirs(os.path.dirname(settings), exist_ok=True)
        shutil.copy2(os.path.join(root, '.claude', 'settings.json'), settings)

print(f'{len(added)} files {verb} added, {len(replaced)} {verb} replaced with the current version, {same} were already current.')
for name, files in (('Added', added), ('Replaced', replaced)):
    if files:
        print(f'{name}: ' + ', '.join(sorted(files)[:12]) + (f' and {len(files) - 12} more' if len(files) > 12 else ''))
if kept:
    print(f'Kept, because the project changed them ({len(kept)}):')
    for f in sorted(kept):
        print(f'  {f}')
print('This project holds a copy of GameFrame. scripts/update-project.sh DIR --plugin moves it to the plugin.')
PY
