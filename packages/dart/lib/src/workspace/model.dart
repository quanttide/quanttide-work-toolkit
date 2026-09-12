import '../task/model.dart';
import '../workflow/model.dart';

/// 工作区聚合 / 模型：一次工作的边界，把定义与任务系在一起。
///
/// 工作区是建模单位，不是目录——它只装内容（装载起来的定义与任务），不装位置。
/// 物理位置由平台给：落点与核对要用的目录基准，当参数传进来。
/// 出处：`docs/specification/place/workspace.md`。
class Workspace {
  const Workspace({this.workflows = const [], this.tasks = const []});

  /// 拿装载好的定义与任务装一个工作区。
  factory Workspace.of(List<Workflow> workflows, List<Task> tasks) =>
      Workspace(workflows: workflows, tasks: tasks);

  final List<Workflow> workflows;
  final List<Task> tasks;

  /// 按名字取装载起来的工作流定义。
  Workflow? workflow(String name) {
    for (final flow in workflows) {
      if (flow.name == name) return flow;
    }
    return null;
  }

  /// 按名字取装载起来的任务。
  Task? task(String name) {
    for (final item in tasks) {
      if (item.name == name) return item;
    }
    return null;
  }
}
