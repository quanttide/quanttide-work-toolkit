/// 路径：目录与剩下的路径相接，占位展开到数据仓。
///
/// 中立处：`task`（落点）与 `workflow`（定义核对）都向它对齐。
library;

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

/// 判据里的占位先按数据仓展开（够核对用）。
String expandPlaceholders(String value, String data) => value
    .replaceAll('{{artifacts}}', join(data, 'artifacts'))
    .replaceAll('{{report}}', join(data, 'artifacts/report'))
    .replaceAll('{{journal}}', join(data, 'artifacts/journal'))
    .replaceAll('{{log}}', join(data, 'tasks'));
