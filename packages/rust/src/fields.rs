//! 从定义里读字段：取值助手与各层认得的字段表。
//!
//! 这是全库共用的「读定义」公件，不属于任何聚合——放在中立处，聚合只向下依赖它。

use serde_yaml::{Mapping, Value as Yaml};

/// 定义顶层认得的字段。
pub(crate) const TOP_FIELDS: [&str; 3] = ["name", "description", "steps"];

/// 步骤认得的字段。
pub(crate) const STEP_FIELDS: [&str; 4] = ["name", "description", "executor", "criteria"];

/// 一条判据认得的字段。
pub(crate) const CRITERION_FIELDS: [&str; 7] = [
    "executor",
    "description",
    "path",
    "absent",
    "file",
    "contains",
    "run",
];

/// 取一个字符串字段，去掉两侧空白；不是字符串就当没写。
pub(crate) fn text_of(value: &Yaml, key: &str) -> String {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string()
}

/// 这次给的字段里，哪些是不认识的。
pub(crate) fn unknown_fields(mapping: &Mapping, allowed: &[&str]) -> Vec<String> {
    mapping
        .keys()
        .filter_map(|key| key.as_str())
        .filter(|key| !allowed.contains(key))
        .map(|key| key.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn yaml(value: serde_json::Value) -> Yaml {
        serde_yaml::to_value(value).expect("JSON 装成 YAML 值")
    }

    #[test]
    fn text_of_trims_and_ignores_non_strings() {
        let value = yaml(json!({"name": "  demo  ", "count": 3, "flag": true}));
        assert_eq!(text_of(&value, "name"), "demo");
        assert_eq!(text_of(&value, "count"), "", "数字不当字符串");
        assert_eq!(text_of(&value, "flag"), "");
        assert_eq!(text_of(&value, "missing"), "");
    }

    #[test]
    fn unknown_fields_names_the_odd_ones() {
        let mut mapping = Mapping::new();
        mapping.insert(Yaml::String("name".into()), Yaml::String("x".into()));
        mapping.insert(Yaml::String("extra".into()), Yaml::Bool(true));
        mapping.insert(Yaml::Number(1.into()), Yaml::Null);
        assert_eq!(unknown_fields(&mapping, &["name"]), vec!["extra"]);
        assert!(unknown_fields(&mapping, &["name", "extra"]).is_empty());
    }
}
