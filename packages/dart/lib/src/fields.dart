/// 从定义里读字段：取值助手与各层认得的字段表。
///
/// 这是全库共用的「读定义」公件，不属于任何聚合——放在中立处，聚合只向下依赖它。
library;

/// 定义顶层认得的字段。
const List<String> topFields = ['name', 'description', 'steps'];

/// 步骤认得的字段。
const List<String> stepFields = ['name', 'description', 'executor', 'criteria'];

/// 一条判据认得的字段。
const List<String> criterionFields = [
  'executor',
  'description',
  'path',
  'absent',
  'file',
  'contains',
  'run',
];

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
