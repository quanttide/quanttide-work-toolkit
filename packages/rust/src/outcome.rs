//! 结果：一次动作的答复。
//!
//! 规范（`docs/specification/process/outcome.md`）：一次动作一份，命令行与窗口都从它取，
//! 推进任务的动作还记进流水。这一份是它的不变部分——四样（`ok` 通不通、`lines` 话、
//! `columns` 与 `rows` 同一份表格、`data` 给界面的那一栏）与编解码；话怎么拼、
//! 路径怎么显示、退出码怎么定，留各自的平台。

use serde_json::{Value as Json, json};

/// 一次动作的答复。四样：通不通、话、表格、给界面的那一栏。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Outcome {
    pub ok: bool,
    /// 给人看的话，一行一句。
    pub lines: Vec<String>,
    /// 同一份表格：表头与行，命令行与窗口共用。
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    /// 给窗口与脚本的那一栏（要交原文就托在这里）；没有就不写。
    pub data: Option<Json>,
}

impl Outcome {
    pub fn new(ok: bool) -> Self {
        Outcome {
            ok,
            ..Default::default()
        }
    }

    /// 只有话的结果。
    pub fn lines(ok: bool, lines: Vec<String>) -> Self {
        Outcome {
            ok,
            lines,
            ..Default::default()
        }
    }

    /// 往话的开头添一句。
    pub fn with_first(mut self, line: String) -> Self {
        self.lines.insert(0, line);
        self
    }

    /// 托上给界面的那一栏。
    pub fn with_data(mut self, data: Json) -> Self {
        self.data = Some(data);
        self
    }

    /// 信封：四样，`data` 有才写。
    pub fn to_json(&self) -> Json {
        let mut envelope = serde_json::Map::new();
        envelope.insert("ok".to_string(), json!(self.ok));
        envelope.insert("lines".to_string(), json!(self.lines));
        envelope.insert("columns".to_string(), json!(self.columns));
        envelope.insert("rows".to_string(), json!(self.rows));
        if let Some(data) = &self.data {
            envelope.insert("data".to_string(), data.clone());
        }
        Json::Object(envelope)
    }

    /// 原文那一栏（`--out` 落的就是它）；没托东西就给信封。
    pub fn data_json(&self) -> Json {
        match &self.data {
            Some(data) => data.clone(),
            None => self.to_json(),
        }
    }

    /// 从信封装回来。缺样按空算。
    pub fn from_json(value: &Json) -> Self {
        Outcome {
            ok: value.get("ok").and_then(Json::as_bool).unwrap_or(false),
            lines: strings(value.get("lines")),
            columns: strings(value.get("columns")),
            rows: value
                .get("rows")
                .and_then(Json::as_array)
                .map(|rows| rows.iter().map(|row| strings(Some(row))).collect())
                .unwrap_or_default(),
            data: value.get("data").cloned(),
        }
    }
}

/// 一栏字符串：不是数组按空算，元素照原样写成文字。
fn strings(value: Option<&Json>) -> Vec<String> {
    value
        .and_then(Json::as_array)
        .map(|items| {
            items
                .iter()
                .map(|item| match item {
                    Json::String(text) => text.clone(),
                    other => other.to_string(),
                })
                .collect()
        })
        .unwrap_or_default()
}
