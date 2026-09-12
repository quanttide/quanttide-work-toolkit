import 'criterion.dart';
import 'executor.dart';
import 'fields.dart';

/// 定义顶层认得的字段。
const List<String> _topFields = ['name', 'description', 'steps'];

/// 步骤认得的字段。
const List<String> _stepFields = ['name', 'description', 'executor', 'criteria'];

/// 工作流聚合：一串有序的步骤。
///
/// 定义的**语法与不变量**都在这个文件里——字段名、取值、判据种类，不认识、缺了、
/// 越界，当场报错。规矩的出处是 `docs/specification/process/workflow.md`·语法。
///
/// 模型不可变：[Workflow.fromValue] 读进来顺带校验，[Workflow.of] 读已经校验过的，
/// [toMap] 写成同样的字段形状。YAML 怎么读写是各语言自己的事。
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
          Step.fromValue(steps[index], file: file, position: index + 1),
      ],
    );
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

  /// 核对这条定义：判据里的路径在不在、描述提到的小节有没有判据覆盖。
  ///
  /// [exists] 由调用方给——工具箱不碰文件系统。
  List<Finding> check(String data, bool Function(String path) exists) {
    final found = <Finding>[];
    for (final step in steps) {
      for (final criterion in step.rules) {
        final literal = switch (criterion) {
          PathExists(:final path) => path,
          FileContains(:final file) => file,
          _ => '',
        };
        if (literal.isEmpty) continue;
        if (literal.contains('{{report}}') ||
            literal.contains('{{journal}}') ||
            literal.contains('{{log}}')) {
          continue;
        }
        final written = expandPlaceholders(literal, data);
        found.add(
          Finding(
            where: '${step.name}·$literal',
            what: '判据里的路径在不在：$written',
            ok: exists(written),
          ),
        );
      }
    }

    final covered = steps
        .expand((step) => step.rules)
        .whereType<FileContains>()
        .map((criterion) => criterion.contains)
        .toList();

    final mentioned = <String>[];
    for (final step in steps) {
      final text = step.description;
      for (final piece in text.split('## ').skip(1)) {
        final name = piece.split(RegExp(r'[\s`」]')).first.trim();
        if (looksLikeSection(name) && !mentioned.contains(name)) {
          mentioned.add(name);
        }
      }
      var rest = text;
      while (true) {
        final at = rest.indexOf('「');
        if (at < 0) break;
        final after = rest.substring(at + 1);
        final end = after.indexOf('」');
        if (end < 0) break;
        final name = after.substring(0, end).trim();
        final tail = after.substring(end + 1).trimLeft();
        final isSection =
            tail.startsWith('一节') ||
            tail.startsWith('节') ||
            tail.startsWith('两节');
        if (isSection && looksLikeSection(name) && !mentioned.contains(name)) {
          mentioned.add(name);
        }
        rest = after.substring(end + 1);
      }
    }
    for (final name in mentioned) {
      found.add(
        Finding(
          where: 'description',
          what: 'description 提到的报告小节有没有判据覆盖：$name',
          ok: covered.any((value) => value.contains(name)),
        ),
      );
    }
    return found;
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
              .map(Criterion.fromMap)
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

/// 定义核对出来的一件事：在哪里、核的是什么、过没过。
///
/// 它是「把这条定义对着工作区核一遍」的回执，不单列成一个文件——
/// `workflow --check` 的实现产物，规范里还没有这一节。
class Finding {
  Finding({required this.where, required this.what, required this.ok});

  final String where;
  final String what;
  final bool ok;
}

/// 像不像报告小节的名字：中文短词。版本号写法、占位、路径都不算。
bool looksLikeSection(String name) {
  if (name.isEmpty || name.length > 12) return false;
  return !RegExp(r'[0-9\[\]{}./`<>_-]').hasMatch(name);
}

/// 判据里的占位先按数据仓展开（够核对用）。
String expandPlaceholders(String value, String data) => value
    .replaceAll('{{artifacts}}', '$data/artifacts')
    .replaceAll('{{report}}', '$data/artifacts/report')
    .replaceAll('{{journal}}', '$data/artifacts/journal')
    .replaceAll('{{log}}', '$data/tasks');
