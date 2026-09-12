//! 工作流聚合 / 整体校验：字段缺了、取值越界、有不认识的字段，当场报错。
//!
//! 规矩的出处是 `docs/specification/process/workflow.md`·语法。
//! 模型在 [`super::model`]；字段表与取值助手在 `crate::fields`，错误在 `crate::error`。

use super::model::{Step, Workflow};
use crate::criterion::read_criterion;
use crate::error::{DefinitionError, Fault, Position};
use crate::executor::{AGENT, EXECUTORS};
use crate::fields::{STEP_FIELDS, TOP_FIELDS, text_of, unknown_fields};
use serde_yaml::Value as Yaml;

/// 语法校验：不是映射、缺字段、取值不对，当场报错。
pub fn validate(payload: &Yaml) -> Result<(), DefinitionError> {
    let top = payload
        .as_mapping()
        .ok_or_else(|| DefinitionError::new(Position::Top, Fault::TopNotMapping))?;
    if text_of(payload, "name").is_empty() {
        return Err(DefinitionError::new(Position::Top, Fault::MissingName));
    }
    let steps = payload
        .get("steps")
        .and_then(|v| v.as_sequence())
        .filter(|items| !items.is_empty())
        .ok_or_else(|| DefinitionError::new(Position::Top, Fault::MissingSteps))?;
    let unknown = unknown_fields(top, &TOP_FIELDS);
    if !unknown.is_empty() {
        return Err(DefinitionError::new(
            Position::Top,
            Fault::UnknownTopFields(unknown),
        ));
    }
    for (index, step) in steps.iter().enumerate() {
        validate_step(step, index + 1)?;
    }
    Ok(())
}

/// 一个步骤的语法校验。`position` 只用来说话。
fn validate_step(value: &Yaml, position: usize) -> Result<(), DefinitionError> {
    let at = || Position::Step(position);
    let step_map = value
        .as_mapping()
        .ok_or_else(|| DefinitionError::new(at(), Fault::MissingStepName))?;
    if text_of(value, "name").is_empty() {
        return Err(DefinitionError::new(at(), Fault::MissingStepName));
    }
    let extra = unknown_fields(step_map, &STEP_FIELDS);
    if !extra.is_empty() {
        return Err(DefinitionError::new(at(), Fault::UnknownStepFields(extra)));
    }
    let mut executor = text_of(value, "executor");
    if executor.is_empty() {
        executor = AGENT.to_string();
    }
    if !EXECUTORS.contains(&executor.as_str()) {
        return Err(DefinitionError::new(
            at(),
            Fault::BadStepExecutor { got: executor },
        ));
    }
    let criteria: &[Yaml] = match value.get("criteria") {
        None | Some(Yaml::Null) => &[],
        Some(Yaml::Sequence(items)) => items.as_slice(),
        Some(_) => return Err(DefinitionError::new(at(), Fault::CriteriaNotList)),
    };
    for (order, criterion) in criteria.iter().enumerate() {
        read_criterion(criterion, position, order + 1)?;
    }
    Ok(())
}

impl Workflow {
    /// 从定义里的字段读出，顺带把语法过一遍。
    pub fn from_value(payload: &Yaml) -> Result<Workflow, DefinitionError> {
        validate(payload)?;
        Ok(Workflow::of(payload))
    }
}

impl Step {
    /// 从定义里的字段读出，顺带把语法过一遍；`position` 是它在定义里的序号（从 1 数）。
    pub fn from_value(value: &Yaml, position: usize) -> Result<Step, DefinitionError> {
        validate_step(value, position)?;
        Ok(Step::of(value))
    }
}
