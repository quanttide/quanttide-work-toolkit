# 平台开发者：接入工具箱

工具箱装的是知识工作领域**不因平台而变的那部分**——模型的字段与不变量、判据的取值、流水的语义、落点与占位的展开。平台侧（命令行、工作台、以后的语言）向它对齐，**不各写一份**。

语言不同，接法一样：**先读这篇的总则与货架，再按你要用的模型读对应那一篇。**

## 三条总则（不因语言而变，违反任一条，接进来的就不是正本）

1. **只读对齐**——模型不可变：读进来顺带校验（`from_value` / `fromValue`），改动用 `with_*` 拿新值，**落盘由端侧做**（工具箱不碰文件）
2. **工具箱翻单，端侧跑腿**——工具箱把判据翻成「要跑什么」，**真去跑**（查文件、起进程）是端侧的事；**说法**（拼句、退出码、路径怎么显示）与**文案**（提示词）也都留在端侧
3. **一份正本**——不 fork、不复制模型；端侧向同一批**契约向量**（`tests/contract/*.json`，13 份）对齐

界限一句话：**能两处一致的，进工具箱；只能一处有的，留端侧。**

## 六个模型（这些是出口）

| 模型 | Rust 路径 | Dart（桶文件内同名） | 管什么 | 接入说明 |
| :-- | :-- | :-- | :-- | :-- |
| 判据 | `quanttide_work::criterion` | `criterion` | 判据的取值与读法、翻成「要跑什么」 | [criterion.md](criterion.md) |
| 任务 | `quanttide_work::task` | `task` | 任务与流水（`JournalEvent`） | [task.md](task.md) |
| 工作流 | `quanttide_work::workflow` | `workflow` | 步骤、定义的语法与不变量 | [workflow.md](workflow.md) |
| 工作区 | `quanttide_work::workspace` | `workspace` | 把定义与任务系在一起：流水判定、落点、定义核对 | [workspace.md](workspace.md) |
| 结果 | `quanttide_work::outcome` | `outcome` | 结果信封（`ok` / `lines` / `columns` / `rows` / `data`） | [outcome.md](outcome.md) |
| 执行者 | `quanttide_work::executor` | `executor` | 三个取值常量 | [executor.md](executor.md) |

目录里的**分件**（`model` / `read` / `items` / `journal` / `check` / `artifact` / `progress`）是内部结构；横切的 `error`（`DefinitionError`）与 `paths`（认占位名、替换）另立中立模块，接入者用到时按名字取。

## 各语言速查

| 语言 | 包名 | 从哪儿装 | 现状 | 装 | 引 |
| :-- | :-- | :-- | :-- | :-- | :-- |
| **Rust** | `quanttide-work` | crates.io | **有实现** | `cargo add quanttide-work` | `use quanttide_work::criterion::Criterion;` |
| **Dart** | `quanttide_work` | pub.dev | **有实现** | `dart pub add quanttide_work` | `import 'package:quanttide_work/quanttide_work.dart';` |
| Go | `github.com/quanttide/quanttide-work-toolkit/packages/go` | 模块路径 | 空壳（只有 `Domain` / `Version`） | — | — |
| Python | `quanttide-work` | PyPI | 空壳（只有 `DOMAIN` / `__version__`） | — | — |
| TypeScript | `quanttide-work` | npm | 空壳（只有 `DOMAIN` / `VERSION`） | — | — |

> **空壳语言先别接**：模型还没实现，装进来只有两个常量。实现状态以包内 `STATUS.md` / `ROADMAP.md` 为准。

Dart 只有一个入口：**桶文件** `package:quanttide_work/quanttide_work.dart`——接一次全拿到，不必按目录引。

## 接入五步走哪几个模型

| 步 | 做什么 | 去哪篇 |
| :-- | :-- | :-- |
| 一 | **引包**——用版本号，不用本地路径（挂本地路径不算接入） | 上面「各语言速查」+ [versioning.md](versioning.md) |
| 二 | **读定义**——读进来顺带校验，别在端侧重写检查 | [workflow.md](workflow.md) |
| 三 | **判流水**——走过哪几步、下一步是哪，别自己实现 | [workspace.md](workspace.md) |
| 四 | **自己跑**——工具箱翻单，端侧真去查文件、起进程 | [criterion.md](criterion.md) |
| 五 | **向量验收**——`sh scripts/contract.sh`，不靠自报 | [versioning.md](versioning.md) |

## 这一篇怎么分的

照代码的模块划分：**一个模型一件**（`criterion` / `task` / `workflow` / `workspace` / `outcome` / `executor`），与 `packages/rust/src/`、`packages/dart/lib/src/` 的目录一一对应；不属于任何模型的横切纪律（版本与对齐）另立一件。入口（本文件）只管总则、货架与动线，不装模型细节。
