//! 定义读不通时的错误。
//!
//! 只存位置（顶层 / 第 n 个步骤 / 第 n 个步骤第 m 条判据）与种类（[`Fault`]）；
//! 文件名不进错误，由端侧在渲染时给——[`DefinitionError::message`] 出 canonical 文案。

use crate::executor::{CRITERION_TYPES, EXECUTORS};
use crate::fields::{CRITERION_FIELDS, STEP_FIELDS, TOP_FIELDS};

/// 定义里的位置。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Position {
    /// 顶层。
    Top,
    /// 第 n 个步骤（从 1 数）。
    Step(usize),
    /// 第 n 个步骤第 m 条判据（都从 1 数）。
    Criterion { step: usize, criterion: usize },
}

impl Position {
    /// 位置的话头，比如「第 1 个步骤」；顶层是空串。
    pub fn phrase(&self) -> String {
        match self {
            Position::Top => String::new(),
            Position::Step(step) => format!("第 {step} 个步骤"),
            Position::Criterion { step, criterion } => {
                format!("第 {step} 个步骤第 {criterion} 条判据")
            }
        }
    }
}

/// 这一条错在哪，带上渲染所需的取值。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fault {
    /// 定义顶层不是映射。
    TopNotMapping,
    /// 顶层少了 name。
    MissingName,
    /// 顶层少了 steps（至少一个步骤）。
    MissingSteps,
    /// 顶层有不认识的字段。
    UnknownTopFields(Vec<String>),
    /// 步骤少了 name（步骤不是映射也算）。
    MissingStepName,
    /// 步骤有不认识的字段。
    UnknownStepFields(Vec<String>),
    /// 步骤的 executor 越界。
    BadStepExecutor { got: String },
    /// 步骤的 criteria 不是列表。
    CriteriaNotList,
    /// 判据不是映射。
    CriterionNotMapping,
    /// 判据的 executor 越界。
    BadCriterionExecutor,
    /// 判据有不认识的字段。
    UnknownCriterionFields(Vec<String>),
    /// 判据是 rule，却没写判法。
    RuleNeedsJudgement,
    /// 写了 contains，没写 file。
    ContainsNeedsFile,
    /// 写了 file，没写 contains。
    FileNeedsContains,
    /// 判法混着写：只能一种。
    OnlyOneJudgement,
    /// 判据是 agent / human，却没写 description。
    NeedsDescription { kind: String },
    /// 判据是 agent / human，却带了 rule 的字段。
    NoRuleFields { kind: String, given: Vec<String> },
}

impl Fault {
    /// 位置之后的那一句（canonical 文案的尾巴）。
    pub fn text(&self) -> String {
        match self {
            Fault::TopNotMapping => "的顶层不是映射（name / steps）".to_string(),
            Fault::MissingName => "少了 name".to_string(),
            Fault::MissingSteps => "少了 steps（至少一个步骤）".to_string(),
            Fault::UnknownTopFields(unknown) => format!(
                "顶层有不认识的字段：{}（只认 {}）",
                unknown.join("、"),
                TOP_FIELDS.join("、")
            ),
            Fault::MissingStepName => "少了 name".to_string(),
            Fault::UnknownStepFields(unknown) => format!(
                "有不认识的字段：{}（只认 {}）",
                unknown.join("、"),
                STEP_FIELDS.join("、")
            ),
            Fault::BadStepExecutor { got } => {
                format!("的 executor 只能是 {}，实得 {got}", EXECUTORS.join(" 或 "))
            }
            Fault::CriteriaNotList => "的 criteria 应当是列表".to_string(),
            Fault::CriterionNotMapping => "不是映射".to_string(),
            Fault::BadCriterionExecutor => format!(
                "的 executor 只能是 {}（谁判：规则引擎 / 智能体 / 人）",
                CRITERION_TYPES.join(" / ")
            ),
            Fault::UnknownCriterionFields(unknown) => format!(
                "有不认识的字段：{}（只认 {}）",
                unknown.join("、"),
                CRITERION_FIELDS.join("、")
            ),
            Fault::RuleNeedsJudgement => {
                "是 rule，得写一条判法（path / absent / file+contains / run）".to_string()
            }
            Fault::ContainsNeedsFile => "写了 contains，还得写 file".to_string(),
            Fault::FileNeedsContains => "写了 file，还得写 contains".to_string(),
            Fault::OnlyOneJudgement => {
                "的判法只能一种：path / absent / file+contains / run".to_string()
            }
            Fault::NeedsDescription { kind } => {
                format!("是 {kind}，必须写 description（判准 / 要人拍板的事）")
            }
            Fault::NoRuleFields { kind, given } => {
                format!("是 {kind}，不该带 {}（那是 rule 的字段）", given.join("、"))
            }
        }
    }
}

/// 一份定义读不通。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefinitionError {
    /// 在哪一层读不通。
    pub position: Position,
    /// 哪一条不成立。
    pub fault: Fault,
}

impl DefinitionError {
    pub fn new(position: Position, fault: Fault) -> Self {
        DefinitionError { position, fault }
    }

    /// canonical 报错文字：`{file} {位置}{这一条}`；文件由端侧在渲染时给。
    pub fn message(&self, file: &str) -> String {
        format!("{file} {}{}", self.position.phrase(), self.fault.text())
    }
}

impl std::fmt::Display for DefinitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.position.phrase(), self.fault.text())
    }
}

impl std::error::Error for DefinitionError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_prefixes_the_file_and_keeps_the_position() {
        let error = DefinitionError::new(Position::Top, Fault::MissingName);
        assert_eq!(error.message("demo.yaml"), "demo.yaml 少了 name");
        assert_eq!(error.to_string(), "少了 name");

        let error = DefinitionError::new(
            Position::Criterion {
                step: 2,
                criterion: 3,
            },
            Fault::RuleNeedsJudgement,
        );
        assert_eq!(
            error.message("demo.yaml"),
            "demo.yaml 第 2 个步骤第 3 条判据是 rule，得写一条判法（path / absent / file+contains / run）"
        );
    }
}
