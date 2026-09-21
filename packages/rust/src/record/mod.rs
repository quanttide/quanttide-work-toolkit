//! 工作记录聚合：模型、错误与语法。
//!
//! 账本级的事（账本接口、追加服务、事件负载）按方案逐步落，
//! 见 `docs/dev-guide/record.md`。

mod errors;
mod models;

pub use errors::*;
pub use models::*;
