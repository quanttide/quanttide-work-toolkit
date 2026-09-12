//! 工作流聚合：一串有序的步骤。
//!
//! 定义的**语法与不变量**都在这个文件里——字段名、取值、判据种类，不认识、缺了、
//! 越界，当场报错。规矩的出处是 `docs/specification/process/workflow.md`·语法。
//!
//! 模型不可变：[`Workflow::from_yaml`] 读进来顺带校验，[`Workflow::of`] 读已经校验过的，
//! [`Workflow::to_yaml`] 写成同样的字段形状。YAML 怎么读写是各自包的事。

use crate::criterion::Criterion;
use crate::executor::{AGENT, CRITERION_TYPES, EXECUTORS, HUMAN, RULE};
use serde_yaml::{Mapping, Value as Yaml};

/// 定义顶层认得的字段。
const TOP_FIELDS: [&str; 3] = ["name", "description", "steps"];

/// 步骤认得的字段。
const STEP_FIELDS: [&str; 4] = ["name", "description", "executor", "criteria"];

/// 一条判据认得的字段。
const CRITERION_FIELDS: [&str; 7] = [
    "executor",
    "description",
    "path",
    "absent",
    "file",
    "contains",
    "run",
];

/// 一份定义读不通：字段缺了、取值越界、有不认识的字段。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefinitionError(pub String);

impl std::fmt::Display for DefinitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for DefinitionError {}

/// 取一个字符串字段，去掉两侧空白；不是字符串就当没写。
pub fn text_of(value: &Yaml, key: &str) -> String {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string()
}

/// 这次给的字段里，哪些是不认识的。
pub fn unknown_fields(mapping: &Mapping, allowed: &[&str]) -> Vec<String> {
    mapping
        .keys()
        .filter_map(|key| key.as_str())
        .filter(|key| !allowed.contains(key))
        .map(|key| key.to_string())
        .collect()
}

/// 从定义里的字段认出一条判据（不校验）。
pub fn criterion_of(value: &Yaml) -> Criterion {
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

/// 读一条判据：取值不对、缺该有的字段，当场报错。
///
/// `file` 与 `place` 只用来说话；返回的是认好的值对象。
pub fn read_criterion(value: &Yaml, file: &str, place: &str) -> Result<Criterion, DefinitionError> {
    let kind = text_of(value, "executor");
    if !CRITERION_TYPES.contains(&kind.as_str()) {
        return Err(DefinitionError(format!(
            "{file} {place}的 executor 只能是 {}（谁判：规则引擎 / 智能体 / 人）",
            CRITERION_TYPES.join(" / ")
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
    Ok(criterion_of(value))
}

/// 读一份定义：不是映射、缺字段、取值不对，当场报错。`file` 只用来说话。
pub fn validate(payload: &Yaml, file: &str) -> Result<(), DefinitionError> {
    Workflow::from_yaml(payload, file).map(|_| ())
}

/// 一个工作步骤：叫什么、做什么、谁执行、怎么算完。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub name: String,
    pub description: String,
    pub executor: String,
    pub criteria: Vec<Criterion>,
}

impl Step {
    /// 从定义里的字段读出（不校验）。用在已经校验过的定义上。
    pub fn of(value: &Yaml) -> Step {
        let criteria = value
            .get("criteria")
            .and_then(|v| v.as_sequence())
            .map(|items| items.iter().map(criterion_of).collect())
            .unwrap_or_default();
        let mut executor = text_of(value, "executor");
        if executor.is_empty() {
            executor = AGENT.to_string();
        }
        Step {
            name: text_of(value, "name"),
            description: text_of(value, "description"),
            executor,
            criteria,
        }
    }

    /// 从定义里的字段读出，顺带把语法过一遍。
    pub fn from_yaml(value: &Yaml, file: &str, position: usize) -> Result<Step, DefinitionError> {
        let step_map = value
            .as_mapping()
            .ok_or_else(|| DefinitionError(format!("{file} 第 {position} 个步骤少了 name")))?;
        if text_of(value, "name").is_empty() {
            return Err(DefinitionError(format!(
                "{file} 第 {position} 个步骤少了 name"
            )));
        }
        let extra = unknown_fields(step_map, &STEP_FIELDS);
        if !extra.is_empty() {
            return Err(DefinitionError(format!(
                "{file} 第 {position} 个步骤有不认识的字段：{}（只认 {}）",
                extra.join("、"),
                STEP_FIELDS.join("、")
            )));
        }
        let mut executor = text_of(value, "executor");
        if executor.is_empty() {
            executor = AGENT.to_string();
        }
        if !EXECUTORS.contains(&executor.as_str()) {
            return Err(DefinitionError(format!(
                "{file} 第 {position} 个步骤的 executor 只能是 {}，实得 {executor}",
                EXECUTORS.join(" 或 ")
            )));
        }
        let criteria: &[Yaml] = match value.get("criteria") {
            None | Some(Yaml::Null) => &[],
            Some(Yaml::Sequence(items)) => items.as_slice(),
            Some(_) => {
                return Err(DefinitionError(format!(
                    "{file} 第 {position} 个步骤的 criteria 应当是列表"
                )));
            }
        };
        let mut parsed = Vec::with_capacity(criteria.len());
        for (order, criterion) in criteria.iter().enumerate() {
            let place = format!("第 {position} 个步骤第 {} 条判据", order + 1);
            parsed.push(read_criterion(criterion, file, &place)?);
        }
        Ok(Step {
            name: text_of(value, "name"),
            description: text_of(value, "description"),
            executor,
            criteria: parsed,
        })
    }

    /// 这一步的执行者是不是人。
    pub fn human(&self) -> bool {
        self.executor == HUMAN
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn description(&self) -> String {
        self.description.clone()
    }

    pub fn executor(&self) -> String {
        self.executor.clone()
    }

    pub fn criteria(&self) -> Vec<Criterion> {
        self.criteria.clone()
    }

    pub fn rules(&self) -> Vec<Criterion> {
        self.of_kind(RULE)
    }

    pub fn agents(&self) -> Vec<Criterion> {
        self.of_kind(AGENT)
    }

    pub fn gates(&self) -> Vec<Criterion> {
        self.of_kind(HUMAN)
    }

    fn of_kind(&self, kind: &str) -> Vec<Criterion> {
        self.criteria
            .iter()
            .filter(|item| item.executor() == kind)
            .cloned()
            .collect()
    }

    /// 写回定义里的字段形状。
    pub fn to_yaml(&self) -> Yaml {
        let mut map = Mapping::new();
        map.insert(Yaml::String("name".into()), Yaml::String(self.name.clone()));
        if !self.description.is_empty() {
            map.insert(
                Yaml::String("description".into()),
                Yaml::String(self.description.clone()),
            );
        }
        map.insert(
            Yaml::String("executor".into()),
            Yaml::String(self.executor.clone()),
        );
        if !self.criteria.is_empty() {
            map.insert(
                Yaml::String("criteria".into()),
                Yaml::Sequence(self.criteria.iter().map(Criterion::to_yaml).collect()),
            );
        }
        Yaml::Mapping(map)
    }
}

/// 工作流聚合：一串步骤（不含文件位置——那是各自包的事）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workflow {
    pub name: String,
    pub description: String,
    pub steps: Vec<Step>,
}

impl Workflow {
    /// 从定义里的字段读出（不校验）；`name` 由调用方给（比如文件名）。
    pub fn of(name: &str, payload: &Yaml) -> Workflow {
        Workflow {
            name: name.to_string(),
            description: text_of(payload, "description"),
            steps: payload
                .get("steps")
                .and_then(|v| v.as_sequence())
                .map(|items| items.iter().map(Step::of).collect())
                .unwrap_or_default(),
        }
    }

    /// 从定义里的字段读出，顺带把语法过一遍。
    pub fn from_yaml(payload: &Yaml, file: &str) -> Result<Workflow, DefinitionError> {
        let top = payload
            .as_mapping()
            .ok_or_else(|| DefinitionError(format!("{file} 的顶层不是映射（name / steps）")))?;
        if text_of(payload, "name").is_empty() {
            return Err(DefinitionError(format!("{file} 少了 name")));
        }
        let steps = payload
            .get("steps")
            .and_then(|v| v.as_sequence())
            .filter(|items| !items.is_empty())
            .ok_or_else(|| DefinitionError(format!("{file} 少了 steps（至少一个步骤）")))?;
        let unknown = unknown_fields(top, &TOP_FIELDS);
        if !unknown.is_empty() {
            return Err(DefinitionError(format!(
                "{file} 顶层有不认识的字段：{}（只认 {}）",
                unknown.join("、"),
                TOP_FIELDS.join("、")
            )));
        }
        let mut parsed = Vec::with_capacity(steps.len());
        for (index, step) in steps.iter().enumerate() {
            parsed.push(Step::from_yaml(step, file, index + 1)?);
        }
        Ok(Workflow {
            name: text_of(payload, "name"),
            description: text_of(payload, "description"),
            steps: parsed,
        })
    }

    /// 读已校验的定义；`name` 由调用方给（比如文件名）。
    pub fn new(name: &str, payload: &Yaml) -> Workflow {
        Workflow::of(name, payload)
    }

    pub fn description(&self) -> String {
        self.description.clone()
    }

    pub fn steps(&self) -> Vec<Step> {
        self.steps.clone()
    }

    /// 步骤名，按定义顺序。
    pub fn step_names(&self) -> Vec<String> {
        self.steps.iter().map(|step| step.name.clone()).collect()
    }

    pub fn step(&self, name: &str) -> Option<Step> {
        self.steps.iter().find(|step| step.name == name).cloned()
    }

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

    /// 写回定义里的字段形状。
    pub fn to_yaml(&self) -> Yaml {
        let mut map = Mapping::new();
        map.insert(Yaml::String("name".into()), Yaml::String(self.name.clone()));
        if !self.description.is_empty() {
            map.insert(
                Yaml::String("description".into()),
                Yaml::String(self.description.clone()),
            );
        }
        map.insert(
            Yaml::String("steps".into()),
            Yaml::Sequence(self.steps.iter().map(Step::to_yaml).collect()),
        );
        Yaml::Mapping(map)
    }
}

/// 定义核对出来的一件事：在哪里、核的是什么、过没过。
///
/// 它是「把这条定义对着工作区核一遍」的回执，不单列成一个文件——
/// `workflow --check` 的实现产物，规范里还没有这一节。
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

/// 判据里的占位先按数据仓展开（够核对用）。
pub fn expand_placeholders(value: &str, data: &str) -> String {
    value
        .replace("{{artifacts}}", &format!("{data}/artifacts"))
        .replace("{{report}}", &format!("{data}/artifacts/report"))
        .replace("{{journal}}", &format!("{data}/artifacts/journal"))
        .replace("{{log}}", &format!("{data}/tasks"))
}
