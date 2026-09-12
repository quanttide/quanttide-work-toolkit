//! 判据的读法：从定义里的字段读出 [`Criterion`]，顺带把语法过一遍。
//!
//! 判据是**字段**，不是一行小语法：`path` 存在、`absent` 不存在、
//! `file` + `contains` 含这段文字、`run` 这条命令退出码为零。字段名、取值、
//! 不认识、缺了、越界，当场报错。规矩的出处是 `docs/specification/process/workflow.md`·语法。

use super::model::Criterion;
use crate::executor::{AGENT, CRITERION_TYPES, HUMAN, RULE};
use crate::workflow::{DefinitionError, text_of, unknown_fields};
use serde_yaml::Value as Yaml;

/// 一条判据认得的字段。
const CRITERION_FIELDS: [&str; 7] = [
    "executor",
    "description",
    "path",
    "absent",
    "file",
    "contains",
    "run",
];

/// 从定义里的字段认出一条判据（不校验）。
pub fn criterion_of(value: &Yaml) -> Criterion {
    let description = text_of(value, "description");
    let kind = text_of(value, "executor");
    if kind == AGENT {
        return Criterion::AgentJudgement { description };
    }
    if kind == HUMAN {
        return Criterion::HumanGate { description };
    }
    let path = text_of(value, "path");
    if !path.is_empty() {
        return Criterion::PathExists { path, description };
    }
    let absent = text_of(value, "absent");
    if !absent.is_empty() {
        return Criterion::PathAbsent {
            absent,
            description,
        };
    }
    let file = text_of(value, "file");
    if !file.is_empty() {
        return Criterion::FileContains {
            file,
            contains: text_of(value, "contains"),
            description,
        };
    }
    Criterion::CommandRun {
        run: text_of(value, "run"),
        description,
    }
}

/// 读一条判据：取值不对、缺该有的字段，当场报错。
///
/// `file` 与 `place` 只用来说话；返回的是认好的值对象。
pub fn read_criterion(value: &Yaml, file: &str, place: &str) -> Result<Criterion, DefinitionError> {
    let kind = text_of(value, "executor");
    if !CRITERION_TYPES.contains(&kind.as_str()) {
        return Err(DefinitionError(format!(
            "{file} {place}的 executor 只能是 {}（谁判：规则引擎 / 智能体 / 人）",
            CRITERION_TYPES.join(" / ")
        )));
    }
    let criterion_map = value
        .as_mapping()
        .ok_or_else(|| DefinitionError(format!("{file} {place}不是映射")))?;
    let odd = unknown_fields(criterion_map, &CRITERION_FIELDS);
    if !odd.is_empty() {
        return Err(DefinitionError(format!(
            "{file} {place}有不认识的字段：{}（只认 {}）",
            odd.join("、"),
            CRITERION_FIELDS.join("、")
        )));
    }
    let given: Vec<&str> = ["path", "absent", "file", "contains", "run"]
        .into_iter()
        .filter(|name| value.get(*name).is_some())
        .collect();
    if kind == RULE {
        if given.is_empty() {
            return Err(DefinitionError(format!(
                "{file} {place}是 rule，得写一条判法（path / absent / file+contains / run）"
            )));
        }
        if given.contains(&"contains") && !given.contains(&"file") {
            return Err(DefinitionError(format!(
                "{file} {place}写了 contains，还得写 file"
            )));
        }
        if given.contains(&"file") && !given.contains(&"contains") {
            return Err(DefinitionError(format!(
                "{file} {place}写了 file，还得写 contains"
            )));
        }
        let others: Vec<&str> = given
            .iter()
            .copied()
            .filter(|name| *name != "file" && *name != "contains")
            .collect();
        if others.len() > 1 || (!others.is_empty() && given.contains(&"file")) {
            return Err(DefinitionError(format!(
                "{file} {place}的判法只能一种：path / absent / file+contains / run"
            )));
        }
    } else {
        if text_of(value, "description").is_empty() {
            return Err(DefinitionError(format!(
                "{file} {place}是 {kind}，必须写 description（判准 / 要人拍板的事）"
            )));
        }
        if !given.is_empty() {
            return Err(DefinitionError(format!(
                "{file} {place}是 {kind}，不该带 {}（那是 rule 的字段）",
                given.join("、")
            )));
        }
    }
    Ok(criterion_of(value))
}
