# 代码约定

横切约束集中在这一处。目录怎么分、一个文件装多少、多写的东西放哪，都看这一份。
与 Rust 侧是同一套，只把文件名与命令换成 Dart 的。

这个库只装**不因平台而变的核心逻辑**——模型的字段与不变量、判据的取值、流水的语义、
落点与占位的展开。它没有 IO，也不出声（拼句、退出码、路径怎么显示、提示词都在端侧），
所以契约是端侧那份的裁剪版：不分「服务 / 适配」，只留下面五条。

## 一个模型一个目录

有自己的定义（身份、字段、取值规矩）的模型，各自成一个目录；目录里的分件按**事**落位。

| 目录 | 模型 | 分件 |
| :-- | :-- | :-- |
| `criterion/` | 判据 | `model.dart` 模型（`RuleKind` / `Criterion`）、`read.dart` 读法、`items.dart` 翻成「要跑什么」 |
| `task/` | 任务 | `model.dart` 模型（`Task`）、`journal.dart` 流水（`JournalEvent`） |
| `workflow/` | 工作流 | `model.dart` 模型（`Step` / `Workflow`）、`read.dart` 读法（取值 + 语法校验） |
| `workspace/` | 工作区 | `model.dart` 模型（`Workspace`）、`check.dart` 定义核对、`artifact.dart` 落点、`progress.dart` 流水判定 |

没有定义、只是常量或信封的，仍单文件：`executor.dart`、`outcome.dart`。
横切的公件另立中立文件：`fields.dart`（读字段与字段表）、`error.dart`（`DefinitionError`）、`paths.dart`（路径拼接与占位展开）——聚合只向下依赖它们。
每个目录一个同名出口文件（`criterion/criterion.dart` 等，对应 Rust 的 `mod.rs`），只留出口。

## 一个文件只讲一件事

主判据是「这一个文件讲几件事」，不是行数。同一件事在 Dart 与 Rust 里行数差很多
（`task` 模型 Dart 132 行 / Rust 176 行），所以**行数 ≤250 只作辅助信号**——
它提示「该看看是不是装了不止一件事」，不单独定罪。

一件事一件；出口文件只留类型与导出，不装东西。

## 一件事只写一处

判据的模型、读法、翻成要跑什么都在 `criterion/`；工作流不再解析判据字段。
从定义里读与校验在 `workflow/read.dart`，字段助手在 `fields.dart`、报错在 `error.dart`、路径与占位在 `paths.dart`——三者是中立的横切件，聚合只向下依赖，别处只借不抄。改一处，改一处。

## 说法与文案不进库

拼句、退出码、路径怎么显示给人看、提示词，都留在各自的端侧（studio / cli）。
这一份只出模型与值对象。

## 收进来的每一样都要有规范出处

每个文件头引 `quanttide-work/docs/specification` 里对应的条款（如
`process/workflow.md`·语法、`process/task.md`·读法）。规范里没有的，先补规范，再进库。
