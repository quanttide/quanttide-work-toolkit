import '../executor.dart';
import '../paths.dart';

/// 判据：规则引擎那几种机械核对。
///
/// 判据是**字段**，不是一行小语法：`path` 存在、`absent` 不存在、
/// `file` + `contains` 含这段文字、`run` 这条命令退出码为零。
/// 工具箱只把判据翻成「要跑什么」——真去跑（文件系统、起进程）是各自包的事。
///
/// 读字段、认取值、报错在 `read.dart`（与模型同处）。
/// 判据的种类：四种机械核对。枚举名与 [Criterion] 的四个子类型一致；
/// 线值（[wire]）是定义里的字段名，不随枚举名变。
enum RuleKind {
  pathExists('path'),
  pathAbsent('absent'),
  fileContains('contains'),
  commandRun('run');

  const RuleKind(this.wire);

  /// 线上写法：判据里的字段名。
  final String wire;
}

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

  /// 占位展开：每个字段里的 `{{name}}` 交给 [resolve] 换成哪条路径。
  ///
  /// [resolve] 认不得的名字原样留着。换成哪条路径是场所的事——
  /// 见 `WorkspacePlace.expanded`。
  Criterion expanded(String? Function(String name) resolve) {
    String ex(String value) => replacePlaceholders(value, resolve);
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
