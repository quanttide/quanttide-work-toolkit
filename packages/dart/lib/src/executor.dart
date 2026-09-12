/// 执行者：这一步谁做、这一条判据谁判。
///
/// 三个取值由规范定死，不因平台而变。步骤上只能写 [agent] 或 [human]；
/// 判据上还能写 [rule]（机械核对，不用智能体）。
library;

/// 智能体。
const String agent = 'agent';

/// 规则引擎：机械核对。
const String rule = 'rule';

/// 人：只在闸门拍板。
const String human = 'human';

/// 步骤的执行者——能用 AI 都用 AI，人只在闸门。
const List<String> executors = [agent, human];

/// 判据的执行者——规则引擎 / 智能体 / 人。
const List<String> criterionTypes = [rule, agent, human];
