import '../paths.dart';
import '../task/model.dart';
import 'model.dart';

/// 工作区聚合 / 落点：产物落在哪。
///
/// 声明了按声明的（相对平台给的目录基准）；没声明落
/// `artifacts/<种类>/<任务名>.md`；流水是任务文件本身。相接的规矩在 `paths.dart`。
/// 出处：`docs/specification/process/task.md`·语法（落点）。

/// 流水这种产物的种类名。
const _log = 'log';

extension WorkspaceArtifact on Workspace {
  /// 这次执行往哪写这种产物（规范「任务 / 语法」里的落点）。
  ///
  /// [base] 是平台给的目录基准；任务自己不带位置。
  String artifact(Task task, String kind, String base) {
    if (kind == _log) return join(base, 'tasks/${task.name}.yaml');
    final written = task.declared(kind);
    if (written != null) {
      return written.startsWith('/') ? written : join(base, written);
    }
    return join(base, 'artifacts/$kind/${task.name}.md');
  }
}
