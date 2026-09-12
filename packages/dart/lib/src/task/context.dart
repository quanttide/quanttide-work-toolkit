import '../workflow/validate.dart';

/// 这次执行自带的运行上下文：工作区根、数据仓、工作流目录。
///
/// 三个字段怎么读、怎么写由各自的包定（工具箱只管托着它们）。
class RunContext {
  const RunContext({this.root = '', this.data = '', this.workflows = ''});

  factory RunContext.of(Map value) => RunContext(
    root: textOf(value, 'root'),
    data: textOf(value, 'data'),
    workflows: textOf(value, 'workflows'),
  );

  final String root;
  final String data;
  final String workflows;

  Map<String, Object?> toMap() => {
    'root': root,
    'data': data,
    'workflows': workflows,
  };
}
