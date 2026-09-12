import '../task/model.dart';
import '../workflow/model.dart';
import 'model.dart';

/// 工作区聚合 / 流水判定：走过哪几步、下一步是哪。
///
/// 「走过哪几步」从任务流水读出：带后缀的流水（`·审` / `·判`）给这一步**投票**，
/// 不带后缀的（重走一遍）**把结论从头算**。工作流从工作区里按名字取。
/// 出处：`docs/specification/process/task.md`·读法。
extension WorkspaceProgress on Workspace {
  /// 这件任务走过哪几步（按流水里的先后）。
  List<String> doneSteps(Task task) {
    final flow = workflow(task.workflowName);
    if (flow == null) return const [];
    return _doneSteps(task, flow);
  }

  /// 这件任务下一个没走到的步骤（按定义里的顺序）；都走过或没有定义给 `null`。
  String? nextStep(Task task) {
    final flow = workflow(task.workflowName);
    if (flow == null) return null;
    return _nextStep(task, flow);
  }

  /// 状态行：下一步是谁，或者都走过了。
  String stateLine(Task task) {
    final flow = workflow(task.workflowName);
    if (flow == null) return '工作区里没有这条工作流：${task.workflowName}';
    return _stateLine(task, flow);
  }
}

/// 走过哪几步：工作流上没有的步骤名不算数。
List<String> _doneSteps(Task task, Workflow workflow) {
  final names = workflow.stepNames;
  final verdict = <String, bool>{};
  final order = <String>[];
  for (final event in task.journal) {
    final at = event.step.indexOf('·');
    final step = at < 0 ? event.step : event.step.substring(0, at);
    if (!names.contains(step)) continue;
    if (verdict.containsKey(step)) {
      verdict[step] = at >= 0 ? (verdict[step]! && event.ok) : event.ok;
    } else {
      verdict[step] = event.ok;
      order.add(step);
    }
  }
  return order.where((step) => verdict[step] == true).toList();
}

/// 第一个没走到的步骤（按工作流里的顺序）。
String? _nextStep(Task task, Workflow workflow) {
  final finished = _doneSteps(task, workflow);
  for (final name in workflow.stepNames) {
    if (!finished.contains(name)) return name;
  }
  return null;
}

/// 状态行：下一步是谁，或者都走过了。
String _stateLine(Task task, Workflow workflow) {
  final names = workflow.stepNames;
  if (names.isEmpty) {
    return '这条工作流没有步骤：${workflow.name}';
  }
  final next = _nextStep(task, workflow);
  return next == null ? '${names.length} 个步骤都走过了' : '下一步：$next';
}
