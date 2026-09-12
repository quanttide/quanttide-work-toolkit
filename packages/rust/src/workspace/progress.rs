//! 工作区聚合 / 流水判定：走过哪几步、下一步是哪。
//!
//! 「走过哪几步」从任务流水读出：带后缀的流水（`·审` / `·判`）给这一步**投票**，
//! 不带后缀的（重走一遍）**把结论从头算**。工作流从工作区里按名字取。
//! 出处：`docs/specification/process/task.md`·读法。

use super::model::Workspace;
use crate::task::Task;
use crate::workflow::Workflow;

impl Workspace {
    /// 这件任务走过哪几步（按流水里的先后）。
    pub fn done_steps(&self, task: &Task) -> Vec<String> {
        match self.workflow(&task.workflow_name) {
            Some(workflow) => done_steps(task, workflow),
            None => Vec::new(),
        }
    }

    /// 这件任务下一个没走到的步骤（按定义里的顺序）；都走过或没有定义给 `None`。
    pub fn next_step(&self, task: &Task) -> Option<String> {
        let workflow = self.workflow(&task.workflow_name)?;
        next_step(task, workflow)
    }

    /// 状态行：下一步是谁，或者都走过了。
    pub fn state_line(&self, task: &Task) -> String {
        let Some(workflow) = self.workflow(&task.workflow_name) else {
            return format!("工作区里没有这条工作流：{}", task.workflow_name);
        };
        state_line(task, workflow)
    }
}

/// 走过哪几步：工作流上没有的步骤名不算数。
fn done_steps(task: &Task, workflow: &Workflow) -> Vec<String> {
    let names = workflow.step_names();
    let mut verdict: Vec<(String, bool)> = Vec::new();
    for event in &task.journal {
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
fn next_step(task: &Task, workflow: &Workflow) -> Option<String> {
    let finished = done_steps(task, workflow);
    workflow
        .step_names()
        .into_iter()
        .find(|name| !finished.contains(name))
}

/// 状态行：下一步是谁，或者都走过了。
fn state_line(task: &Task, workflow: &Workflow) -> String {
    let names = workflow.step_names();
    if names.is_empty() {
        return format!("这条工作流没有步骤：{}", workflow.name);
    }
    match next_step(task, workflow) {
        Some(step) => format!("下一步：{step}"),
        None => format!("{} 个步骤都走过了", names.len()),
    }
}
