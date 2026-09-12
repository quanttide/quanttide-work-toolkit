import '../criterion/criterion.dart';
import '../executor.dart';
import '../fields.dart';
import 'validate.dart';

/// 工作流聚合：一串有序的步骤。
///
/// 本文件只装模型——字段名、不变量、取值校验与报错在 `validate.dart`。
/// 规矩的出处是 `docs/specification/process/workflow.md`·语法。
///
/// 模型不可变：[Workflow.fromValue] 读进来顺带校验，[Workflow.of] 读已经校验过的，
/// [toMap] 写成同样的字段形状。YAML 怎么读写是各语言自己的事。
/// 定义核对（[Workflow.check]）在 `check.dart`。
class Workflow {
  const Workflow({
    required this.name,
    this.description = '',
    this.steps = const [],
  });

  /// 从定义里的字段读出（不校验）。用在已经校验过的定义上；
  /// [name] 由调用方给（文件名即工作流名）。
  factory Workflow.of(String name, Map payload) => Workflow(
    name: name,
    description: textOf(payload, 'description'),
    steps:
        (payload['steps'] as List?)
            ?.cast<Map>()
            .map(Step.of)
            .toList(growable: false) ??
        const [],
  );

  /// 从定义里的字段读出，顺带把语法过一遍。读不通就抛 [DefinitionError]。
  factory Workflow.fromValue(Object? value, {String file = '定义'}) {
    validateWorkflow(value, file: file);
    return Workflow.of(textOf(value, 'name'), value as Map);
  }

  final String name;
  final String description;

  /// 步骤：按定义里的顺序——这就是「串联」。
  final List<Step> steps;

  /// 步骤名，按定义顺序。
  List<String> get stepNames =>
      steps.map((step) => step.name).toList(growable: false);

  Step? step(String name) {
    for (final item in steps) {
      if (item.name == name) return item;
    }
    return null;
  }

  Map<String, Object?> toMap() => {
    'name': name,
    if (description.isNotEmpty) 'description': description,
    'steps': [for (final step in steps) step.toMap()],
  };
}

/// 一个工作步骤：叫什么、做什么、谁执行、怎么算完。
class Step {
  const Step({
    required this.name,
    this.description = '',
    this.executor = agent,
    this.criteria = const [],
  });

  /// 从定义里的字段读出（不校验）。
  factory Step.of(Map value) {
    final executor = textOf(value, 'executor');
    return Step(
      name: textOf(value, 'name'),
      description: textOf(value, 'description'),
      executor: executor.isEmpty ? agent : executor,
      criteria:
          (value['criteria'] as List?)
              ?.cast<Map>()
              .map(criterionOf)
              .toList(growable: false) ??
          const [],
    );
  }

  /// 从定义里的字段读出，顺带把语法过一遍。
  factory Step.fromValue(
    Object? value, {
    required String file,
    required int position,
  }) {
    validateStep(value, file: file, position: position);
    return Step.of(value as Map);
  }

  final String name;
  final String description;
  final String executor;
  final List<Criterion> criteria;

  bool get isHuman => executor == human;

  List<Criterion> _ofKind(String kind) =>
      criteria.where((item) => item.executor == kind).toList(growable: false);

  List<Criterion> get rules => _ofKind(rule);
  List<Criterion> get agents => _ofKind(agent);
  List<Criterion> get gates => _ofKind(human);

  Map<String, Object?> toMap() => {
    'name': name,
    if (description.isNotEmpty) 'description': description,
    'executor': executor,
    if (criteria.isNotEmpty)
      'criteria': [for (final criterion in criteria) criterion.toMap()],
  };
}
