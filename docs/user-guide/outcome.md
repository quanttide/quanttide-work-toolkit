# outcome · 结果

结果是一个**信封**：一次动作算完，交出来的就是它。命令行与工作台共用同一个形状——`--json` 吐的、界面读的，都是这一份。

## 工具箱管什么

信封装**四样**：通不通、给人看的话、给界面画表的表格、给窗口与脚本的那一栏。

```rust
use quanttide_work::outcome::Outcome;
```

```dart
import 'package:quanttide_work/quanttide_work.dart';   // Outcome
```

- `ok`——这次成没成（**退出码怎么定是端侧的事**）
- `lines`——给人看的那几行
- `columns` + `rows`——同一份表格：表头与行，命令行与窗口共用
- `data`——给窗口与脚本的那一栏（要交原文就托在这里；任意 JSON，对象、数组、字符串都托得住）

## 端侧接哪一步

**每一步的出口**——算完就装进信封：

```rust
let result = Outcome::new(true).with_first(format!("工作区：{}", root));      // 起一个信封
let failed = Outcome::lines(false, vec!["未找到：案例".into()]);              // 直接给几行
```

```dart
final result = Outcome(true).withFirst('工作区：$root');
```

**读回来**（端侧之间传，比如命令行给窗口）：

```rust
let result = Outcome::from_json(&value);      // to_json() 是它的反操作
```

```dart
final result = Outcome.fromStdout(stdout);    // 命令行吐的 JSON 直接装回来
```

## 值语义（两端一致）

`with_first` / `with_data`（Dart `withFirst` / `withData`）都**拿新值**，不改原来那份：原信封保持不动，端侧组装信封、工具箱不管落盘。两端同一套值语义，不再有「一边原地改、一边拿新值」的差异。`to_json`（Dart `toJson`）出信封；`to_output_json`（Dart `toOutputJson`）出「要写出去的那一份」——托了原文给原文，没托给信封。两端各有 `from_stdout` / `fromStdout` 把命令行吐的 JSON 装回来。

## 端侧不做什么

- **不各造一套信封**——端侧之间对表，比的就是这四样加 `data`，形状一歪就对不上
- **不把退出码塞进信封**——信封里只有 `ok`，退出码是你的说法
