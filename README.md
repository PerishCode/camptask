# camptask

`camptask` is a minimal, agent-agnostic runtime guard CLI.

This repository currently provides a CLI skeleton with a `hello` smoke command and placeholders for:

- `resources init|update|status|doctor`
- `camp init|check-lite|archive`
- `self-update`

## Quick start

```bash
cargo run --bin camptask -- hello
cargo run --bin camptask -- resources init
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
