/// 路径：目录与剩下的路径相接，占位展开到平台给的目录。
///
/// 中立处：`workspace`（落点、定义核对）向它对齐。
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

/// 判据里的占位按平台给的目录基准展开（够核对用）。
String expandPlaceholders(String value, String base) => value
    .replaceAll('{{artifacts}}', join(base, 'artifacts'))
    .replaceAll('{{report}}', join(base, 'artifacts/report'))
    .replaceAll('{{journal}}', join(base, 'artifacts/journal'))
    .replaceAll('{{log}}', join(base, 'tasks'));
