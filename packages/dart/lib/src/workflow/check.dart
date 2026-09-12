import '../criterion/criterion.dart';
import '../paths.dart';
import 'model.dart';

/// 定义核对出来的一件事：在哪里、核的是什么、过没过。
///
/// 它是「把这条定义对着工作区核一遍」的回执——`workflow --check` 的实现产物，
/// 规范里还没有这一节。
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

extension WorkflowCheck on Workflow {
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
}
