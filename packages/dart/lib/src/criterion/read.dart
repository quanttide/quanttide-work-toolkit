import '../executor.dart';
import '../workflow/validate.dart';
import 'model.dart';

/// 一条判据认得的字段。
const List<String> _criterionFields = [
  'executor',
  'description',
  'path',
  'absent',
  'file',
  'contains',
  'run',
];

/// 从定义里的字段认出一条判据（不校验）。
Criterion criterionOf(Map map) {
  final description = textOf(map, 'description');
  final kind = textOf(map, 'executor');
  if (kind == agent) return AgentJudgement(description);
  if (kind == human) return HumanGate(description);
  final path = textOf(map, 'path');
  if (path.isNotEmpty) return PathExists(path, description: description);
  final absent = textOf(map, 'absent');
  if (absent.isNotEmpty) return PathAbsent(absent, description: description);
  final file = textOf(map, 'file');
  if (file.isNotEmpty) {
    return FileContains(file, textOf(map, 'contains'), description: description);
  }
  return CommandRun(textOf(map, 'run'), description: description);
}

/// 读一条判据：不是映射、取值不对、缺该有的字段，当场报错。
///
/// `file` 与 `place` 只用来说话；返回的是认好的值对象。
Criterion readCriterion(
  Object? value, {
  required String file,
  required String place,
}) {
  // 先看是不是映射：不是映射时要说「不是映射」，不能先说 executor 该怎么写——
  // 那样报错会指错方向（2026-09-12 之前正是这个顺序，这条分支因此永远走不到）。
  if (value is! Map) {
    throw DefinitionError('$file $place不是映射');
  }
  final kind = textOf(value, 'executor');
  if (!criterionTypes.contains(kind)) {
    throw DefinitionError(
      '$file $place的 executor 只能是 ${criterionTypes.join(' / ')}（谁判：规则引擎 / 智能体 / 人）',
    );
  }
  final odd = unknownFields(value, _criterionFields);
  if (odd.isNotEmpty) {
    throw DefinitionError(
      '$file $place有不认识的字段：${odd.join('、')}（只认 ${_criterionFields.join('、')}）',
    );
  }
  final given = [
    'path',
    'absent',
    'file',
    'contains',
    'run',
  ].where((name) => value[name] != null).toList();
  if (kind == rule) {
    if (given.isEmpty) {
      throw DefinitionError(
        '$file $place是 rule，得写一条判法（path / absent / file+contains / run）',
      );
    }
    if (given.contains('contains') && !given.contains('file')) {
      throw DefinitionError('$file $place写了 contains，还得写 file');
    }
    if (given.contains('file') && !given.contains('contains')) {
      throw DefinitionError('$file $place写了 file，还得写 contains');
    }
    final others = given
        .where((name) => name != 'file' && name != 'contains')
        .toList();
    if (others.length > 1 || (others.isNotEmpty && given.contains('file'))) {
      throw DefinitionError('$file $place的判法只能一种：path / absent / file+contains / run');
    }
  } else {
    if (textOf(value, 'description').isEmpty) {
      throw DefinitionError(
        '$file $place是 $kind，必须写 description（判准 / 要人拍板的事）',
      );
    }
    if (given.isNotEmpty) {
      throw DefinitionError(
        '$file $place是 $kind，不该带 ${given.join('、')}（那是 rule 的字段）',
      );
    }
  }
  return criterionOf(value);
}
