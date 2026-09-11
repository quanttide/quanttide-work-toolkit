/// 量潮知识工作工具箱 Dart 包。
///
/// 知识工作领域里，命令行与 studio 两侧**意义相同**的那几样东西抽在这里：
/// 信封、工作流定义（字段表、校验、视图、核对）、机械判据、任务流水与「走过」的算法、
/// 给智能体的两段话。
///
/// 抽的是**纯逻辑**：从已经解析好的 Map / List 进、从值出。文件读写、YAML 解析与序列化、
/// 起进程，各语言各自的库去管——那些不进工具箱。
library;

export 'src/criteria.dart';
export 'src/definition.dart';
export 'src/envelope.dart';
export 'src/prompts.dart';
export 'src/tasklog.dart' hide textOf;

/// 领域英文名。
const String domain = 'knowledge-work';

/// 包版本（与 pubspec.yaml 保持一致）。
const String version = '0.1.0-alpha.5';
