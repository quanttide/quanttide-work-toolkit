//! 执行者：这一步谁做、这一条判据谁判。
//!
//! 三个取值由规范定死，不因平台而变。步骤上只能写 [`AGENT`] 或 [`HUMAN`]；
//! 判据上还能写 [`RULE`]（机械核对，不用智能体）。
//! 出处：`docs/specification/process/workflow.md`·语法（executor 取值）。

/// 智能体。
pub const AGENT: &str = "agent";

/// 规则引擎：机械核对。
pub const RULE: &str = "rule";

/// 人：只在闸门拍板。
pub const HUMAN: &str = "human";

/// 步骤的执行者——能用 AI 都用 AI，人只在闸门。
pub const EXECUTORS: [&str; 2] = [AGENT, HUMAN];

/// 判据的执行者——规则引擎 / 智能体 / 人。
pub const CRITERION_TYPES: [&str; 3] = [RULE, AGENT, HUMAN];
