import 'criteria.dart';

/// 给智能体的两段话：走一步要它做什么、审一遍要它按什么判。
///
/// 这两段话是产品的一部分——说什么、不说什么是定死的，所以抽出来两侧共用。

/// 这一步的判据清单：每条一行「谁判：说明」。
String criteriaText(Iterable<Criterion> criteria) {
  final lines = criteria
      .map((criterion) => '- ${criterion.executor}：${criterion.text}')
      .toList();
  return lines.isEmpty ? '（这一步没有判据）' : lines.join('\n');
}

/// 走一步那件事的现场：任务与这一步的已知事实（路径由各自包算好递进来）。
class Facts {
  const Facts({
    required this.root,
    required this.data,
    required this.name,
    required this.start,
    required this.workflowName,
    required this.workflowDescription,
    required this.steps,
    required this.step,
    required this.what,
    required this.report,
    required this.journal,
    required this.log,
    required this.artifacts,
  });

  final String root;
  final String data;
  final String name;
  final String start;
  final String workflowName;
  final String workflowDescription;
  final String steps;
  final String step;
  final String what;
  final String report;
  final String journal;
  final String log;
  final String artifacts;
}

/// 交给 AI 的那一段话。
String promptFor(Facts facts, Iterable<Criterion> criteria) =>
    '你在按一条工作流走一步。只做这一步，做完就停。\n\n'
    '工作区：${facts.root}\n'
    '数据仓：${facts.data}\n'
    '任务：${facts.name}（开工：${facts.start}）\n'
    '工作流：${facts.workflowName}——${facts.workflowDescription}\n'
    '步骤：${facts.steps}\n'
    '这一步：${facts.step}\n'
    '做什么：\n${facts.what}\n\n'
    '判据（程序随后自己核对，你不能改判据、也不许改判据文件）：\n${criteriaText(criteria)}\n\n'
    '本任务的三样东西（报告与日志是产物，流水是执行痕迹）：\n'
    '  产物：${facts.report}（程序不碰产物内容，谁写谁定；闸门项记在任务文件里）\n'
    '  日志：${facts.journal}\n'
    '  流水：${facts.log}（就在任务文件里）\n'
    '工作流里用 {{report}} / {{journal}} / {{log}} 指这三样；工作内容写进报告，别动程序那两节。\n'
    '规矩：数据只写数据仓；工作区里只动「做什么」点名的东西。最后用一句话说明你做了什么。\n';

/// 交给智能体审的那一段话：产物 + 判准，逐条回答。
String judgePrompt(Facts facts, List<Criterion> criteria) {
  final listed = <String>[];
  for (var i = 0; i < criteria.length; i++) {
    listed.add('${i + 1}. ${criteria[i].text}');
  }
  return '你是审查者，不是执行者。别改产物、别改判据文件。\n\n'
      '工作区：${facts.root}\n'
      '要审的东西：这一步的产物在 ${facts.artifacts}（也可以看工作区里相关文件）\n'
      '这一步做什么：${facts.what}\n\n'
      '判准（逐条判）：\n${listed.join('\n')}\n\n'
      '对每条输出一行，格式只能是「序号. 通过 — 一句话理由」或「序号. 不通过 — 一句话理由」，最后不要写别的。\n';
}
