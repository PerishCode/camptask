# AGENTS

camptask 是一个 CLI 工具。
它用于固化 `.camp` / `.work` 这套工具无关、Agent 无关的结构化工作范式。

## Identity
- `camptask`: automation CLI for convention operations.
- `.camp` / `.work`: convention directories, not product-specific runtime components.

## Shared Terms
- `rule`: 可直接执行的行为约束。
- `prompt`: 面向 agent 的任务指令骨架。
- `template`: 可复用的输入/输出结构。
- `tool action`: 标准化工具调用动作。

## Vocabulary v0
- 编排层状态目录使用 `.camp/`，执行层状态目录使用 `.work/`。
- 编排流程单元使用 `campaign_run`，执行流程单元使用 `work_unit`。
- `phase` 去歧义：leader 使用 `camp_stage`，worker 使用 `work_step`。
- `handoff` 去歧义：leader 下发为 `camp_handoff`，worker 回传为 `work_report`。
- 禁止在规范文本中使用裸 `task/phase/state` 指代跨层对象。

## Core Mission
- 固化 `.camp` / `.work` 常用动作为稳定 CLI 命令。
- 降低协作范式的使用成本与心智负担。
- 把可复用做法沉淀成 `rule` / `prompt` / `template` / `tool action`。

## Command Surface (v0)
- `init`: 初始化 `.camp` 或 `.work` 最小结构。
- `switch`: 切换 `camp_stage` 或 `work_step`。
- `log`: 追加结构化记录。
- `status`: 展示当前状态与最近活动。
- `close`: 关闭当前单元并执行收口动作。

## Decision Rules
- 优先单一命令语义，不做兼容别名。
- 优先最小命令面，不提前扩展。
- 优先可验证输出，不返回模糊状态。

## Output Contract
- 每轮迭代至少固化一项 CLI 行为约束或结构模板。
- 新命令先以 v0 最小能力落地，再扩展参数。
