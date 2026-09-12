# workspace · 工作区

工作区是**一次工作的边界**：把定义、任务系在一起，让它们互相可见、可以一起工作。

工作区是建模单位，**不是目录**——它只装内容（装载起来的定义与任务），不装位置。物理位置由平台给：落点与核对要用的目录基准，当参数传进来。落点（产物落哪）、流水判定（走过哪几步）、定义核对三件事跨着定义与现场，都是工作区的操作。

## 工具箱管什么

- 工作区模型：`Workspace::of` / `Workspace.of` 装装载好的定义与任务；`workflow` / `task` 按名字取
- 流水判定：走过哪几步、下一步是哪、给用户看的一句话
- 落点与占位：产物落在哪、占位怎么展开
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
let place = workspace.artifact(&task, "report", "/d");          // 声明了按声明的，没声明落默认处
let text = expand_placeholders("{{report}}/清单.md", "/d");     // 占位展开
```

```dart
final place = workspace.artifact(task, 'report', '/d');
final text = expandPlaceholders('{{report}}/清单.md', '/d');
```

落点与占位的第二个参数是**平台给的目录基准**。占位只有四个：`{{artifacts}}`、`{{report}}`、`{{journal}}`、`{{log}}`，展开成 `<目录基准>/artifacts/...`。

目录怎么写都行——**末尾斜杠与重复斜杠不用管**：`/d/`、`/d//` 都当 `/d` 用（相对目录按原样接，空串与 `/` 等价）。这是规范定的，工具箱在**拼的那一处**统一处理（`docs/specification/process/task.md`·落点），你不需要在端侧自己 trim。

### 定义核对

对应 `workflow --check` 一类命令。

```rust
let findings = workspace.check(&workflow, "/d", |path| std::path::Path::new(path).exists());
```

```dart
final findings = workspace.check(workflow, '/d', (path) => File(path).existsSync());
```

`check` 接收三个参数：要核的定义、平台给的目录基准（把 `{{report}}` 一类占位展开成实际路径）、以及「该路径是否存在」的判断函数（由端侧针对真实文件系统实现）。工具箱不访问文件系统，仅依据端侧给出的判断结果核对定义。

判据里的路径带上述占位时不核——那几处要等任务执行时才落。这类回执的 `Finding.ok` 是 `None`（Dart 为 `null`），即「未核」，不静默丢掉；核过的给 `Some(true)` / `Some(false)`（Dart `true` / `false`）。

## 端侧不做什么

- **不自己实现「走过」的算法**——附加判定投票、重跑从头算，正本在工作区
- **不自己拼产物落点**——声明了按声明的、没声明落默认处的规矩在工作区
- **不把位置塞进模型**——工作区只装内容，目录由你在调用时传进来
- **不重复实现核对**——判据路径在不在、小节有没有覆盖，判定方式与回执文字均以工作区为准
