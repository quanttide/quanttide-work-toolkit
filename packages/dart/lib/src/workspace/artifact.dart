import '../criterion/model.dart';
import '../task/model.dart';
import 'model.dart';

/// 工作区聚合 / 落点：产物落在哪，判据里的占位换成哪。
///
/// 落点只给**相对工作区根的路径**：声明了按声明的（绝对路径原样），没声明落
/// `artifacts/<种类>/<任务名>.md`；流水是任务文件本身 `tasks/<任务名>.yaml`。
/// 拼上目录是平台的事（规范 `process/task.md`·语法）。

/// 流水这种产物的种类名。
const _log = 'log';

/// 产物目录那个占位的名字。
const _artifacts = 'artifacts';

extension WorkspaceArtifact on Workspace {
  /// 这次执行往哪写这种产物（规范「任务 / 语法」里的落点）。
  ///
  /// 给的是相对工作区根的路径；平台拿自己的目录接上。
  String artifact(Task task, String kind) {
    if (kind == _log) return 'tasks/${task.name}.yaml';
    final written = task.declared(kind);
    if (written != null) return written;
    return 'artifacts/$kind/${task.name}.md';
  }

  /// 把判据里的占位换成本次任务的落点（规范「任务 / 语法」：写了路径就把判据里的占位换成它）。
  ///
  /// 与 [artifact] 同一处算，所以判据里写占位与直接写落点等价。
  Criterion expanded(Criterion criterion, Task task) {
    return criterion.expanded((name) {
      if (name == _artifacts) return _artifacts;
      if (name == 'report' || name == 'journal' || name == _log) {
        return artifact(task, name);
      }
      return null;
    });
  }
}
