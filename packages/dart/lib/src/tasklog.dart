/// 任务流水与「走过」的算法。
///
/// 一条流水＝时间、步骤名、一句话、过没过。步骤名带后缀的（`·审` 审查、`·判` 机器判据）
/// **给这一步的结论投票**；不带后缀的（重新执行一次）**把结论从头算**。
String textOf(Object? value, String key) {
  if (value is! Map) return '';
  final item = value[key];
  return item is String ? item.trim() : '';
}

/// 哪些步骤走过了。[steps] 是工作流上的步骤名，按定义顺序。
List<String> done(List<String> steps, List<Map> events) {
  final verdict = <String, bool>{};
  final order = <String>[];
  for (final event in events) {
    final ok = event['ok'] == true;
    final raw = '${event['step'] ?? ''}';
    final at = raw.indexOf('·');
    final step = at < 0 ? raw : raw.substring(0, at);
    final hasExtra = at >= 0;
    if (!steps.contains(step)) continue;
    if (verdict.containsKey(step)) {
      verdict[step] = hasExtra ? (verdict[step]! && ok) : ok;
    } else {
      verdict[step] = ok;
      order.add(step);
    }
  }
  return order.where((step) => verdict[step] == true).toList();
}

/// 第一个没走到的步骤。
String? nextStep(List<String> steps, List<Map> events) {
  final finished = done(steps, events);
  for (final name in steps) {
    if (!finished.contains(name)) return name;
  }
  return null;
}

/// 状态行：下一步是谁，或者都走过了。
String stateLine(List<String> steps, List<Map> events, String workflowName) {
  if (steps.isEmpty) {
    return '这条工作流没有步骤——在 workflows/$workflowName.yaml 的 steps 里写步骤';
  }
  final next = nextStep(steps, events);
  return next == null ? '${steps.length} 个步骤都走过了' : '下一步：$next';
}
