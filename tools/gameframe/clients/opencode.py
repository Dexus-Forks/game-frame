"""OpenCode: .opencode/agents/<name>.md, Markdown with description, mode and a permission map.

OpenCode names an agent after its file. `mode: subagent` keeps the agent out of the list of
primary agents. `permission` denies each OpenCode permission that only denied Claude Code tools
map to (`task` for Agent, `webfetch` and `websearch` for the web); the rest stay as the user's
own settings say.
"""
import json

from clients import allowed, body

DIR = '.opencode/agents'


def path(name):
    return f'{DIR}/{name}.md'


def render(agent, generated):
    keep = allowed(agent['cannot_use'], 2)
    deny = [p for p in allowed([], 2, every=True) if p not in keep]
    perms = ''.join(f'\n  {p}: deny' for p in deny)
    return (f"---\ndescription: {json.dumps(agent['description'])}\nmode: subagent\n"
            f"permission:{perms or ' {}'}\n---\n\n<!-- {generated} -->\n\n{body(agent)}")
