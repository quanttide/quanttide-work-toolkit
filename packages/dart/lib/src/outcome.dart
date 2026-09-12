import 'dart:convert';

/// 结果：一次动作的答复。
///
/// 规范（`docs/specification/process/outcome.md`）：一次动作一份，命令行与窗口都从它取，
/// 推进任务的动作还记进流水。这一份是它的不变部分——四样（`ok` 通不通、`lines` 话、
/// `columns` 与 `rows` 同一份表格、`data` 给界面的那一栏）与编解码；话怎么拼、
/// 路径怎么显示、退出码怎么定，留各自的平台。
///
/// 不可变：[withFirst] / [withData] 返回新的结果。
class Outcome {
  const Outcome(
    this.ok, {
    this.lines = const [],
    this.columns = const [],
    this.rows = const [],
    this.data,
  });

  /// 不成：只带话。
  const Outcome.failed(this.lines)
    : ok = false,
      columns = const [],
      rows = const [],
      data = null;

  /// 从信封装回来。缺样按空算。
  factory Outcome.fromJson(Map<String, dynamic> json) => Outcome(
    json['ok'] == true,
    lines: _strings(json['lines']),
    columns: _strings(json['columns']),
    rows: (json['rows'] as List? ?? const [])
        .map(_strings)
        .toList(growable: false),
    data: (json['data'] as Map?)?.cast<String, Object?>(),
  );

  /// 命令行印的那一份（`--json`）。
  factory Outcome.fromStdout(String stdout) =>
      Outcome.fromJson(jsonDecode(stdout) as Map<String, dynamic>);

  final bool ok;

  /// 给人看的话，一行一句。
  final List<String> lines;

  /// 同一份表格：表头与行，命令行与窗口共用。
  final List<String> columns;
  final List<List<String>> rows;

  /// 给窗口与脚本的那一栏（要交原文就托在这里）。
  final Map<String, Object?>? data;

  /// 往话的开头添一句。
  Outcome withFirst(String line) =>
      Outcome(ok, lines: [line, ...lines], columns: columns, rows: rows, data: data);

  /// 托上给界面的那一栏。
  Outcome withData(Map<String, Object?> data) =>
      Outcome(ok, lines: lines, columns: columns, rows: rows, data: data);

  /// 信封：四样，`data` 有才写。
  Map<String, Object?> toJson() => {
    'ok': ok,
    'lines': lines,
    'columns': columns,
    'rows': rows,
    if (data != null) 'data': data,
  };

  /// 原文那一栏（`--out` 落的就是它）；没托东西就给信封。
  Object? dataJson() => data ?? toJson();

  /// 一栏字符串：不是数组按空算，元素照原样写成文字。
  static List<String> _strings(Object? value) =>
      (value as List? ?? const []).map((item) => '$item').toList(growable: false);
}
