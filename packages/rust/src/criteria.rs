//! 判据：规则引擎那几种机械核对。
//!
//! 判据是**字段**，不是一行小语法：`path` 存在、`absent` 不存在、
//! `file` + `contains` 含这段文字、`run` 这条命令退出码为零。
//! 工具箱只把判据翻成「要跑什么」——真去跑（文件系统、起进程）是各自包的事。

use crate::schema::{
    AGENT, CRITERION_FIELDS, DefinitionError, HUMAN, RULE, TYPES, text_of, unknown_fields,
};
use serde_yaml::{Mapping, Value as Yaml};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleKind {
    Path,
    Absent,
    Contains,
    Run,
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
    /// 从定义里的字段读出（不校验）。
    pub fn from_yaml(value: &Yaml) -> Criterion {
        let description = text_of(value, "description");
        let kind = text_of(value, "executor");
        if kind == AGENT {
            return Criterion::AgentJudgement { description };
        }
        if kind == HUMAN {
            return Criterion::HumanGate { description };
        }
        let path = text_of(value, "path");
        if !path.is_empty() {
            return Criterion::PathExists { path, description };
        }
        let absent = text_of(value, "absent");
        if !absent.is_empty() {
            return Criterion::PathAbsent {
                absent,
                description,
            };
        }
        let file = text_of(value, "file");
        if !file.is_empty() {
            return Criterion::FileContains {
                file,
                contains: text_of(value, "contains"),
                description,
            };
        }
        Criterion::CommandRun {
            run: text_of(value, "run"),
            description,
        }
    }

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

    /// 占位展开：每个字段里的 `{{…}}` 交给 `expand` 换掉。
    pub fn expanded<F>(&self, expand: F) -> Criterion
    where
        F: Fn(&str) -> String,
    {
        let ex = |value: &str| {
            if value.contains("{{") {
                expand(value)
            } else {
                value.to_string()
            }
        };
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

/// 读一条判据：取值不对、缺该有的字段，当场报错。
///
/// `file` 与 `place` 只用来说话；返回的是认好的值对象。
pub fn read_criterion(value: &Yaml, file: &str, place: &str) -> Result<Criterion, DefinitionError> {
    let kind = text_of(value, "executor");
    if !TYPES.contains(&kind.as_str()) {
        return Err(DefinitionError(format!(
            "{file} {place}的 executor 只能是 {}（谁判：规则引擎 / 智能体 / 人）",
            TYPES.join(" / ")
        )));
    }
    let criterion_map = value
        .as_mapping()
        .ok_or_else(|| DefinitionError(format!("{file} {place}不是映射")))?;
    let odd = unknown_fields(criterion_map, &CRITERION_FIELDS);
    if !odd.is_empty() {
        return Err(DefinitionError(format!(
            "{file} {place}有不认识的字段：{}（只认 {}）",
            odd.join("、"),
            CRITERION_FIELDS.join("、")
        )));
    }
    let given: Vec<&str> = ["path", "absent", "file", "contains", "run"]
        .into_iter()
        .filter(|name| value.get(*name).is_some())
        .collect();
    if kind == RULE {
        if given.is_empty() {
            return Err(DefinitionError(format!(
                "{file} {place}是 rule，得写一条判法（path / absent / file+contains / run）"
            )));
        }
        if given.contains(&"contains") && !given.contains(&"file") {
            return Err(DefinitionError(format!(
                "{file} {place}写了 contains，还得写 file"
            )));
        }
        if given.contains(&"file") && !given.contains(&"contains") {
            return Err(DefinitionError(format!(
                "{file} {place}写了 file，还得写 contains"
            )));
        }
        let others: Vec<&str> = given
            .iter()
            .copied()
            .filter(|name| *name != "file" && *name != "contains")
            .collect();
        if others.len() > 1 || (!others.is_empty() && given.contains(&"file")) {
            return Err(DefinitionError(format!(
                "{file} {place}的判法只能一种：path / absent / file+contains / run"
            )));
        }
    } else {
        if text_of(value, "description").is_empty() {
            return Err(DefinitionError(format!(
                "{file} {place}是 {kind}，必须写 description（判准 / 要人拍板的事）"
            )));
        }
        if !given.is_empty() {
            return Err(DefinitionError(format!(
                "{file} {place}是 {kind}，不该带 {}（那是 rule 的字段）",
                given.join("、")
            )));
        }
    }
    Ok(Criterion::from_yaml(value))
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

/// 把判据翻成要跑的东西：rule 的跑，agent / human 的不跑。
pub fn items_of(criteria: &[Criterion]) -> Vec<RuleItem> {
    criteria
        .iter()
        .map(|criterion| {
            let description = criterion.text();
            let (kind, args) = match criterion {
                Criterion::PathExists { path, .. } => (Some(RuleKind::Path), vec![path.clone()]),
                Criterion::PathAbsent { absent, .. } => {
                    (Some(RuleKind::Absent), vec![absent.clone()])
                }
                Criterion::FileContains { file, contains, .. } => (
                    Some(RuleKind::Contains),
                    vec![file.clone(), contains.clone()],
                ),
                Criterion::CommandRun { run, .. } => (Some(RuleKind::Run), vec![run.clone()]),
                Criterion::AgentJudgement { .. } | Criterion::HumanGate { .. } => {
                    (None, Vec::new())
                }
            };
            RuleItem {
                description,
                kind,
                args,
            }
        })
        .collect()
}
