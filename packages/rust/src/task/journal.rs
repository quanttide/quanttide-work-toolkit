//! 任务聚合 / 流水：什么时候、哪一步、一句话、过没过。
//!
//! 「走过哪几步」从流水读出：带后缀的流水（`·审` / `·判`）给这一步**投票**，
//! 不带后缀的（重走一遍）**把结论从头算**。规矩的出处是
//! `docs/specification/process/task.md`·读法。

use super::model::Task;
use crate::workflow::Workflow;
use crate::workflow::text_of;
use serde_yaml::{Mapping, Value as Yaml};

/// 流水里的一条：什么时候、哪一步、一句话、过没过。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalEvent {
    pub at: String,
    pub step: String,
    pub detail: String,
    pub ok: bool,
}

impl JournalEvent {
    pub fn of(value: &Yaml) -> JournalEvent {
        JournalEvent {
            at: text_of(value, "at"),
            step: text_of(value, "step"),
            detail: text_of(value, "detail"),
            ok: value.get("ok").and_then(|v| v.as_bool()).unwrap_or(false),
        }
    }

    pub fn to_yaml(&self) -> Yaml {
        let mut map = Mapping::new();
        for (key, value) in [
            ("at", &self.at),
            ("step", &self.step),
            ("detail", &self.detail),
        ] {
            map.insert(
                Yaml::String(key.to_string()),
                Yaml::String(value.to_string()),
            );
        }
        map.insert(Yaml::String("ok".into()), Yaml::Bool(self.ok));
        Yaml::Mapping(map)
    }
}

impl Task {
    /// 走过哪几步。
    ///
    /// 带后缀的流水（`·审` / `·判`）给这一步**投票**，不带后缀的（重走一遍）
    /// **把结论从头算**；工作流上没有的步骤名不算数。
    pub fn done_steps(&self, workflow: &Workflow) -> Vec<String> {
        let names = workflow.step_names();
        let mut verdict: Vec<(String, bool)> = Vec::new();
        for event in &self.journal {
            let (step, extra) = match event.step.split_once('·') {
                Some((step, rest)) => (step.to_string(), Some(rest.to_string())),
                None => (event.step.clone(), None),
            };
            if !names.contains(&step) {
                continue;
            }
            match verdict.iter_mut().find(|(name, _)| *name == step) {
                Some((_, last)) => {
                    if extra.is_some() {
                        *last = *last && event.ok;
                    } else {
                        *last = event.ok;
                    }
                }
                None => verdict.push((step, event.ok)),
            }
        }
        verdict
            .into_iter()
            .filter(|(_, ok)| *ok)
            .map(|(name, _)| name)
            .collect()
    }

    /// 第一个没走到的步骤（按工作流里的顺序）。
    pub fn next_step(&self, workflow: &Workflow) -> Option<String> {
        let finished = self.done_steps(workflow);
        workflow
            .step_names()
            .into_iter()
            .find(|name| !finished.contains(name))
    }

    /// 状态行：下一步是谁，或者都走过了。
    pub fn state_line(&self, workflow: &Workflow) -> String {
        let names = workflow.step_names();
        if names.is_empty() {
            return format!(
                "这条工作流没有步骤——在 workflows/{}.yaml 的 steps 里写步骤",
                self.workflow_name
            );
        }
        match self.next_step(workflow) {
            Some(step) => format!("下一步：{step}"),
            None => format!("{} 个步骤都走过了", names.len()),
        }
    }
}
