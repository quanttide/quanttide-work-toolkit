//! 定义的 schema：字段名、取值、判据种类，以及读字段、挑陌生字段的小工具。
//!
//! 定义要有固定的意义，所以这些都定死。这一层不认模型，只认已经解析好的 YAML 值。

use serde_yaml::{Mapping, Value as Yaml};

pub const AGENT: &str = "agent";
pub const HUMAN: &str = "human";
pub const RULE: &str = "rule";
pub const EXECUTORS: [&str; 2] = [AGENT, HUMAN];
pub const TYPES: [&str; 3] = [RULE, AGENT, HUMAN];
pub const TOP_FIELDS: [&str; 3] = ["name", "description", "steps"];
pub const STEP_FIELDS: [&str; 4] = ["name", "description", "executor", "criteria"];
pub const CRITERION_FIELDS: [&str; 7] = [
    "executor",
    "description",
    "path",
    "absent",
    "file",
    "contains",
    "run",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefinitionError(pub String);

impl std::fmt::Display for DefinitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for DefinitionError {}

pub fn text_of(value: &Yaml, key: &str) -> String {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string()
}

pub fn unknown_fields(mapping: &Mapping, allowed: &[&str]) -> Vec<String> {
    mapping
        .keys()
        .filter_map(|key| key.as_str())
        .filter(|key| !allowed.contains(key))
        .map(|key| key.to_string())
        .collect()
}
