//! 工作区聚合 / 落点：产物落在哪。
//!
//! 声明了按声明的（相对平台给的目录基准）；没声明落
//! `artifacts/<种类>/<任务名>.md`；流水是任务文件本身。相接的规矩在 `crate::paths`。
//! 出处：`docs/specification/process/task.md`·语法（落点）。

use super::model::Workspace;
use crate::paths::{Placeholders, join};
use crate::task::Task;

/// 流水这种产物的种类名。
const LOG: &str = "log";

impl Workspace {
    /// 这次执行往哪写这种产物（规范「任务 / 语法」里的落点）。
    ///
    /// `base` 是平台给的目录基准；任务自己不带位置。
    pub fn artifact(&self, task: &Task, kind: &str, base: &str) -> String {
        if kind == LOG {
            return join(base, &format!("tasks/{}.yaml", task.name));
        }
        if let Some(written) = task.declared(kind) {
            return if written.starts_with('/') {
                written
            } else {
                join(base, &written)
            };
        }
        join(base, &format!("artifacts/{kind}/{}.md", task.name))
    }

    /// 本次任务的占位表：四个占位各换成哪个落点（规范 `process/workflow.md`·语法）。
    ///
    /// `{{report}}` / `{{journal}}` / `{{log}}` 换的是产物的落点，`{{artifacts}}` 换产物目录——
    /// 与 [`Workspace::artifact`] 同一处算，所以判据里写占位与直接写落点等价。
    pub fn placeholders(&self, task: &Task, base: &str) -> Placeholders {
        Placeholders {
            artifacts: join(base, "artifacts"),
            report: self.artifact(task, "report", base),
            journal: self.artifact(task, "journal", base),
            log: self.artifact(task, "log", base),
        }
    }
}
