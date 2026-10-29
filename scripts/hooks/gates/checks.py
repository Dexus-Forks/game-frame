"""The checks behind every gate. No agent's hook format appears here.

A Check names one action: a commit, a push, a write or a shell command that may delete. reason(check) returns why the action is
blocked, or None. The adapters build Check records from each agent's hook input; the git
pre-commit hook builds one from the environment.
"""
import os
import re
import subprocess
from dataclasses import dataclass

try:
    from . import deletes, version  # imported as gates.checks (tools/site/check.py)
except ImportError:
    import deletes
    import version

# Folders that hold the game, its derivatives or decompiled code. They are ignored by git; a
# staged path there was force-added. game/, media/, saves/, analysis/ and derived/ are links into the
# GameFrame home in a project made by new-project.sh; the links themselves never go in either.
GAME_DIRS = ('game/', 'media/', 'saves/', 'target/', 'analysis/', 'derived/', 'game-data/', 'save-game/')
HOME_LINKS = {'game', 'media', 'saves', 'analysis', 'derived'}
# Extensions of game binaries and data files that never belong in a repository.
GAME_EXTS = {
    '.exe', '.dll', '.ovl', '.com', '.sys', '.pkg', '.crs', '.rsc', '.str', '.sav', '.wad',
    '.pak', '.pk3', '.grp', '.mpq', '.bif', '.gob', '.rez', '.vpk', '.iso', '.cue', '.bin',
    '.nes', '.sfc', '.smc', '.gba', '.gb', '.gbc', '.n64', '.z64', '.v64', '.md', '.gen',
}
# .md is a Mega Drive ROM extension too; Markdown is far more common in a repository.
GAME_EXTS.discard('.md')
# Lines that Ghidra's decompiler writes and people do not. The English word "undefined" and
# FUN_/DAT_ names cited in a sentence pass; Ghidra's types, declarations and statements do not.
GHIDRA_NAME = r'(?:FUN|DAT)_[0-9a-fA-F]{4,}(?:_[0-9a-fA-F]{4})?'
PSEUDOCODE = re.compile(
    r'\bundefined[1248]\b|\bundefined\s*\*|'                         # sized types, pointers
    r'\bundefined\s+\**(?:[a-z]{0,3}Var\d+|local_\w+|param_\d+|[a-z]{0,3}Stack_\w+)|'  # variables
    r'\bCONCAT\d\d\(|\bSUB\d\d\(|\bZEXT\d\d\(|\(\*\*\(code \*\*\)|'
    r'/\* WARNING: |\bunaff_\w+|\bin_stack_\w+|\bextraout_\w+|'
    r'\b' + GHIDRA_NAME + r'\s*\([^()]*\)\s*;|'                        # a call statement
    r'\b' + GHIDRA_NAME + r'\s*=[^=]|=\s*(?:\([\w\s*]+\)\s*)?' + GHIDRA_NAME + r'\s*;|'  # assignments
    r'\b(?:void|int|uint|char|short|long|bool|float|double|code)\s+\**' + GHIDRA_NAME + r'\s*\('  # declarations
)
# Paths where decompiled code may live (all ignored by git).
PRIVATE_DIRS = ('analysis/', 'derived/', 'game/', 'saves/', 'target/')
LANDER = 're-lander'
# The variable that names the GameFrame agent where the hook input does not.
AGENT_VAR = 'GAMEFRAME_AGENT'
# `git commit`, `git -C dir commit`, `git --no-pager commit`, and the same for push.
GIT_VERB = r'\bgit(?:\s+-[cC]\s+\S+|\s+--\S+)*\s+{}\b'
# `git commit -a` stages tracked changes as it commits.
COMMIT_ALL = re.compile(r'(^|\s)(-a|--all|-[a-zA-Z]*a[a-zA-Z]*)(\s|$)')


@dataclass
class Check:
    kind: str                # 'commit', 'push', 'write' or 'delete'
    cwd: str
    agent: str = None        # a GameFrame agent (re-tracer), or None for a person or main session
    command: str = ''        # the shell command, for commit, push and delete
    path: str = ''           # the file written, for write
    text: str = ''           # the text written, for write


def agent_name(raw):
    """re-tracer from re-tracer or gameframe:re-tracer; None for anything else."""
    name = (raw or '').strip().split(':')[-1]
    return name if name.startswith('re-') else None


def command_agent(command):
    """The agent a command names with a leading GAMEFRAME_AGENT=re-tracer, or None."""
    m = re.search(AGENT_VAR + r'=([\'"]?)([\w:.-]+)\1', command or '')
    return agent_name(m.group(2)) if m else None


def env_agent():
    return agent_name(os.environ.get(AGENT_VAR))


def git_verbs(command):
    """Which of commit and push a shell command runs."""
    return [v for v in ('commit', 'push') if re.search(GIT_VERB.format(v), command or '')]


def git(*args, cwd):
    return subprocess.run(['git', *args], cwd=cwd, capture_output=True, text=True).stdout


def pseudocode_lines(text):
    return [line for line in text.splitlines() if PSEUDOCODE.search(line)]


def agent_reason(kind, agent):
    if agent and agent != LANDER:
        return (f'{agent} does not {kind}: re-lander commits and pushes. End with your hand-off'
                + (' and name re-lander as Next.' if kind == 'commit' else '.'))
    return None


def staged_reason(cwd, with_tracked=False):
    """Game material or decompiled code in the index; with_tracked adds unstaged tracked changes."""
    staged = [p for p in git('diff', '--cached', '--name-only', cwd=cwd).splitlines() if p]
    if with_tracked:
        staged += [p for p in git('diff', '--name-only', cwd=cwd).splitlines() if p]
    bad = [p for p in staged
           if p.startswith(GAME_DIRS) or p in HOME_LINKS or os.path.splitext(p)[1].lower() in GAME_EXTS]
    if bad:
        return ('This commit stages game material, which never goes into git:\n  '
                + '\n  '.join(bad[:10])
                + '\nUnstage it (git restore --staged PATH). See docs/LEGAL.md.')
    added = [l[1:] for l in git('diff', '--cached', '--unified=0', cwd=cwd).splitlines()
             if l.startswith('+') and not l.startswith('+++')]
    hits = pseudocode_lines('\n'.join(added))
    if len(hits) >= 2:
        return ('The staged changes contain lines that look like decompiled code:\n  '
                + '\n  '.join(h.strip()[:100] for h in hits[:5])
                + '\nDescribe the behaviour in prose with addresses and constants instead.')
    return None


def write_reason(cwd, path, text):
    rel = os.path.relpath(path, cwd) if os.path.isabs(path) else os.path.normpath(path)
    if rel.startswith(PRIVATE_DIRS) or rel.startswith('..'):
        return None
    hits = pseudocode_lines(text)
    if len(hits) >= 2:
        return (f'{rel}: this looks like decompiled code, which stays in analysis/:\n  '
                + '\n  '.join(h.strip()[:100] for h in hits[:5])
                + '\nWrite the finding in prose, with the function address, offsets and constants.')
    return None


def reason(check):
    """Why this action is blocked, or None."""
    if check.kind == 'write':
        return write_reason(check.cwd, check.path, check.text)
    if check.kind == 'delete':
        return deletes.reason(check.cwd, check.command)
    why = agent_reason(check.kind, check.agent)
    if why or check.kind == 'push':
        return why
    return staged_reason(check.cwd, bool(COMMIT_ALL.search(check.command))) or version.reason(check.cwd)
