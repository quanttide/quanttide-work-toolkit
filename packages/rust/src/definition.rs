//! 工作流定义：一串有序的步骤。
//!
//! 定义要有固定的意义，所以字段名、取值、判据种类都由 schema 定死，不认识的字段直接报错。
//! 这一层只管**已经解析好的 JSON 值**；YAML 怎么读进来，各语言各自的库去管。

use serde_json::Value as Json;

pub const AGENT: &str = "agent";
pub const HUMAN: &str = "human";
pub const RULE: &str = "rule";
pub const EXECUTORS: [&str; 2] = [AGENT, HUMAN];
pub const TYPES: [&str; 3] = [RULE, AGENT, HUMAN];
pub const TOP_FIELDS: [&str; 3] = ["name", "description", "steps"];
pub const STEP_FIELDS: [&str; 4] = ["name", "description", "executor", "criteria"];
pub const CRITERION_FIELDS: [&str; 7] = [
    "executor",
    "description",
    "path",
    "absent",
    "file",
    "contains",
    "run",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefinitionError(pub String);

impl std::fmt::Display for DefinitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for DefinitionError {}

pub fn text_of(value: &Json, key: &str) -> String {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string()
}

fn unknown_fields(mapping: &serde_json::Map<String, Json>, allowed: &[&str]) -> Vec<String> {
    mapping
        .keys()
        .filter(|key| !allowed.contains(&key.as_str()))
        .map(|key| key.to_string())
        .collect()
}

/// 读一份定义：不是映射、缺字段、取值不对，当场报错。`file` 只用来说话。
pub fn validate(payload: &Json, file: &str) -> Result<(), DefinitionError> {
    let top = payload
        .as_object()
        .ok_or_else(|| DefinitionError(format!("{file} 的顶层不是映射（name / steps）")))?;
    if text_of(payload, "name").is_empty() {
        return Err(DefinitionError(format!("{file} 少了 name")));
    }
    let steps = payload
        .get("steps")
        .and_then(|v| v.as_array())
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
    for (index, step) in steps.iter().enumerate() {
        let position = index + 1;
        let step_map = step
            .as_object()
            .ok_or_else(|| DefinitionError(format!("{file} 第 {position} 个步骤少了 name")))?;
        if text_of(step, "name").is_empty() {
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
        let mut executor = text_of(step, "executor");
        if executor.is_empty() {
            executor = AGENT.to_string();
        }
        if !EXECUTORS.contains(&executor.as_str()) {
            return Err(DefinitionError(format!(
                "{file} 第 {position} 个步骤的 executor 只能是 {}，实得 {executor}",
                EXECUTORS.join(" 或 ")
            )));
        }
        let criteria: &[Json] = match step.get("criteria") {
            None | Some(Json::Null) => &[],
            Some(Json::Array(items)) => items.as_slice(),
            Some(_) => {
                return Err(DefinitionError(format!(
                    "{file} 第 {position} 个步骤的 criteria 应当是列表"
                )));
            }
        };
        for (order, criterion) in criteria.iter().enumerate() {
            let place = format!("第 {position} 个步骤第 {} 条判据", order + 1);
            let kind = text_of(criterion, "executor");
            if !TYPES.contains(&kind.as_str()) {
                return Err(DefinitionError(format!(
                    "{file} {place}的 executor 只能是 {}（谁判：规则引擎 / 智能体 / 人）",
                    TYPES.join(" / ")
                )));
            }
            let criterion_map = criterion
                .as_object()
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
                .filter(|name| criterion.get(*name).is_some())
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
                if text_of(criterion, "description").is_empty() {
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
        }
    }
    Ok(())
}

/// 一个工作步骤：叫什么、做什么、谁执行、怎么算完。
#[derive(Clone)]
pub struct Step {
    payload: Json,
}

impl Step {
    pub fn of(payload: Json) -> Self {
        Step { payload }
    }

    pub fn name(&self) -> String {
        text_of(&self.payload, "name")
    }

    pub fn description(&self) -> String {
        text_of(&self.payload, "description")
    }

    pub fn executor(&self) -> String {
        let value = text_of(&self.payload, "executor");
        if value.is_empty() {
            AGENT.to_string()
        } else {
            value
        }
    }

    pub fn human(&self) -> bool {
        self.executor() == HUMAN
    }

    pub fn criteria(&self) -> Vec<Json> {
        self.payload
            .get("criteria")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default()
    }

    pub fn of_kind(&self, kind: &str) -> Vec<Json> {
        self.criteria()
            .into_iter()
            .filter(|item| text_of(item, "executor") == kind)
            .collect()
    }

    pub fn rules(&self) -> Vec<Json> {
        self.of_kind(RULE)
    }
    pub fn agents(&self) -> Vec<Json> {
        self.of_kind(AGENT)
    }
    pub fn gates(&self) -> Vec<Json> {
        self.of_kind(HUMAN)
    }
}

/// 过程的编排定义：一串步骤（不含文件位置——那是各自包的事）。
#[derive(Clone)]
pub struct Workflow {
    pub name: String,
    pub payload: Json,
}

impl Workflow {
    pub fn new(name: &str, payload: Json) -> Self {
        Workflow {
            name: name.to_string(),
            payload,
        }
    }

    pub fn description(&self) -> String {
        text_of(&self.payload, "description")
    }

    /// 步骤：按定义里的顺序——这就是「串联」。
    pub fn steps(&self) -> Vec<Step> {
        self.payload
            .get("steps")
            .and_then(|v| v.as_array())
            .map(|items| items.iter().cloned().map(Step::of).collect())
            .unwrap_or_default()
    }

    pub fn step(&self, name: &str) -> Option<Step> {
        self.steps().into_iter().find(|step| step.name() == name)
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
    for step in flow.steps() {
        for criterion in step.rules() {
            let literal = match criterion.get("path").or_else(|| criterion.get("file")) {
                Some(Json::String(text)) => text.clone(),
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
                where_: format!("{}·{}", step.name(), literal),
                what: format!("判据里的路径在不在：{written}"),
                ok: exists(&written),
            });
        }
    }

    let covered: Vec<String> = flow
        .steps()
        .into_iter()
        .flat_map(|step| step.rules())
        .filter_map(|criterion| {
            criterion
                .get("contains")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
        .collect();
    let mut mentioned: Vec<String> = Vec::new();
    for step in flow.steps() {
        let text = step.description();
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
