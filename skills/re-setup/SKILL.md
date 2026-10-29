---
name: re-setup
description: Install and check the reverse-engineering toolchain (Ghidra, JDK 21, Python with Unicorn/pefile/capstone and optional Frida, objdump, Node for REA, Zig for the sample) and register the REA MCP server for the agent. Use at the start of a project, when a tool is missing or fails to start, or when REA tools do not appear in the session.
---

# Set up the toolchain

## 1. Check, then install

```bash
scripts/setup.sh              # a table: ok / MISSING for each tool, with a hint
scripts/setup.sh --install    # installs what is missing (Homebrew on macOS, apt on Debian/Ubuntu)
                              # and creates .venv with requirements.txt
```

`./gameframe doctor` runs the same check, and passes options on (`./gameframe doctor
--install`). `./gameframe run NAME ARGS` then runs any Python tool with the `.venv`.

The script writes `.env` with `GHIDRA_INSTALL_DIR` and `JAVA_HOME` once both are found. Every
script and the MCP files read these, so set them there if your installs are elsewhere.

Sizes (tell the user before installing on their machine if they have not approved it): Ghidra
about 500 MB, JDK 21 about 300 MB, the Python packages about 40 MB, Zig about 200 MB (only for
building the sample game).

On Linux, Ghidra is not packaged: download the release zip from
https://github.com/NationalSecurityAgency/ghidra/releases, unzip it, and put its folder in
`GHIDRA_INSTALL_DIR`. On Windows, use WSL2 and follow the Linux steps.

On Linux, in a container or in a cloud agent's workspace, read
`docs/working/environments.md` as well. In each new shell, `source scripts/env.sh` loads `.env`
and puts `gameframe` and the `.venv` Python on `PATH`.

## 2. Prove it works

```bash
tests/run.sh            # ~20 s: builds the sample game, runs every tool against it
tests/run.sh --ghidra   # + a real Ghidra export (~1 min)
```

All checks must pass. A failure names the tool and shows its output.

## 3. REA (the reverse-engineering MCP server)

REA ("Reverse Engineer Anything", https://github.com/morluto/rea, npm package `rea-agents`)
gives the agent tools to open binaries, archives, Android packages, .NET assemblies, browser
pages and Electron apps, decompile through Ghidra, and keep every result as cited Evidence.
See `re-rea` for when to use it. External MCPs are opt-in. Start with:

```bash
./gameframe mcp --list
./gameframe mcp --enable rea
```

This pins REA to `rea-agents` 6.8.0 in the project config and refreshes client settings already
present in the project. It needs Node.js 22.23.4+, 24.22.1+, or 26.12.1+. Restart the agent
session; Claude Code may ask to approve the project's MCP server.

For an existing REA entry created by GameFrame, refresh its old pin after updating GameFrame:

```bash
./gameframe mcp --disable rea
./gameframe mcp --enable rea
```

Restart the agent session. `--emit` keeps the old pin; `--enable` refuses to overwrite it. If the project
uses a custom REA command, update that command directly instead. A `REA_MCP_VERSION` value in
`.env` still overrides the generated pin.

- **Project settings:** the generated entry calls `./gameframe mcp-provider rea 6.8.0`. The
  launcher reads `GHIDRA_INSTALL_DIR`, `JAVA_HOME` and `REA_MCP_VERSION` from the project's
  ignored `.env`. To use another exact REA version, set `REA_MCP_VERSION` there, then restart the
  agent. GameFrame's own MCP server does not proxy REA tools.
- **GameFrame's own server:** `gameframe` (`./gameframe serve`) gives the know-how to any MCP
  client. Its tools are `advise`, `knowhow_entry`, `knowhow_list` and `find`
  (`docs/working/knowhow.md`).
- **Other agents:** provider settings and unchanged Copilot agent allowlists are refreshed when you
  run `./gameframe mcp --enable` or `--disable`. To regenerate client settings by hand, run
  `./gameframe mcp --emit all`. The test suite checks that the generated files match the project
  `.mcp.json`.

  | Agent | File | Reads AGENTS.md |
  |---|---|---|
  | Claude Code | `.mcp.json` (the source) | through `CLAUDE.md` |
  | Codex | `.codex/config.toml`, in trusted projects only | yes |
  | Gemini CLI | `.gemini/settings.json` | through `context.fileName` in the same file |
  | Cursor | `.cursor/mcp.json` | yes |
  | GitHub Copilot in VS Code | `.vscode/mcp.json` | yes (`chat.useAgentsMdFile`, on by default) |
  | OpenCode | `opencode.json` | yes |

  The generated wrapper loads the project `.env` for each client, so the same local paths and
  version pins work across clients.
- **REA's own setup** writes REA into the agent's user configuration and installs REA's skill.
  Its client names are `claude_code`, `codex`, `gemini_cli`, `cursor`, `vscode` and `opencode`:
  ```bash
  scripts/setup.sh --rea                          # dry run: shows the plan for Claude Code
  scripts/setup.sh --rea --client codex           # the plan for another client
  npx -y rea-agents@6.8.0 setup --client codex --yes   # apply, after the user agrees
  ```
- REA's own skill (`reverse-engineer-anything`) explains every tool. Install it with REA's setup,
  or from its repository.

REA lists the hosts it supports. On others (for example Debian 13) its doctor warns, but the MCP
connection, Ghidra-backed tools and the sample checks have worked. Try REA on the sample game
before relying on it.

Ghidra analysis of a large binary can take minutes, longer than many clients wait for one
tool call. Raise the client's tool timeout for REA (in Codex, `tool_timeout_sec` in the
server's entry).

If REA tools do not appear: restart the agent (MCP servers start with the session), then check
`npx -y rea-agents@6.8.0 doctor --client claude_code --json`. A tool that reports
`provider_unavailable` usually means Ghidra or Java is not where `.env` says.

## 4. DOS games: DOSBox-X and its debugger

- DOSBox-X: `scripts/setup.sh --install` (on macOS the formula, `brew install --formula
  dosbox-x`; the cask is disabled for failing Gatekeeper). Homebrew's build has the heavy
  debugger; `brew install --HEAD --formula dosbox-x` adds the newest commands (`STEPIN`). Check:
  `scripts/dosbox-run.sh` on the DOS sample prints `exit code 23` (tests/run.sh does this).
- **The debugger control channel needs DOSBox-X 2026.10 or later**, built with the debugger.
  `dosbox-run.sh --mcp`, `tools/dos/dbx_control.py` and the `dosbox` MCP server all use it.
  Distribution packages are often older (Debian 13 ships a 2025 build). `scripts/setup.sh`
  then shows a `dosbox-x debugger OLD` row, `--mcp` stops with a message, and the debugger test
  is skipped. Build a current one on Linux (ask the user first: it is a download and a long
  build):
  ```bash
  sudo apt install libtool autogen autoconf automake libncurses-dev gcc g++ make nasm \
    libsdl2-dev libsdl2-net-dev libpcap-dev libslirp-dev fluidsynth libfluidsynth-dev \
    libavformat-dev libavcodec-dev libavcodec-extra libswscale-dev libfreetype-dev \
    libxkbfile-dev libxrandr-dev
  git clone --depth 1 https://github.com/joncampbell123/dosbox-x.git
  cd dosbox-x && ./build-debug-sdl2      # SDL2 with the heavy debugger
  ```
  The package list follows DOSBox-X's `BUILD.md`; the debugger needs `libncurses-dev`. Use
  the repository's build script. It passes the compiler flags the code needs; a plain
  `./configure && make` with a newer compiler can stop on a warning treated as an error.
  Put the result on `PATH` (or `sudo make install`) and run `scripts/setup.sh` again.
- Opt into the debugger server with `./gameframe mcp --enable dosbox`. Install its pinned release
  with `cargo install --git https://github.com/caiiiycuk/dosbox-x-mcp-server --tag v0.1.2 --locked`.
  If you keep the executable elsewhere, set `DOSBOX_X_MCP_SERVER=/path/to/dosbox-x-mcp-server` in
  the project's ignored `.env`. Set `DOSBOX_X_BINARY=/path/to/custom/dosbox-x` there to use a local
  emulator build; GameFrame's runner and version check use that path. Restart the agent session,
  ask `dosbox_mcp_port`, then run `scripts/dosbox-run.sh --mcp PORT ...`.

For the opt-in CPU and whole-PC providers, follow [the MCP tool surface guide](../../docs/working/mcp-toolchain.md).

For 86Box GDB debugging, the local build needs both `-D AUTOMATION=ON` and `-D GDBSTUB=ON`.
Enable the separate server with `./gameframe mcp --enable 86box`, then restart the agent session.
In the pinned MCP checkout, run `docker/build.sh -D GDBSTUB=ON`; it enables automation itself.
For a native build, add that option to the CMake command with `AUTOMATION=ON`. Check the build
cache for `AUTOMATION:BOOL=ON`, `GDBSTUB:BOOL=ON` and `DYNAREC:BOOL=OFF`. The provider's `vm_start`
tool must use `debugger=true` to attach. The MCP tool surface guide has the pinned server, local
settings and platform build instructions. Reuse one `86box-mcp` checkout per host. Put only VM
profiles and disks in separate game projects. Make another build directory only when you need a
different CMake configuration, such as a faster run without the GDB stub.

The server creates its automation socket in the host's temporary directory. GameFrame sets
`TMPDIR` to `$HOME/.cache/gameframe/86box-tmp`, a path the Docker wrapper mounts into the
container. Set `GAMEFRAME_86BOX_TMPDIR` in the project's ignored `.env` to another directory under
`$HOME` or the shared MCP checkout. Restart the agent session after changing it. This avoids the
macOS default under `/var/folders`, which the container cannot see.

GameFrame uses loopback TCP with the provider token by default. Set
`GAMEFRAME_86BOX_TRANSPORT=unix` only when the shared filesystem passes Unix sockets. The GDB
stub uses a separate dynamic TCP port. The Docker wrapper uses host networking; Docker Desktop
needs host networking enabled in Settings > Resources > Network. Check that the port reported by
86Box is reachable from the host if startup stalls at debugger attach.

## Common problems

- `dosbox-x not found` after `brew install --HEAD`: run `brew link dosbox-x`.
- `dbx_control` or the dosbox MCP server times out waiting for DOSBox-X: check that
  `scripts/setup.sh` shows no `OLD` row. An older DOSBox-X ignores `mcp_server` and never
  connects.

| Symptom | Cause and fix |
|---|---|
| `Ghidra not found` | Set `GHIDRA_INSTALL_DIR` in `.env` to the folder that contains `support/analyzeHeadless` |
| Ghidra exits with `UnsupportedClassVersionError` | Wrong Java: set `JAVA_HOME` to a JDK 21 |
| Ghidra runs out of memory on a large game | `MAXMEM=16G scripts/ghidra-export.sh ...` |
| `Unable to lock project` | Another headless job uses the same Ghidra project: wait, or pass `--project` |
| `pip install unicorn` fails to build | Use a Python version that has a Unicorn wheel (3.10 to 3.13 are safest): `PYTHON=python3.12` |
| The sample game does not build | Install Zig (`brew install zig`, or https://ziglang.org/download) |
| `frida` MISSING in the check | Optional, for live tracing: `scripts/setup.sh --install`, or `.venv/bin/pip install -r requirements-live.txt` |
| Frida cannot attach on macOS | The program is signed with the hardened runtime. Trace a copy re-signed without it, or one you built |

## Working in a git worktree

A worktree has no `game/`, `media/`, `saves/`, `analysis/` or `derived/`, since git keeps
none of them. Run `./gameframe game link ID .` in the worktree: it makes the same links into the
GameFrame home as the main checkout has, and `.gitignore` already ignores them. An older
project with real folders links them from the main checkout and adds the links to
`$(git rev-parse --git-common-dir)/info/exclude`. The shared Python is in the home's
`python/`, so a worktree needs no `.venv` of its own.
