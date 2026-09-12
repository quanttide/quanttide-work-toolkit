import 'package:quanttide_work/quanttide_work.dart';
import 'package:test/test.dart';

void main() {
  test('领域英文名', () {
    expect(domain, 'knowledge-work');
  });

  test('包版本与 pubspec 一致', () {
    expect(version, '0.1.0-beta.5');
  });
}
