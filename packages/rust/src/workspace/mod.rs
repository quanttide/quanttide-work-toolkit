//! 工作区聚合：一次工作的边界，把定义与任务系在一起。
//!
//! 模型（[`Workspace`]）在 [`model`]；跨着定义与现场的操作分在三处——
//! 定义核对（[`Workspace::check`]）在 [`check`]，落点（[`Workspace::place`]）与
//! 占位展开（[`Workspace::expanded`]）在 [`place`]，流水判定（[`Workspace::done_steps`] 等）在 [`progress`]。
//! 工作区只装内容，不装位置；落点只算相对工作区根的路径，拼目录是平台的事。

mod check;
mod model;
mod place;
mod progress;

pub use check::{Finding, looks_like_section};
pub use model::Workspace;
