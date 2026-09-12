# task · 任务

任务是**工作流的一次执行实例**：跑哪条工作流 + 流水（只增不改）+ 闸门项 + 产物声明。任务不带位置——它在哪、产物落哪，由工作区算（见 [workspace.md](workspace.md)）。

## 工具箱管什么

- 任务模型：`Task::of` / `Task.of` 读进来——**不校验**（任务文件没有语法好查；有语法可查的是定义，见 [workflow.md](workflow.md)）
- 流水：`JournalEvent` 记什么时候、哪一步、一句话、过没过
- 产物声明：`declared` 读这次执行往哪写这种产物

```rust
use quanttide_work::task::{JournalEvent, Task};
```

```dart
import 'package:quanttide_work/quanttide_work.dart';   // Task / JournalEvent
```

## 端侧接哪一步

「走过哪几步」「产物落哪」跨着任务与定义、要拿平台给的目录，是**工作区**的操作——见 [workspace.md](workspace.md)。任务模型只回答「这次执行是哪条工作流、流水里记了什么、产物声明成什么」。

## 只读对齐（这条最容易踩）

模型不可变：改动一律**拿新值**，**落盘由端侧做**——工具箱不碰文件，它只交给你一个新对象。

```rust
let after = task.recorded("2026-09-12", "outline", "走了一步", true);   // at / step / detail / ok
// 端侧：把 after 写回任务文件
```

```dart
final after = task.recorded(at: '2026-09-12', step: 'outline', detail: '走了一步', ok: true);
```

## 端侧不做什么

- **不自己实现「走过」的算法**——附加判定投票、重跑从头算，正本在工作区
- **不自己拼产物落点**——声明了按声明的、没声明落默认处的规矩在工作区
- **不改任务文件**——工具箱只算，写是你的事
- **不把位置塞进任务**——任务只装内容，目录由你在调用时传进来
