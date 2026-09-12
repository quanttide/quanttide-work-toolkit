/// 工作区聚合：一次工作的边界，把定义与任务系在一起。
///
/// 模型（`Workspace`）在 `model.dart`；跨着定义与现场的操作分在三处——
/// 定义核对（`WorkspaceCheck.check`）在 `check.dart`，落点（`WorkspaceArtifact.artifact`）在
/// `artifact.dart`，流水判定（`WorkspaceProgress.doneSteps` 等）在 `progress.dart`。
/// 工作区只装内容，不装位置；目录基准由平台当参数给。
library;

export 'artifact.dart';
export 'check.dart';
export 'model.dart';
export 'progress.dart';
