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

The catalog currently has 116 entries. The runner can call all 94 programs and shell scripts.
`tests/mcp_tool_coverage.py` checks the catalog and help for every entry, and runs each command's
help path through MCP in a temporary project. It also checks a sample PE analysis and runner
progress, cancellation and timeout handling.

The catalog also lists import-only libraries. The runner does not import or execute those
modules. They are shared Python APIs, not standalone commands. Use the documented programs and
scripts that call them. Add a purpose-built MCP operation when an agent needs a library API
directly; the generic runner must not execute arbitrary Python. Ghidra scripts run through the
catalog's `ghidra-run` and `ghidra-export` scripts.

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

The default configuration includes GameFrame, REA and DOSBox-X. Other MCPs are opt-in because they
need separate software or emulator builds. Add an optional server to a project's client config
only when that project needs it.

### Optional x86 providers

Add either entry to the project's local MCP configuration. These examples show the full
`mcpServers` object; merge the selected server into an existing object.

[`MCPEmulate`](https://github.com/LabGuy94/MCPEmulate) adds general x86 and x86-64 CPU emulation,
assembly, disassembly, register and memory access, breakpoints and instruction traces. It
complements `pe32_emu`; it does not provide GameFrame's PE call harness or a whole Windows system.
It is GPL-2.0-only, so keep it separate and check the licence before bundling it.
The upstream project documents this local MCP client entry:

```json
{
  "mcpServers": {
    "mcp-emulate": {
      "command": "uvx",
      "args": ["mcp-emulate"]
    }
  }
}
```

[`86box-mcp`](https://github.com/dafugg/86box-mcp) drives an 86Box PC through keyboard and mouse
input, screenshots, memory inspection and held emulated time. It covers whole-machine DOS work
that a function emulator cannot. The control socket was [merged into 86Box](https://github.com/86Box/86Box/pull/8365),
but it is disabled in official builds. Build 86Box with `-D AUTOMATION=ON`, then install the server
from its repository. The server needs the path to that build:

```json
{
  "mcpServers": {
    "86box": {
      "command": "/absolute/path/to/86box-mcp/server/.venv/bin/box86mcp",
      "args": [],
      "env": {
        "BOX86_BINARY": "/absolute/path/to/86Box"
      }
    }
  }
}
```

The 86Box MCP project describes itself as an early release; it has not been tested on macOS. It
is GPL-2.0-or-later. Follow its setup guide for ROMs and other optional tools. These providers
stay out of GameFrame's default `.mcp.json`; they are not installed or started automatically.

The optional 86Box MCP is an external provider. GameFrame has no in-process 86Box adapter, and its
MCP does not proxy the external provider's tools. The 86Box GDB stub remains separate.

## Checks when extending the server

Keep protocol messages on stdout and diagnostics off it. Preserve newline-delimited JSON-RPC and
the four original know-how/search tool names. Use repository sample programs and files to check
new operations. Run `tests/mcp_tool_coverage.py` to check every catalog entry and runnable command
through MCP, plus cancellation and timeout behavior. Do not leave child processes behind. Do not
put game binaries, outputs or other game material in the repository.
