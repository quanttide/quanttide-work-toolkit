/// 信封：动作结果。`ok` 定退出码，`lines` 给人看，`columns` 与 `rows` 给窗口画。
class Outcome {
  Outcome(
    this.ok, {
    List<String>? lines,
    List<String>? columns,
    List<List<String>>? rows,
    this.payload,
  }) : lines = lines ?? <String>[],
       columns = columns ?? <String>[],
       rows = rows ?? <List<String>>[];

  Outcome.failed(List<String> lines) : this(false, lines: lines);

  bool ok;
  List<String> lines;
  List<String> columns;
  List<List<String>> rows;

  /// 有些动作要交出结构化的东西，而不是行列。
  Object? payload;

  Outcome withFirst(String line) {
    lines.insert(0, line);
    return this;
  }

  Map<String, Object?> toJson() => payload != null && payload is Map
      ? (payload! as Map<String, Object?>)
      : {'ok': ok, 'lines': lines, 'columns': columns, 'rows': rows};
}

/// 路径相对根写短一点；不在根底下就原样。
String short(String root, String path) {
  final prefix = root.endsWith('/') ? root : '$root/';
  return path.startsWith(prefix) ? path.substring(prefix.length) : path;
}
