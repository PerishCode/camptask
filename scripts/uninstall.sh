#!/usr/bin/env bash
set -euo pipefail

PREFIX="${HOME}/.local"

usage() {
  cat <<'EOF'
Usage: uninstall.sh [--prefix <path>]

Uninstall camptask from a cargo install root.

Options:
  --prefix <path>   Install root prefix (default: ~/.local)
  -h, --help        Show this help
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

if cargo uninstall --root "${PREFIX}" camptask >/dev/null 2>&1; then
  echo "Uninstalled camptask from ${PREFIX}"
else
  echo "camptask not found under ${PREFIX}" >&2
  exit 1
fi
