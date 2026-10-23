"""The note Claude Code adds at the start of a session: the tools found, and the next open items."""
import json
import os
import re


def context(root, gameframe):
    """The note for the project at root; gameframe is the GameFrame folder in use."""
    facts = []
    env = os.path.join(root, '.env')
    copy = os.path.exists(os.path.join(root, 'tools', 'pe', 'peinfo.py'))
    setup = 'scripts/setup.sh' if copy else './gameframe doctor'
    facts.append('.env present' if os.path.exists(env) else f'no .env: run {setup}')
    facts.append('.venv present' if os.path.isdir(os.path.join(root, '.venv')) else
                 f'no .venv: run {setup} --install')
    if os.path.isdir(os.path.join(root, 'analysis', 'export')):
        facts.append('Ghidra export present in analysis/export')
    checklist = os.path.join(root, 'docs', 'CHECKLIST.md')
    todo = []
    if os.path.exists(checklist):
        for line in open(checklist, encoding='utf-8', errors='replace'):
            if re.match(r'\s*[-*] \[ \]', line):
                todo.append(line.strip()[6:].strip())
            if len(todo) == 3:
                break
    text = 'GameFrame project: ' + '; '.join(facts) + '.'
    if not copy and os.path.realpath(gameframe) != os.path.realpath(root):
        text += (f' GameFrame comes from its plugin, at {gameframe}. Its skills and agents name guides and tools'
                 f' by GameFrame\'s paths: read docs/NAME.md as {gameframe}/docs/NAME.md, and run a tool from'
                 ' this folder with ./gameframe run NAME (./gameframe tools lists them; ./gameframe python FILE'
                 ' runs a project script with the toolkit on its path).')
    if todo:
        text += ' Next open items in docs/CHECKLIST.md: ' + ' | '.join(todo) + '.'
    text += (' When a command fails, or the work turns to a new era, runtime or hardware, run'
             ' ./gameframe advise "<error text or topic>" for known failures, fixes and dead ends.')
    return json.dumps({'hookSpecificOutput': {'hookEventName': 'SessionStart',
                                              'additionalContext': text}})
