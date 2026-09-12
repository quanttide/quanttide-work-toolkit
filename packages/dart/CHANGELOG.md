## [Unreleased]

### Added

- `artifact` 聚合（`src/artifact/`）：产物 = **名字 + 规格**（一组验收判据）——名字是身份，不限定种类（报告、日志只是两个名字），不带位置。出处 `docs/specification/piece/artifact.md`
- `workspace` 聚合（`src/workspace/`）：按规范三轴补上「场所」——`Workspace` 装装载好的定义与任务，跨着定义与现场的操作都用扩展挂上去（`WorkspaceCheck.check` / `WorkspacePlace.place` / `WorkspaceProgress.doneSteps` 等）

### Changed

- **破坏性**：`Workspace.artifact(task, kind)` 改名 `WorkspacePlace.place(task, artifact)`，收 `Artifact` 而不是裸种类名——`report` / `journal` / `log` 的特例下线。`{{report}}` / `{{journal}}` 按产物的名字算；`{{artifacts}}`（产物目录）与 `{{log}}`（任务文件）本就不是产物，仍归落点一侧
- **破坏性**：落点不再拼目录——`artifact` / `expanded` 只给**相对工作区根的路径**（声明成绝对路径就原样），目录由平台接；`join` 随之下线（规范 `process/task.md`·落点）
- **破坏性**：占位展开收敛成一套——删掉 `expandPlaceholders` 与 `Placeholders`（占位表只是 `artifact` 的四个名字），改用 `Workspace.expanded(criterion, task)` 把判据里的占位换成本次任务的落点；`Criterion.expanded` 改收「名字 → 路径」的解析函数
- **破坏性**：占位换的是**落点**（`{{report}}` / `{{journal}}` / `{{log}}` 是产物文件，`{{artifacts}}` 是产物目录），与 `Workspace.artifact` 同一处算——原先 `expandPlaceholders` 把前三个也换成目录，与规范对不上
- **破坏性**：`check` 去掉 `base` 参数——判据路径带占位时本来就跳过，`base` 是空转（相对基准由端侧的 `exists` 定）
- 判据路径里的占位只认四个；写别的（如 `{{foo}}`）算定义错误（`UnknownPlaceholder`）；小工具 `runtimePlaceholder` 删除（改用 `placeholdersIn`）。契约向量补 `validate-unknown-placeholder`（第 13 份）
- **破坏性**：删掉 `RunContext`（`src/context.dart`，桶文件不再导出）。位置不进模型：落点与核对要用的目录基准改由平台当参数传进来
- **破坏性**：`WorkflowCheck.check` 与 `looksLikeSection` 从 `workflow/` 搬到 `workspace/`；`Task.doneSteps` / `nextStep` / `stateLine` / `artifact` 从 `task/` 搬到 `workspace/`——落点与流水判定要拿工作区里装载的定义，不是任务自己能算的
- **破坏性**：`check(workflow, base, exists)` 与 `artifact(task, kind, base)` 都多一个 `base`（平台给的目录基准）；`Task` 不再持 `context`，`toMap()` 也不再写 `root` / `data` / `workflows`
- `expandPlaceholders(value, base)` 第二参数改名（语义：平台给的目录基准，不再是数据仓）；`stateLine` 对没有步骤的工作流改说「这条工作流没有步骤：<名>」
- 契约向量：`artifact-location` / `expand-placeholders` 的 `data` 改 `base`，`check-skip` 的 `context` 改 `base`；落点收成单一目录基准，`artifact-location` 补「目录基准为空」一格
- 工作流的读法归位：`workflowOf` / `stepOf` 进 `read.dart`（原 `validate.dart`），`model.dart` 的 `of` 只留一行委托；与 `criterion/{model,read}` 同形（公共 API 不变）
- **破坏性**：`RuleKind` 枚举值与 `Criterion` 子类型同名（`path` → `pathExists`、`absent` → `pathAbsent`、`contains` → `fileContains`、`run` → `commandRun`）；线上写法走新增的 `wire`（字段名），契约不变
- **破坏性**：`Outcome` 改为不可变——字段全 `final`，`withFirst` / `withData` 返回新的一份（与 Rust 侧值语义一致，原先原地改）；`data` 放宽成任意 JSON（`Object?`，原先只收 `Map`）；`dataJson` 改名 `toOutputJson`
- **破坏性**：`check` 不再静默跳过含运行时占位的判据：四个占位都跳过并各出一条 `Finding`，其 `ok` 为 `null`（未核）；`Finding.ok` 由 `bool` 改成 `bool?`。契约向量补 `check-skip`（第 12 份）
- **破坏性**：`Workflow.of` / `Task.of` 不再收名字，改从 `name` 字段读；`check` 第一参数由 `String` 换成 `RunContext`
- `RunContext` 搬到中立件 `src/context.dart`（桶文件照旧导出，`package:quanttide_work` 下无感）
- **破坏性**：`Workflow.fromValue` / `Step.fromValue` / `readCriterion` / `validateWorkflow` / `validateStep` 不再收 `file` / `place`；`DefinitionError` 改成结构化（`position` + `fault`），`message(file)` 出 canonical 文案，`toString()` 给不带文件的那一句
- 抽出中立件 `fields.dart`（`textOf` / `unknownFields`，不经桶文件导出）、`error.dart`（`DefinitionError`）、`paths.dart`（`join` + `expandPlaceholders`）：切断 `criterion ↔ workflow` 与 `workflow ↔ task` 两个环；桶文件照旧导出 `DefinitionError` / `expandPlaceholders`，接入者无感
- **破坏性**：`validateWorkflow` / `validateStep` 改为只做语法校验（返回 `void`），不再顺带建模型；建模走 `Workflow.fromValue` / `Step.fromValue`——与 Rust 侧 `validate` / `from_value` 对齐
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
