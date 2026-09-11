/// 工作流定义：一串有序的步骤。
///
/// 定义要有固定的意义，所以字段名、取值、判据种类都由 schema 定死，不认识的字段直接报错。
/// 这一层只管**已经解析好的 Map / List**；YAML 怎么读进来，各语言各自的库去管。
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

/// 读一份定义：不是映射、缺字段、取值不对，当场报错。[file] 只用来说话。
void validateDefinition(Object? payload, String file) {
  if (payload is! Map) {
    throw DefinitionError('$file 的顶层不是映射（name / steps）');
  }
  if (textOf(payload, 'name').isEmpty) {
    throw DefinitionError('$file 少了 name');
  }
  final steps = payload['steps'];
  if (steps is! List || steps.isEmpty) {
    throw DefinitionError('$file 少了 steps（至少一个步骤）');
  }
  final unknown = unknownFields(payload, topFields);
  if (unknown.isNotEmpty) {
    throw DefinitionError(
      '$file 顶层有不认识的字段：${unknown.join('、')}（只认 ${topFields.join('、')}）',
    );
  }
  for (var index = 0; index < steps.length; index++) {
    final position = index + 1;
    final step = steps[index];
    if (step is! Map) {
      throw DefinitionError('$file 第 $position 个步骤少了 name');
    }
    if (textOf(step, 'name').isEmpty) {
      throw DefinitionError('$file 第 $position 个步骤少了 name');
    }
    final extra = unknownFields(step, stepFields);
    if (extra.isNotEmpty) {
      throw DefinitionError(
        '$file 第 $position 个步骤有不认识的字段：${extra.join('、')}（只认 ${stepFields.join('、')}）',
      );
    }
    var executor = textOf(step, 'executor');
    if (executor.isEmpty) executor = agent;
    if (!executors.contains(executor)) {
      throw DefinitionError(
        '$file 第 $position 个步骤的 executor 只能是 ${executors.join(' 或 ')}，实得 $executor',
      );
    }
    final rawCriteria = step['criteria'];
    final List criteria;
    if (rawCriteria == null) {
      criteria = const [];
    } else if (rawCriteria is List) {
      criteria = rawCriteria;
    } else {
      throw DefinitionError('$file 第 $position 个步骤的 criteria 应当是列表');
    }
    for (var order = 0; order < criteria.length; order++) {
      final place = '第 $position 个步骤第 ${order + 1} 条判据';
      final criterion = criteria[order];
      final kind = textOf(criterion, 'executor');
      if (!criterionTypes.contains(kind)) {
        throw DefinitionError(
          '$file $place的 executor 只能是 ${criterionTypes.join(' / ')}（谁判：规则引擎 / 智能体 / 人）',
        );
      }
      if (criterion is! Map) {
        throw DefinitionError('$file $place不是映射');
      }
      final odd = unknownFields(criterion, criterionFields);
      if (odd.isNotEmpty) {
        throw DefinitionError(
          '$file $place有不认识的字段：${odd.join('、')}（只认 ${criterionFields.join('、')}）',
        );
      }
      final given = [
        'path',
        'absent',
        'file',
        'contains',
        'run',
      ].where((name) => criterion[name] != null).toList();
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
        if (others.length > 1 ||
            (others.isNotEmpty && given.contains('file'))) {
          throw DefinitionError(
            '$file $place的判法只能一种：path / absent / file+contains / run',
          );
        }
      } else {
        if (textOf(criterion, 'description').isEmpty) {
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
    }
  }
}

/// 一个工作步骤：叫什么、做什么、谁执行、怎么算完。
class Step {
  Step(this.payload);

  final Map payload;

  String get name => textOf(payload, 'name');

  String get description => textOf(payload, 'description');

  String get executor {
    final value = textOf(payload, 'executor');
    return value.isEmpty ? agent : value;
  }

  bool get isHuman => executor == human;

  List<Map> get criteria =>
      (payload['criteria'] as List?)?.cast<Map>().toList() ?? const [];

  List<Map> ofKind(String kind) =>
      criteria.where((item) => textOf(item, 'executor') == kind).toList();

  List<Map> get rules => ofKind(rule);
  List<Map> get agents => ofKind(agent);
  List<Map> get gates => ofKind(human);
}

/// 过程的编排定义：一串步骤（不含文件位置——那是各自包的事）。
class Workflow {
  Workflow({required this.name, required this.payload});

  final String name;
  final Map payload;

  String get description => textOf(payload, 'description');

  /// 步骤：按定义里的顺序——这就是「串联」。
  List<Step> get steps =>
      (payload['steps'] as List?)?.map((item) => Step(item as Map)).toList() ??
      const [];

  Step? step(String name) {
    for (final item in steps) {
      if (item.name == name) return item;
    }
    return null;
  }
}

// ---- 定义核对 ----

/// 一条定义核对出来的一件事。
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

/// 核对一条工作流：判据里的路径在不在；描述里提到的报告小节有没有判据覆盖。
///
/// [exists] 由调用方给——工具箱不碰文件系统。
List<Finding> checkWorkflow(
  Workflow flow,
  String data,
  bool Function(String path) exists,
) {
  final found = <Finding>[];
  for (final step in flow.steps) {
    for (final criterion in step.rules) {
      final literal = criterion['path'] ?? criterion['file'];
      if (literal is! String || literal.isEmpty) continue;
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

  final covered = flow.steps
      .expand((step) => step.rules)
      .map((criterion) => criterion['contains'])
      .whereType<String>()
      .toList();

  final mentioned = <String>[];
  for (final step in flow.steps) {
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

/// 核对结果写成人读的一段。
List<String> describeFindings(List<Finding> found) {
  final lines = <String>['核对 ${found.length} 件事'];
  for (final item in found) {
    lines.add('  ${item.ok ? '✓' : '✗'} ${item.where}——${item.what}');
  }
  if (found.isEmpty) {
    lines.add('  （这条定义里没有可核对的路径与小节）');
  }
  return lines;
}

bool allOk(List<Finding> found) => found.every((item) => item.ok);
