import '../criterion/criterion.dart';
import '../paths.dart';
import '../workflow/model.dart';
import 'model.dart';

/// 工作区聚合 / 定义核对：声明与判据对不对得上。
///
/// 判据里的路径在不在、描述提到的报告小节有没有判据覆盖。判据里的路径按平台给的
/// 目录基准展开；「在不在」由调用方给——工具箱不碰文件系统。
/// 出处：`docs/specification/process/workflow.md`·定义核对。

/// 定义核对出来的一件事：在哪里、核的是什么、过没过。
///
/// [ok] 为 `null` 表示没核——判据里带的运行时占位要等任务执行时才落，
/// 核对时给一条「未核」的回执，不静默丢掉。
class Finding {
  const Finding({required this.where, required this.what, required this.ok});

  final String where;
  final String what;

  /// 核过的结果；没核给 `null`。
  final bool? ok;
}

/// 路径里带的运行时占位（`{{report}}` 等）；没有给 `null`。
String? runtimePlaceholder(String path) {
  for (final placeholder in const [
    '{{report}}',
    '{{journal}}',
    '{{log}}',
    '{{artifacts}}',
  ]) {
    if (path.contains(placeholder)) return placeholder;
  }
  return null;
}

/// 像不像报告小节的名字：中文短词。版本号写法、占位、路径都不算。
bool looksLikeSection(String name) {
  if (name.isEmpty || name.length > 12) return false;
  return !RegExp(r'[0-9\[\]{}./`<>_-]').hasMatch(name);
}

extension WorkspaceCheck on Workspace {
  /// 核对一条定义：判据里的路径在不在、描述提到的小节有没有判据覆盖。
  ///
  /// [base] 是平台给的目录基准，占位按它展开；[exists] 由调用方给——
  /// 工具箱不碰文件系统。
  List<Finding> check(
    Workflow workflow,
    String base,
    bool Function(String path) exists,
  ) {
    final found = <Finding>[];
    for (final step in workflow.steps) {
      for (final criterion in step.rules) {
        final literal = switch (criterion) {
          PathExists(:final path) => path,
          FileContains(:final file) => file,
          _ => '',
        };
        if (literal.isEmpty) continue;
        final placeholder = runtimePlaceholder(literal);
        if (placeholder != null) {
          found.add(
            Finding(
              where: '${step.name}·$literal',
              what: '判据里的路径含 $placeholder，未核（等任务执行时再核）',
              ok: null,
            ),
          );
          continue;
        }
        final written = expandPlaceholders(literal, base);
        found.add(
          Finding(
            where: '${step.name}·$literal',
            what: '判据里的路径在不在：$written',
            ok: exists(written),
          ),
        );
      }
    }

    final covered = workflow.steps
        .expand((step) => step.rules)
        .whereType<FileContains>()
        .map((criterion) => criterion.contains)
        .toList();

    final mentioned = <String>[];
    for (final step in workflow.steps) {
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
}
