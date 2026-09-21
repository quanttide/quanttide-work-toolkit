//! 工作记录聚合 / 模型：账上的一笔。
//!
//! 工作记录不独立落盘，内嵌在工单的 `records` 字段里，随读工单文档取用。
//! `of` / `to_yaml` 只搬运不判断；[`validate`] 把语法关——字段表之外拒、
//! 必选缺失拒、类型不符拒。「一笔在一本账里合不合法」（凭证不重、页码不跳、
//! 时刻不倒流）是账本级纪律，归 [`crate::record`] 的账本侧。
//! 出处：`docs/specification/process/work-record.md`·约束。

use crate::fields::{RECORD_FIELDS, text_of, unknown_fields};
use serde_yaml::{Mapping, Value as Yaml};

/// 必选的文本字段：缺了或空串都算没写。
const REQUIRED_TEXT: [&str; 4] = ["id", "created_at", "order_id", "step_id"];

/// 账上的一笔：什么时候、哪一站、一句话、过没过，外加两样锚点。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkRecord {
    /// 凭证号：追加时定，落笔后永不改变；引用一律认它。
    pub id: String,
    /// 页码：账本方分配，自 1 起严格递增不跳号；排序一律认它。
    pub seq: u64,
    /// 什么时候发生——取步骤发生的时刻，不取落笔时刻。
    pub created_at: String,
    /// 所属工单的凭证。
    pub order_id: String,
    /// 这一站的机器锚点，与所引工作流里的步骤同指；落笔后永不失配。
    pub step_id: String,
    /// 一句话证词，记录里唯一的自由文本；默认为空。
    pub description: String,
    /// 过没过；缺省 `false`——没记「过」就当作没过。
    pub is_succeeded: bool,
}

impl WorkRecord {
    /// 从账上的字段读出（不校验）；`is_succeeded` 缺省 `false`。
    pub fn of(value: &Yaml) -> WorkRecord {
        WorkRecord {
            id: text_of(value, "id"),
            seq: value.get("seq").and_then(|v| v.as_u64()).unwrap_or(0),
            created_at: text_of(value, "created_at"),
            order_id: text_of(value, "order_id"),
            step_id: text_of(value, "step_id"),
            description: text_of(value, "description"),
            is_succeeded: value
                .get("is_succeeded")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        }
    }

    /// 落回账上的形状：七个字段全写，不带账本外的东西。
    pub fn to_yaml(&self) -> Yaml {
        let mut map = Mapping::new();
        map.insert(Yaml::String("id".into()), Yaml::String(self.id.clone()));
        map.insert(Yaml::String("seq".into()), Yaml::Number(self.seq.into()));
        map.insert(
            Yaml::String("created_at".into()),
            Yaml::String(self.created_at.clone()),
        );
        map.insert(
            Yaml::String("order_id".into()),
            Yaml::String(self.order_id.clone()),
        );
        map.insert(
            Yaml::String("step_id".into()),
            Yaml::String(self.step_id.clone()),
        );
        map.insert(
            Yaml::String("description".into()),
            Yaml::String(self.description.clone()),
        );
        map.insert(
            Yaml::String("is_succeeded".into()),
            Yaml::Bool(self.is_succeeded),
        );
        Yaml::Mapping(map)
    }
}

/// 一笔的写法错在哪；「第 n 笔」的话头由调用方给。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyntaxFault {
    /// 这一笔不是映射。
    NotMapping,
    /// 有不认识的字段。
    UnknownFields(Vec<String>),
    /// 缺了必选字段（或空串）。
    MissingField(&'static str),
    /// `seq` 缺了，或不是自 1 起的整数。
    BadSeq,
    /// 字段写了，但类型不对。
    BadType(&'static str),
}

impl SyntaxFault {
    /// canonical 文案的尾巴。
    pub fn text(&self) -> String {
        match self {
            SyntaxFault::NotMapping => "不是映射（工作记录是七个字段的账）".to_string(),
            SyntaxFault::UnknownFields(unknown) => format!(
                "有不认识的字段：{}（只认 {}）",
                unknown.join("、"),
                RECORD_FIELDS.join("、")
            ),
            SyntaxFault::MissingField(name) => format!("少了 {name}"),
            SyntaxFault::BadSeq => "的 seq 缺了，或不是自 1 起的整数".to_string(),
            SyntaxFault::BadType(name) => format!("的 {name} 类型不对"),
        }
    }
}

/// 语法校验：不是映射、不认识的字段、必选缺失、类型不符，当场报错。
/// 一笔写得对不算账对——账本级纪律在账本侧对账时把守。
pub fn validate(value: &Yaml) -> Result<(), SyntaxFault> {
    let map = value.as_mapping().ok_or(SyntaxFault::NotMapping)?;
    let unknown = unknown_fields(map, &RECORD_FIELDS);
    if !unknown.is_empty() {
        return Err(SyntaxFault::UnknownFields(unknown));
    }
    for name in REQUIRED_TEXT {
        if text_of(value, name).is_empty() {
            return Err(SyntaxFault::MissingField(name));
        }
    }
    match value.get("seq").and_then(|v| v.as_u64()) {
        Some(seq) if seq >= 1 => {}
        _ => return Err(SyntaxFault::BadSeq),
    }
    if value
        .get("description")
        .is_some_and(|v| v.as_str().is_none())
    {
        return Err(SyntaxFault::BadType("description"));
    }
    if value
        .get("is_succeeded")
        .is_some_and(|v| v.as_bool().is_none())
    {
        return Err(SyntaxFault::BadType("is_succeeded"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn yaml(value: serde_json::Value) -> Yaml {
        serde_yaml::to_value(value).expect("JSON 装成 YAML 值")
    }

    fn sample() -> serde_json::Value {
        json!({
            "id": "0f0e5b1a-9d0c-4c7e-8d1e-2b6a5f4e3d2c",
            "seq": 1,
            "created_at": "2026-02-11T10:00:00Z",
            "order_id": "3b8a2c1d-4e5f-4a6b-8c9d-0e1f2a3b4c5d",
            "step_id": "8c7d6e5f-4a3b-4c2d-9e1f-0a2b3c4d5e6f",
            "description": "草拟了提纲",
            "is_succeeded": true
        })
    }

    #[test]
    fn valid_record_round_trips() {
        let value = yaml(sample());
        validate(&value).expect("合法一笔");
        let record = WorkRecord::of(&value);
        assert_eq!(record.seq, 1);
        assert!(record.is_succeeded);
        let again = WorkRecord::of(&record.to_yaml());
        assert_eq!(record, again, "落形再读出，一笔不变");
    }

    #[test]
    fn optional_fields_fill_in() {
        let mut bare = sample();
        bare.as_object_mut().unwrap().remove("description");
        bare.as_object_mut().unwrap().remove("is_succeeded");
        let value = yaml(bare);
        validate(&value).expect("推荐字段可省");
        let record = WorkRecord::of(&value);
        assert_eq!(record.description, "");
        assert!(!record.is_succeeded, "没记「过」就当作没过");
    }

    #[test]
    fn unknown_fields_are_rejected() {
        let mut odd = sample();
        odd.as_object_mut()
            .unwrap()
            .insert("step".into(), json!("草拟"));
        assert_eq!(
            validate(&yaml(odd)),
            Err(SyntaxFault::UnknownFields(vec!["step".into()])),
            "站名不落账，落 step_id"
        );
    }

    #[test]
    fn missing_required_is_rejected() {
        for name in REQUIRED_TEXT {
            let mut bare = sample();
            bare.as_object_mut().unwrap().remove(name);
            assert_eq!(
                validate(&yaml(bare)),
                Err(SyntaxFault::MissingField(name)),
                "缺 {name} 当场报"
            );
        }
    }

    #[test]
    fn bad_values_are_rejected() {
        for seq in [json!(0), json!(-1), json!("1")] {
            let mut odd = sample();
            odd.as_object_mut().unwrap().insert("seq".into(), seq);
            assert_eq!(validate(&yaml(odd)), Err(SyntaxFault::BadSeq));
        }
        let mut odd = sample();
        odd.as_object_mut()
            .unwrap()
            .insert("is_succeeded".into(), json!("yes"));
        assert_eq!(
            validate(&yaml(odd)),
            Err(SyntaxFault::BadType("is_succeeded"))
        );
        let mut odd = sample();
        odd.as_object_mut()
            .unwrap()
            .insert("description".into(), json!(3));
        assert_eq!(
            validate(&yaml(odd)),
            Err(SyntaxFault::BadType("description"))
        );
    }

    #[test]
    fn not_a_mapping_is_rejected() {
        assert_eq!(
            validate(&yaml(json!(["a", "b"]))),
            Err(SyntaxFault::NotMapping)
        );
    }
}
