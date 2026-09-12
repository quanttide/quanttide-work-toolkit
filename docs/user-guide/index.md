# 平台开发者：接入工具箱

工具箱装的是知识工作领域**不因平台而变的那部分**——模型的字段与不变量、判据的取值、流水的语义、落点与占位的展开。平台侧（命令行、工作台、以后的语言）向它对齐，**不各写一份**。

这一篇讲怎么接。语言不同，接法一样。

## 一、接入前先记住三条

这三条不因语言而变，**违反任一条，接进来的就不是正本**：

1. **只读对齐**——模型不可变：读进来顺带校验（`from_yaml` / `fromValue`），改动用 `with_*` 拿新值，**落盘由端侧做**（工具箱不碰文件）
2. **工具箱翻单，端侧跑腿**——工具箱把判据翻成「要跑什么」（`items_of` / `itemsOf`），**真去跑**（查文件、起进程）是端侧的事；**说法**（拼句、退出码、路径怎么显示）与**文案**（提示词）也都留在端侧
3. **一份正本**——不 fork、不复制模型；端侧向同一批**契约向量**（`tests/contract/*.json`，11 份）对齐

## 二、各语言速查

| 语言 | 包名 | 从哪儿装 | 现状 | 装 | 引 |
| :-- | :-- | :-- | :-- | :-- | :-- |
| **Rust** | `quanttide-work` | crates.io | **有实现** | `cargo add quanttide-work` | `use quanttide_work::criterion::Criterion;` |
| **Dart** | `quanttide_work` | pub.dev | **有实现** | `dart pub add quanttide_work` | `import 'package:quanttide_work/quanttide_work.dart';` |
| Go | `.../quanttide-work-toolkit/packages/go` | 模块路径 | 空壳（只有 `Domain` / `Version`） | — | — |
| Python | `quanttide-work` | PyPI | 空壳（只有 `__version__`） | — | — |
| TypeScript | `quanttide-work` | npm | 空壳（只有 `DOMAIN` / `VERSION`） | — | — |

> **空壳语言先别接**：模型还没实现，装进来只有两个常量。实现状态以包内 `STATUS.md` / `ROADMAP.md` 为准。

## 三、五个模型（这些是出口）

| 模型 | Rust 路径 | Dart（桶文件内同名） | 管什么 |
| :-- | :-- | :-- | :-- |
| 判据 | `quanttide_work::criterion` | `criterion` | 判据的取值与读法、翻成「要跑什么」 |
| 任务 | `quanttide_work::task` | `task` | 任务、流水与「走过」的判定、运行上下文、落点与占位展开 |
| 工作流 | `quanttide_work::workflow` | `workflow` | 步骤、定义的语法与不变量、定义核对 |
| 结果 | `quanttide_work::outcome` | `outcome` | 结果信封（`ok` / `lines` / `columns` / `rows` / `data`） |
| 执行者 | `quanttide_work::executor` | `executor` | 三个取值常量（`agent` / `rule` / `human`） |

目录里的**分件**（`model` / `read` / `items` / `validate` / `check` / `journal` / `context`）是内部结构，接入者只用上面五个出口。

## 四、接入五步

### 第一步：引包——用版本号，不用本地路径

```toml
# Cargo.toml
quanttide-work = "0.1.0-beta.4"
```

```yaml
# pubspec.yaml
quanttide_work: ^0.1.0-beta.5
```

**挂本地路径不算接入**——抽出来的东西要发布，两端引的必须是发布版本号。

### 第二步：读定义——读进来顺带校验

```rust
use quanttide_work::workflow::Workflow;

let workflow = Workflow::from_yaml(&payload, "pre-release.md")?;   // 不合法当场 Err
```

```dart
final workflow = Workflow.fromValue(payload, file: 'pre-release.md');  // 不合法当场抛
```

端侧只负责**把文件读成值**；**校验是工具箱的事**，别在端侧重写一遍检查。

### 第三步：判流水——「走过哪几步、下一步是哪」

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

### 第四步：自己实现「跑」——这是端侧的本分

```rust
use quanttide_work::criterion::items_of;

let items = items_of(&step.rules());        // 工具箱：翻成「要跑什么」
// 端侧：真的去查文件、起进程，把结果装进 Outcome
```

```dart
final items = itemsOf(step.rules);
```

同理留在端侧：写文件与落盘、给用户看的话、给智能体的提示词、路径怎么显示、退出码怎么定。

### 第五步：用契约向量验收——不靠自报

```bash
sh scripts/contract.sh      # 11 份向量，Rust 与 Dart 各跑一遍，结论必须一样
```

端侧自己也该有尺子：命令行与工作台就是这么对的——同一处工作区、同一条命令，两侧各跑一次，比 `ok` / `columns` / `rows` / `data`（见 `apps/qtcloud-work/src/studio/scripts/parity.sh`，24 条）。

## 五、版本与对齐纪律

- 工具箱两侧（Rust / Dart）**发同一个版本号**；**向量一致时才发**
- 两侧一致后，端侧引同一号——不要一端 `beta.4`、一端 `beta.5`，对表会翻车
- 动端侧之前先跑一次 `contract.sh` 拿基线；红了先查是不是引的版本不对

## 六、真实接入长什么样

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

## 七、不做什么

- 不复制模型、不 fork 一份改
- 不把「说法」塞进工具箱（退出码、路径显示、提示词）
- 不让工具箱碰 IO（查文件、起进程、发网络请求）
- 不在端侧重算「走过哪几步」或判据的取值
