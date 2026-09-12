# Changelog

## [Unreleased]

### Changed

- 改成按领域模型分文件：`workflow`（工作流聚合：工作流 + 步骤 + 语法与不变量 + 定义核对）、`task`（任务聚合：流水 + 闸门 + 落点 + 运行上下文 + 走过 / 下一步 / 状态行）、`criterion`（判据）、`executor`（执行者）；删掉 `schema` / `definition` / `tasklog`
- `Finding` 收进 `workflow`，不再单列；定义核对的拼句与退出码（`describe` / `all_ok`）移出工具箱
- `prompts`（含 `Facts` / `criteria_text`）移出工具箱，回各自的平台
- `validate(payload, file)` 不变；`check(flow, data, exists)` → `Workflow::check`；`tasklog::done / next_step / state_line` → `Task::done_steps / next_step / state_line`
- 字段表不再进公开面（`TOP_FIELDS` / `STEP_FIELDS` / `CRITERION_FIELDS` / `text_of` / `unknown_fields`）；`DefinitionError` 仍导出

### Added

- `Task` 聚合（不可变：`recorded` / `with_gates` 返回新的）、`JournalEvent`、`RunContext`

## [0.1.0-beta.1] - 2026-09-12

### Changed

- 判据改密封类型：`Criterion` 枚举六变体（`PathExists` / `PathAbsent` / `FileContains` / `CommandRun` / `AgentJudgement` / `HumanGate`），定义的字段形状不变
- `Step` / `Workflow` 改成不可变值：`Workflow::from_yaml` 校验并建模型、`Workflow::of` 读已校验的定义、`to_yaml` 写回字段形状；`Step.criteria` 是 `Vec<Criterion>`
- `prompts` 的判据入参改 `&[Criterion]`

### Added

- `schema.rs`：字段名、取值、判据种类与 `text_of` / `unknown_fields` 单列；消掉 `definition.rs` 与 `tasklog.rs` 重复的 `text_of`

## [0.1.0-alpha.6] - 2026-09-12

### Changed

- 定义这一类模型（工作流定义、步骤、判据、流水、给 AI 的两段话）改吃 YAML 值——工作流定义本来就是 YAML，不该在工具箱里被挤成 JSON；信封仍是 JSON（那是给窗口与命令行的输出格式）

### Note

- 对消费者是破坏性改动：`quanttide-work = "0.1.0-alpha.6"` 起，定义类的入参是 `serde_yaml::Value`


## [0.1.0-alpha.5] - 2026-09-12

### Fixed

- Dart 发布改用 `PUBDEV_CREDENTIAL_JSON` 落到 pub 缓存的 `credentials.json`（alpha.4 那一步取不到 token）


## [0.1.0-alpha.4] - 2026-09-12

### Fixed

- pub.dev 的 token 认几个常见名字（alpha.3 的 Dart 发布卡在 `PUB_DEV_TOKEN` 取不到）


## [0.1.0-alpha.3] - 2026-09-12

### Fixed

- 版本改动与 `Cargo.lock` 同步（alpha.2 的发布工作流卡在 `--locked`）


## [0.1.0-alpha.2] - 2026-09-12

### Fixed

- Rust 侧按 `cargo fmt` 格式化（alpha.1 的发布工作流卡在这一步）


## [0.1.0-alpha.1] - 2026-09-12

### Added

- 抽出命令行与 studio 两侧意义相同的领域模型：信封、工作流定义（字段表、校验、视图、核对）、机械判据、任务流水与「走过」的算法、给智能体的两段话
- 契约测试：`contract/` 下的用例向量两侧共用，`scripts/contract.sh` 跑 rust 与 dart 各一遍


- 初始化 Rust 库骨架：领域常量与版本导出。
