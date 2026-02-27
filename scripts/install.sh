#!/usr/bin/env bash
set -euo pipefail

PREFIX="${HOME}/.local"

usage() {
  cat <<'EOF'
Usage: scripts/install.sh [--prefix <path>]

Install camptask from local source using cargo install.

Options:
  --prefix <path>   Install root prefix (default: ~/.local)
  -h, --help        Show this help

Installed binary path:
  <prefix>/bin/camptask
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --prefix)
      PREFIX="${2:-}"
      if [[ -z "${PREFIX}" ]]; then
        echo "--prefix requires a value" >&2
        exit 1
      fi
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if ! command -v cargo >/dev/null 2>&1; then
  echo "missing required command: cargo" >&2
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

mkdir -p "${PREFIX}/bin"

cargo install \
  --path "${PROJECT_ROOT}" \
  --locked \
  --force \
  --root "${PREFIX}"

echo "Installed camptask to ${PREFIX}/bin/camptask"
