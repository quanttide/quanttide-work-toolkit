# task · 任务

任务是**工作流的一次执行实例**：跑哪条工作流 + 流水（只增不改）+ 闸门项 + 产物落点 + 这次执行的运行上下文。

## 工具箱管什么

- 任务模型：`Task::of` / `Task.of` 读进来——**不校验**（任务文件没有语法好查；有语法可查的是定义，见 [workflow.md](workflow.md)）
- 流水与「走过」的判定：哪些步骤走过了、下一步是哪、给用户看的一句话
- 运行上下文：三处位置（`RunContext`）
- 落点与占位：产物落在哪、占位怎么展开

```rust
use quanttide_work::task::{expand_placeholders, JournalEvent, RunContext, Task};
```

```dart
import 'package:quanttide_work/quanttide_work.dart';   // Task / JournalEvent / RunContext / expandPlaceholders
```

## 端侧接哪一步

**第三步「判流水」**：

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

落点、占位与上下文：

```rust
let context = RunContext::of(&payload);                  // 三处位置
let place = task.artifact("report", &context);           // 落点：声明了按声明的，没声明落数据仓
let text = expand_placeholders("{{report}}/清单.md", &context.data);   // 占位展开
```

```dart
final context = RunContext.of(payload);
final place = task.artifact('report', context);
final text = expandPlaceholders('{{report}}/清单.md', context.data);
```

占位只有四个：`{{artifacts}}`、`{{report}}`、`{{journal}}`、`{{log}}`；第二个参数给的是**数据目录**（`context.data`），展开成 `<数据目录>/artifacts/...`。

目录怎么写都行——**末尾斜杠与重复斜杠不用管**：`/d/`、`/d//` 都当 `/d` 用（相对目录按原样接，空串与 `/` 等价）。这是规范定的，工具箱在**拼的那一处**统一处理（`docs/specification/process/task.md`·落点），你不需要在端侧自己 trim。

## 只读对齐（这条最容易踩）

模型不可变：改动一律**拿新值**，**落盘由端侧做**——工具箱不碰文件，它只交给你一个新对象。

```rust
let after = task.recorded("2026-09-12", "outline", "走了一步", true);   // at / step / detail / ok
// 端侧：把 after 写回任务文件
```

```dart
final after = task.recorded(at: '2026-09-12', step: 'outline', detail: '走了一步', ok: true);
```

> 端侧**可以**包一层（命令行就是这么做的：把自己的三处位置记在包装类型里，对外只暴露 `artifact(kind)`）——但**算法仍在工具箱**，包一层只是让调用点顺手。

## 端侧不做什么

- **不自己实现「走过」的算法**——附加判定投票、重跑从头算，正本在工具箱
- **不自己拼产物落点**——声明了按声明的、没声明落数据仓的规矩在工具箱
- **不改任务文件**——工具箱只算，写是你的事
