# workflow · 工作流

工作流以数据形式定义，由一串有序的步骤组成；新增流程不需要修改程序。

## 工具箱管什么

- 定义的语法与不变量：字段名、取值、判据种类；出现未知字段、缺失字段或越界取值时立即报错
- 步骤与工作流的模型（`Step` / `Workflow`）
- 从外部值读进来：`from_value` 读并校验，`of` 读已经校验过的值

定义核对（判据引用的路径在不在、描述提到的小节有没有覆盖）跨着工作区，见 [workspace.md](workspace.md)。

```rust
use quanttide_work::error::DefinitionError;
use quanttide_work::workflow::{validate, Step, Workflow};
```

```dart
import 'package:quanttide_work/quanttide_work.dart';   // Workflow / Step / DefinitionError
```

## 端侧接哪一步

**第二步「读定义」**：读取定义时同步完成校验。

```rust
let workflow = Workflow::from_value(&payload)?;   // 定义不合法时返回 Err
```

```dart
final workflow = Workflow.fromValue(payload);  // 定义不合法时抛出异常
```

`payload` 是端侧解析 YAML 得到的值（Rust 为 `serde_yaml::Value`，Dart 为 `Map`），顶层是映射。它既不是文件路径，也不是文件内容的字符串；端侧负责读取文件并解析为值，YAML 的读写方式由各语言包实现。工具箱只接收这个值，校验由工具箱完成。

定义读不通时，工具箱返回（Rust）或抛出（Dart）一个结构化的 `DefinitionError`：它记着位置（顶层 / 第几个步骤 / 第几条判据）与种类，**不含文件名**。要得到给人看的报错文字，端侧把文件名交给 `error.message("code-implement.yaml")`——工具箱出 canonical 文案（如 `code-implement.yaml 少了 steps`），文件由端侧在渲染时补上。工具箱不访问文件系统。

## 端侧不做什么

- 不重复实现校验：缺少 `name`、存在未知顶层字段、判据未声明 `executor`、取值越界等情况的判定方式与报错文字，均与工具箱保持一致
- 不自行判断哪些小节被引用：该判定由工作区的 `looks_like_section` 完成，见 [workspace.md](workspace.md)
- 不重复保存定义：`Workflow::of` 读取已校验的值，端侧不应另行定义同一结构
