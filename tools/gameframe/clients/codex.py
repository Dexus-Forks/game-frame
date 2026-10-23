"""Codex: .codex/agents/<name>.toml, with name, description and developer_instructions.

Codex takes no per-agent tool list: an agent inherits the session's tools, sandbox and MCP
servers. The instructions say what the agent may not do (clients.limits), and a denied Bash
turns off the shell tool, the one tool a Codex agent file can remove.
"""
import json

from clients import body as full_body

DIR = '.codex/agents'


def path(name):
    return f'{DIR}/{name}.toml'


def render(agent, generated):
    body = full_body(agent, with_limits=True)
    if "'''" in body:
        raise ValueError(f"{agent['name']}: the body holds ''' and cannot be a TOML literal string")
    lines = [f'# {generated}',
             f"name = {json.dumps(agent['name'])}",
             f"description = {json.dumps(agent['description'])}",
             f"developer_instructions = '''\n{body.rstrip()}\n'''"]
    if 'Bash' in agent['cannot_use']:
        lines += ['', '[features]', 'shell_tool = false']
    return '\n'.join(lines) + '\n'
