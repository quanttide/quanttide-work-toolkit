## [Unreleased]

### Changed

- 判据的读法归位：`criterionOf` / `readCriterion` 从 `workflow` 搬到 `criterion`（桶文件导入方式不变，端侧无感）
- 文件按「一件事一件」重排：`workflow.dart`（405 行）与 `task.dart`（216 行）各拆成一个目录多件，最长 169 行（`criterion/model.dart`）；`Finding` 与 `looksLikeSection` 归 `workflow/check`、占位展开归 `task/model`

### Added

- 两个扩展：`TaskJournal`（`doneSteps` / `nextStep` / `stateLine`）、`WorkflowCheck`（`check`）——Dart 不能跨文件写实现，用扩展承接分件；调用写法不变（`task.doneSteps(...)`、`workflow.check(...)`）

### Fixed

- 判据不是映射时不再指错方向：原先 `readCriterion` 先校验 `executor`，非映射的值会报「executor 只能是 rule / agent / human」；现在先判是不是映射，报「不是映射」
- 落点拼接按规范收口：末尾斜杠忽略、重复斜杠折叠（`/w/data/`、`/w/data//` 都当 `/w/data`）——原先 `artifact` 只去掉一个尾斜杠、`expandPlaceholders` 完全不处理，同一件事两处两种做法；现在都走同一个 `_join`（规范 `process/task.md`·落点）。契约向量 `artifact-location`、`expand-placeholders` 各补了边界格（含空串与相对目录）

## [0.1.0-beta.5] - 2026-09-12

### Changed

- 声明表正名为 `artifacts`（原先叫 `products`）：任务文件里的键、`Task` 的字段、读它的访问器都跟着改；`product(kind)` → `declared(kind)`

### Added

- `artifact(kind, context)`：产物落点收进聚合——声明了按声明的（相对工作区根），没声明落数据仓的 `artifacts/<种类>/<任务名>.md`，流水是任务文件本身（规范 `process/task.md`）
- 契约向量 `artifact-location`（第 11 份）：落点六格——没声明 / 流水 / 相对 / 绝对 / 声明成空白

## [0.1.0-beta.4] - 2026-09-12

### Changed

- `Outcome` 的四个字段改成可写、`lines` / `columns` / `rows` 缺省给能增的列表：答复是边算边拼的（`lines.add(...)` 直接能用），`withFirst` / `withData` 就地添上再返回自己

## [0.1.0-beta.3] - 2026-09-12

### Added

- `Outcome`（结果）：规范「过程 / 结果」那一节结成的模型——`ok` 通不通、`lines` 话、`columns` 与 `rows` 同一份表格、`data` 给界面的那一栏；`toJson`（信封）、`dataJson`（原文，`--out` 落的）、`Outcome.fromJson` / `Outcome.fromStdout`（缺样按空算）、`withFirst` / `withData`

### Changed

- 结果与它的编解码收进工具箱（原先两侧各写一份）：话怎么拼、路径怎么显示、`ok` 怎么定退出码，仍留各自的平台
- 契约向量加回 `outcome-json`（第 10 份）：四样的字段名与「`data` 有才写」两侧一致

## [0.1.0-beta.2] - 2026-09-12

### Changed

- 改成按领域模型分文件，一个模型一个文件：`workflow`（工作流聚合：工作流 + 步骤 + 定义的语法与不变量 + 定义核对）、`task`（任务聚合：流水 + 闸门 + 落点 + 运行上下文 + 走过 / 下一步 / 状态行）、`criterion`（判据）、`executor`（执行者）
- 定义的语法（字段表、读字段、`DefinitionError`）并进 `workflow`；`Finding` 收进 `workflow`，不单列
- `validateDefinition(payload, file)` → `Workflow.fromValue`；`checkWorkflow` → `flow.check`；`tasklog.done / nextStep / stateLine` → `Task.doneSteps / nextStep / stateLine`；`Criterion.fromMap` → `criterionOf`
- 移出工具箱、回各自的平台：提示词（`prompts`）、定义核对的拼句与退出码（`describeFindings` / `allOk`）、信封（`envelope`）
- 契约向量去掉 `envelope-json`：信封不再是工具箱的东西

### Added

- `Task` 聚合（不可变：`recorded` / `withGates` 返回新的）、`JournalEvent`、`RunContext`
- `Criterion.expanded(f)`：判据各字段里的占位一次换掉（Rust 侧 beta.1 已有）

## [0.1.0-beta.1] - 2026-09-12

### Changed

- 判据改密封类型：`path` / `absent` / `file`+`contains` / `run` 各一个子类型，`agent` / `human` 各一个；定义的字段形状不变
- `Workflow` / `Step` 改成不可变值：`Workflow.fromValue` 校验并建模型、`Workflow.of` 读已校验的定义、`toMap` 写回字段形状；`Step.criteria` 是 `List<Criterion>`
- `itemsOf` 吃 `Iterable<Criterion>`；`prompts` 的判据入参同步

### Added

- `schema.dart`：字段名、取值、判据种类与 `textOf` / `unknownFields` 单列；消掉 `definition.dart` 与 `tasklog.dart` 重复的 `textOf`（公开面不再需要 `hide`）

## [0.1.0-alpha.10] - 2026-09-12

### Fixed

- 发布改用与 quanttide-data-toolkit 同一个 action（k-paxian/dart-package-publisher），不再手写凭证落点


## [0.1.0-alpha.8] - 2026-09-12

### Fixed

- 凭证改走 pub 自己的 token 仓（以前把整份 credentials.json 写进 pub-cache，新版 pub 不读它）


## [0.1.0-alpha.7] - 2026-09-12

### Fixed

- 凭证到位后重发


## [0.1.0-alpha.6] - 2026-09-12

### Fixed

- 发布步不让 pub 挂等交互：验凭证形状、关 stdin、加 300 秒上限（alpha.5 那一步静默挂了十几分钟）


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


- 初始化 Dart 包骨架：`quanttide_work` 库入口（领域名与版本常量）与测试。
