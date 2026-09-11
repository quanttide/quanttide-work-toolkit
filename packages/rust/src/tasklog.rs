//! 任务流水与「走过」的算法。
//!
//! 一条流水＝时间、步骤名、一句话、过没过。步骤名带后缀的（`·审` 审查、`·判` 机器判据）
//! **给这一步的结论投票**；不带后缀的（重新执行一次）**把结论从头算**。

use serde_json::Value as Json;

pub fn text_of(value: &Json, key: &str) -> String {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string()
}

/// 哪些步骤走过了。`steps` 是工作流上的步骤名，按定义顺序。
pub fn done(steps: &[String], events: &[Json]) -> Vec<String> {
    let mut verdict: Vec<(String, bool)> = Vec::new();
    for event in events {
        let ok = event.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
        let raw = text_of(event, "step");
        let (step, extra) = match raw.split_once('·') {
            Some((step, rest)) => (step.to_string(), Some(rest.to_string())),
            None => (raw.clone(), None),
        };
        if !steps.contains(&step) {
            continue;
        }
        match verdict.iter_mut().find(|(name, _)| *name == step) {
            Some((_, last)) => {
                if extra.is_some() {
                    *last = *last && ok;
                } else {
                    *last = ok;
                }
            }
            None => verdict.push((step, ok)),
        }
    }
    verdict
        .into_iter()
        .filter(|(_, ok)| *ok)
        .map(|(name, _)| name)
        .collect()
}

/// 第一个没走到的步骤。
pub fn next_step(steps: &[String], events: &[Json]) -> Option<String> {
    let finished = done(steps, events);
    steps.iter().find(|name| !finished.contains(name)).cloned()
}

/// 状态行：下一步是谁，或者都走过了。
pub fn state_line(steps: &[String], events: &[Json], workflow_name: &str) -> String {
    if steps.is_empty() {
        return format!("这条工作流没有步骤——在 workflows/{workflow_name}.yaml 的 steps 里写步骤");
    }
    match next_step(steps, events) {
        Some(step) => format!("下一步：{step}"),
        None => format!("{} 个步骤都走过了", steps.len()),
    }
}
