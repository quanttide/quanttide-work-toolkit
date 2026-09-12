# 包与模型

工具箱的货架：先看引哪个包，再看用哪个模型。

## 各语言速查

| 语言 | 包名 | 从哪儿装 | 现状 | 装 | 引 |
| :-- | :-- | :-- | :-- | :-- | :-- |
| **Rust** | `quanttide-work` | crates.io | **有实现** | `cargo add quanttide-work` | `use quanttide_work::criterion::Criterion;` |
| **Dart** | `quanttide_work` | pub.dev | **有实现** | `dart pub add quanttide_work` | `import 'package:quanttide_work/quanttide_work.dart';` |
| Go | `.../quanttide-work-toolkit/packages/go` | 模块路径 | 空壳（只有 `Domain` / `Version`） | — | — |
| Python | `quanttide-work` | PyPI | 空壳（只有 `__version__`） | — | — |
| TypeScript | `quanttide-work` | npm | 空壳（只有 `DOMAIN` / `VERSION`） | — | — |

> **空壳语言先别接**：模型还没实现，装进来只有两个常量。实现状态以包内 `STATUS.md` / `ROADMAP.md` 为准。

Dart 只有一个入口：**桶文件** `package:quanttide_work/quanttide_work.dart`——接一次全拿到，不必按目录引。

## 五个模型（这些是出口）

| 模型 | Rust 路径 | Dart（桶文件内同名） | 管什么 |
| :-- | :-- | :-- | :-- |
| 判据 | `quanttide_work::criterion` | `criterion` | 判据的取值与读法、翻成「要跑什么」 |
| 任务 | `quanttide_work::task` | `task` | 任务、流水与「走过」的判定、运行上下文、落点与占位展开 |
| 工作流 | `quanttide_work::workflow` | `workflow` | 步骤、定义的语法与不变量、定义核对 |
| 结果 | `quanttide_work::outcome` | `outcome` | 结果信封（`ok` / `lines` / `columns` / `rows` / `data`） |
| 执行者 | `quanttide_work::executor` | `executor` | 三个取值常量（`agent` / `rule` / `human`） |

目录里的**分件**（`model` / `read` / `items` / `validate` / `check` / `journal` / `context`）是内部结构，接入者只用上面五个出口。
