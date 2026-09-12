//! 判据聚合：模型、读法、翻成「要跑什么」。
//!
//! 模型（[`RuleKind`] / [`Criterion`]）在 [`model`]，从定义里读出判据
//! （[`criterion_of`] / [`read_criterion`]）在 [`read`]，翻成要跑的东西
//! （[`RuleItem`] / [`items_of`]）在 [`items`]。端侧只 `use crate::criterion::` 就能用全。

mod items;
mod model;
mod read;

pub use items::*;
pub use model::*;
pub use read::*;
