"""GitHub Copilot: .github/agents/<name>.agent.md, Markdown with name, description and a tools allowlist.

Copilot takes only an allowlist: `tools` names the aliases (read, search, edit, execute, todo,
web, agent) the agent's denied Claude Code tools leave, and `server/*` for each MCP server in
.mcp.json. VS Code also reads .claude/agents directly; these files serve Copilot on GitHub.com
and the Copilot CLI.
"""
import json

from clients import allowed, mcp_servers

DIR = '.github/agents'


def path(name):
    return f'{DIR}/{name}.agent.md'


def render(agent, generated):
    tools = allowed(agent['cannot_use'], 1)
    if 'mcp' not in agent['cannot_use']:
        tools += [f'{s}/*' for s in mcp_servers()]
    return (f"---\nname: {agent['name']}\ndescription: {json.dumps(agent['description'])}\n"
            f"tools: {json.dumps(tools)}\n---\n\n<!-- {generated} -->\n\n{agent['body']}")
