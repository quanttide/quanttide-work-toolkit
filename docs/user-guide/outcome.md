# outcome · 结果

结果是一个**信封**：一次动作算完，交出来的就是它。命令行与工作台共用同一个形状——`--json` 吐的、界面读的，都是这一份。

## 工具箱管什么

信封的形状与五种装法：`ok` / `lines` / `columns` / `rows` / `data`。

```rust
use quanttide_work::outcome::Outcome;
```

```dart
import 'package:quanttide_work/quanttide_work.dart';   // Outcome
```

- `ok`——这次成没成（**退出码怎么定是端侧的事**）
- `lines`——给人看的那几行
- `columns` / `rows`——给界面画表的那两栏
- `data`——给窗口装领域对象的原文

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

## 一处语言差异（不是分歧）

Rust 的 `with_first` / `with_data` 是**拿新值**（`mut self` → `Self`），Dart 的 `withFirst` / `withData` 是**原地改**——两种语言在这件事上的写法就不同。**意思一样**：端侧组装信封，工具箱不管落盘。

## 端侧不做什么

- **不各造一套信封**——端侧之间对表，比的就是这四样加 `data`，形状一歪就对不上
- **不把退出码塞进信封**——信封里只有 `ok`，退出码是你的说法
