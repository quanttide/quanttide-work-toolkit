//! 量潮知识工作工具箱（Rust）。
//!
//! 这一份是**不变的核心逻辑**：把知识工作的规范（`docs/specification`）里
//! 不因平台而变的那部分，封成一套可执行的正本——定义的语法与不变量、
//! 判据的取值、任务流水的语义与「走过」的判定、落点与占位的展开。
//!
//! 分成一个一个领域模型（工作流 / 任务 / 结果 / 判据 / 执行者），一个模型一个目录
//! （只有常量或信封的仍单文件）；读定义（`fields`）、错误（`error`）、路径（`paths`）
//! 是横切件，另立中立模块，聚合只向下依赖它们。
//! 各实现（命令行、工作台、后来的语言）向它对齐，不各写一份；说法（拼句、退出码、
//! 路径怎么显示）与文案（提示词）留在各自的平台，不进这里。
//!
//! 收进来的每一样，都要能在规范里找到出处；规范里没有的，先补规范。

/// 这个工具箱管哪个领域。
pub const DOMAIN: &str = "knowledge-work";

/// 版本号，与 Cargo.toml 一致。
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

mod fields;

pub mod criterion;
pub mod error;
pub mod executor;
pub mod outcome;
pub mod paths;
pub mod task;
pub mod workflow;
