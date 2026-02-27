#!/usr/bin/env bash
set -euo pipefail

PREFIX="${HOME}/.local"
REPO_URL="https://github.com/PerishCode/camptask.git"
REF="main"

usage() {
  cat <<'EOF'
Usage: install.sh [--prefix <path>] [--repo-url <url>] [--ref <git-ref>]

Install camptask from remote source using cargo install.

Options:
  --prefix <path>   Install root prefix (default: ~/.local)
  --repo-url <url>  Git repository URL (default: https://github.com/PerishCode/camptask.git)
  --ref <git-ref>   Git ref for install (default: main)
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
    --repo-url)
      REPO_URL="${2:-}"
      if [[ -z "${REPO_URL}" ]]; then
        echo "--repo-url requires a value" >&2
        exit 1
      fi
      shift 2
      ;;
    --ref)
      REF="${2:-}"
      if [[ -z "${REF}" ]]; then
        echo "--ref requires a value" >&2
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

mkdir -p "${PREFIX}/bin"

cargo install \
  --git "${REPO_URL}" \
  --branch "${REF}" \
  --bin camptask \
  --locked \
  --force \
  --root "${PREFIX}"

echo "Installed camptask (${REF}) to ${PREFIX}/bin/camptask"
