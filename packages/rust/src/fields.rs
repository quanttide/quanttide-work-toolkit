//! 读字段、挑陌生字段的小工具，以及定义读不通时报的错。
//!
//! 内部用：只认已经解析好的 YAML 值。怎么读写是各自包的事。

use serde_yaml::{Mapping, Value as Yaml};

/// 一份定义（或一件任务）读不通：字段缺了、取值越界、有不认识的字段。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefinitionError(pub String);

impl std::fmt::Display for DefinitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for DefinitionError {}

/// 取一个字符串字段，去掉两侧空白；不是字符串就当没写。
pub fn text_of(value: &Yaml, key: &str) -> String {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string()
}

/// 这次给的字段里，哪些是不认识的。
pub fn unknown_fields(mapping: &Mapping, allowed: &[&str]) -> Vec<String> {
    mapping
        .keys()
        .filter_map(|key| key.as_str())
        .filter(|key| !allowed.contains(key))
        .map(|key| key.to_string())
        .collect()
}
