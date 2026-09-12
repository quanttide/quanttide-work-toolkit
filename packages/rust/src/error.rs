//! 定义读不通时的错误。
//!
//! 中立处：不寄居任何聚合，`criterion` / `workflow` 都向它对齐，免得依赖成环。

/// 一份定义读不通：字段缺了、取值越界、有不认识的字段。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefinitionError(pub String);

impl std::fmt::Display for DefinitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for DefinitionError {}
