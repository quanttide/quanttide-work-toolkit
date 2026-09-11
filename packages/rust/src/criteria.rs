//! 判据：规则引擎那几种机械核对。
//!
//! 判据是**字段**，不是一行小语法：`path` 存在、`absent` 不存在、
//! `file` + `contains` 含这段文字、`run` 这条命令退出码为零。
//! 工具箱只把判据翻成「要跑什么」——真去跑（文件系统、起进程）是各自包的事。

use serde_json::Value as Json;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleKind {
    Path,
    Absent,
    Contains,
    Run,
}

/// 一条要跑的判据：说明 + 怎么判（`kind` 为空即不跑，交给智能体或人）。
#[derive(Debug, Clone)]
pub struct RuleItem {
    pub description: String,
    pub kind: Option<RuleKind>,
    pub args: Vec<String>,
}

impl RuleItem {
    pub fn machine(&self) -> bool {
        self.kind.is_some()
    }
}

fn text(criterion: &Json, key: &str) -> Option<String> {
    criterion
        .get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// 说明：写了就用写的，没写按字段拼一句。
pub fn description_of(criterion: &Json) -> String {
    if let Some(written) = text(criterion, "description")
        && !written.trim().is_empty()
    {
        return written.trim().to_string();
    }
    if let Some(path) = text(criterion, "path") {
        return format!("存在：{path}");
    }
    if let Some(path) = text(criterion, "absent") {
        return format!("不存在：{path}");
    }
    if let Some(file) = text(criterion, "file") {
        let needle = text(criterion, "contains").unwrap_or_default();
        return format!("含「{needle}」：{file}");
    }
    if let Some(run) = text(criterion, "run") {
        return format!("跑通：{run}");
    }
    String::new()
}

/// 把定义里的判据翻成要跑的东西：rule 的跑，agent / human 的不跑。
pub fn items_of(criteria: &[Json]) -> Vec<RuleItem> {
    let mut items = Vec::new();
    for criterion in criteria {
        let description = description_of(criterion);
        let is_rule = text(criterion, "executor").as_deref() == Some("rule");
        if !is_rule {
            items.push(RuleItem {
                description,
                kind: None,
                args: Vec::new(),
            });
            continue;
        }
        if let Some(path) = text(criterion, "path") {
            items.push(RuleItem {
                description,
                kind: Some(RuleKind::Path),
                args: vec![path],
            });
        } else if let Some(path) = text(criterion, "absent") {
            items.push(RuleItem {
                description,
                kind: Some(RuleKind::Absent),
                args: vec![path],
            });
        } else if let Some(file) = text(criterion, "file") {
            let needle = text(criterion, "contains").unwrap_or_default();
            items.push(RuleItem {
                description,
                kind: Some(RuleKind::Contains),
                args: vec![file, needle],
            });
        } else if let Some(run) = text(criterion, "run") {
            items.push(RuleItem {
                description,
                kind: Some(RuleKind::Run),
                args: vec![run],
            });
        }
    }
    items
}
