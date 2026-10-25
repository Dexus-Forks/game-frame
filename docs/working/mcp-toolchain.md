# The GameFrame MCP tool surface

GameFrame's stdio MCP server gives agents one discoverable interface to the local toolkit. The
project's `.mcp.json` can also register provider servers such as REA and DOSBox-X. Those providers
remain independent processes owned by the MCP client. The local server lives in
`tools/gameframe/mcp_server/` and reuses the existing GameFrame APIs and command catalog.

## Tools available to an agent

| Group | MCP tools | What they return or do |
|---|---|---|
| Know-how | `advise`, `knowhow_entry`, `knowhow_list`, `find` | Read failure guidance, context entries, and matching lines in skill and tool help. |
| Workflow | `gameframe_steps`, `gameframe_skills`, `gameframe_skill`, `gameframe_agents` | Read the method, skill text, and agent hand-offs. |
| Tool discovery | `gameframe_tool_catalog`, `gameframe_tool_help` | List each tool, its kind and run support, then read its own usage and limits. |
| Tool execution | `gameframe_run_tool` | Run any catalog program or script with a separate string for each command-line argument. |
| Project state | `gameframe_progress`, `gameframe_mcp_providers` | Read progress logs and list configured provider names and launch metadata without argument or environment values. |

Use the catalog, then the tool's help, then the runner. The runner calls only entries that the
catalog marks runnable. It passes an argument array to the process without shell parsing. It
returns the tool name and path, arguments, working directory, exit status, duration and captured
output. It caps output size, reports progress for long calls and stops a child process when the
client cancels the request or the timeout expires. A tool's own output names files it created.

The catalog also lists import-only libraries. The runner does not import or execute those
modules. Read their help and use a documented program, script or a dedicated MCP handler. Ghidra
scripts run through the catalog's `ghidra-run` and `ghidra-export` scripts.

## Modules

Keep the server in small modules, each with one responsibility.

| Module | Responsibility |
|---|---|
| `serve.py` | Compatibility entry point for `gameframe serve`. |
| `mcp_server/server.py` | Stdio lifecycle, concurrent calls and cancellation routing. |
| `mcp_server/protocol.py` | JSON-RPC and MCP message construction. |
| `mcp_server/registry.py` | Tool definitions, schemas and dispatch. |
| `mcp_server/context.py` | Request progress and cancellation state. |
| `mcp_server/toolsets/knowledge.py` | Know-how and text search. |
| `mcp_server/toolsets/discovery.py` | Workflow, skills, agents, catalog and project state. |
| `mcp_server/toolsets/runner.py` | Allow-listed execution through the existing catalog. |

Add a tool to the module for its capability. Give it an input schema and a short description.
Keep handlers small. Put process execution in `runner.py` and transport behavior in `server.py`.
Do not add all new behavior to `serve.py`.

## Provider boundaries and limits

The client starts and connects to each server in `.mcp.json`. `gameframe_mcp_providers` reports
configuration only; the client owns live connection state and provider tools. REA and DOSBox-X
therefore keep their own protocols and APIs. GameFrame does not proxy them through a second
server.

The current catalog runner covers runnable GameFrame programs and shell scripts. Import-only
libraries need dedicated operations where agents need direct access to their APIs. 86Box control
also needs its own adapter after a stable control protocol is available. The optional 86Box GDB
stub is not connected by this server.

## Checks when extending the server

Keep protocol messages on stdout and diagnostics off it. Preserve newline-delimited JSON-RPC and
the four original know-how/search tool names. Use repository sample programs and files to check
new operations. Check cancellation and timeout behavior without leaving child processes behind.
Do not put game binaries, outputs or other game material in the repository.
