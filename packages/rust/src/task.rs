//! 任务聚合：工作流的一次执行实例。
//!
//! 指令是跑哪条工作流（`workflow_name`，**按名字**引用）与从哪开工（`start`）；
//! 状态是流水（只增不改）、闸门项、产物落点；另带这次执行的运行上下文。
//! 任务是运行数据，不是产物——程序只维护它，不往产物里写字。
//!
//! 不可变：`recorded` / `with_gates` 都返回新的任务，改动由调用方落盘。

use std::collections::BTreeMap;

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

/// 这次执行自带的运行上下文：工作区根、数据仓、工作流目录。
///
/// 三个字段怎么读、怎么写由各自的包定（工具箱只管托着它们）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RunContext {
    pub root: String,
    pub data: String,
    pub workflows: String,
}

impl RunContext {
    pub fn of(value: &Yaml) -> RunContext {
        RunContext {
            root: text_of(value, "root"),
            data: text_of(value, "data"),
            workflows: text_of(value, "workflows"),
        }
    }

    pub fn to_yaml(&self) -> Yaml {
        let mut map = Mapping::new();
        for (key, value) in [
            ("root", &self.root),
            ("data", &self.data),
            ("workflows", &self.workflows),
        ] {
            map.insert(
                Yaml::String(key.to_string()),
                Yaml::String(value.to_string()),
            );
        }
        Yaml::Mapping(map)
    }
}

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
    pub products: BTreeMap<String, String>,
}

impl Task {
    /// 从任务文件里的字段读出；`name` 由调用方给（文件名即任务名）。
    pub fn of(name: &str, payload: &Yaml) -> Task {
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
        let products = payload
            .get("products")
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
            name: name.to_string(),
            workflow_name: text_of(payload, "workflow"),
            start: text_of(payload, "start"),
            context: RunContext::of(payload),
            journal,
            gates,
            products,
        }
    }

    /// 这种产物声明了往哪写；没声明给 `None`（落哪是各自包的事）。
    pub fn product(&self, kind: &str) -> Option<String> {
        let written = self.products.get(kind)?.trim();
        if written.is_empty() {
            None
        } else {
            Some(written.to_string())
        }
    }

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
        let mut products = Mapping::new();
        for (key, value) in &self.products {
            products.insert(Yaml::String(key.clone()), Yaml::String(value.clone()));
        }
        map.insert(Yaml::String("products".into()), Yaml::Mapping(products));
        if let Yaml::Mapping(context) = self.context.to_yaml() {
            for (key, value) in context {
                map.insert(key, value);
            }
        }
        Yaml::Mapping(map)
    }
}
