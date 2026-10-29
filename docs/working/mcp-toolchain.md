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

The catalog currently has 122 entries. The runner can call all 99 programs and shell scripts.
`tests/mcp_tool_coverage.py` checks the catalog and help for every entry, and runs each command's
help path through MCP in a temporary project. It also checks a sample PE analysis and runner
progress, cancellation and timeout handling.

The catalog also lists import-only libraries. The runner does not import or execute those
modules. They are shared Python APIs, not standalone commands. Use the documented programs and
scripts that call them. Add a purpose-built MCP operation when an agent needs a library API
directly; the generic runner must not execute arbitrary Python. Ghidra scripts run through the
catalog's `ghidra-run` and `ghidra-export` scripts.

## Providers from a script

An agent's provider servers live inside the agent. `tools/rea/mcp_client.py` lets a script start
its own copy from the project's `.mcp.json` (`provider_command(NAME)`) and call its tools.
`tools/rea/rea_session.py` uses it for REA: function dossiers with the project's labels, and a
saved analysis snapshot ([re-rea](../../skills/re-rea/SKILL.md)).

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

The client starts and connects to each server in its project settings. `gameframe_mcp_providers`
reports configuration only; the client owns live connection state and provider tools. GameFrame
does not proxy external providers through its own server.

The default configuration enables only GameFrame. List available providers and opt the project
into the ones it needs:

```bash
./gameframe mcp --list
./gameframe mcp --enable rea dosbox
./gameframe mcp --disable dosbox
```

The command updates `.mcp.json`, refreshes MCP settings for configured clients, and updates
unchanged generated Copilot agent allowlists. It preserves unrelated client settings and keeps
edited Copilot agent files. Restart the agent session after enabling or disabling a server. No
provider is installed by these commands.

`mcp-provider.sh` reads the project's ignored `.env` before starting an opted-in provider. The
catalog pins each package or source revision. Use `.env` to select a different exact package
version or a local executable. The project commits only the provider choice and default pin.

| Provider | Pin | Setup and local settings |
|---|---|---|
| REA | `rea-agents` 6.7.0 | Node.js 22.23.4+, 24.22.1+, or 26.12.1+. `.env` can set `REA_MCP_VERSION`. |
| DOSBox-X debugger | `dosbox-x-mcp-server` tag v0.1.2 | Install with the pinned Cargo command in [re-setup](../../skills/re-setup/SKILL.md). `.env` can set `DOSBOX_X_MCP_SERVER` to a local server. Set `DOSBOX_X_BINARY` to use a custom DOSBox-X build; `scripts/setup.sh` checks its version. |
| MCPEmulate | PyPI 0.1.0 | Requires `uv`. Its launcher pins MCP SDK 1.30.0 because newer SDK 2.x releases removed the API this server uses. It defaults to Python 3.11 because its Keystone binding still imports `distutils`; `.env` can set `MCP_EMULATE_SDK_VERSION`, `MCP_EMULATE_PYTHON` and `MCP_EMULATE_VERSION`. On macOS, install the native Keystone library and set `DYLD_LIBRARY_PATH` to its library directory if the binding cannot load it. This GPL-2.0-only server complements `pe32_emu`; it is a CPU emulator, not a Windows system or GameFrame's PE call harness. |
| 86Box | `86box-mcp` 0.1.0 at commit `9e6cdd02f6956e3ff32ec99c1dd4b0be7798d3ca` | No packaged release is published. Build 86Box at or after `1be46d317107f5219f160547d77614ff8e19a7c7` with `-D AUTOMATION=ON`. Add `-D GDBSTUB=ON` when using its debugger tools. Set `BOX86_MCP_DIR`, `BOX86_MCP_REVISION`, `BOX86_MCP_SERVER` and `BOX86_BINARY` in `.env`. The launcher checks the selected source revision and executable path. |

For 86Box, the upstream MCP source and server need a local checkout and venv. The server's
automation socket is off in official 86Box builds. Build instructions and platform requirements
are in the [86Box MCP guide](https://github.com/dafugg/86box-mcp/tree/9e6cdd02f6956e3ff32ec99c1dd4b0be7798d3ca).
Build with both `AUTOMATION=ON` and `GDBSTUB=ON` to use its CPU debugger tools. In 86Box's CMake
configuration, enabling `GDBSTUB` forces `DYNAREC=OFF`. From the MCP checkout, the Docker helper
can build this variant with:

```bash
docker/build.sh -D GDBSTUB=ON
```

For a native build, add `-D GDBSTUB=ON` to the CMake command that enables `AUTOMATION`. The GameFrame
launcher checks the pinned server revision and executable path. It cannot check CMake options when
`BOX86_BINARY` is a wrapper. Inspect the build cache and confirm `AUTOMATION:BOOL=ON`,
`GDBSTUB:BOOL=ON` and `DYNAREC:BOOL=OFF`. The Docker helper writes `.cache/build/CMakeCache.txt`;
for a native build, use the cache under the folder passed to CMake with `-B`. Use
`vm_start(debugger=true)` to attach the stub; ordinary automation runs do not need it. See
[Using 86Box](../observe/86box.md) for scope and limits.

The MCP catalog's minimum 86Box source revision already contains the GDB stub. The same source
checkout can build the automation and debugger variant.

Keep one shared `86box-mcp` checkout and server environment per host. Its nested `86box/` folder
holds the emulator source, and its `.cache/build/` folder holds the Docker build. Reuse these
paths in each project's ignored `.env`; do not clone the server and emulator source for each game.
For the Docker build, set `BOX86_BINARY` to `docker/86box.sh`, which runs `.cache/build/src/86Box`.
For a native build, point it to `86box/build-mcp/src/86Box`. The official 86Box app or release
binary lacks the automation socket needed by this provider; use it for interactive runs. Keep VM
profiles and disks separate from the shared executable, and copy a VM before changing it.

Use one CMake build directory for one build configuration. `GDBSTUB=ON` turns off the dynamic
recompiler. Keep a second, separately named build directory only when a faster non-debug build is
needed, and point `BOX86_BINARY` at the intended variant.

The MCP server creates the automation socket under Python's temporary directory and passes that
host path to 86Box. GameFrame sets `TMPDIR` to `$HOME/.cache/gameframe/86box-tmp` before launching
the provider, where the Docker wrapper's home bind mount makes the same path visible in the
container. Set `GAMEFRAME_86BOX_TMPDIR` in the project's ignored `.env` to choose a different
directory under `$HOME` or the shared `86box-mcp` checkout. Restart the agent session after
changing this setting so the provider starts with the new path. Do not use macOS's default
`/var/folders/...` temporary path with the Docker build; that host path is not mounted in the
container.

GameFrame defaults the automation transport to loopback TCP with the provider's token. Set
`GAMEFRAME_86BOX_TRANSPORT=unix` in `.env` only when the shared filesystem supports Unix sockets.
The GDB stub uses a separate dynamic TCP port, so both the automation connection and debugger
connection must reach the host. The Docker wrapper uses `--network host`. Docker Desktop supports
host networking from version 4.34 when enabled under **Settings > Resources > Network**; see its
[host networking guide](https://docs.docker.com/engine/network/drivers/host/). If the MCP start
call reaches 86Box but stalls while attaching GDB, check host access to the port 86Box reported.

## Checks when extending the server

Keep protocol messages on stdout and diagnostics off it. Preserve newline-delimited JSON-RPC and
the four original know-how/search tool names. Use repository sample programs and files to check
new operations. Run `tests/mcp_tool_coverage.py` to check every catalog entry and runnable command
through MCP, plus cancellation and timeout behavior. Do not leave child processes behind. Do not
put game binaries, outputs or other game material in the repository.
