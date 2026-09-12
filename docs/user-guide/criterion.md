# criterion · 判据

判据是规则引擎那几种**机械核对**。工具箱管它的取值与读法，**不管它跑不跑得通**。

## 工具箱管什么

**判据在 YAML 里就四种写法**（机械核对那类都写 `executor: rule`）：

| 写法 | 意思 |
| :-- | :-- |
| `path: <路径>` | 路径存在 |
| `absent: <路径>` | 路径不存在 |
| `file: <路径>` + `contains: <文字>` | 文件里含这段文字 |
| `run: <命令>` | 这条命令退出码为零 |

外加一件事：**翻成「要跑什么」**——`items_of` / `itemsOf` 交给端侧一份清单，每条是

```
description 说明（如「存在：docs/index.md」）、kind 怎么判、args 参数
```

`kind` 为空就是**不用跑**（那几条的 `executor` 是 `agent` 或 `human`，见 [executor.md](executor.md)）。

```rust
use quanttide_work::criterion::{criterion_of, items_of, read_criterion, Criterion, RuleItem, RuleKind};
```

```dart
import 'package:quanttide_work/quanttide_work.dart';   // criterionOf / readCriterion / itemsOf / Criterion / RuleItem / RuleKind
```

## 端侧接哪一步

**第四步「自己跑」**——工具箱翻单，端侧跑腿：

```rust
let items = items_of(&step.rules());     // 工具箱：翻成「要跑什么」
// 端侧：真的去查文件、起进程，把结果装进 Outcome
```

```dart
final items = itemsOf(step.rules);
```

端侧的本分：**真的去跑**（查文件系统、起进程），把每条的结果（通过 / 不通过 + 说明）写回信封。

## 端侧不做什么

- **不重写判据的取值**——四种 kind 长什么样、字段怎么填，正本在工具箱
- **不自己认判据**——YAML 值怎么翻成判据（`read_criterion` / `readCriterion`）用工具箱的，报错文字也照它的
- **不自己写执行者的字符串**——`rule` / `agent` / `human` 用 [executor.md](executor.md) 里的常量
