//! 工作流定义：一串有序的步骤。
//!
//! 字段名、取值、判据种类由 `schema` 定死，不认识的字段直接报错。
//! 模型是不可变的值：从已经解析好的 YAML 读进来（`from_yaml`），
//! 出去写成同样的字段形状（`to_yaml`）。YAML 怎么读写，各语言各自的库去管。

use crate::criteria::{Criterion, read_criterion};
use crate::schema::{DefinitionError, STEP_FIELDS, TOP_FIELDS, unknown_fields};
use serde_yaml::{Mapping, Value as Yaml};

// 字段表与取值：一处定义、两侧共用。
pub use crate::schema::{AGENT, CRITERION_FIELDS, EXECUTORS, HUMAN, RULE, TYPES, text_of};

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
            .map(|items| items.iter().map(Criterion::from_yaml).collect())
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

    /// 从定义里的字段读出，顺带校验。
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

    pub fn human(&self) -> bool {
        self.executor == HUMAN
    }

    // 旧访问器：字段与同名方法并存，消费方不用改。
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

/// 过程的编排定义：一串步骤（不含文件位置——那是各自包的事）。
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

    /// 从定义里的字段读出，顺带校验。
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

    /// 读已校验的定义（旧访问器）。`name` 由调用方给（比如文件名）。
    pub fn new(name: &str, payload: Yaml) -> Workflow {
        Workflow::of(name, &payload)
    }

    pub fn description(&self) -> String {
        self.description.clone()
    }

    pub fn steps(&self) -> Vec<Step> {
        self.steps.clone()
    }

    pub fn step(&self, name: &str) -> Option<Step> {
        self.steps.iter().find(|step| step.name == name).cloned()
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

// ---- 定义核对 ----

/// 一条定义核对出来的一件事。
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

/// 核对一条工作流：判据里的路径在不在；描述里提到的报告小节有没有判据覆盖。
///
/// `exists` 由调用方给——工具箱不碰文件系统。
pub fn check<F>(flow: &Workflow, data: &str, exists: F) -> Vec<Finding>
where
    F: Fn(&str) -> bool,
{
    let mut found: Vec<Finding> = Vec::new();
    for step in &flow.steps {
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

    let covered: Vec<String> = flow
        .steps
        .iter()
        .flat_map(|step| step.rules())
        .filter_map(|criterion| match criterion {
            Criterion::FileContains { contains, .. } => Some(contains),
            _ => None,
        })
        .collect();
    let mut mentioned: Vec<String> = Vec::new();
    for step in &flow.steps {
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

/// 核对结果写成人读的一段。
pub fn describe(found: &[Finding]) -> Vec<String> {
    let mut lines = vec![format!("核对 {} 件事", found.len())];
    for item in found {
        lines.push(format!(
            "  {} {}——{}",
            if item.ok { "✓" } else { "✗" },
            item.where_,
            item.what
        ));
    }
    if found.is_empty() {
        lines.push("  （这条定义里没有可核对的路径与小节）".to_string());
    }
    lines
}

pub fn all_ok(found: &[Finding]) -> bool {
    found.iter().all(|item| item.ok)
}
