# executor · 执行者

这一步谁做、这一条判据谁判。三个取值由规范定死，**不因平台而变**。

## 工具箱管什么

| 取值 | Rust | Dart | 意思 |
| :-- | :-- | :-- | :-- |
| 智能体 | `AGENT` | `agent` | 交给 AI |
| 规则引擎 | `RULE` | `rule` | 机械核对，不用智能体 |
| 人 | `HUMAN` | `human` | 只在闸门拍板 |

外加两组「哪些取值合法」：

- **步骤**的执行者：`EXECUTORS` / `executors`——只能 `agent` 或 `human`（**能用 AI 都用 AI，人只在闸门**）
- **判据**的执行者：`CRITERION_TYPES` / `criterionTypes`——还能写 `rule`

```rust
use quanttide_work::executor::{AGENT, CRITERION_TYPES, EXECUTORS, HUMAN, RULE};
```

```dart
import 'package:quanttide_work/quanttide_work.dart';   // agent / rule / human / executors / criterionTypes
```

## 端侧怎么用

**比较时用常量，别写字符串字面量**：

```rust
if step.executor() == AGENT { /* 这一段交给智能体 */ }
```

```dart
if (step.executor == agent) { /* 这一段交给智能体 */ }
```

判断"这条判据要不要端侧自己跑"时，也按执行者分：`rule` 是端侧去跑（见 [criterion.md](criterion.md)），`agent` / `human` 另说。

## 端侧不做什么

- **不自造执行者名字**——写 `"auto"` / `"ai"` 这类同义词，校验会不认，对表也会对不上
- **不自己决定哪些取值合法**——组别在工具箱（步骤两选一、判据三选一），校验在 [workflow.md](workflow.md) 的 `validate` 里
