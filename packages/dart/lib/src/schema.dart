/// 定义的 schema：字段名、取值、判据种类，以及读字段、挑陌生字段的小工具。
///
/// 定义要有固定的意义，所以这些都定死。这一层不认模型，只认已经解析好的 Map / List。
library;

const String agent = 'agent';
const String human = 'human';
const String rule = 'rule';
const List<String> executors = [agent, human];
const List<String> criterionTypes = [rule, agent, human];
const List<String> topFields = ['name', 'description', 'steps'];
const List<String> stepFields = ['name', 'description', 'executor', 'criteria'];
const List<String> criterionFields = [
  'executor',
  'description',
  'path',
  'absent',
  'file',
  'contains',
  'run',
];

class DefinitionError implements Exception {
  DefinitionError(this.message);

  final String message;

  @override
  String toString() => message;
}

String textOf(Object? value, String key) {
  if (value is! Map) return '';
  final item = value[key];
  return item is String ? item.trim() : '';
}

List<String> unknownFields(Map mapping, List<String> allowed) => mapping.keys
    .map((key) => '$key')
    .where((key) => !allowed.contains(key))
    .toList();
