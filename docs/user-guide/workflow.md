# workflow · 工作流

工作流以数据形式定义，由一串有序的步骤组成；新增流程不需要修改程序。

## 工具箱管什么

- 定义的语法与不变量：字段名、取值、判据种类；出现未知字段、缺失字段或越界取值时立即报错
- 步骤与工作流的模型（`Step` / `Workflow`）
- 定义核对：判据引用的路径是否存在、描述中提及的小节是否都有判据覆盖；结果以 `Finding` 返回

```rust
use quanttide_work::workflow::{looks_like_section, validate, DefinitionError, Finding, Step, Workflow};
```

```dart
import 'package:quanttide_work/quanttide_work.dart';   // Workflow / Step / DefinitionError / Finding / looksLikeSection
```

## 端侧接哪一步

**第二步「读定义」**：读取定义时同步完成校验。

```rust
let workflow = Workflow::from_yaml(&payload, "code-implement.yaml")?;   // 定义不合法时返回 Err
```

```dart
final workflow = Workflow.fromValue(payload, file: 'code-implement.yaml');  // 定义不合法时抛出异常
```

端侧负责将文件读取并解析为值，YAML 的读写方式由各语言包实现；校验由工具箱完成。

第二个参数是定义的文件名（例如 `code-implement.yaml`），仅用于构造报错信息（如 `code-implement.yaml 少了 steps`），并不表示工具箱要读取该文件：工具箱不访问文件系统。命令行工具即采用这一方式：由端侧读取文件、解析为值，再把文件名交给工具箱校验。

### 定义核对

对应 `workflow --check` 一类命令。

```rust
let findings = workflow.check(&context.data, |path| std::path::Path::new(path).exists());
```

```dart
final findings = workflow.check(data, (path) => File(path).existsSync());
```

两个参数均由端侧提供：第一个是数据目录，用于将 `{{report}}` 一类占位符展开为实际路径；第二个是「该路径是否存在」的判断结果。工具箱不访问文件系统，仅依据端侧提供的判断结果核对定义。

## 端侧不做什么

- 不重复实现校验：缺少 `name`、存在未知顶层字段、判据未声明 `executor`、取值越界等情况的判定方式与报错文字，均与工具箱保持一致
- 不自行判断哪些小节被引用：该判定由 `looks_like_section` / `looksLikeSection` 完成
- 不重复保存定义：`Workflow::of` 读取已校验的值，端侧不应另行定义同一结构
