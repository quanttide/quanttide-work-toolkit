# 接入五步

从引包到验收，五步。每一步都给两侧的写法——**接法一样，只是字不同**。

## 第一步：引包——用版本号，不用本地路径

```toml
# Cargo.toml
quanttide-work = "0.1.0-beta.4"
```

```yaml
# pubspec.yaml
quanttide_work: ^0.1.0-beta.5
```

**挂本地路径不算接入**——抽出来的东西要发布，两端引的必须是发布版本号。

> 两侧版本号现在是错开的（`beta.4` / `beta.5`），要引请见 [versioning.md](versioning.md)。

## 第二步：读定义——读进来顺带校验

```rust
use quanttide_work::workflow::Workflow;

let workflow = Workflow::from_yaml(&payload, "pre-release.md")?;   // 不合法当场 Err
```

```dart
final workflow = Workflow.fromValue(payload, file: 'pre-release.md');  // 不合法当场抛
```

端侧只负责**把文件读成值**；**校验是工具箱的事**，别在端侧重写一遍检查。

## 第三步：判流水——「走过哪几步、下一步是哪」

```rust
let done = task.done_steps(&workflow);      // 附加判定投票、重跑从头算
let next = task.next_step(&workflow);
let line = task.state_line(&workflow);      // 给用户看的一句话
```

```dart
final done = task.doneSteps(workflow);
final next = task.nextStep(workflow);
final line = task.stateLine(workflow);
```

**别在端侧自己实现「走过」的算法**——投票与重跑这些判定，正本在工具箱。

## 第四步：自己实现「跑」——这是端侧的本分

```rust
use quanttide_work::criterion::items_of;

let items = items_of(&step.rules());        // 工具箱：翻成「要跑什么」
// 端侧：真的去查文件、起进程，把结果装进 Outcome
```

```dart
final items = itemsOf(step.rules);
```

同理留在端侧：写文件与落盘、给用户看的话、给智能体的提示词、路径怎么显示、退出码怎么定。

## 第五步：用契约向量验收——不靠自报

```bash
sh scripts/contract.sh      # 11 份向量，Rust 与 Dart 各跑一遍，结论必须一样
```

端侧自己也该有尺子：命令行与工作台就是这么对的——同一处工作区、同一条命令，两侧各跑一次，比 `ok` / `columns` / `rows` / `data`（见 `apps/qtcloud-work/src/studio/scripts/parity.sh`，24 条）。

## 接完长什么样

命令行（Rust）：

```rust
use quanttide_work::criterion::{RuleItem, RuleKind, items_of};
use quanttide_work::outcome::Outcome;
use quanttide_work::task::{RunContext, Task};
```

工作台（Dart）：

```dart
import 'package:quanttide_work/quanttide_work.dart' as qt;

qt.Task.of(name, payload);
```
