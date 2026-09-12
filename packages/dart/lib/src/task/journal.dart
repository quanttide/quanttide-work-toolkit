import '../workflow/model.dart';
import '../fields.dart';
import 'model.dart';

/// 流水里的一条：什么时候、哪一步、一句话、过没过。
class JournalEvent {
  const JournalEvent({
    required this.at,
    required this.step,
    required this.detail,
    required this.ok,
  });

  factory JournalEvent.of(Map value) => JournalEvent(
    at: textOf(value, 'at'),
    step: textOf(value, 'step'),
    detail: textOf(value, 'detail'),
    ok: value['ok'] == true,
  );

  final String at;
  final String step;
  final String detail;
  final bool ok;

  Map<String, Object?> toMap() => {
    'at': at,
    'step': step,
    'detail': detail,
    'ok': ok,
  };
}

/// 从流水读出「走过哪几步」。
extension TaskJournal on Task {
  /// 走过哪几步。
  ///
  /// 带后缀的流水（`·审` / `·判`）给这一步**投票**，不带后缀的（重走一遍）
  /// **把结论从头算**；工作流上没有的步骤名不算数。
  List<String> doneSteps(Workflow workflow) {
    final names = workflow.stepNames;
    final verdict = <String, bool>{};
    final order = <String>[];
    for (final event in journal) {
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
  String? nextStep(Workflow workflow) {
    final finished = doneSteps(workflow);
    for (final name in workflow.stepNames) {
      if (!finished.contains(name)) return name;
    }
    return null;
  }

  /// 状态行：下一步是谁，或者都走过了。
  String stateLine(Workflow workflow) {
    final names = workflow.stepNames;
    if (names.isEmpty) {
      return '这条工作流没有步骤——在 workflows/$workflowName.yaml 的 steps 里写步骤';
    }
    final next = nextStep(workflow);
    return next == null ? '${names.length} 个步骤都走过了' : '下一步：$next';
  }
}
