//! 路径：目录与剩下的路径相接，占位换成落点。
//!
//! 中立处：`criterion`（判据的字段里怎么写占位）与 `workspace`（落点、定义核对）向它对齐。
//! 落点本身由 `workspace` 按任务声明与默认处算，这里只管「占位换成哪条路径」。

/// 定义里只认这四个占位。
pub const PLACEHOLDER_NAMES: [&str; 4] = ["artifacts", "report", "journal", "log"];

/// 拼目录与剩下的路径：末尾斜杠忽略、重复斜杠折叠（`/` 与空串等价——都落在根）。
///
/// 规矩的出处是 `docs/specification/process/task.md`·落点。落点只有这一处拼法。
pub(crate) fn join(dir: &str, rest: &str) -> String {
    let mut clean = String::with_capacity(dir.len() + rest.len() + 1);
    let mut last_was_slash = false;
    for ch in dir.chars() {
        if ch == '/' {
            if last_was_slash {
                continue;
            }
            last_was_slash = true;
        } else {
            last_was_slash = false;
        }
        clean.push(ch);
    }
    format!("{}/{rest}", clean.trim_end_matches('/'))
}

/// 一条文字里的占位名（`{{name}}` 的 name，按出现次序，重复的也留）。
pub fn placeholders_in(value: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut rest = value;
    while let Some(at) = rest.find("{{") {
        let after = &rest[at + 2..];
        let Some(end) = after.find("}}") else { break };
        names.push(after[..end].to_string());
        rest = &after[end + 2..];
    }
    names
}

/// 占位表：四个占位各换成哪个落点。
///
/// 落点由 `workspace` 按任务声明与默认处算好（见 `Workspace::placeholders`）；
/// 判据里的字段与落点不再各拼一套。四个占位认哪几个，以 [`PLACEHOLDER_NAMES`] 为准。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Placeholders {
    /// `{{artifacts}}`：产物目录。
    pub artifacts: String,
    /// `{{report}}`：报告的落点。
    pub report: String,
    /// `{{journal}}`：日志的落点。
    pub journal: String,
    /// `{{log}}`：流水（任务文件本身）。
    pub log: String,
}

impl Placeholders {
    /// 这个占位换成哪个落点；不认识给 `None`。
    pub fn path_of(&self, name: &str) -> Option<&str> {
        match name {
            "artifacts" => Some(&self.artifacts),
            "report" => Some(&self.report),
            "journal" => Some(&self.journal),
            "log" => Some(&self.log),
            _ => None,
        }
    }

    /// 把一条文字里的占位换成落点；不认识的占位原样留着（读定义时已经拦过）。
    pub fn expand(&self, value: &str) -> String {
        if !value.contains("{{") {
            return value.to_string();
        }
        let mut out = String::with_capacity(value.len());
        let mut rest = value;
        while let Some(at) = rest.find("{{") {
            out.push_str(&rest[..at]);
            let after = &rest[at + 2..];
            let Some(end) = after.find("}}") else {
                out.push_str("{{");
                rest = after;
                continue;
            };
            match self.path_of(&after[..end]) {
                Some(path) => out.push_str(path),
                None => {
                    out.push_str("{{");
                    out.push_str(&after[..end]);
                    out.push_str("}}");
                }
            }
            rest = &after[end + 2..];
        }
        out.push_str(rest);
        out
    }
}
