//! 工作区聚合 / 落点：产物落在哪，判据里的占位换成哪。
//!
//! 落点只给**相对工作区根的路径**：声明了按声明的（绝对路径原样），没声明落
//! `artifacts/<产物的名字>/<任务名>.md`。拼上目录是平台的事（规范 `process/task.md`·语法）。
//! 占位里有两处不是产物：`{{artifacts}}` 是产物目录，`{{log}}` 是任务文件本身。

use super::model::Workspace;
use crate::artifact::Artifact;
use crate::criterion::Criterion;
use crate::paths::PLACEHOLDER_NAMES;
use crate::task::Task;

/// 产物目录那个占位的名字。
const ARTIFACTS: &str = "artifacts";

/// 任务文件那个占位的名字——流水不是产物。
const LOG: &str = "log";

impl Workspace {
    /// 这件产物落哪（规范「任务 / 语法」里的落点）。
    ///
    /// 给的是相对工作区根的路径；平台拿自己的目录接上。
    pub fn place(&self, task: &Task, artifact: &Artifact) -> String {
        if let Some(written) = task.declared(&artifact.name) {
            return written;
        }
        format!("artifacts/{}/{}.md", artifact.name, task.name)
    }

    /// 把判据里的占位换成本次任务的落点（规范「任务 / 语法」：写了路径就把判据里的占位换成它）。
    ///
    /// 与 [`Workspace::place`] 同一处算，所以判据里写占位与直接写落点等价。
    pub fn expanded(&self, criterion: &Criterion, task: &Task) -> Criterion {
        criterion.expanded(|name| self.placeholder(task, name))
    }

    /// 四个占位各换成哪条路径；不认识给 `None`。
    ///
    /// 两处不是产物：`{{artifacts}}` 是产物目录，`{{log}}` 是任务文件；
    /// 其余在册的占位是产物的名字——产物不限种类，落点按名字算。
    fn placeholder(&self, task: &Task, name: &str) -> Option<String> {
        match name {
            ARTIFACTS => Some(ARTIFACTS.to_string()),
            LOG => Some(format!("tasks/{}.yaml", task.name)),
            named if PLACEHOLDER_NAMES.contains(&named) => {
                Some(self.place(task, &Artifact::named(named)))
            }
            _ => None,
        }
    }
}
