
import 'workflow.dart';

/// 去掉尾巴上的斜杠，拼路径不出双斜杠。
String _trim(String path) => path.endsWith('/') ? path.substring(0, path.length - 1) : path;

/// 任务聚合：工作流的一次执行实例。
///
/// 指令是跑哪条工作流（[workflowName]，**按名字**引用）与从哪开工（[start]）；
/// 状态是流水（只增不改）、闸门项、产物落点；另带这次执行的运行上下文。
/// 任务是运行数据，不是产物——程序只维护它，不往产物里写字。
///
/// 不可变：[recorded] / [withGates] 都返回新的任务，改动由调用方落盘。
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
    final data = _trim(context.data);
    if (kind == 'log') return '$data/tasks/$name.yaml';
    final written = declared(kind);
    if (written != null) {
      return written.startsWith('/') ? written : '${_trim(context.root)}/$written';
    }
    return '$data/artifacts/$kind/$name.md';
  }

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

/// 这次执行自带的运行上下文：工作区根、数据仓、工作流目录。
///
/// 三个字段怎么读、怎么写由各自的包定（工具箱只管托着它们）。
class RunContext {
  const RunContext({this.root = '', this.data = '', this.workflows = ''});

  factory RunContext.of(Map value) => RunContext(
    root: textOf(value, 'root'),
    data: textOf(value, 'data'),
    workflows: textOf(value, 'workflows'),
  );

  final String root;
  final String data;
  final String workflows;

  Map<String, Object?> toMap() => {
    'root': root,
    'data': data,
    'workflows': workflows,
  };
}
