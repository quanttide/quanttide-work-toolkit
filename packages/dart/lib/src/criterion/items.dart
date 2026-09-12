import 'model.dart';

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
