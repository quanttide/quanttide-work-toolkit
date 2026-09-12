/// 产物：名字是它的身份，规格是它凭以通过验收的那组判据。

import 'package:quanttide_work/quanttide_work.dart';
import 'package:test/test.dart';

void main() {
  test('named 只知其名，还没有规格', () {
    const artifact = Artifact.named('report');
    expect(artifact.name, 'report');
    expect(artifact.spec, isEmpty);
  });

  test('of 带上规格', () {
    const criterion = PathExists('artifacts/report/甲.md');
    const artifact = Artifact.of('report', [criterion]);
    expect(artifact.name, 'report');
    expect(artifact.spec, [criterion]);
  });
}
