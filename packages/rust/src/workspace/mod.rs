//! 工作区聚合：一次工作的边界，把定义与任务系在一起。
//!
//! 模型（[`Workspace`]）在 [`model`]；跨着定义与现场的操作分在三处——
//! 定义核对（[`Workspace::check`]）在 [`check`]，落点（[`Workspace::artifact`]）在
//! [`artifact`]，流水判定（[`Workspace::done_steps`] 等）在 [`progress`]。
//! 工作区只装内容，不装位置；目录基准由平台当参数给。

mod artifact;
mod check;
mod model;
mod progress;

pub use check::{Finding, looks_like_section};
pub use model::Workspace;
