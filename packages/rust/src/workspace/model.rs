//! 工作区聚合 / 模型：一次工作的边界，把定义与任务系在一起。
//!
//! 工作区是建模单位，不是目录——它只装内容（装载起来的定义与任务），不装位置。
//! 物理位置由平台给：落点与核对要用的目录基准，当参数传进来。
//! 出处：`docs/specification/place/workspace.md`。

use crate::task::Task;
use crate::workflow::Workflow;

/// 工作区：装载起来的定义与任务。跨着定义与现场的操作都挂在这里。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Workspace {
    pub workflows: Vec<Workflow>,
    pub tasks: Vec<Task>,
}

impl Workspace {
    /// 拿装载好的定义与任务装一个工作区。
    pub fn of(workflows: Vec<Workflow>, tasks: Vec<Task>) -> Workspace {
        Workspace { workflows, tasks }
    }

    /// 按名字取装载起来的工作流定义。
    pub fn workflow(&self, name: &str) -> Option<&Workflow> {
        self.workflows.iter().find(|flow| flow.name == name)
    }

    /// 按名字取装载起来的任务。
    pub fn task(&self, name: &str) -> Option<&Task> {
        self.tasks.iter().find(|task| task.name == name)
    }
}
