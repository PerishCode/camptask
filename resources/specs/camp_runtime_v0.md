# camp runtime spec (v0)

Purpose: keep `.camp` minimal, machine-checkable, and agent-agnostic.

## 1) Runtime asset boundary

`.camp/` is the local runtime anchor for current repo + branch.
It stores only minimal execution context, not full decision history.

External event logs are stored outside repo by default:

- `${CAMPTASK_HOME}/logs/<log_stream_id>.ndjson`

`CAMPTASK_HOME` default is `~/.camptask`.

## 2) Minimal `.camp/` layout

Required:

- `.camp/metadata.json`
- `.camp/MAIN.md`
- `.camp/phases/`

Optional (agent-owned):

- `.camp/resources/`
- `.camp/scripts/`

Phase files are free-form markdown with recommended naming:

- `.camp/phases/<X.Y>.md`

Examples: `1.1.md`, `2.3.md`, `10.7.md`.

## 3) Hard-gated metadata contract

`metadata.json` must follow `schemas/camp_metadata.v0.schema.json`.
This file is the only machine-truth file inside `.camp/`.

Required fields:

- `schema_version`
- `run_id`
- `phase_stack`
- `current_phase`
- `log_stream_id`
- `last_sync_at`

## 4) Event envelope contract

Each NDJSON line must follow `schemas/camp_event.v0.schema.json`.

Only envelope is hard-gated:

- `ts`
- `run_id`
- `phase_id`
- `actor`
- `event`
- `result`

`payload` stays open for agent/runtime-specific detail.

## 5) Ownership split

camptask-owned (hard constraints):

- `.camp` required structure
- metadata schema validation
- event envelope schema validation
- check-lite result generation
- archive action

Agent-owned (soft decisions):

- decomposition strategy
- dependency analysis
- command sequence selection
- event payload content
- phase markdown content

## 6) Check-lite scope

`check-lite` only blocks low-level drift:

- missing required files/directories
- invalid schema shape
- pointer mismatch (e.g., missing current phase file)
- log sink not writable

It does not judge strategy quality.
