"""Gemini CLI: .gemini/agents/<name>.md, Markdown with name, description, kind and a tools allowlist.

Gemini CLI takes only an allowlist, so `tools` names every built-in tool the agent's denied
Claude Code tools leave, and `mcp_*` for the MCP servers. A Gemini subagent cannot start other
subagents whatever its list says, so a denied Agent needs no entry.
"""
import json

from clients import allowed

DIR = '.gemini/agents'


def path(name):
    return f'{DIR}/{name}.md'


def render(agent, generated):
    tools = '\n'.join(f'  - {t}' for t in allowed(agent['cannot_use'], 0))
    return (f"---\nname: {agent['name']}\ndescription: {json.dumps(agent['description'])}\n"
            f"kind: local\ntools:\n{tools}\n---\n\n<!-- {generated} -->\n\n{agent['body']}")
