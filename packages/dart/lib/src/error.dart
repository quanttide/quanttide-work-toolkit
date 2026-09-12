/// 定义读不通时的错误。
///
/// 中立处：不寄居任何聚合，`criterion` / `workflow` 都向它对齐，免得依赖成环。
library;

/// 一份定义读不通：字段缺了、取值越界、有不认识的字段。
class DefinitionError implements Exception {
  DefinitionError(this.message);

  final String message;

  @override
  String toString() => message;
}
