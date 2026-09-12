import '../error.dart';
import '../executor.dart';
import '../fields.dart';
import '../paths.dart';
import 'model.dart';

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
/// [step] 与 [order] 是这条判据的位置（第几个步骤、第几条判据），只用来说话；
/// 返回的是认好的值对象。
Criterion readCriterion(Object? value, int step, int order) {
  final at = CriterionPosition(step, order);
  // 先看是不是映射：不是映射时要说「不是映射」，不能先说 executor 该怎么写——
  // 那样报错会指错方向（2026-09-12 之前正是这个顺序，这条分支因此永远走不到）。
  if (value is! Map) {
    throw DefinitionError(at, const CriterionNotMapping());
  }
  final kind = textOf(value, 'executor');
  if (!criterionTypes.contains(kind)) {
    throw DefinitionError(at, const BadCriterionExecutor());
  }
  final odd = unknownFields(value, criterionFields);
  if (odd.isNotEmpty) {
    throw DefinitionError(at, UnknownCriterionFields(odd));
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
      throw DefinitionError(at, const RuleNeedsJudgement());
    }
    if (given.contains('contains') && !given.contains('file')) {
      throw DefinitionError(at, const ContainsNeedsFile());
    }
    if (given.contains('file') && !given.contains('contains')) {
      throw DefinitionError(at, const FileNeedsContains());
    }
    final others = given
        .where((name) => name != 'file' && name != 'contains')
        .toList();
    if (others.length > 1 || (others.isNotEmpty && given.contains('file'))) {
      throw DefinitionError(at, const OnlyOneJudgement());
    }
  } else {
    if (textOf(value, 'description').isEmpty) {
      throw DefinitionError(at, NeedsDescription(kind));
    }
    if (given.isNotEmpty) {
      throw DefinitionError(at, NoRuleFields(kind, given));
    }
  }
  // 路径里只认四个占位；写别的（如 `{{foo}}`）算不合语法（规范 `process/workflow.md`·语法）。
  final unknown = <String>[];
  for (final field in ['path', 'absent', 'file']) {
    for (final name in placeholdersIn(textOf(value, field))) {
      if (!placeholderNames.contains(name) && !unknown.contains(name)) {
        unknown.add(name);
      }
    }
  }
  if (unknown.isNotEmpty) {
    throw DefinitionError(at, UnknownPlaceholder(unknown));
  }
  return criterionOf(value);
}
