# Prompt Ref Migration Map (v1 draft)

Purpose: translate early `*.ref.md` ideas into current metatask/camptask vocabulary and execution path.

## 1) Vocabulary Translation

- `Global Command` / `dx execute ...` -> `camptask` lifecycle action + one worker run command
- `.task` state -> `.work` state
- `Phase X.Y` -> `camp_stage` (leader) and `work_step` (worker)
- `Milestone` / `Objective` -> `campaign_run` objective + current `work_unit` goal
- `Peer info` / cross-repo status -> `camp_handoff` constraints and dependency notes
- worker exit summary -> `work_report`

## 2) Leader Ref -> Leader v1 Rules

Source intent from `LEADER.ref.md`:

- Distill standards before execution
- Keep cross-repo dependency visibility
- Emit executable command for each repo
- Reflect and feed improvements back to standards

Mapped leader rules in current system:

- **R1 (Scope):** leader handles architecture/decomposition and lifecycle decisions; no deep implementation execution.
- **R2 (Init-first):** when execution starts, run `camptask work init --branch=<branch>` before worker run.
- **R3 (Single-step handoff):** each worker run targets one granular `work_unit` with clear acceptance criteria.
- **R4 (Constraint distill):** include only minimum necessary constraints in `camp_handoff` (3-5 key items preferred).
- **R5 (Dependency clarity):** encode external dependency status directly in `camp_handoff`.
- **R6 (Lifecycle decision):** after `work_report`, choose `camptask work update` or `camptask work finish`.
- **R7 (Reflection loop):** recurring failure pattern becomes metatask asset (`rule`/`prompt`/`template`/`tool action`).

## 3) Worker Ref -> Worker v1 Rules

Source intent from `WORKER.ref.md`:

- one-time non-interactive execution
- strict handling of contract fields
- linear update/exit discipline
- explicit failure and reflection handling

Mapped worker rules in current system:

- **W1 (One-shot):** execute one `work_unit` per run and terminate.
- **W2 (Contract-first):** treat `camp_handoff` constraints as hard constraints for this run.
- **W3 (Locality):** only change files required for current `work_unit`.
- **W4 (Structured output):** return explicit `work_report` including result, failure reason, and next-action hint.
- **W5 (Failure sharpness):** if blocked, provide concrete blocker + minimal unblock proposal.
- **W6 (No role drift):** worker does not redefine architecture; unresolved architecture questions return to leader.

## 4) Keep / Drop Decisions

Keep (translated):

- Distill-before-execute
- Dependency-aware handoff
- One-time execution
- Reflection-to-governance

Drop (not carried as-is):

- `dx` command syntax
- `.task` naming and file semantics
- X/Y phase indexing as protocol requirement

## 5) Candidate Prompt Shape (next step)

- `LEADER.md`: role boundary + init-first + handoff schema + update/finish decision rule
- `WORKER.md`: one-shot execution + constraint compliance + structured work_report + failure sharpness
- `*.ref.md`: historical reference only, non-runtime
