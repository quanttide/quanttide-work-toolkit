import '../criterion/criterion.dart';
import '../error.dart';
import '../executor.dart';
import '../fields.dart';

/// 语法校验一份定义：不是映射、缺字段、取值不对，读不通就抛 [DefinitionError]。
void validateWorkflow(Object? value) {
  if (value is! Map) {
    throw DefinitionError(const TopPosition(), const TopNotMapping());
  }
  if (textOf(value, 'name').isEmpty) {
    throw DefinitionError(const TopPosition(), const MissingName());
  }
  final steps = value['steps'];
  if (steps is! List || steps.isEmpty) {
    throw DefinitionError(const TopPosition(), const MissingSteps());
  }
  final unknown = unknownFields(value, topFields);
  if (unknown.isNotEmpty) {
    throw DefinitionError(const TopPosition(), UnknownTopFields(unknown));
  }
  for (var index = 0; index < steps.length; index++) {
    validateStep(steps[index], position: index + 1);
  }
}

/// 语法校验一个步骤：读不通就抛 [DefinitionError]。
void validateStep(Object? value, {required int position}) {
  final at = StepPosition(position);
  if (value is! Map) {
    throw DefinitionError(at, const MissingStepName());
  }
  if (textOf(value, 'name').isEmpty) {
    throw DefinitionError(at, const MissingStepName());
  }
  final extra = unknownFields(value, stepFields);
  if (extra.isNotEmpty) {
    throw DefinitionError(at, UnknownStepFields(extra));
  }
  var executor = textOf(value, 'executor');
  if (executor.isEmpty) executor = agent;
  if (!executors.contains(executor)) {
    throw DefinitionError(at, BadStepExecutor(executor));
  }
  final raw = value['criteria'];
  final List items;
  if (raw == null) {
    items = const [];
  } else if (raw is List) {
    items = raw;
  } else {
    throw DefinitionError(at, const CriteriaNotList());
  }
  for (var order = 0; order < items.length; order++) {
    readCriterion(items[order], position, order + 1);
  }
}
