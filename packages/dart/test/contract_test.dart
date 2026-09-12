/// 契约：同一批用例向量（`../contract/*.json`），两侧各跑一遍，结论必须一样。
///
/// 向量是两侧**共用**的同一批文件——契约写在向量里，不写在各自的测试代码里。
library;

import 'dart:convert';
import 'dart:io';

import 'package:quanttide_work/quanttide_work.dart';
import 'package:test/test.dart';

List<(String, Map<String, Object?>)> vectors() {
  final dir = Directory('../../tests/contract');
  expect(dir.existsSync(), isTrue, reason: '找不到 contract 目录');
  final found = dir
      .listSync()
      .whereType<File>()
      .where((file) => file.path.endsWith('.json'))
      .map(
        (file) => (
          file.path.split('/').last,
          jsonDecode(file.readAsStringSync()) as Map<String, Object?>,
        ),
      )
      .toList();
  found.sort((a, b) => a.$1.compareTo(b.$1));
  return found;
}

/// 向量里的步骤名串成一条工作流（判出「走过哪几步」够用）。
Workflow workflowOf(Object? steps) => Workflow(
  name: 'v',
  steps: [for (final name in (steps as List).cast<String>()) Step(name: name)],
);

/// 向量里的流水装成一件任务。
Task taskOf(Object? events) => Task(
  name: 'v',
  workflowName: 'v',
  journal: [
    for (final event in (events as List).cast<Map>()) JournalEvent.of(event),
  ],
);

void main() {
  test('契约：同一批向量，两侧同一个结论', () {
    final all = vectors();
    for (final (name, vector) in all) {
      switch (vector['kind']) {
        case 'check':
          final parsed = Workflow.fromValue(vector['workflow']);
          final context = RunContext(
            data: (vector['context'] as Map)['data'] as String? ?? '',
          );
          final exists = (vector['exists'] as List? ?? const []).cast<String>();
          final got = parsed
              .check(context, (path) => exists.contains(path))
              .map((f) => {'where': f.where, 'what': f.what, 'ok': f.ok})
              .toList();
          expect(got, vector['expect'], reason: '$name：核对回执不一样');
        case 'validate':
          final input = vector['input'];
          final want = vector['expect'] as Map;
          if (want['error'] != null) {
            expect(
              () => Workflow.fromValue(input),
              throwsA(
                isA<DefinitionError>().having(
                  (e) => e.message(vector['file'] as String),
                  'message',
                  want['error'],
                ),
              ),
              reason: '$name：报错文字不一样',
            );
          } else {
            Workflow.fromValue(input);
          }
        case 'items':
          final got = itemsOf(
            (vector['input'] as List).cast<Map>().map(criterionOf),
          )
              .map(
                (item) => {
                  'description': item.description,
                  'kind': item.kind?.wire,
                  'args': item.args,
                },
              )
              .toList();
          expect(got, vector['expect'], reason: '$name：判据翻出来的不一样');
        case 'done':
          final task = taskOf(vector['events']);
          expect(
            task.doneSteps(workflowOf(vector['steps'])),
            vector['expect'],
            reason: '$name：走过哪几步不一样',
          );
        case 'section':
          for (final c in (vector['cases'] as List).cast<Map>()) {
            expect(
              looksLikeSection(c['input'] as String),
              c['expect'],
              reason: '$name：${c['input']} 算不算小节名',
            );
          }
        case 'artifact':
          // 每一格可以自带 root / data（换目录的那几格）；不写就按向量顶层的。
          for (final c in (vector['cases'] as List).cast<Map>()) {
            final context = RunContext(
              root: (c['root'] ?? vector['root']) as String,
              data: (c['data'] ?? vector['data']) as String,
            );
            final task = Task.of({'name': c['name'], 'artifacts': c['artifacts']});
            expect(
              task.artifact('${c['artifact']}', context),
              c['expect'],
              reason: '$name：${c['note']} 落点算得不对',
            );
          }
        case 'outcome':
          for (final c in (vector['cases'] as List).cast<Map>()) {
            expect(
              Outcome.fromJson(
                (c['input'] as Map).cast<String, dynamic>(),
              ).toJson(),
              c['expect'],
              reason: '$name：${c['note']} 编解码不一样',
            );
          }
        case 'expand':
          for (final c in (vector['cases'] as List).cast<Map>()) {
            expect(
              expandPlaceholders(
                c['input'] as String,
                // 每一格可以自带 data（换目录的那几格）；不写就按向量顶层的。
                (c['data'] ?? vector['data']) as String,
              ),
              c['expect'],
              reason: '$name：${c['input']} 展开得不对',
            );
          }
        default:
          fail('$name：不认得的向量类型 ${vector['kind']}');
      }
    }
    expect(all.length, greaterThanOrEqualTo(11), reason: '向量太少');
  });
}
