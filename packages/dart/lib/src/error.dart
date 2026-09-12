/// 定义读不通时的错误。
///
/// 只存位置（顶层 / 第 n 个步骤 / 第 n 个步骤第 m 条判据）与种类（[Fault]）；
/// 文件名不进错误，由端侧在渲染时给——[DefinitionError.message] 出 canonical 文案。
library;

import 'executor.dart';
import 'fields.dart';

/// 定义里的位置。
sealed class Position {
  const Position();

  /// 位置的话头，比如「第 1 个步骤」；顶层是空串。
  String get phrase;
}

/// 顶层。
class TopPosition extends Position {
  const TopPosition();

  @override
  String get phrase => '';
}

/// 第 n 个步骤（从 1 数）。
class StepPosition extends Position {
  const StepPosition(this.step);

  final int step;

  @override
  String get phrase => '第 $step 个步骤';
}

/// 第 n 个步骤第 m 条判据（都从 1 数）。
class CriterionPosition extends Position {
  const CriterionPosition(this.step, this.criterion);

  final int step;
  final int criterion;

  @override
  String get phrase => '第 $step 个步骤第 $criterion 条判据';
}

/// 这一条错在哪，带上渲染所需的取值。
sealed class Fault {
  const Fault();

  /// 位置之后的那一句（canonical 文案的尾巴）。
  String text();
}

/// 定义顶层不是映射。
class TopNotMapping extends Fault {
  const TopNotMapping();

  @override
  String text() => '的顶层不是映射（name / steps）';
}

/// 顶层少了 name。
class MissingName extends Fault {
  const MissingName();

  @override
  String text() => '少了 name';
}

/// 顶层少了 steps（至少一个步骤）。
class MissingSteps extends Fault {
  const MissingSteps();

  @override
  String text() => '少了 steps（至少一个步骤）';
}

/// 顶层有不认识的字段。
class UnknownTopFields extends Fault {
  const UnknownTopFields(this.unknown);

  final List<String> unknown;

  @override
  String text() =>
      '顶层有不认识的字段：${unknown.join('、')}（只认 ${topFields.join('、')}）';
}

/// 步骤少了 name（步骤不是映射也算）。
class MissingStepName extends Fault {
  const MissingStepName();

  @override
  String text() => '少了 name';
}

/// 步骤有不认识的字段。
class UnknownStepFields extends Fault {
  const UnknownStepFields(this.unknown);

  final List<String> unknown;

  @override
  String text() =>
      '有不认识的字段：${unknown.join('、')}（只认 ${stepFields.join('、')}）';
}

/// 步骤的 executor 越界。
class BadStepExecutor extends Fault {
  const BadStepExecutor(this.got);

  final String got;

  @override
  String text() => '的 executor 只能是 ${executors.join(' 或 ')}，实得 $got';
}

/// 步骤的 criteria 不是列表。
class CriteriaNotList extends Fault {
  const CriteriaNotList();

  @override
  String text() => '的 criteria 应当是列表';
}

/// 判据不是映射。
class CriterionNotMapping extends Fault {
  const CriterionNotMapping();

  @override
  String text() => '不是映射';
}

/// 判据的 executor 越界。
class BadCriterionExecutor extends Fault {
  const BadCriterionExecutor();

  @override
  String text() =>
      '的 executor 只能是 ${criterionTypes.join(' / ')}（谁判：规则引擎 / 智能体 / 人）';
}

/// 判据有不认识的字段。
class UnknownCriterionFields extends Fault {
  const UnknownCriterionFields(this.unknown);

  final List<String> unknown;

  @override
  String text() =>
      '有不认识的字段：${unknown.join('、')}（只认 ${criterionFields.join('、')}）';
}

/// 判据是 rule，却没写判法。
class RuleNeedsJudgement extends Fault {
  const RuleNeedsJudgement();

  @override
  String text() => '是 rule，得写一条判法（path / absent / file+contains / run）';
}

/// 写了 contains，没写 file。
class ContainsNeedsFile extends Fault {
  const ContainsNeedsFile();

  @override
  String text() => '写了 contains，还得写 file';
}

/// 写了 file，没写 contains。
class FileNeedsContains extends Fault {
  const FileNeedsContains();

  @override
  String text() => '写了 file，还得写 contains';
}

/// 判法混着写：只能一种。
class OnlyOneJudgement extends Fault {
  const OnlyOneJudgement();

  @override
  String text() => '的判法只能一种：path / absent / file+contains / run';
}

/// 判据是 agent / human，却没写 description。
class NeedsDescription extends Fault {
  const NeedsDescription(this.kind);

  final String kind;

  @override
  String text() => '是 $kind，必须写 description（判准 / 要人拍板的事）';
}

/// 判据是 agent / human，却带了 rule 的字段。
class NoRuleFields extends Fault {
  const NoRuleFields(this.kind, this.given);

  final String kind;
  final List<String> given;

  @override
  String text() => '是 $kind，不该带 ${given.join('、')}（那是 rule 的字段）';
}

/// 一份定义读不通。
class DefinitionError implements Exception {
  DefinitionError(this.position, this.fault);

  /// 在哪一层读不通。
  final Position position;

  /// 哪一条不成立。
  final Fault fault;

  /// canonical 报错文字：`{file} {位置}{这一条}`；文件由端侧在渲染时给。
  String message(String file) => '$file ${position.phrase}${fault.text()}';

  @override
  String toString() => '${position.phrase}${fault.text()}';
}
