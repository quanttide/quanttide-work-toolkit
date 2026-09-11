import 'definition.dart';

/// 判据：规则引擎那几种机械核对。
///
/// 判据是**字段**，不是一行小语法：`path` 存在、`absent` 不存在、
/// `file` + `contains` 含这段文字、`run` 这条命令退出码为零。
/// 工具箱只把判据翻成「要跑什么」——真去跑（文件系统、起进程）是各自包的事。
enum RuleKind { path, absent, contains, run }

/// 一条要跑的判据：说明 + 怎么判（[kind] 为空即不跑，交给智能体或人）。
class RuleItem {
  RuleItem(this.description, {this.kind, this.args = const []});

  final String description;
  final RuleKind? kind;
  final List<String> args;

  bool get machine => kind != null;
}

String _text(Map criterion, String key) {
  final value = criterion[key];
  return value is String ? value : '';
}

/// 说明：写了就用写的，没写按字段拼一句。
String descriptionOf(Map criterion) {
  final written = _text(criterion, 'description');
  if (written.trim().isNotEmpty) return written.trim();
  final path = _text(criterion, 'path');
  if (path.isNotEmpty) return '存在：$path';
  final absent = _text(criterion, 'absent');
  if (absent.isNotEmpty) return '不存在：$absent';
  final file = _text(criterion, 'file');
  if (file.isNotEmpty) return '含「${_text(criterion, 'contains')}」：$file';
  final run = _text(criterion, 'run');
  if (run.isNotEmpty) return '跑通：$run';
  return '';
}

/// 把定义里的判据翻成要跑的东西：rule 的跑，agent / human 的不跑。
List<RuleItem> itemsOf(List<Map> criteria) {
  final items = <RuleItem>[];
  for (final criterion in criteria) {
    final description = descriptionOf(criterion);
    final isRule = _text(criterion, 'executor') == rule;
    if (!isRule) {
      items.add(RuleItem(description));
      continue;
    }
    final path = _text(criterion, 'path');
    final absent = _text(criterion, 'absent');
    final file = _text(criterion, 'file');
    final run = _text(criterion, 'run');
    if (path.isNotEmpty) {
      items.add(RuleItem(description, kind: RuleKind.path, args: [path]));
    } else if (absent.isNotEmpty) {
      items.add(RuleItem(description, kind: RuleKind.absent, args: [absent]));
    } else if (file.isNotEmpty) {
      items.add(
        RuleItem(
          description,
          kind: RuleKind.contains,
          args: [file, _text(criterion, 'contains')],
        ),
      );
    } else if (run.isNotEmpty) {
      items.add(RuleItem(description, kind: RuleKind.run, args: [run]));
    }
  }
  return items;
}
