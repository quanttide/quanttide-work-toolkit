//! 工作流聚合 / 整体校验：字段缺了、取值越界、有不认识的字段，当场报错。
//!
//! 规矩的出处是 `docs/specification/process/workflow.md`·语法。
//! 模型在 [`super::model`]；字段表与取值助手在 `crate::fields`，错误在 `crate::error`。

use super::model::{Step, Workflow};
use crate::criterion::read_criterion;
use crate::error::DefinitionError;
use crate::executor::{AGENT, EXECUTORS};
use crate::fields::{STEP_FIELDS, TOP_FIELDS, text_of, unknown_fields};
use serde_yaml::Value as Yaml;

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
