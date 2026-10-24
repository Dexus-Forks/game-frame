#!/usr/bin/env python3
"""Shared code for the tools that call Claude with an API key (tools/llm/).

    from client import api, text_of, run_claude, MODEL

These tools spend Claude Platform credits, such as the monthly credits of a Max or Team plan
(docs/working/api-credits.md). Every call needs ANTHROPIC_API_KEY, a key from the Console
organization that holds the credits. They never use your Claude plan's login.

  api()           an anthropic.Anthropic client. ANTHROPIC_BASE_URL points it elsewhere
                  (the tests point it at fake_api.py)
  text_of(msg)    the text blocks of a response, joined
  run_claude(...) runs `claude -p --bare` (Claude Code without your settings, hooks or login:
                  --bare reads only ANTHROPIC_API_KEY) and returns its JSON result.
                  GAMEFRAME_CLAUDE replaces the command (the tests use fake_api.py claude)
  MODEL           GAMEFRAME_MODEL, else claude-opus-5-5

A key that is not scoped to a workspace needs ANTHROPIC_WORKSPACE_ID, the workspace to bill;
both clients send it as the anthropic-workspace-id header. A key scoped to a workspace needs
nothing more, and that workspace's spend limit then applies.

ANTHROPIC_API_KEY, ANTHROPIC_WORKSPACE_ID and GAMEFRAME_MODEL are read from the environment, else from the .env of the
current folder, the project or GameFrame (the first that exists). GAMEFRAME_NO_DOTENV=1 skips
.env (the tests set it).

Run directly, it checks the setup: the SDK, the key and the claude command.

What it does not do: track spending. Set a spend limit on the workspace in the Console, and use
each tool's --budget or --max limits.
"""
import json
import os
import shlex
import shutil
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
PROJECT = os.environ.get('GAMEFRAME_PROJECT') or ROOT


KEYS = ('ANTHROPIC_API_KEY', 'ANTHROPIC_WORKSPACE_ID', 'GAMEFRAME_MODEL')


def load_env():
    """Take ANTHROPIC_API_KEY and GAMEFRAME_MODEL from the project's .env, when the environment
    has none (./gameframe run does not export .env)."""
    if os.environ.get('GAMEFRAME_NO_DOTENV'):
        return
    for folder in (os.getcwd(), PROJECT, ROOT):
        path = os.path.join(folder, '.env')
        if os.path.exists(path):
            for line in open(path, encoding='utf-8'):
                key, eq, value = line.strip().partition('=')
                if eq and key in KEYS and key not in os.environ:
                    os.environ[key] = value.strip().strip('"\'')
            return


load_env()
MODEL = os.environ.get('GAMEFRAME_MODEL', 'claude-opus-5-5')


def need_key():
    if not os.environ.get('ANTHROPIC_API_KEY'):
        sys.exit('ANTHROPIC_API_KEY is not set. Create a key in the Console organization that '
                 'holds your credits, and put it in .env (docs/working/api-credits.md).')


def api():
    need_key()
    try:
        import anthropic
    except ImportError:
        sys.exit('the anthropic package is missing: scripts/setup.sh --install, or '
                 '.venv/bin/pip install -r requirements-llm.txt')
    workspace = os.environ.get('ANTHROPIC_WORKSPACE_ID')
    return anthropic.Anthropic(default_headers={'anthropic-workspace-id': workspace} if workspace else None)


def text_of(msg):
    return ''.join(b.text for b in msg.content if b.type == 'text')


def claude_command():
    if os.environ.get('GAMEFRAME_CLAUDE'):
        return shlex.split(os.environ['GAMEFRAME_CLAUDE'])
    if not shutil.which('claude'):
        sys.exit('the claude command is missing: install Claude Code')
    return ['claude']


def run_claude(prompt, system=None, cwd=None, tools='Read,Glob,Grep,Bash', budget=2.0,
               model=None, timeout=3600):
    """Run one headless Claude Code task with the API key. Returns the parsed JSON result:
    'result' (the final text), 'total_cost_usd', 'is_error' and the rest of what -p prints."""
    need_key()
    cmd = claude_command() + ['-p', '--bare', '--output-format', 'json', '--no-session-persistence',
                              '--model', model or MODEL, '--max-budget-usd', str(budget),
                              '--permission-mode', 'bypassPermissions', '--tools', tools]
    if system:
        cmd += ['--system-prompt', system]
    # A run started from inside a Claude Code session inherits its CLAUDE* variables (its entry
    # point, session id and sockets), and would join that session. Leave them out.
    env = {k: v for k, v in os.environ.items() if not k.startswith('CLAUDE')}
    if env.get('ANTHROPIC_WORKSPACE_ID'):
        env['ANTHROPIC_CUSTOM_HEADERS'] = f'anthropic-workspace-id: {env["ANTHROPIC_WORKSPACE_ID"]}'
    # The prompt goes on stdin: --tools takes a list, and would take a prompt after it as a tool.
    run = subprocess.run(cmd, input=prompt, cwd=cwd, env=env, capture_output=True, text=True, timeout=timeout)
    try:
        return json.loads(run.stdout)
    except json.JSONDecodeError:
        return {'is_error': True, 'result': (run.stdout + run.stderr).strip()[-2000:], 'total_cost_usd': 0}


def main():
    ok = True
    try:
        import anthropic
        print(f'anthropic {anthropic.__version__}')
    except ImportError:
        print('anthropic: missing (scripts/setup.sh --install)'); ok = False
    print('ANTHROPIC_API_KEY: ' + ('set' if os.environ.get('ANTHROPIC_API_KEY') else 'missing'))
    ok = ok and bool(os.environ.get('ANTHROPIC_API_KEY'))
    print('claude: ' + (' '.join(claude_command()) if os.environ.get('GAMEFRAME_CLAUDE') or shutil.which('claude') else 'missing'))
    print(f'model: {MODEL}')
    return 0 if ok else 1


if __name__ == '__main__':
    if sys.argv[1:] in (['-h'], ['--help']):
        print(__doc__); sys.exit(0)
    sys.exit(main())
