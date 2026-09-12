/// 路径：占位认哪几个、怎么换。
///
/// 中立处：`criterion`（判据的字段里怎么写占位）与 `workspace`（落点、定义核对）向它对齐。
/// 换成哪条路径由调用方给（`workspace` 按任务声明与默认处算）；这里只管认名字与替换。
library;

/// 定义里只认这四个占位。
const List<String> placeholderNames = ['artifacts', 'report', 'journal', 'log'];

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

/// 把一条文字里的占位按 [resolve] 换掉；[resolve] 认不得的占位原样留着（读定义时已经拦过）。
String replacePlaceholders(String value, String? Function(String name) resolve) {
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
    out.write(resolve(name) ?? '{{$name}}');
    rest = after.substring(end + 2);
  }
  out.write(rest);
  return out.toString();
}
