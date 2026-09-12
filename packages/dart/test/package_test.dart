import 'dart:io';

import 'package:quanttide_work/quanttide_work.dart';
import 'package:test/test.dart';

void main() {
  test('领域英文名', () {
    expect(domain, 'knowledge-work');
  });

  // 版本常量与清单同源：Dart 没得从 pubspec 编译期注入，常量只能手写一份，
  // 这里对着清单核一遍，错开就红。
  test('包版本与 pubspec 一致', () {
    final declared = File('pubspec.yaml')
        .readAsLinesSync()
        .firstWhere((line) => line.startsWith('version:'))
        .split(':')[1]
        .trim();
    expect(version, declared);
  });
}
