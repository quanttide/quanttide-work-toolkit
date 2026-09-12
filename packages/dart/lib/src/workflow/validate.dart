import '../criterion/criterion.dart';
import '../executor.dart';
import 'model.dart';

/// 定义顶层认得的字段。
const List<String> _topFields = ['name', 'description', 'steps'];

/// 步骤认得的字段。
const List<String> _stepFields = ['name', 'description', 'executor', 'criteria'];

/// 一份定义读不通：字段缺了、取值越界、有不认识的字段。
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

/// 读一份定义，顺带把整体语法过一遍。读不通就抛 [DefinitionError]。
Workflow validateWorkflow(Object? value, {String file = '定义'}) {
  if (value is! Map) {
    throw DefinitionError('$file 的顶层不是映射（name / steps）');
  }
  if (textOf(value, 'name').isEmpty) {
    throw DefinitionError('$file 少了 name');
  }
  final steps = value['steps'];
  if (steps is! List || steps.isEmpty) {
    throw DefinitionError('$file 少了 steps（至少一个步骤）');
  }
  final unknown = unknownFields(value, _topFields);
  if (unknown.isNotEmpty) {
    throw DefinitionError(
      '$file 顶层有不认识的字段：${unknown.join('、')}（只认 ${_topFields.join('、')}）',
    );
  }
  return Workflow(
    name: textOf(value, 'name'),
    description: textOf(value, 'description'),
    steps: [
      for (var index = 0; index < steps.length; index++)
        validateStep(steps[index], file: file, position: index + 1),
    ],
  );
}

/// 读一个步骤，顺带把语法过一遍。读不通就抛 [DefinitionError]。
Step validateStep(
  Object? value, {
  required String file,
  required int position,
}) {
  if (value is! Map) {
    throw DefinitionError('$file 第 $position 个步骤少了 name');
  }
  if (textOf(value, 'name').isEmpty) {
    throw DefinitionError('$file 第 $position 个步骤少了 name');
  }
  final extra = unknownFields(value, _stepFields);
  if (extra.isNotEmpty) {
    throw DefinitionError(
      '$file 第 $position 个步骤有不认识的字段：${extra.join('、')}（只认 ${_stepFields.join('、')}）',
    );
  }
  var executor = textOf(value, 'executor');
  if (executor.isEmpty) executor = agent;
  if (!executors.contains(executor)) {
    throw DefinitionError(
      '$file 第 $position 个步骤的 executor 只能是 ${executors.join(' 或 ')}，实得 $executor',
    );
  }
  final raw = value['criteria'];
  final List items;
  if (raw == null) {
    items = const [];
  } else if (raw is List) {
    items = raw;
  } else {
    throw DefinitionError('$file 第 $position 个步骤的 criteria 应当是列表');
  }
  final parsed = <Criterion>[];
  for (var order = 0; order < items.length; order++) {
    parsed.add(
      readCriterion(
        items[order],
        file: file,
        place: '第 $position 个步骤第 ${order + 1} 条判据',
      ),
    );
  }
  return Step(
    name: textOf(value, 'name'),
    description: textOf(value, 'description'),
    executor: executor,
    criteria: parsed,
  );
}
