//! 工作流聚合 / 整体校验：字段缺了、取值越界、有不认识的字段，当场报错。
//!
//! 规矩的出处是 `docs/specification/process/workflow.md`·语法。
//! 模型在 [`super::model`]；这里只装「读一份定义」所需的字段表、错误与校验。

use super::model::{Step, Workflow};
use crate::criterion::read_criterion;
use crate::executor::{AGENT, EXECUTORS};
use serde_yaml::{Mapping, Value as Yaml};

/// 定义顶层认得的字段。
const TOP_FIELDS: [&str; 3] = ["name", "description", "steps"];

/// 步骤认得的字段。
const STEP_FIELDS: [&str; 4] = ["name", "description", "executor", "criteria"];

/// 一份定义读不通：字段缺了、取值越界、有不认识的字段。
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

/// 语法校验：不是映射、缺字段、取值不对，当场报错。`file` 只用来说话。
pub fn validate(payload: &Yaml, file: &str) -> Result<(), DefinitionError> {
    let top = payload
        .as_mapping()
        .ok_or_else(|| DefinitionError(format!("{file} 的顶层不是映射（name / steps）")))?;
    if text_of(payload, "name").is_empty() {
        return Err(DefinitionError(format!("{file} 少了 name")));
    }
    let steps = payload
        .get("steps")
        .and_then(|v| v.as_sequence())
        .filter(|items| !items.is_empty())
        .ok_or_else(|| DefinitionError(format!("{file} 少了 steps（至少一个步骤）")))?;
    let unknown = unknown_fields(top, &TOP_FIELDS);
    if !unknown.is_empty() {
        return Err(DefinitionError(format!(
            "{file} 顶层有不认识的字段：{}（只认 {}）",
            unknown.join("、"),
            TOP_FIELDS.join("、")
        )));
    }
    for (index, step) in steps.iter().enumerate() {
        validate_step(step, file, index + 1)?;
    }
    Ok(())
}

/// 一个步骤的语法校验。`file` 与 `position` 只用来说话。
fn validate_step(value: &Yaml, file: &str, position: usize) -> Result<(), DefinitionError> {
    let step_map = value
        .as_mapping()
        .ok_or_else(|| DefinitionError(format!("{file} 第 {position} 个步骤少了 name")))?;
    if text_of(value, "name").is_empty() {
        return Err(DefinitionError(format!(
            "{file} 第 {position} 个步骤少了 name"
        )));
    }
    let extra = unknown_fields(step_map, &STEP_FIELDS);
    if !extra.is_empty() {
        return Err(DefinitionError(format!(
            "{file} 第 {position} 个步骤有不认识的字段：{}（只认 {}）",
            extra.join("、"),
            STEP_FIELDS.join("、")
        )));
    }
    let mut executor = text_of(value, "executor");
    if executor.is_empty() {
        executor = AGENT.to_string();
    }
    if !EXECUTORS.contains(&executor.as_str()) {
        return Err(DefinitionError(format!(
            "{file} 第 {position} 个步骤的 executor 只能是 {}，实得 {executor}",
            EXECUTORS.join(" 或 ")
        )));
    }
    let criteria: &[Yaml] = match value.get("criteria") {
        None | Some(Yaml::Null) => &[],
        Some(Yaml::Sequence(items)) => items.as_slice(),
        Some(_) => {
            return Err(DefinitionError(format!(
                "{file} 第 {position} 个步骤的 criteria 应当是列表"
            )));
        }
    };
    for (order, criterion) in criteria.iter().enumerate() {
        let place = format!("第 {position} 个步骤第 {} 条判据", order + 1);
        read_criterion(criterion, file, &place)?;
    }
    Ok(())
}

impl Workflow {
    /// 从定义里的字段读出，顺带把语法过一遍。
    pub fn from_value(payload: &Yaml, file: &str) -> Result<Workflow, DefinitionError> {
        validate(payload, file)?;
        Ok(Workflow::of(&text_of(payload, "name"), payload))
    }
}

impl Step {
    /// 从定义里的字段读出，顺带把语法过一遍。
    pub fn from_value(value: &Yaml, file: &str, position: usize) -> Result<Step, DefinitionError> {
        validate_step(value, file, position)?;
        Ok(Step::of(value))
    }
}
