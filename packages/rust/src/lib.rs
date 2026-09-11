//! 量潮知识工作工具箱（Rust）。
//!
//! 知识工作领域里，命令行与 studio 两侧**意义相同**的那几样东西抽在这里：
//! 信封、工作流定义（字段表、校验、视图、核对）、机械判据、任务流水与「走过」的算法、
//! 给智能体的两段话。
//!
//! 抽的是**纯逻辑**：从已经解析好的 JSON 值进、从值出。文件读写、YAML 解析与序列化、
//! 起进程，各语言各自的库去管——那些不进工具箱。

/// 这个工具箱管哪个领域。
pub const DOMAIN: &str = "knowledge-work";

/// 版本号，与 Cargo.toml 一致。
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod criteria;
pub mod definition;
pub mod envelope;
pub mod prompts;
pub mod tasklog;
