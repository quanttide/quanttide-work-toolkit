/// 读字段、挑陌生字段的小工具，以及定义读不通时报的错。
///
/// 内部用：只认已经解析好的 Map / List。YAML 怎么读、怎么写是各语言自己的事。
library;

/// 一份定义（或一件任务）读不通：字段缺了、取值越界、有不认识的字段。
class DefinitionError implements Exception {
  DefinitionError(this.message);

  final String message;

  @override
  String toString() => message;
}

/// 取一个字符串字段，去掉两侧空白；不是字符串就当没写。
String textOf(Object? value, String key) {
  if (value is! Map) return '';
  final item = value[key];
  return item is String ? item.trim() : '';
}

/// 这次给的字段里，哪些是不认识的。
List<String> unknownFields(Map mapping, List<String> allowed) => mapping.keys
    .map((key) => '$key')
    .where((key) => !allowed.contains(key))
    .toList();
