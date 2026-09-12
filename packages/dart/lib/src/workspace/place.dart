import '../artifact/model.dart';
import '../criterion/model.dart';
import '../paths.dart';
import '../task/model.dart';
import 'model.dart';

/// 工作区聚合 / 落点：产物落在哪，判据里的占位换成哪。
///
/// 落点只给**相对工作区根的路径**：声明了按声明的（绝对路径原样），没声明落
/// `artifacts/<产物的名字>/<任务名>.md`。拼上目录是平台的事（规范 `process/task.md`·语法）。
/// 占位里有两处不是产物：`{{artifacts}}` 是产物目录，`{{log}}` 是任务文件本身。

/// 产物目录那个占位的名字。
const _artifacts = 'artifacts';

/// 任务文件那个占位的名字——流水不是产物。
const _log = 'log';

extension WorkspacePlace on Workspace {
  /// 这件产物落哪（规范「任务 / 语法」里的落点）。
  ///
  /// 给的是相对工作区根的路径；平台拿自己的目录接上。
  String place(Task task, Artifact artifact) {
    final written = task.declared(artifact.name);
    if (written != null) return written;
    return 'artifacts/${artifact.name}/${task.name}.md';
  }

  /// 把判据里的占位换成本次任务的落点（规范「任务 / 语法」：写了路径就把判据里的占位换成它）。
  ///
  /// 与 [place] 同一处算，所以判据里写占位与直接写落点等价。
  Criterion expanded(Criterion criterion, Task task) {
    return criterion.expanded((name) {
      if (name == _artifacts) return _artifacts;
      if (name == _log) return 'tasks/${task.name}.yaml';
      // 其余在册的占位是产物的名字——产物不限种类，落点按名字算。
      if (placeholderNames.contains(name))
        return place(task, Artifact.named(name));
      return null;
    });
  }
}
