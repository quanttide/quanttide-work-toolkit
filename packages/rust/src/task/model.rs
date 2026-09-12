//! 任务聚合 / 模型：工作流的一次执行实例。
//!
//! 指令是跑哪条工作流（`workflow_name`，**按名字**引用）与从哪开工（`start`）；
//! 状态是流水（只增不改）、闸门项、产物落点；另带这次执行的运行上下文。
//! 任务是运行数据，不是产物——程序只维护它，不往产物里写字。
//!
//! 不可变：`recorded` / `with_gates` 都返回新的任务，改动由调用方落盘。
//! 「走过哪几步」从流水读出，在 [`super::journal`]；运行上下文在 [`super::context`]。
//! 落点在 `artifact`；占位展开在中立的 `crate::paths`。
//! 出处：`docs/specification/process/task.md`·语法。

use super::journal::JournalEvent;
use crate::context::RunContext;
use crate::fields::text_of;
use crate::paths::join;
use serde_yaml::{Mapping, Value as Yaml};
use std::collections::BTreeMap;

/// 任务聚合。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    pub name: String,
    pub workflow_name: String,
    pub start: String,
    pub context: RunContext,
    pub journal: Vec<JournalEvent>,
    /// 等人拍板的事项。
    pub gates: Vec<String>,
    /// 这次执行往哪写产物（声明写成什么就是什么，相对工作区根）。
    pub artifacts: BTreeMap<String, String>,
}

impl Task {
    /// 从任务文件里的字段读出（不校验）；任务名取自 `name` 字段。
    pub fn of(payload: &Yaml) -> Task {
        let journal = payload
            .get("log")
            .and_then(|v| v.as_sequence())
            .map(|items| items.iter().map(JournalEvent::of).collect())
            .unwrap_or_default();
        let gates = payload
            .get("gates")
            .and_then(|v| v.as_sequence())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str().map(|text| text.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        let artifacts = payload
            .get("artifacts")
            .and_then(|v| v.as_mapping())
            .map(|mapping| {
                mapping
                    .iter()
                    .filter_map(|(key, value)| {
                        Some((key.as_str()?.to_string(), value.as_str()?.to_string()))
                    })
                    .collect()
            })
            .unwrap_or_default();
        Task {
            name: text_of(payload, "name"),
            workflow_name: text_of(payload, "workflow"),
            start: text_of(payload, "start"),
            context: RunContext::of(payload),
            journal,
            gates,
            artifacts,
        }
    }

    /// 这种产物声明了往哪写；没声明给 `None`。
    pub fn declared(&self, kind: &str) -> Option<String> {
        let written = self.artifacts.get(kind)?.trim();
        if written.is_empty() {
            None
        } else {
            Some(written.to_string())
        }
    }

    /// 这次执行往哪写这种产物（规范「任务 / 语法」里的落点）。
    ///
    /// 声明了按声明的（相对工作区根）；没声明落数据仓的
    /// `artifacts/<种类>/<任务名>.md`；流水是任务文件本身。
    pub fn artifact(&self, kind: &str, context: &RunContext) -> String {
        if kind == "log" {
            return join(&context.data, &format!("tasks/{}.yaml", self.name));
        }
        if let Some(written) = self.declared(kind) {
            return if written.starts_with('/') {
                written
            } else {
                join(&context.root, &written)
            };
        }
        join(&context.data, &format!("artifacts/{kind}/{}.md", self.name))
    }

    /// 记一笔流水：流水只增不改，所以返回新的任务。
    pub fn recorded(&self, at: &str, step: &str, detail: &str, ok: bool) -> Task {
        let mut task = self.clone();
        task.journal.push(JournalEvent {
            at: at.to_string(),
            step: step.to_string(),
            detail: detail.to_string(),
            ok,
        });
        task
    }

    /// 记下闸门项；已有的不重复记。同样返回新的任务。
    pub fn with_gates(&self, notes: &[String]) -> Task {
        let mut task = self.clone();
        for note in notes {
            if !task.gates.contains(note) {
                task.gates.push(note.clone());
            }
        }
        task
    }

    /// 写回任务文件的字段形状。
    pub fn to_yaml(&self) -> Yaml {
        let mut map = Mapping::new();
        map.insert(Yaml::String("name".into()), Yaml::String(self.name.clone()));
        map.insert(
            Yaml::String("start".into()),
            Yaml::String(self.start.clone()),
        );
        map.insert(
            Yaml::String("workflow".into()),
            Yaml::String(self.workflow_name.clone()),
        );
        map.insert(
            Yaml::String("log".into()),
            Yaml::Sequence(self.journal.iter().map(JournalEvent::to_yaml).collect()),
        );
        map.insert(
            Yaml::String("gates".into()),
            Yaml::Sequence(
                self.gates
                    .iter()
                    .map(|note| Yaml::String(note.clone()))
                    .collect(),
            ),
        );
        let mut artifacts = Mapping::new();
        for (key, value) in &self.artifacts {
            artifacts.insert(Yaml::String(key.clone()), Yaml::String(value.clone()));
        }
        map.insert(Yaml::String("artifacts".into()), Yaml::Mapping(artifacts));
        if let Yaml::Mapping(context) = self.context.to_yaml() {
            for (key, value) in context {
                map.insert(key, value);
            }
        }
        Yaml::Mapping(map)
    }
}
