import '../criterion/model.dart';

/// 产物聚合 / 模型：一件产出物的名字与规格。
///
/// 名字是它的身份，产物不限种类——报告、日志只是两个名字。规格是一组验收判据，
/// 产物的质量由规格控制：判据核过了，产物才算通过验收。
/// 产物不带位置——位置不进模型，它落在哪由工作区按名字算（`WorkspacePlace.place`）。
/// 出处：`docs/specification/piece/artifact.md`。
class Artifact {
  const Artifact({required this.name, this.spec = const []});

  /// 只知其名：还没有规格的产物。
  const Artifact.named(String name) : this(name: name);

  /// 名字加规格。
  const Artifact.of(String name, List<Criterion> spec)
    : this(name: name, spec: spec);

  /// 产物的名字，也是它的身份。
  final String name;

  /// 规格：验收这组判据。
  final List<Criterion> spec;
}
