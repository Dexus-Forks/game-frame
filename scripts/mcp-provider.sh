#!/usr/bin/env bash
# Launch one opt-in external MCP provider for a GameFrame project.
#
# Usage: scripts/mcp-provider.sh PROVIDER PINNED_VERSION
# Input: provider id and version from tools/gameframe/mcp_providers.json; local overrides from
#        the project's ignored .env, loaded by scripts/lib.sh.
# Output: replaces this process with the provider's stdio MCP server. Diagnostics go to stderr.
# It does not install providers, change emulator builds, or proxy provider tools through GameFrame.
set -euo pipefail
. "$(dirname "$0")/lib.sh"

usage() {
  echo 'Usage: scripts/mcp-provider.sh PROVIDER PINNED_VERSION' >&2
  echo 'Providers: rea, dosbox, mcp-emulate, 86box' >&2
}

if [[ $# != 2 || $1 == --help || $1 == -h ]]; then
  usage
  exit $([[ $# == 2 ]] && echo 0 || echo 2)
fi
provider="$1"; pinned="$2"

version() {
  local value="$1" name="$2"
  [[ "$value" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || {
    echo "$name must be an exact x.y.z version (got '$value')" >&2
    exit 2
  }
  printf '%s' "$value"
}

case "$provider" in
  rea)
    selected="$(version "${REA_MCP_VERSION:-$pinned}" REA_MCP_VERSION)"
    exec npx -y "rea-agents@$selected" mcp
    ;;
  dosbox)
    server="${DOSBOX_X_MCP_SERVER:-dosbox-x-mcp-server}"
    command -v "$server" >/dev/null 2>&1 || {
      echo "DOSBox-X MCP server $pinned is not installed; run the pinned install command in the MCP guide or set DOSBOX_X_MCP_SERVER in .env" >&2
      exit 127
    }
    exec "$server"
    ;;
  mcp-emulate)
    selected="$(version "${MCP_EMULATE_VERSION:-$pinned}" MCP_EMULATE_VERSION)"
    sdk_version="$(version "${MCP_EMULATE_SDK_VERSION:-1.30.0}" MCP_EMULATE_SDK_VERSION)"
    python_version="${MCP_EMULATE_PYTHON:-3.11}"
    command -v uv >/dev/null 2>&1 || {
      echo 'MCPEmulate needs uv; install uv or set it on PATH before starting the agent' >&2
      exit 127
    }
    exec uv run --python "$python_version" --with "mcp==$sdk_version" --with "mcp-emulate==$selected" --no-project mcp-emulate
    ;;
  86box)
    revision="${BOX86_MCP_REVISION:-$pinned}"
    source="${BOX86_MCP_DIR:-}"
    server="${BOX86_MCP_SERVER:-}"
    binary="${BOX86_BINARY:-}"
    [[ -n "$source" && -d "$source" ]] || {
      echo '86Box MCP needs BOX86_MCP_DIR set in the project .env to its pinned source checkout' >&2
      exit 2
    }
    actual="$(git -C "$source" rev-parse HEAD 2>/dev/null || true)"
    [[ "$actual" == "$revision" ]] || {
      echo "86Box MCP checkout is $actual; expected $revision. Run: git -C '$source' checkout $revision" >&2
      exit 2
    }
    [[ -n "$server" ]] || {
      echo '86Box MCP needs BOX86_MCP_SERVER set in the project .env to server/.venv/bin/box86mcp' >&2
      exit 2
    }
    command -v "$server" >/dev/null 2>&1 || {
      echo "86Box MCP server not found: $server" >&2
      exit 127
    }
    [[ -n "$binary" && -x "$binary" ]] || {
      echo '86Box MCP needs BOX86_BINARY set to an executable local 86Box build with AUTOMATION=ON' >&2
      exit 2
    }
    exec "$server"
    ;;
  *)
    echo "unknown MCP provider '$provider'; choose rea, dosbox, mcp-emulate or 86box" >&2
    exit 2
    ;;
esac
