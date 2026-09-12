//! 路径：占位认哪几个、怎么换。
//!
//! 中立处：`criterion`（判据的字段里怎么写占位）与 `workspace`（落点、定义核对）向它对齐。
//! 换成哪条路径由调用方给（`workspace` 按任务声明与默认处算）；这里只管认名字与替换。

/// 定义里只认这四个占位。
pub const PLACEHOLDER_NAMES: [&str; 4] = ["artifacts", "report", "journal", "log"];

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

/// 把一条文字里的占位按 `resolve` 换掉；`resolve` 认不得的占位原样留着（读定义时已经拦过）。
pub(crate) fn replace_placeholders<F>(value: &str, resolve: F) -> String
where
    F: Fn(&str) -> Option<String>,
{
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
        let name = &after[..end];
        match resolve(name) {
            Some(path) => out.push_str(&path),
            None => {
                out.push_str("{{");
                out.push_str(name);
                out.push_str("}}");
            }
        }
        rest = &after[end + 2..];
    }
    out.push_str(rest);
    out
}
