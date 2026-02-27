#!/usr/bin/env bash
set -euo pipefail

PREFIX="${HOME}/.local"
REPO_URL="https://github.com/PerishCode/camptask.git"
VERSION=""

usage() {
  cat <<'EOF'
Usage: install.sh --version vX.Y.Z|X.Y.Z [--prefix <path>] [--repo-url <url>]

Install camptask from remote source using cargo install.

Options:
  --prefix <path>   Install root prefix (default: ~/.local)
  --version <ver>   Install from git tag (required; vX.Y.Z or X.Y.Z)
  --repo-url <url>  Git repository URL (default: https://github.com/PerishCode/camptask.git)
  -h, --help        Show this help

Installed binary path:
  <prefix>/bin/camptask
EOF
}

normalize_version() {
  local raw="$1"
  if [[ "${raw}" == v* ]]; then
    printf '%s\n' "${raw}"
  else
    printf 'v%s\n' "${raw}"
  fi
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
    --version)
      VERSION="${2:-}"
      if [[ -z "${VERSION}" ]]; then
        echo "--version requires a value" >&2
        exit 1
      fi
      VERSION="$(normalize_version "${VERSION}")"
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

if [[ -z "${VERSION}" ]]; then
  echo "--version is required" >&2
  usage >&2
  exit 1
fi

cargo install \
  --git "${REPO_URL}" \
  --tag "${VERSION}" \
  --bin camptask \
  --locked \
  --force \
  --root "${PREFIX}"

echo "Installed camptask (${VERSION}) to ${PREFIX}/bin/camptask"
