import 'schema.dart';

/// 判据：规则引擎那几种机械核对。
///
/// 判据是**字段**，不是一行小语法：`path` 存在、`absent` 不存在、
/// `file` + `contains` 含这段文字、`run` 这条命令退出码为零。
/// 工具箱只把判据翻成「要跑什么」——真去跑（文件系统、起进程）是各自包的事。
enum RuleKind { path, absent, contains, run }

/// 一条判据：谁判、怎么判、说明。
///
/// 值对象，不可变。`rule` 四种判法各一个子类型；`agent` / `human` 只有说明。
/// 线上形状不变——进出定义文件时仍写成 `path` / `absent` / `file`+`contains` / `run` 字段。
sealed class Criterion {
  const Criterion({this.description = ''});

  /// 写了的说明；空即按判法拼一句。
  final String description;

  /// 谁判：`rule` / `agent` / `human`。
  String get executor;

  /// 人读的说明：写了就用写的，没写按判法拼。
  String get text;

  /// 写回定义里的字段形状。
  Map<String, Object?> toMap();

  /// 占位展开：每个字段里的 `{{…}}` 交给 [expand] 换掉。
  Criterion expanded(String Function(String) expand) {
    String ex(String value) => value.contains('{{') ? expand(value) : value;
    return switch (this) {
      PathExists(:final path, :final description) => PathExists(
        ex(path),
        description: ex(description),
      ),
      PathAbsent(:final absent, :final description) => PathAbsent(
        ex(absent),
        description: ex(description),
      ),
      FileContains(:final file, :final contains, :final description) =>
        FileContains(ex(file), ex(contains), description: ex(description)),
      CommandRun(:final run, :final description) => CommandRun(
        ex(run),
        description: ex(description),
      ),
      AgentJudgement(:final description) => AgentJudgement(ex(description)),
      HumanGate(:final description) => HumanGate(ex(description)),
    };
  }

  /// 从定义里的字段读出。字段已经校验过——这里只管认。
  static Criterion fromMap(Map map) {
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
      return FileContains(
        file,
        textOf(map, 'contains'),
        description: description,
      );
    }
    return CommandRun(textOf(map, 'run'), description: description);
  }
}

/// 读一条判据：不是映射、取值不对、缺该有的字段，当场报错。
///
/// `file` 与 `place` 只用来说话；返回的是认好的值对象。
Criterion readCriterion(
  Object? value, {
  required String file,
  required String place,
}) {
  final kind = textOf(value, 'executor');
  if (!criterionTypes.contains(kind)) {
    throw DefinitionError(
      '$file $place的 executor 只能是 ${criterionTypes.join(' / ')}（谁判：规则引擎 / 智能体 / 人）',
    );
  }
  if (value is! Map) {
    throw DefinitionError('$file $place不是映射');
  }
  final odd = unknownFields(value, criterionFields);
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
  return Criterion.fromMap(value);
}

/// `path` 存在。
class PathExists extends Criterion {
  const PathExists(this.path, {super.description});

  final String path;

  @override
  String get executor => rule;

  @override
  String get text => description.isNotEmpty ? description : '存在：$path';

  @override
  Map<String, Object?> toMap() => {
    'executor': rule,
    'path': path,
    if (description.isNotEmpty) 'description': description,
  };
}

/// `absent` 不存在。
class PathAbsent extends Criterion {
  const PathAbsent(this.absent, {super.description});

  final String absent;

  @override
  String get executor => rule;

  @override
  String get text => description.isNotEmpty ? description : '不存在：$absent';

  @override
  Map<String, Object?> toMap() => {
    'executor': rule,
    'absent': absent,
    if (description.isNotEmpty) 'description': description,
  };
}

/// `file` 含 `contains` 这段文字。
class FileContains extends Criterion {
  const FileContains(this.file, this.contains, {super.description});

  final String file;
  final String contains;

  @override
  String get executor => rule;

  @override
  String get text => description.isNotEmpty ? description : '含「$contains」：$file';

  @override
  Map<String, Object?> toMap() => {
    'executor': rule,
    'file': file,
    'contains': contains,
    if (description.isNotEmpty) 'description': description,
  };
}

/// `run` 这条命令退出码为零。
class CommandRun extends Criterion {
  const CommandRun(this.run, {super.description});

  final String run;

  @override
  String get executor => rule;

  @override
  String get text => description.isNotEmpty ? description : '跑通：$run';

  @override
  Map<String, Object?> toMap() => {
    'executor': rule,
    'run': run,
    if (description.isNotEmpty) 'description': description,
  };
}

/// 交给智能体审的判准。
class AgentJudgement extends Criterion {
  const AgentJudgement(String description) : super(description: description);

  @override
  String get executor => agent;

  @override
  String get text => description;

  @override
  Map<String, Object?> toMap() => {
    'executor': agent,
    'description': description,
  };
}

/// 留给人拍板的事项。
class HumanGate extends Criterion {
  const HumanGate(String description) : super(description: description);

  @override
  String get executor => human;

  @override
  String get text => description;

  @override
  Map<String, Object?> toMap() => {
    'executor': human,
    'description': description,
  };
}

/// 一条要跑的判据：说明 + 怎么判（[kind] 为空即不跑，交给智能体或人）。
class RuleItem {
  RuleItem(this.description, {this.kind, this.args = const []});

  final String description;
  final RuleKind? kind;
  final List<String> args;

  bool get machine => kind != null;
}

/// 把判据翻成要跑的东西：rule 的跑，agent / human 的不跑。
List<RuleItem> itemsOf(Iterable<Criterion> criteria) {
  final items = <RuleItem>[];
  for (final criterion in criteria) {
    final description = criterion.text;
    switch (criterion) {
      case PathExists(:final path):
        items.add(RuleItem(description, kind: RuleKind.path, args: [path]));
      case PathAbsent(:final absent):
        items.add(RuleItem(description, kind: RuleKind.absent, args: [absent]));
      case FileContains(:final file, :final contains):
        items.add(
          RuleItem(
            description,
            kind: RuleKind.contains,
            args: [file, contains],
          ),
        );
      case CommandRun(:final run):
        items.add(RuleItem(description, kind: RuleKind.run, args: [run]));
      case AgentJudgement() || HumanGate():
        items.add(RuleItem(description));
    }
  }
  return items;
}
