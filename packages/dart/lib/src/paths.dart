/// 路径：目录与剩下的路径相接，占位换成落点。
///
/// 中立处：`criterion`（判据的字段里怎么写占位）与 `workspace`（落点、定义核对）向它对齐。
/// 落点本身由 `workspace` 按任务声明与默认处算，这里只管「占位换成哪条路径」。
library;

/// 定义里只认这四个占位。
const List<String> placeholderNames = ['artifacts', 'report', 'journal', 'log'];

/// 拼目录与剩下的路径：末尾斜杠忽略、重复斜杠折叠（`/` 与空串等价——都落在根）。
///
/// 规矩的出处是 `docs/specification/process/task.md`·落点。落点只有这一处拼法。
String join(String dir, String rest) {
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

/// 一条文字里的占位名（`{{name}}` 的 name，按出现次序，重复的也留）。
List<String> placeholdersIn(String value) {
  final names = <String>[];
  var rest = value;
  while (true) {
    final at = rest.indexOf('{{');
    if (at < 0) break;
    final after = rest.substring(at + 2);
    final end = after.indexOf('}}');
    if (end < 0) break;
    names.add(after.substring(0, end));
    rest = after.substring(end + 2);
  }
  return names;
}

/// 占位表：四个占位各换成哪个落点。
///
/// 落点由 `workspace` 按任务声明与默认处算好（见 `WorkspaceArtifact.placeholders`）；
/// 判据里的字段与落点不再各拼一套。四个占位认哪几个，以 [placeholderNames] 为准。
class Placeholders {
  const Placeholders({
    required this.artifacts,
    required this.report,
    required this.journal,
    required this.log,
  });

  /// `{{artifacts}}`：产物目录。
  final String artifacts;

  /// `{{report}}`：报告的落点。
  final String report;

  /// `{{journal}}`：日志的落点。
  final String journal;

  /// `{{log}}`：流水（任务文件本身）。
  final String log;

  /// 这个占位换成哪个落点；不认识给 `null`。
  String? pathOf(String name) => switch (name) {
    'artifacts' => artifacts,
    'report' => report,
    'journal' => journal,
    'log' => log,
    _ => null,
  };

  /// 把一条文字里的占位换成落点；不认识的占位原样留着（读定义时已经拦过）。
  String expand(String value) {
    if (!value.contains('{{')) return value;
    final out = StringBuffer();
    var rest = value;
    while (true) {
      final at = rest.indexOf('{{');
      if (at < 0) break;
      out.write(rest.substring(0, at));
      final after = rest.substring(at + 2);
      final end = after.indexOf('}}');
      if (end < 0) {
        out.write('{{');
        rest = after;
        continue;
      }
      final name = after.substring(0, end);
      final path = pathOf(name);
      out.write(path ?? '{{$name}}');
      rest = after.substring(end + 2);
    }
    out.write(rest);
    return out.toString();
  }
}
