//! 工作流聚合 / 定义核对：声明与判据对不对得上。
//!
//! 判据里的路径在不在、描述提到的报告小节有没有判据覆盖。
//! 落点/占位在 [`crate::task`]；这里是 `workflow --check` 的模型侧。
//! 模型在 [`super::model`]，语法校验在 [`super::validate`]。

use super::model::Workflow;
use crate::criterion::Criterion;
use crate::paths::expand_placeholders;

/// 定义核对出来的一件事：在哪里、核的是什么、过没过。
///
/// 它是「把这条定义对着工作区核一遍」的回执——`workflow --check` 的实现产物，
/// 规范里还没有这一节。
#[derive(Debug, Clone)]
pub struct Finding {
    pub where_: String,
    pub what: String,
    pub ok: bool,
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

impl Workflow {
    /// 核对这条定义：判据里的路径在不在、描述提到的小节有没有判据覆盖。
    ///
    /// `exists` 由调用方给——工具箱不碰文件系统。
    pub fn check<F>(&self, data: &str, exists: F) -> Vec<Finding>
    where
        F: Fn(&str) -> bool,
    {
        let mut found: Vec<Finding> = Vec::new();
        for step in &self.steps {
            for criterion in step.rules() {
                let literal = match &criterion {
                    Criterion::PathExists { path, .. } => path.clone(),
                    Criterion::FileContains { file, .. } => file.clone(),
                    _ => continue,
                };
                if literal.contains("{{report}}")
                    || literal.contains("{{journal}}")
                    || literal.contains("{{log}}")
                {
                    continue;
                }
                let written = expand_placeholders(&literal, data);
                found.push(Finding {
                    where_: format!("{}·{}", step.name, literal),
                    what: format!("判据里的路径在不在：{written}"),
                    ok: exists(&written),
                });
            }
        }

        let covered: Vec<String> = self
            .steps
            .iter()
            .flat_map(|step| step.rules())
            .filter_map(|criterion| match criterion {
                Criterion::FileContains { contains, .. } => Some(contains),
                _ => None,
            })
            .collect();
        let mut mentioned: Vec<String> = Vec::new();
        for step in &self.steps {
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
                ok: covered.iter().any(|value| value.contains(&name)),
            });
        }
        found
    }
}
