import '../fields.dart';
import 'journal.dart';

/// 任务聚合：工作流的一次执行实例。
///
/// 指令是跑哪条工作流（[workflowName]，**按名字**引用）与从哪开工（[start]）；
/// 状态是流水（只增不改）、闸门项、产物声明。任务是运行数据，不是产物——
/// 程序只维护它，不往产物里写字。任务不带位置：它在哪、产物落哪，由工作区算。
///
/// 不可变：[recorded] / [withGates] 都返回新的任务，改动由调用方落盘。
/// 「走过哪几步」与落点在 `workspace/`。
class Task {
  const Task({
    required this.name,
    required this.workflowName,
    this.start = '',
    this.journal = const [],
    this.gates = const [],
    this.artifacts = const {},
  });

  /// 从任务文件里的字段读出。[name] 由调用方给（文件名即任务名）。
  factory Task.of(Map payload) => Task(
    name: textOf(payload, 'name'),
    workflowName: textOf(payload, 'workflow'),
    start: textOf(payload, 'start'),
    journal: [
      for (final event in (payload['log'] as List? ?? const []).cast<Map>())
        JournalEvent.of(event),
    ],
    gates: [
      for (final note in payload['gates'] as List? ?? const [])
        if (note is String) note,
    ],
    artifacts: {
      for (final entry in (payload['artifacts'] as Map? ?? const {}).entries)
        if (entry.key is String && entry.value is String)
          '${entry.key}': '${entry.value}',
    },
  );

  final String name;
  final String workflowName;
  final String start;
  final List<JournalEvent> journal;

  /// 等人拍板的事项。
  final List<String> gates;

  /// 这次执行往哪写产物（声明写成什么就是什么，相对平台给的目录基准）。
  final Map<String, String> artifacts;

  /// 这种产物声明了往哪写；没声明给 null。
  String? declared(String kind) {
    final written = artifacts[kind]?.trim() ?? '';
    return written.isEmpty ? null : written;
  }

  /// 记一笔流水：流水只增不改，所以返回新的任务。
  Task recorded({
    required String at,
    required String step,
    required String detail,
    required bool ok,
  }) => Task(
    name: name,
    workflowName: workflowName,
    start: start,
    journal: [
      ...journal,
      JournalEvent(at: at, step: step, detail: detail, ok: ok),
    ],
    gates: gates,
    artifacts: artifacts,
  );

  /// 记下闸门项；已有的不重复记。同样返回新的任务。
  Task withGates(List<String> notes) {
    final all = [...gates];
    for (final note in notes) {
      if (!all.contains(note)) all.add(note);
    }
    return Task(
      name: name,
      workflowName: workflowName,
      start: start,
      journal: journal,
      gates: all,
      artifacts: artifacts,
    );
  }

  Map<String, Object?> toMap() => {
    'name': name,
    'start': start,
    'workflow': workflowName,
    'log': [for (final event in journal) event.toMap()],
    'gates': gates,
    'artifacts': artifacts,
  };
}
