/// 量潮知识工作工具箱 Dart 包。
///
/// 这一份是**不变的核心逻辑**：把知识工作的规范（`docs/specification`）里
/// 不因平台而变的那部分，封成一套可执行的正本——定义的语法与不变量、
/// 判据的取值、任务流水的语义与「走过」的判定、落点与占位的展开。
///
/// 分成一个一个领域模型（工作流 / 任务 / 结果 / 判据 / 执行者），一个模型一个目录
/// （只有常量或信封的仍单文件）。目录与文件怎么分见 `lib/CONVENTIONS.md`。
/// 各实现（命令行、工作台、后来的语言）向它对齐，不各写一份；说法（拼句、退出码、
/// 路径怎么显示）与文案（提示词）留在各自的平台，不进这里。
///
/// 收进来的每一样，都要能在规范里找到出处；规范里没有的，先补规范。
library;

export 'src/criterion/criterion.dart';
export 'src/error.dart';
export 'src/executor.dart';
export 'src/outcome.dart';
export 'src/paths.dart';
export 'src/task/task.dart';
export 'src/workflow/workflow.dart';

/// 领域英文名。
const String domain = 'knowledge-work';

/// 包版本（与 pubspec.yaml 保持一致）。
const String version = '0.1.0-beta.5';
