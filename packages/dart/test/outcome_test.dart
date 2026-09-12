/// 结果信封：四样与编解码。
///
/// 断言对着形状与往返：缺样按空算、`data` 有才写、`dataJson` 有托给原文没托给信封。
library;

import 'package:quanttide_work/quanttide_work.dart';
import 'package:test/test.dart';

void main() {
  test('起一个信封：四样缺省按空算', () {
    final result = Outcome(true);
    expect(result.ok, isTrue);
    expect(result.lines, isEmpty);
    expect(result.columns, isEmpty);
    expect(result.rows, isEmpty);
    expect(result.data, isNull);
    expect(result.toJson(), {
      'ok': true,
      'lines': <String>[],
      'columns': <String>[],
      'rows': <List<String>>[],
    });
  });

  test('不成：只带话', () {
    final failed = Outcome.failed(['未找到：案例']);
    expect(failed.ok, isFalse);
    expect(failed.lines, ['未找到：案例']);
    expect(failed.columns, isEmpty);
    expect(failed.rows, isEmpty);
    expect(failed.data, isNull);
    expect(failed.toJson(), {
      'ok': false,
      'lines': ['未找到：案例'],
      'columns': <String>[],
      'rows': <List<String>>[],
    });
  });

  test('withFirst 添在开头、withData 托上那一栏，都返回自己', () {
    final result = Outcome(true)
      ..withFirst('第二句')
      ..withFirst('第一句')
      ..withData({'payload': 7});

    expect(result.lines, ['第一句', '第二句']);
    expect(result.data, {'payload': 7});
    expect(result.toJson()['data'], {'payload': 7});
    expect(result.dataJson(), {'payload': 7});
  });

  test('dataJson：没托东西就给信封', () {
    final result = Outcome(false, lines: ['没成']);
    expect(result.dataJson(), result.toJson());
  });

  test('fromJson：缺样按空算，元素照原样写成文字', () {
    final result = Outcome.fromJson({'ok': true, 'lines': ['a', 2]});
    expect(result.ok, isTrue);
    expect(result.lines, ['a', '2']);
    expect(result.columns, isEmpty);
    expect(result.rows, isEmpty);
    expect(result.data, isNull);
  });

  test('fromJson：rows 是同一份表，data 原样托着', () {
    final result = Outcome.fromJson({
      'ok': true,
      'lines': ['走过 2 步'],
      'columns': ['步骤', '状态'],
      'rows': [
        ['甲', '走过'],
        ['乙', '没走'],
      ],
      'data': {
        'payload': {'name': 'x'},
      },
    });
    expect(result.columns, ['步骤', '状态']);
    expect(result.rows, [
      ['甲', '走过'],
      ['乙', '没走'],
    ]);
    expect(result.data, {
      'payload': {'name': 'x'},
    });
  });

  test('fromStdout：命令行吐的 JSON 直接装回来', () {
    final result = Outcome.fromStdout(
      '{"ok":false,"lines":["给一条命令"],"columns":[],"rows":[]}',
    );
    expect(result.ok, isFalse);
    expect(result.lines, ['给一条命令']);
    expect(result.toJson(), {
      'ok': false,
      'lines': ['给一条命令'],
      'columns': <String>[],
      'rows': <List<String>>[],
    });
  });
}
