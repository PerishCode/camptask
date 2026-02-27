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

## Local Install

```bash
curl -fsSL -o /tmp/camptask-install.sh https://raw.githubusercontent.com/PerishCode/camptask/main/scripts/install.sh
bash /tmp/camptask-install.sh --version v0.1.3
```

Install to a custom prefix:

```bash
bash /tmp/camptask-install.sh --version v0.1.3 --prefix /tmp/camptask-local
```

Initialize resources (default target `~/.camptask/resources`, overwrite by default):

```bash
camptask init
```

Custom target and no-overwrite:

```bash
camptask init --target /tmp/camptask-resources --no-overwrite
```

Initialize opencode agent config from prompt files:

```bash
camptask agent init
```

Environment overrides:

- `CAMPTASK_HOME` (default: `~/.camptask`)
- `CAMPTASK_AGENT_OPENCODE_HOME` (default: `~/.config/opencode`)

`camptask agent init` updates `${CAMPTASK_AGENT_OPENCODE_HOME}/opencode.json` by creating/updating:

- `agent.camptask.leader.prompt = {file:${CAMPTASK_HOME}/resources/prompts/LEADER.md}`
- `agent.camptask.worker.prompt = {file:${CAMPTASK_HOME}/resources/prompts/WORKER.md}`

Uninstall:

```bash
curl -fsSL -o /tmp/camptask-uninstall.sh https://raw.githubusercontent.com/PerishCode/camptask/main/scripts/uninstall.sh
bash /tmp/camptask-uninstall.sh --prefix /tmp/camptask-local
```
