# artifact · 产物

产物是工件的出口态：一件已稳定的产出物。它由**名字**与**规格**两样定，不因平台而变。

- **名字是身份**——产物不限种类，报告、日志只是两个名字，有哪些名字由端侧定；
- **规格是验收标准**——一组判据，产物的质量由规格控制：判据核过了才算通过验收（判据见 [criterion.md](criterion.md)）；
- **不带位置**——位置不进模型；它落在哪由工作区按名字算（见 [workspace.md](workspace.md)）。

## 工具箱管什么

```rust
use quanttide_work::artifact::Artifact;
```

```dart
import 'package:quanttide_work/quanttide_work.dart';   // Artifact
```

| 构造 | Rust | Dart | 意思 |
| :-- | :-- | :-- | :-- |
| 只知其名 | `Artifact::named("report")` | `Artifact.named('report')` | 还没有规格 |
| 名字加规格 | `Artifact::of("report", spec)` | `Artifact.of('report', spec)` | 规格就是验收它的那组判据 |

```rust
let report = Artifact::of("report", vec![criterion.clone()]);
assert_eq!(report.name, "report");
assert_eq!(report.spec, vec![criterion]);
```

```dart
final report = Artifact.of('report', [criterion]);
expect(report.name, 'report');
expect(report.spec, [criterion]);
```

界面上要的是落点，不是产物本身——落点按名字算，见 [workspace.md](workspace.md)。

## 端侧不做什么

- **不把产物种类写死进模型**——有哪些产物是端侧的事；工具箱只认「名字 + 规格」这两样
- **不把落点塞进产物**——位置不进模型，落点在端侧给目录、工作区给相对路径
- **不自己发明验收**——规格里的判据按 [criterion.md](criterion.md) 的几种判法走，端侧照单去跑
