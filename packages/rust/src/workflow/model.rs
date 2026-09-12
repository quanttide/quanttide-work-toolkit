//! 工作流聚合：一串有序的步骤。
//!
//! 本文件只装模型——字段名、不变量、取值校验与报错在 [`super::validate`]，
//! 定义核对（[`Workflow::check`]）在 [`super::check`]。
//! 规矩的出处是 `docs/specification/process/workflow.md`·语法。
//!
//! 模型不可变：[`Workflow::from_value`] 读进来顺带校验，[`Workflow::of`] 读已经校验过的，
//! [`Workflow::to_yaml`] 写成同样的字段形状。YAML 怎么读写是各自包的事。

use crate::criterion::{Criterion, criterion_of};
use crate::executor::{AGENT, HUMAN, RULE};
use crate::fields::text_of;
use serde_yaml::{Mapping, Value as Yaml};

/// 一个工作步骤：叫什么、做什么、谁执行、怎么算完。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub name: String,
    pub description: String,
    pub executor: String,
    pub criteria: Vec<Criterion>,
}

impl Step {
    /// 从定义里的字段读出（不校验）。用在已经校验过的定义上。
    pub fn of(value: &Yaml) -> Step {
        let criteria = value
            .get("criteria")
            .and_then(|v| v.as_sequence())
            .map(|items| items.iter().map(criterion_of).collect())
            .unwrap_or_default();
        let mut executor = text_of(value, "executor");
        if executor.is_empty() {
            executor = AGENT.to_string();
        }
        Step {
            name: text_of(value, "name"),
            description: text_of(value, "description"),
            executor,
            criteria,
        }
    }

    /// 这一步的执行者是不是人。
    pub fn human(&self) -> bool {
        self.executor == HUMAN
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn description(&self) -> String {
        self.description.clone()
    }

    pub fn executor(&self) -> String {
        self.executor.clone()
    }

    pub fn criteria(&self) -> Vec<Criterion> {
        self.criteria.clone()
    }

    pub fn rules(&self) -> Vec<Criterion> {
        self.of_kind(RULE)
    }

    pub fn agents(&self) -> Vec<Criterion> {
        self.of_kind(AGENT)
    }

    pub fn gates(&self) -> Vec<Criterion> {
        self.of_kind(HUMAN)
    }

    fn of_kind(&self, kind: &str) -> Vec<Criterion> {
        self.criteria
            .iter()
            .filter(|item| item.executor() == kind)
            .cloned()
            .collect()
    }

    /// 写回定义里的字段形状。
    pub fn to_yaml(&self) -> Yaml {
        let mut map = Mapping::new();
        map.insert(Yaml::String("name".into()), Yaml::String(self.name.clone()));
        if !self.description.is_empty() {
            map.insert(
                Yaml::String("description".into()),
                Yaml::String(self.description.clone()),
            );
        }
        map.insert(
            Yaml::String("executor".into()),
            Yaml::String(self.executor.clone()),
        );
        if !self.criteria.is_empty() {
            map.insert(
                Yaml::String("criteria".into()),
                Yaml::Sequence(self.criteria.iter().map(Criterion::to_yaml).collect()),
            );
        }
        Yaml::Mapping(map)
    }
}

/// 工作流聚合：一串步骤（不含文件位置——那是各自包的事）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workflow {
    pub name: String,
    pub description: String,
    pub steps: Vec<Step>,
}

impl Workflow {
    /// 从定义里的字段读出（不校验）；工作流名取自 `name` 字段。
    pub fn of(payload: &Yaml) -> Workflow {
        Workflow {
            name: text_of(payload, "name"),
            description: text_of(payload, "description"),
            steps: payload
                .get("steps")
                .and_then(|v| v.as_sequence())
                .map(|items| items.iter().map(Step::of).collect())
                .unwrap_or_default(),
        }
    }

    pub fn description(&self) -> String {
        self.description.clone()
    }

    pub fn steps(&self) -> Vec<Step> {
        self.steps.clone()
    }

    /// 步骤名，按定义顺序。
    pub fn step_names(&self) -> Vec<String> {
        self.steps.iter().map(|step| step.name.clone()).collect()
    }

    pub fn step(&self, name: &str) -> Option<Step> {
        self.steps.iter().find(|step| step.name == name).cloned()
    }

    /// 写回定义里的字段形状。
    pub fn to_yaml(&self) -> Yaml {
        let mut map = Mapping::new();
        map.insert(Yaml::String("name".into()), Yaml::String(self.name.clone()));
        if !self.description.is_empty() {
            map.insert(
                Yaml::String("description".into()),
                Yaml::String(self.description.clone()),
            );
        }
        map.insert(
            Yaml::String("steps".into()),
            Yaml::Sequence(self.steps.iter().map(Step::to_yaml).collect()),
        );
        Yaml::Mapping(map)
    }
}
