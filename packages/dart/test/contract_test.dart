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

void main() {
  test('契约：同一批向量，两侧同一个结论', () {
    final all = vectors();
    for (final (name, vector) in all) {
      switch (vector['kind']) {
        case 'validate':
          final input = vector['input'];
          final want = vector['expect'] as Map;
          if (want['error'] != null) {
            expect(
              () => validateDefinition(input, vector['file'] as String),
              throwsA(
                isA<DefinitionError>().having(
                  (e) => e.message,
                  'message',
                  want['error'],
                ),
              ),
              reason: '$name：报错文字不一样',
            );
          } else {
            validateDefinition(input, vector['file'] as String);
          }
        case 'items':
          final got = itemsOf(
            (vector['input'] as List).cast<Map>().map(Criterion.fromMap),
          )
              .map(
                (item) => {
                  'description': item.description,
                  'kind': item.kind?.name,
                  'args': item.args,
                },
              )
              .toList();
          expect(got, vector['expect'], reason: '$name：判据翻出来的不一样');
        case 'done':
          final steps = (vector['steps'] as List).cast<String>();
          final events = (vector['events'] as List).cast<Map>();
          expect(
            done(steps, events),
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
        case 'expand':
          for (final c in (vector['cases'] as List).cast<Map>()) {
            expect(
              expandPlaceholders(
                c['input'] as String,
                vector['data'] as String,
              ),
              c['expect'],
              reason: '$name：${c['input']} 展开得不对',
            );
          }
        case 'envelope':
          final input = vector['input'] as Map;
          final outcome = Outcome(
            input['ok'] as bool,
            lines: (input['lines'] as List).cast<String>(),
            columns: (input['columns'] as List).cast<String>(),
            rows: (input['rows'] as List)
                .map((r) => (r as List).cast<String>())
                .toList(),
          );
          expect(
            outcome.toJson(),
            vector['expect'],
            reason: '$name：信封的 JSON 不一样',
          );
        default:
          fail('$name：不认得的向量类型 ${vector['kind']}');
      }
    }
    expect(all.length, greaterThanOrEqualTo(10), reason: '向量太少');
  });
}
