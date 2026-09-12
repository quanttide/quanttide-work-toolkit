//! 路径：目录与剩下的路径相接，占位展开到数据仓。
//!
//! 中立处：`task`（落点）与 `workflow`（定义核对）都向它对齐。

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

/// 判据里的占位先按数据仓展开（够核对用）。
pub fn expand_placeholders(value: &str, data: &str) -> String {
    value
        .replace("{{artifacts}}", &join(data, "artifacts"))
        .replace("{{report}}", &join(data, "artifacts/report"))
        .replace("{{journal}}", &join(data, "artifacts/journal"))
        .replace("{{log}}", &join(data, "tasks"))
}
