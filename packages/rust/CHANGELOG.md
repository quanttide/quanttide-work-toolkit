## [Unreleased]

### Changed

- **破坏性**：`Workflow::from_value` / `Step::from_value` / `read_criterion` / `validate` 不再收文件名与「第 N 个步骤」话头；`DefinitionError` 改成结构化——只存位置（`Position`）与种类（`Fault`），文件由端侧在渲染时交给 `message(file)` 拼 canonical 文案
- **破坏性**：`DefinitionError` 搬到中立件 `quanttide_work::error`（原 `workflow::DefinitionError`）；`expand_placeholders` 搬到 `quanttide_work::paths`（原 `task::expand_placeholders`）
- 抽出中立件 `fields`（`text_of` / `unknown_fields`，转内部、不再从公共出口出去）、`error`、`paths`（`join` + `expand_placeholders`）：`criterion` / `task` / `workflow` 只向下依赖它们，切断 `criterion ↔ workflow` 与 `workflow ↔ task` 两个环
- **破坏性**：`Workflow::from_yaml` / `Step::from_yaml` 改名 `from_value`（收的是已解析的值，不是 YAML 文本，与 Dart 侧 `fromValue` 同名同义）；删掉与 `of` 完全重复的 `Workflow::new`
- `validate` 去掉 `from_yaml(...).map(|_| ())` 的壳：自己承担语法校验，`from_value` 调它再建模，两条路共用同一份检查
- **破坏性**：判据的读法归位——`criterion_of` / `read_criterion` 从 `workflow` 搬到 `criterion`（`quanttide_work::criterion::{criterion_of, read_criterion}`）；`expand_placeholders` 从 `workflow` 搬到 `task`
- 文件按「一件事一件」重排：`workflow.rs`（516 行）与 `task.rs`（292 行）各拆成一个目录多件，最长 195 行（`criterion/model.rs`）；`Finding` 与 `looks_like_section` 归 `workflow/check`、占位展开归 `task/model`

### Fixed

- 判据不是映射时不再指错方向：原先 `read_criterion` 先校验 `executor`，非映射的值会报「executor 只能是 rule / agent / human」；现在先判是不是映射，报「不是映射」
- 落点拼接按规范收口：末尾斜杠忽略、重复斜杠折叠（`/w/data/`、`/w/data//` 都当 `/w/data`）——原先 `artifact` 只去掉一个尾斜杠、`expand_placeholders` 完全不处理，同一件事两处两种做法；现在都走同一个 `join`（规范 `process/task.md`·落点）。契约向量 `artifact-location`、`expand-placeholders` 各补了边界格（含空串与相对目录）

## [0.1.0-beta.4] - 2026-09-12

### Changed

- 声明表正名为 `artifacts`（原先叫 `products`）：任务文件里的键、`Task` 的字段、读它的访问器都跟着改；`Task::product(kind)` → `Task::declared(kind)`

### Added

- `Task::artifact(kind, context)`：产物落点收进聚合——声明了按声明的（相对工作区根），没声明落数据仓的 `artifacts/<种类>/<任务名>.md`，流水是任务文件本身（规范 `process/task.md`）
- 契约向量 `artifact-location`（第 11 份）：落点六格——没声明 / 流水 / 相对 / 绝对 / 声明成空白

## [0.1.0-beta.3] - 2026-09-12

### Added

- `outcome`（结果）：规范「过程 / 结果」那一节结成的模型——`ok` 通不通、`lines` 话、`columns` 与 `rows` 同一份表格、`data` 给界面的那一栏；`to_json`（信封）、`data_json`（原文，`--out` 落的）、`from_json`（缺样按空算）、`with_first` / `with_data`

### Changed

- 结果与它的编解码收进工具箱（原先两侧各写一份）：话怎么拼、路径怎么显示、`ok` 怎么定退出码，仍留各自的平台
- 契约向量加回 `outcome-json`（第 10 份）：四样的字段名与「`data` 有才写」两侧一致

## [0.1.0-beta.2] - 2026-09-12

### Changed

- 改成按领域模型分文件，一个模型一个文件：`workflow`（工作流聚合：工作流 + 步骤 + 定义的语法与不变量 + 定义核对）、`task`（任务聚合：流水 + 闸门 + 落点 + 运行上下文 + 走过 / 下一步 / 状态行）、`criterion`（判据）、`executor`（执行者）
- 定义的语法（字段表、读字段、`DefinitionError`）并进 `workflow`；`Finding` 收进 `workflow`，不单列
- `check` → `Workflow::check`；`tasklog::done / next_step / state_line` → `Task::done_steps / next_step / state_line`；`Criterion::from_yaml` → `criterion_of`；`validate` 不变
- 移出工具箱、回各自的平台：提示词（`prompts`）、定义核对的拼句与退出码（`describe` / `all_ok`）、信封（`envelope`）
- 契约向量去掉 `envelope-json`：信封不再是工具箱的东西

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
