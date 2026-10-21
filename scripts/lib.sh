# Shared settings for the scripts: where Ghidra, Java and the Python environment are.
# Override any of them in the environment, or in a `.env` file at the repository's root.
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
[[ -f "$ROOT/.env" ]] && set -a && . "$ROOT/.env" && set +a

find_ghidra() {
  for d in "${GHIDRA_INSTALL_DIR:-}" /opt/homebrew/opt/ghidra/libexec /usr/local/opt/ghidra/libexec \
           /opt/ghidra /usr/share/ghidra "$HOME"/ghidra_*_PUBLIC; do
    [[ -n "$d" && -x "$d/support/analyzeHeadless" ]] && { echo "$d"; return; }
  done
}

find_java21() {
  for d in "${JAVA_HOME:-}" /opt/homebrew/opt/openjdk@21/libexec/openjdk.jdk/Contents/Home \
           /usr/lib/jvm/java-21-openjdk-amd64 /usr/lib/jvm/java-21-openjdk /usr/lib/jvm/temurin-21-jdk-amd64; do
    [[ -n "$d" && -x "$d/bin/java" ]] && { echo "$d"; return; }
  done
}

PYTHON="${PYTHON:-$ROOT/.venv/bin/python}"
