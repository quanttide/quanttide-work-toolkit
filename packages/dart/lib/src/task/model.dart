import '../workflow/validate.dart';
import 'context.dart';
import 'journal.dart';

/// 拼目录与剩下的路径：末尾斜杠忽略、重复斜杠折叠（`/` 与空串等价——都落在根）。
///
/// 规矩的出处是 `docs/specification/process/task.md`·落点。落点只有这一处拼法。
String _join(String dir, String rest) {
  final clean = StringBuffer();
  var lastWasSlash = false;
  for (final ch in dir.split('')) {
    if (ch == '/') {
      if (lastWasSlash) continue;
      lastWasSlash = true;
    } else {
      lastWasSlash = false;
    }
    clean.write(ch);
  }
  var head = clean.toString();
  while (head.endsWith('/')) {
    head = head.substring(0, head.length - 1);
  }
  return '$head/$rest';
}

/// 任务聚合：工作流的一次执行实例。
///
/// 指令是跑哪条工作流（[workflowName]，**按名字**引用）与从哪开工（[start]）；
/// 状态是流水（只增不改）、闸门项、产物落点；另带这次执行的运行上下文。
/// 任务是运行数据，不是产物——程序只维护它，不往产物里写字。
///
/// 不可变：[recorded] / [withGates] 都返回新的任务，改动由调用方落盘。
/// 「走过哪几步」从流水读出，在 `journal.dart`。
class Task {
  const Task({
    required this.name,
    required this.workflowName,
    this.start = '',
    this.context = const RunContext(),
    this.journal = const [],
    this.gates = const [],
    this.artifacts = const {},
  });

  /// 从任务文件里的字段读出。[name] 由调用方给（文件名即任务名）。
  factory Task.of(String name, Map payload) => Task(
    name: name,
    workflowName: textOf(payload, 'workflow'),
    start: textOf(payload, 'start'),
    context: RunContext.of(payload),
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
  final RunContext context;
  final List<JournalEvent> journal;

  /// 等人拍板的事项。
  final List<String> gates;

  /// 这次执行往哪写产物（声明写成什么就是什么，相对工作区根）。
  final Map<String, String> artifacts;

  /// 这种产物声明了往哪写；没声明给 null。
  String? declared(String kind) {
    final written = artifacts[kind]?.trim() ?? '';
    return written.isEmpty ? null : written;
  }

  /// 这次执行往哪写这种产物（规范「任务 / 语法」里的落点）。
  ///
  /// 声明了按声明的（相对工作区根）；没声明落数据仓的
  /// `artifacts/<种类>/<任务名>.md`；流水是任务文件本身。
  String artifact(String kind, RunContext context) {
    if (kind == 'log') return _join(context.data, 'tasks/$name.yaml');
    final written = declared(kind);
    if (written != null) {
      return written.startsWith('/') ? written : _join(context.root, written);
    }
    return _join(context.data, 'artifacts/$kind/$name.md');
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
    context: context,
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
      context: context,
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
    ...context.toMap(),
  };
}

/// 判据里的占位先按数据仓展开（够核对用）。
String expandPlaceholders(String value, String data) => value
    .replaceAll('{{artifacts}}', _join(data, 'artifacts'))
    .replaceAll('{{report}}', _join(data, 'artifacts/report'))
    .replaceAll('{{journal}}', _join(data, 'artifacts/journal'))
    .replaceAll('{{log}}', _join(data, 'tasks'));
