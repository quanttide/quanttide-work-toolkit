//! 判据：规则引擎那几种机械核对。
//!
//! 判据是**字段**，不是一行小语法：`path` 存在、`absent` 不存在、
//! `file` + `contains` 含这段文字、`run` 这条命令退出码为零。
//! 工具箱只把判据翻成「要跑什么」——真去跑（文件系统、起进程）是各自包的事。
//! 出处：`docs/specification/process/workflow.md`·语法（判据三个字段）。

use crate::executor::{AGENT, HUMAN, RULE};
use crate::paths::replace_placeholders;
use serde_yaml::{Mapping, Value as Yaml};

/// 判据的种类：四种机械核对。名字与 [`Criterion`] 的四个变体一致；
/// 线值（[`RuleKind::as_str`]）是定义里的字段名，不随变体名变。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleKind {
    PathExists,
    PathAbsent,
    FileContains,
    CommandRun,
}

impl RuleKind {
    /// 线上写法：判据里的字段名（`path` / `absent` / `contains` / `run`）。
    pub fn as_str(&self) -> &'static str {
        match self {
            RuleKind::PathExists => "path",
            RuleKind::PathAbsent => "absent",
            RuleKind::FileContains => "contains",
            RuleKind::CommandRun => "run",
        }
    }
}

/// 一条判据：谁判、怎么判、说明。
///
/// 值对象，不可变。`rule` 四种判法各一个变体；`agent` / `human` 只有说明。
/// 线上形状不变——进出定义文件时仍写成 `path` / `absent` / `file`+`contains` / `run` 字段。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Criterion {
    PathExists {
        path: String,
        description: String,
    },
    PathAbsent {
        absent: String,
        description: String,
    },
    FileContains {
        file: String,
        contains: String,
        description: String,
    },
    CommandRun {
        run: String,
        description: String,
    },
    AgentJudgement {
        description: String,
    },
    HumanGate {
        description: String,
    },
}

impl Criterion {
    /// 谁判：`rule` / `agent` / `human`。
    pub fn executor(&self) -> &'static str {
        match self {
            Criterion::PathExists { .. }
            | Criterion::PathAbsent { .. }
            | Criterion::FileContains { .. }
            | Criterion::CommandRun { .. } => RULE,
            Criterion::AgentJudgement { .. } => AGENT,
            Criterion::HumanGate { .. } => HUMAN,
        }
    }

    /// 写了的说明；空即按判法拼一句。
    pub fn description(&self) -> &str {
        match self {
            Criterion::PathExists { description, .. }
            | Criterion::PathAbsent { description, .. }
            | Criterion::FileContains { description, .. }
            | Criterion::CommandRun { description, .. }
            | Criterion::AgentJudgement { description }
            | Criterion::HumanGate { description } => description,
        }
    }

    /// 人读的说明：写了就用写的，没写按判法拼。
    pub fn text(&self) -> String {
        if !self.description().is_empty() {
            return self.description().to_string();
        }
        match self {
            Criterion::PathExists { path, .. } => format!("存在：{path}"),
            Criterion::PathAbsent { absent, .. } => format!("不存在：{absent}"),
            Criterion::FileContains { file, contains, .. } => {
                format!("含「{contains}」：{file}")
            }
            Criterion::CommandRun { run, .. } => format!("跑通：{run}"),
            Criterion::AgentJudgement { .. } | Criterion::HumanGate { .. } => String::new(),
        }
    }

    /// 写回定义里的字段形状。
    pub fn to_yaml(&self) -> Yaml {
        let mut pairs: Vec<(&str, &str)> = Vec::new();
        match self {
            Criterion::PathExists { path, description } => {
                pairs.push(("executor", RULE));
                pairs.push(("path", path));
                if !description.is_empty() {
                    pairs.push(("description", description));
                }
            }
            Criterion::PathAbsent {
                absent,
                description,
            } => {
                pairs.push(("executor", RULE));
                pairs.push(("absent", absent));
                if !description.is_empty() {
                    pairs.push(("description", description));
                }
            }
            Criterion::FileContains {
                file,
                contains,
                description,
            } => {
                pairs.push(("executor", RULE));
                pairs.push(("file", file));
                pairs.push(("contains", contains));
                if !description.is_empty() {
                    pairs.push(("description", description));
                }
            }
            Criterion::CommandRun { run, description } => {
                pairs.push(("executor", RULE));
                pairs.push(("run", run));
                if !description.is_empty() {
                    pairs.push(("description", description));
                }
            }
            Criterion::AgentJudgement { description } => {
                pairs.push(("executor", AGENT));
                pairs.push(("description", description));
            }
            Criterion::HumanGate { description } => {
                pairs.push(("executor", HUMAN));
                pairs.push(("description", description));
            }
        }
        let mut map = Mapping::new();
        for (key, value) in pairs {
            map.insert(
                Yaml::String(key.to_string()),
                Yaml::String(value.to_string()),
            );
        }
        Yaml::Mapping(map)
    }

    /// 占位展开：每个字段里的 `{{name}}` 交给 `resolve` 换成哪条路径。
    ///
    /// `resolve` 认不得的名字原样留着。换成哪条路径是场所的事——
    /// 见 `Workspace::expanded`。
    pub fn expanded<F>(&self, resolve: F) -> Criterion
    where
        F: Fn(&str) -> Option<String>,
    {
        let ex = |value: &str| replace_placeholders(value, &resolve);
        match self {
            Criterion::PathExists { path, description } => Criterion::PathExists {
                path: ex(path),
                description: ex(description),
            },
            Criterion::PathAbsent {
                absent,
                description,
            } => Criterion::PathAbsent {
                absent: ex(absent),
                description: ex(description),
            },
            Criterion::FileContains {
                file,
                contains,
                description,
            } => Criterion::FileContains {
                file: ex(file),
                contains: ex(contains),
                description: ex(description),
            },
            Criterion::CommandRun { run, description } => Criterion::CommandRun {
                run: ex(run),
                description: ex(description),
            },
            Criterion::AgentJudgement { description } => Criterion::AgentJudgement {
                description: ex(description),
            },
            Criterion::HumanGate { description } => Criterion::HumanGate {
                description: ex(description),
            },
        }
    }
}
