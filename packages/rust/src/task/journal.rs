//! 任务聚合 / 流水：什么时候、哪一步、一句话、过没过。
//!
//! 流水只增不改，一条的记录见 [`JournalEvent`]。
//! 「走过哪几步」是跨着任务与定义的操作，在 `crate::workspace`。
//! 出处：`docs/specification/process/task.md`·语法。

use crate::fields::text_of;
use serde_yaml::{Mapping, Value as Yaml};

/// 流水里的一条：什么时候、哪一步、一句话、过没过。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalEvent {
    pub at: String,
    pub step: String,
    pub detail: String,
    pub ok: bool,
}

impl JournalEvent {
    pub fn of(value: &Yaml) -> JournalEvent {
        JournalEvent {
            at: text_of(value, "at"),
            step: text_of(value, "step"),
            detail: text_of(value, "detail"),
            ok: value.get("ok").and_then(|v| v.as_bool()).unwrap_or(false),
        }
    }

    pub fn to_yaml(&self) -> Yaml {
        let mut map = Mapping::new();
        for (key, value) in [
            ("at", &self.at),
            ("step", &self.step),
            ("detail", &self.detail),
        ] {
            map.insert(
                Yaml::String(key.to_string()),
                Yaml::String(value.to_string()),
            );
        }
        map.insert(Yaml::String("ok".into()), Yaml::Bool(self.ok));
        Yaml::Mapping(map)
    }
}
