# Shared settings for the scripts: where Ghidra, Java and the Python environment are.
# They are read from the GameFrame home's config.env ($GAMEFRAME_HOME, ~/gameframe), then from
# a `.env` file at the project's root, which wins. Both replace what the environment holds, so a
# script that inherits GameFrame's own settings still gets the project's.
# ROOT is GameFrame (a clone or the plugin's install). PROJECT is the game's project, where the
# scripts write: the project's own `gameframe` launcher sets GAMEFRAME_PROJECT. In GameFrame
# itself, or in a project that holds a copy of the toolkit, the two are the same folder.
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROJECT="${GAMEFRAME_PROJECT:-$ROOT}"
GAMEFRAME_HOME="${GAMEFRAME_HOME:-$HOME/gameframe}"
load_env() { set -a; . "$1"; set +a; }
[[ -f "$GAMEFRAME_HOME/config.env" ]] && load_env "$GAMEFRAME_HOME/config.env"
if [[ -f "$PROJECT/.env" ]]; then load_env "$PROJECT/.env"
elif [[ -f "$ROOT/.env" ]]; then load_env "$ROOT/.env"; fi

find_ghidra() {
  for d in "${GHIDRA_INSTALL_DIR:-}" /opt/homebrew/opt/ghidra/libexec /usr/local/opt/ghidra/libexec \
           /opt/ghidra /usr/share/ghidra "$HOME"/ghidra_*_PUBLIC; do
    [[ -n "$d" && -x "$d/support/analyzeHeadless" ]] && { echo "$d"; return; }
  done
}

find_java21() {
  for d in "${JAVA_HOME:-}" /opt/homebrew/opt/openjdk@21/libexec/openjdk.jdk/Contents/Home \
           /usr/lib/jvm/java-21-openjdk-amd64 /usr/lib/jvm/java-21-openjdk-arm64 /usr/lib/jvm/java-21-openjdk /usr/lib/jvm/temurin-21-jdk-amd64; do
    [[ -n "$d" && -x "$d/bin/java" ]] && { echo "$d"; return; }
  done
}

if [[ -z "${PYTHON:-}" ]]; then
  PYTHON="$PROJECT/.venv/bin/python"
  [[ -x "$PYTHON" ]] || PYTHON="$ROOT/.venv/bin/python"
  [[ -x "$PYTHON" ]] || PYTHON="$GAMEFRAME_HOME/python/bin/python"
fi

# A folder path for a Ghidra project that Ghidra accepts. Ghidra refuses project paths with a
# folder that starts with "." (a worktree in .claude/worktrees/, a checkout under ~/.cache):
# for those, a link without dots in $TMPDIR points at the real folder.
ghidra_project_dir() {
  local dir; dir="$(cd "$1" && pwd -P)" || return 1
  case "$dir/" in
    */.*) local link="${TMPDIR:-/tmp}"; link="${link%/}/re-ghidra-$(printf %s "$dir" | cksum | cut -d' ' -f1)"
          ln -sfn "$dir" "$link" && echo "$link" ;;
    *) echo "$dir" ;;
  esac
}

# DOSBox-X binary from the local project environment, or the one on PATH.
dosbox_binary() {
  if [[ -n "${DOSBOX_X_BINARY:-}" ]]; then
    echo "$DOSBOX_X_BINARY"
  else
    command -v dosbox-x || return 1
  fi
}

# Refuse a second DOSBox-X before probing or launching it. Match executable names, not
# command arguments: a debugger server or a shell mentioning DOSBox-X is not an instance.
# Exit 75 means busy; exit 2 means process discovery failed.
dosbox_check_idle() {
  local binary processes matches pids on_linux=0 match pid target resolved selected
  binary="$(dosbox_binary || true)"
  processes="$(ps -A -o pid= -o comm=)" || {
    echo 'Cannot check for a running DOSBox-X process; refusing to start it.' >&2
    return 2
  }
  [[ $(uname -s) == Linux ]] && on_linux=1
  matches="$(printf '%s\n' "$processes" | awk -v selected="${binary##*/}" -v on_linux="$on_linux" '
    BEGIN { selected = tolower(selected) }
    {
      pid = $1
      sub(/^[[:space:]]*[0-9]+[[:space:]]+/, "")
      sub(/^.*\//, "")
      sub(/[[:space:]]+$/, "")
      name = tolower($0)
      if (name == "dosbox-x" || (selected != "" && name == selected)) print pid
      else if (on_linux && length(selected) > 15 && name == substr(selected, 1, 15))
        print pid ":truncated"
    }
  ')" || return 2
  pids=""
  selected="$(printf '%s' "${binary##*/}" | tr '[:upper:]' '[:lower:]')"
  for match in $matches; do
    pid="${match%:truncated}"
    if [[ "$match" == *:truncated ]]; then
      # Linux comm holds only 15 characters. Confirm the executable instead of treating
      # an unrelated program with the same prefix as DOSBox-X.
      target="$(readlink "/proc/$pid/exe")" || {
        echo "Cannot identify process $pid while checking DOSBox-X; refusing to start it." >&2
        return 2
      }
      target="${target% (deleted)}"
      resolved="$target"
      target="$(printf '%s' "${target##*/}" | tr '[:upper:]' '[:lower:]')"
      [[ "$target" == dosbox-x || "$target" == "$selected" || "$binary" -ef "$resolved" ]] || continue
    fi
    pids="${pids:+$pids, }$pid"
  done
  [[ -z "$pids" ]] && return 0
  echo "DOSBox-X is already running (PID(s): $pids). Close it before starting another instance. It has been left running." >&2
  return 75
}

# DOSBox-X's version as YYYY.MM.DD, or nothing when it is not installed.
dosbox_version() {
  local binary; binary="$(dosbox_binary)" || return 0
  [[ -x "$binary" ]] || return 0
  { "$binary" -version 2>&1 || true; } | sed -n 's/^DOSBox-X version \([0-9][0-9.]*\).*/\1/p' | head -1
}

# True when DOSBox-X has the debugger control channel (the mcp_server setting), which
# dosbox-run.sh --mcp, dbx_control.py and the dosbox MCP server need: 2026.10 or later.
# Distribution packages are often older; build a current one (the re-setup skill).
dosbox_has_debug_channel() {
  local v; v="$(dosbox_version)"
  [[ -n "$v" ]] || return 1
  local y="${v%%.*}" rest="${v#*.}"; local m="${rest%%.*}"
  (( 10#$y > 2026 || (10#$y == 2026 && 10#$m >= 10) ))
}
