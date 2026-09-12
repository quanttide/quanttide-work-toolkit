# workspace · 工作区

工作区是**一次工作的边界**：把定义、任务系在一起，让它们互相可见、可以一起工作。

工作区是建模单位，**不是目录**——它只装内容（装载起来的定义与任务），不装位置。物理位置由平台给：落点只算相对工作区根的路径，目录由端侧接上。落点（产物落哪）、流水判定（走过哪几步）、定义核对三件事跨着定义与现场，都是工作区的操作。

## 工具箱管什么

- 工作区模型：`Workspace::of` / `Workspace.of` 装装载好的定义与任务；`workflow` / `task` 按名字取
- 流水判定：走过哪几步、下一步是哪、给用户看的一句话
- 落点与占位：产物落在哪、判据里的占位换成哪
- 定义核对：判据引用的路径在不在、描述提到的小节有没有判据覆盖

```rust
use quanttide_work::workspace::{Finding, Workspace, looks_like_section};
```

```dart
import 'package:quanttide_work/quanttide_work.dart';   // Workspace / Finding / looksLikeSection
```

## 端侧接哪一步

### 判流水（第三步）

```rust
let done = workspace.done_steps(&task);      // 附加判定投票、重跑从头算
let next = workspace.next_step(&task);
let line = workspace.state_line(&task);      // 给用户看的一句话
```

```dart
final done = workspace.doneSteps(task);
final next = workspace.nextStep(task);
final line = workspace.stateLine(task);
```

工作区按任务的 `workflow` 字段取定义；取不到就当没走过、下一步为空。

### 落点与占位

```rust
let place = workspace.place(&task, &Artifact::named("report"));   // 声明了按声明的，没声明落默认处
let expanded = workspace.expanded(&criterion, &task);             // 判据里的占位换成本次任务的落点
```

```dart
final place = workspace.place(task, const Artifact.named('report'));
final expanded = workspace.expanded(criterion, task);
```

落点只给**相对工作区根的路径**（声明成绝对路径就原样）——拼上目录是平台的事。占位认四个，各换成同一批落点：`{{report}}` / `{{journal}}` / `{{log}}` 是这三样产物的落点，`{{artifacts}}` 是产物目录——于是判据里写占位与直接写落点等价。

### 定义核对

对应 `workflow --check` 一类命令。

```rust
let findings = workspace.check(&workflow, |path| std::path::Path::new(path).exists());
```

```dart
final findings = workspace.check(workflow, (path) => File(path).existsSync());
```

`check` 接收两个参数：要核的定义，以及「该路径是否存在」的判断函数（由端侧针对真实文件系统实现；相对路径的基准也在端侧定）。工具箱不访问文件系统，仅依据端侧给出的判断结果核对定义。

判据里的路径带四个占位时不核——那几处要等任务执行时才落。这类回执的 `Finding.ok` 是 `None`（Dart 为 `null`），即「未核」，不静默丢掉；核过的给 `Some(true)` / `Some(false)`（Dart `true` / `false`）。

## 端侧不做什么

- **不自己实现「走过」的算法**——附加判定投票、重跑从头算，正本在工作区
- **不自己拼产物落点**——声明的按声明的、没声明落默认处的规矩在工作区；产物只给名字，落点按名字算（见 [artifact.md](artifact.md)）
- **不把位置塞进模型**——工作区只给相对工作区根的路径，拼上目录是端侧的事
- **不重复实现核对**——判据路径在不在、小节有没有覆盖，判定方式与回执文字均以工作区为准
