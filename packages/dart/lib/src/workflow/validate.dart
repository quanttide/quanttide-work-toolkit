import '../criterion/criterion.dart';
import '../error.dart';
import '../executor.dart';
import '../fields.dart';

/// 语法校验一份定义：不是映射、缺字段、取值不对，读不通就抛 [DefinitionError]。
void validateWorkflow(Object? value, {String file = '定义'}) {
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
  final unknown = unknownFields(value, topFields);
  if (unknown.isNotEmpty) {
    throw DefinitionError(
      '$file 顶层有不认识的字段：${unknown.join('、')}（只认 ${topFields.join('、')}）',
    );
  }
  for (var index = 0; index < steps.length; index++) {
    validateStep(steps[index], file: file, position: index + 1);
  }
}

/// 语法校验一个步骤：读不通就抛 [DefinitionError]。
void validateStep(
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
  final extra = unknownFields(value, stepFields);
  if (extra.isNotEmpty) {
    throw DefinitionError(
      '$file 第 $position 个步骤有不认识的字段：${extra.join('、')}（只认 ${stepFields.join('、')}）',
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
  for (var order = 0; order < items.length; order++) {
    readCriterion(
      items[order],
      file: file,
      place: '第 $position 个步骤第 ${order + 1} 条判据',
    );
  }
}
