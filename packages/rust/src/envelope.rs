//! 信封：动作结果。`ok` 定退出码，`lines` 给人看，`columns` 与 `rows` 给窗口画。

use serde_json::{Value as Json, json};

#[derive(Debug, Default, Clone)]
pub struct Outcome {
    pub ok: bool,
    pub lines: Vec<String>,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub payload: Option<Json>,
}

impl Outcome {
    pub fn new(ok: bool) -> Self {
        Outcome {
            ok,
            ..Default::default()
        }
    }

    pub fn lines(ok: bool, lines: Vec<String>) -> Self {
        Outcome {
            ok,
            lines,
            ..Default::default()
        }
    }

    pub fn with_first(mut self, line: String) -> Self {
        self.lines.insert(0, line);
        self
    }

    pub fn to_json(&self) -> Json {
        if let Some(payload) = &self.payload {
            return payload.clone();
        }
        json!({
            "ok": self.ok,
            "lines": self.lines,
            "columns": self.columns,
            "rows": self.rows,
        })
    }
}

/// 路径相对根写短一点；不在根底下就原样。
pub fn short(root: &str, path: &str) -> String {
    let prefix = if root.ends_with('/') {
        root.to_string()
    } else {
        format!("{root}/")
    };
    match path.strip_prefix(&prefix) {
        Some(rest) => rest.to_string(),
        None => path.to_string(),
    }
}
