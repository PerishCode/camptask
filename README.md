# camptask

`camptask` is a minimal, agent-agnostic runtime guard CLI.

This repository currently provides a CLI skeleton with a `hello` smoke command and initial resources workflow:

- `resources init|update|status|doctor`
- `camp init|check-lite|archive` (placeholders)
- `self-update`

## Quick start

```bash
cargo run --bin camptask -- hello
cargo run --bin camptask -- resources init
cargo run --bin camptask -- resources update --dry-run
cargo run --bin camptask -- camp check-lite
```

## Install

```bash
bash scripts/install.sh --version v0.1.6
```

## Uninstall

```bash
bash scripts/uninstall.sh
```

## Self update

```bash
camptask self-update --check
camptask self-update --yes
```
