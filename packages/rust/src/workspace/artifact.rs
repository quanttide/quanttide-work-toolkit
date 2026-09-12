//! 工作区聚合 / 落点：产物落在哪，判据里的占位换成哪。
//!
//! 落点只给**相对工作区根的路径**：声明了按声明的（绝对路径原样），没声明落
//! `artifacts/<种类>/<任务名>.md`；流水是任务文件本身 `tasks/<任务名>.yaml`。
//! 拼上目录是平台的事（规范 `process/task.md`·语法）。

use super::model::Workspace;
use crate::criterion::Criterion;
use crate::task::Task;

/// 流水这种产物的种类名。
const LOG: &str = "log";

/// 产物目录那个占位的名字。
const ARTIFACTS: &str = "artifacts";

impl Workspace {
    /// 这次执行往哪写这种产物（规范「任务 / 语法」里的落点）。
    ///
    /// 给的是相对工作区根的路径；平台拿自己的目录接上。
    pub fn artifact(&self, task: &Task, kind: &str) -> String {
        if kind == LOG {
            return format!("tasks/{}.yaml", task.name);
        }
        if let Some(written) = task.declared(kind) {
            return written;
        }
        format!("artifacts/{kind}/{}.md", task.name)
    }

    /// 把判据里的占位换成本次任务的落点（规范「任务 / 语法」：写了路径就把判据里的占位换成它）。
    ///
    /// 与 [`Workspace::artifact`] 同一处算，所以判据里写占位与直接写落点等价。
    pub fn expanded(&self, criterion: &Criterion, task: &Task) -> Criterion {
        criterion.expanded(|name| self.placeholder(task, name))
    }

    /// 四个占位各换成哪条路径；不认识给 `None`。
    fn placeholder(&self, task: &Task, name: &str) -> Option<String> {
        if name == ARTIFACTS {
            return Some(ARTIFACTS.to_string());
        }
        if name == "report" || name == "journal" || name == LOG {
            return Some(self.artifact(task, name));
        }
        None
    }
}
