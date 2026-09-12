//! 任务聚合 / 运行上下文：工作区根、数据仓、工作流目录。
//!
//! 三个字段怎么读、怎么写由各自的包定（工具箱只管托着它们）。
//! 出处：`docs/specification/process/task.md`·语法（运行上下文）。

use crate::workflow::text_of;
use serde_yaml::{Mapping, Value as Yaml};

/// 这次执行自带的运行上下文：工作区根、数据仓、工作流目录。
///
/// 三个字段怎么读、怎么写由各自的包定（工具箱只管托着它们）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RunContext {
    pub root: String,
    pub data: String,
    pub workflows: String,
}

impl RunContext {
    pub fn of(value: &Yaml) -> RunContext {
        RunContext {
            root: text_of(value, "root"),
            data: text_of(value, "data"),
            workflows: text_of(value, "workflows"),
        }
    }

    pub fn to_yaml(&self) -> Yaml {
        let mut map = Mapping::new();
        for (key, value) in [
            ("root", &self.root),
            ("data", &self.data),
            ("workflows", &self.workflows),
        ] {
            map.insert(
                Yaml::String(key.to_string()),
                Yaml::String(value.to_string()),
            );
        }
        Yaml::Mapping(map)
    }
}
