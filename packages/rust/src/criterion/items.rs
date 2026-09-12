//! 判据翻成「要跑什么」：规则引擎跑，智能体 / 人不跑。
//!
//! 模型在 [`super::model`]；把一条条判据翻成要跑的东西，
//! 真去跑（文件系统、起进程）是各自包的事。
//! 出处：`docs/specification/process/workflow.md`·语法（判据判法）。

use super::model::{Criterion, RuleKind};

/// 一条要跑的判据：说明 + 怎么判（`kind` 为空即不跑，交给智能体或人）。
#[derive(Debug, Clone)]
pub struct RuleItem {
    pub description: String,
    pub kind: Option<RuleKind>,
    pub args: Vec<String>,
}

impl RuleItem {
    pub fn machine(&self) -> bool {
        self.kind.is_some()
    }
}

/// 把判据翻成要跑的东西：rule 的跑，agent / human 的不跑。
pub fn items_of(criteria: &[Criterion]) -> Vec<RuleItem> {
    criteria
        .iter()
        .map(|criterion| {
            let description = criterion.text();
            let (kind, args) = match criterion {
                Criterion::PathExists { path, .. } => (Some(RuleKind::Path), vec![path.clone()]),
                Criterion::PathAbsent { absent, .. } => {
                    (Some(RuleKind::Absent), vec![absent.clone()])
                }
                Criterion::FileContains { file, contains, .. } => (
                    Some(RuleKind::Contains),
                    vec![file.clone(), contains.clone()],
                ),
                Criterion::CommandRun { run, .. } => (Some(RuleKind::Run), vec![run.clone()]),
                Criterion::AgentJudgement { .. } | Criterion::HumanGate { .. } => {
                    (None, Vec::new())
                }
            };
            RuleItem {
                description,
                kind,
                args,
            }
        })
        .collect()
}
