# camptask

Rust CLI workspace for `camptask`.

`camptask` is a CLI tool for convention operations on `.camp` and `.work`.
It is not the convention itself and not tied to any specific agent runtime.

## Scope

- Operate `.camp` / `.work` lifecycle actions
- Keep outputs structured and verifiable
- Stay minimal: no compatibility aliases, no dual naming

## Planned Commands (v0)

- `init`
- `switch`
- `log`
- `status`
- `close`

## Layout

- `src/bin/camptask.rs`: CLI entrypoint
- `src/lib.rs`: core runtime entry
- `src/app.rs`: application state/context
- `docs/`: project docs
- `examples/`: runnable examples
- `scripts/`: helper scripts
- `tests/`: integration tests

## Development

```bash
cargo fmt --check
cargo test
```
