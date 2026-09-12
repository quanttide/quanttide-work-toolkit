# workflow · 工作流

工作流是一串有序的步骤，是数据不是代码——加一条流程不改程序。

## 工具箱管什么

- 定义的语法与不变量：字段名、取值、判据种类——不认识、缺了、越界，当场报错
- 步骤与工作流的模型（`Step` / `Workflow`）
- 定义核对：判据里的路径在不在、描述提到的小节有没有判据覆盖（出 `Finding`）

```rust
use quanttide_work::workflow::{looks_like_section, validate, DefinitionError, Finding, Step, Workflow};
```

```dart
import 'package:quanttide_work/quanttide_work.dart';   // Workflow / Step / DefinitionError / Finding / looksLikeSection
```

## 端侧接哪一步

**第二步「读定义」**——读进来顺带校验：

```rust
let workflow = Workflow::from_yaml(&payload, "code-implement.yaml")?;   // 不合法当场 Err
```

```dart
final workflow = Workflow.fromValue(payload, file: 'code-implement.yaml');  // 不合法当场抛
```

端侧只负责把文件读成值，YAML 怎么读写是各包的事；校验是工具箱的事。

第二个参数是这份定义的文件名，比如 `code-implement.yaml`，只用在报错文字里（`code-implement.yaml 少了 steps`）。它不是要读的那个文件——工具箱不碰文件系统。命令行那份就是这么做的：端侧读文件、解成值，再把文件名交给工具箱校验。

### 定义核对

也就是 `workflow --check` 那一类：

```rust
let findings = workflow.check(&context.data, |path| std::path::Path::new(path).exists());
```

```dart
final findings = workflow.check(data, (path) => File(path).existsSync());
```

两个参数都由端侧给。第一个是数据目录，用来把 `{{report}}` 这类占位展开成真路径；第二个是「这个路径在不在」的答案。工具箱不碰文件系统，只拿你给的答案核对定义。

## 端侧不做什么

- 不重写校验——缺 `name`、未知顶层字段、判据没写 `executor`、取值越界，报法与报错文字都照工具箱
- 不自己判哪个小节被提到了——那是 `looks_like_section` / `looksLikeSection` 的事
- 不自己存一份定义——`Workflow::of` 读已经校验过的值，别在端侧再抄一份结构
