//! 工作区聚合 / 定义核对：声明与判据对不对得上。
//!
//! 判据里的路径在不在、描述提到的报告小节有没有判据覆盖。带占位的判据核不了，
//! 给一条「未核」的回执；「在不在」由调用方给——工具箱不碰文件系统。
//! 出处：`docs/specification/process/workflow.md`·定义核对。

use super::model::Workspace;
use crate::criterion::Criterion;
use crate::paths::placeholders_in;
use crate::workflow::Workflow;

/// 定义核对出来的一件事：在哪里、核的是什么、过没过。
///
/// `ok` 为 `None` 表示没核——判据里带的运行时占位要等任务执行时才落，
/// 核对时给一条「未核」的回执，不静默丢掉。
#[derive(Debug, Clone)]
pub struct Finding {
    pub where_: String,
    pub what: String,
    /// 核过的结果；没核给 `None`。
    pub ok: Option<bool>,
}

/// 像不像报告小节的名字：中文短词。版本号写法、占位、路径都不算。
pub fn looks_like_section(name: &str) -> bool {
    !name.is_empty()
        && name.chars().count() <= 12
        && !name.contains(|ch: char| {
            ch.is_ascii_digit()
                || matches!(
                    ch,
                    '[' | ']' | '{' | '}' | '.' | '/' | '`' | '<' | '>' | '-' | '_'
                )
        })
}

impl Workspace {
    /// 核对一条定义：判据里的路径在不在、描述提到的小节有没有判据覆盖。
    ///
    /// `exists` 由调用方给——工具箱不碰文件系统；相对路径的基准也由它在端侧定。
    /// 带占位的判据核不了（要等任务执行时才落），给一条「未核」的回执。
    pub fn check<F>(&self, workflow: &Workflow, exists: F) -> Vec<Finding>
    where
        F: Fn(&str) -> bool,
    {
        let mut found: Vec<Finding> = Vec::new();
        for step in &workflow.steps {
            for criterion in step.rules() {
                let literal = match &criterion {
                    Criterion::PathExists { path, .. } => path.clone(),
                    Criterion::FileContains { file, .. } => file.clone(),
                    _ => continue,
                };
                let names = placeholders_in(&literal);
                if let Some(placeholder) = names.first() {
                    found.push(Finding {
                        where_: format!("{}·{}", step.name, literal),
                        what: format!(
                            "判据里的路径含 {{{{{placeholder}}}}}，未核（等任务执行时再核）"
                        ),
                        ok: None,
                    });
                    continue;
                }
                found.push(Finding {
                    where_: format!("{}·{}", step.name, literal),
                    what: format!("判据里的路径在不在：{literal}"),
                    ok: Some(exists(&literal)),
                });
            }
        }

        let covered: Vec<String> = workflow
            .steps
            .iter()
            .flat_map(|step| step.rules())
            .filter_map(|criterion| match criterion {
                Criterion::FileContains { contains, .. } => Some(contains),
                _ => None,
            })
            .collect();
        let mut mentioned: Vec<String> = Vec::new();
        for step in &workflow.steps {
            let text = step.description.clone();
            for piece in text.split("## ").skip(1) {
                let name = piece
                    .split(|ch: char| ch.is_whitespace() || ch == '`' || ch == '」')
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string();
                if looks_like_section(&name) && !mentioned.contains(&name) {
                    mentioned.push(name);
                }
            }
            let mut rest: &str = text.as_str();
            while let Some(at) = rest.find('「') {
                let after = &rest[at + '「'.len_utf8()..];
                let Some(end) = after.find('」') else { break };
                let name = after[..end].trim().to_string();
                let tail = after[end + '」'.len_utf8()..].trim_start();
                let is_section =
                    tail.starts_with("一节") || tail.starts_with("节") || tail.starts_with("两节");
                if is_section && looks_like_section(&name) && !mentioned.contains(&name) {
                    mentioned.push(name);
                }
                rest = &after[end + '」'.len_utf8()..];
            }
        }
        for name in mentioned {
            found.push(Finding {
                where_: "description".to_string(),
                what: format!("description 提到的报告小节有没有判据覆盖：{name}"),
                ok: Some(covered.iter().any(|value| value.contains(&name))),
            });
        }
        found
    }
}
