//! 工作记录：工单流水中的单条记录。
//!
//! 一条记录记一件事的发生：发生时刻（`created_at`）、所执行步骤的锚点
//! （`step_id`）、简要描述（`description`）与判定结果（`is_succeeded`）。
//! 跨边界引用以 `id` 为准，排序以 `seq` 为准。记录不独立落盘，内嵌于
//! 工单的 `records` 字段；只增不改，落笔后原样保留，改结论的唯一方式
//! 是追加一条新记录。
//!
//! 本模块只执行单条记录的语法校验（[`validate`]）；单条记录在整本流水
//! 中是否有效——凭证唯一、页码连续、时序递增——属账本级约束，由
//! [`crate::record`] 的账本侧对账执行。

use crate::fields::{RECORD_FIELDS, text_of, unknown_fields};
use serde_yaml::{Mapping, Value as Yaml};

/// 必选的文本字段：缺失或取值为空白均视为未提供。
const REQUIRED_TEXT: [&str; 4] = ["id", "created_at", "order_id", "step_id"];

/// 工作记录实体：凭证、页码、发生时刻、工单与步骤锚点、简要描述、判定结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkRecord {
    /// ID：追加方生成，落笔后永不改变；跨边界引用一律以它为准。
    pub id: String,
    /// 编号：账本方分配，自 1 起严格递增且连续；排序一律以它为准。
    pub seq: u64,
    /// 发生时刻，而非记录录入时刻——流水是证据链，答「何时发生」。
    pub created_at: String,
    /// 所属工单ID。
    pub order_id: String,
    /// 所执行步骤ID：与所引工作流中的步骤同一指向；落笔后不可失配。
    pub step_id: String,
    /// 对已发生事实的简要描述，记录中唯一的自由文本字段；默认为空。
    pub description: String,
    /// 判定结果；缺省 `false`——未记录「通过」即视为未通过。
    pub is_succeeded: bool,
}

impl WorkRecord {
    /// 从记录字段读出（不执行校验）；`is_succeeded` 缺省为 `false`。
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

    /// 序列化为记录字段形态：完整输出七个字段，不产生字段表之外的键。
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

/// 单条记录的语法错误种类；位置信息（第 n 笔）由调用方附加。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyntaxFault {
    /// 记录不是映射结构。
    NotMapping,
    /// 包含字段表之外的字段。
    UnknownFields(Vec<String>),
    /// 缺少必选字段，或取值为空白。
    MissingField(&'static str),
    /// `seq` 缺失，或不是自 1 起的整数。
    BadSeq,
    /// 字段已声明，但取值类型不符。
    BadType(&'static str),
}

impl SyntaxFault {
    /// canonical 错误文案；不含位置前缀，由调用方拼接。
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

/// 执行单条记录的语法校验：非映射、字段表之外、必选缺失、类型不符均报错。
/// 单条语法正确不代表整本流水有效——凭证唯一、页码连续、时序递增由账本侧对账执行。
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

    /// 以 JSON 字面量构造 YAML 值，便于书写测试样本。
    fn yaml(value: serde_json::Value) -> Yaml {
        serde_yaml::to_value(value).expect("JSON 转换为 YAML 值失败")
    }

    /// 合法记录样本：七个字段齐备，`seq` 为 1，判定为通过。
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

    /// 合法记录通过语法校验，且「落形 → 再读出」往返后取值保持不变
    /// （`to_yaml` 的输出仍是合法输入，七字段逐一相等）。
    #[test]
    fn valid_record_round_trips() {
        let value = yaml(sample());
        validate(&value).expect("合法记录应通过校验");
        let record = WorkRecord::of(&value);
        assert_eq!(record.seq, 1);
        assert!(record.is_succeeded);
        assert_eq!(record, WorkRecord::of(&record.to_yaml()), "往返不变性");
    }

    /// 推荐字段可省略：`description` 缺省为空串，`is_succeeded` 缺省为
    /// `false`（规格约束：未记录「通过」即视为未通过）。
    #[test]
    fn optional_fields_fill_in() {
        let mut bare = sample();
        bare.as_object_mut().unwrap().remove("description");
        bare.as_object_mut().unwrap().remove("is_succeeded");
        let value = yaml(bare);
        validate(&value).expect("推荐字段省略不应报错");
        let record = WorkRecord::of(&value);
        assert_eq!(record.description, "");
        assert!(!record.is_succeeded);
    }

    /// 字段表之外的字段被拒收。以 `step` 为反例：站名不落账，
    /// 记录仅以 `step_id`（机器锚点）指认步骤。
    #[test]
    fn unknown_fields_are_rejected() {
        let mut odd = sample();
        odd.as_object_mut()
            .unwrap()
            .insert("step".into(), json!("草拟"));
        assert_eq!(
            validate(&yaml(odd)),
            Err(SyntaxFault::UnknownFields(vec!["step".into()])),
        );
    }

    /// 四个必选文本字段逐一缺失时，均报 `MissingField` 并指名缺失字段。
    #[test]
    fn missing_required_is_rejected() {
        for name in REQUIRED_TEXT {
            let mut bare = sample();
            bare.as_object_mut().unwrap().remove(name);
            assert_eq!(
                validate(&yaml(bare)),
                Err(SyntaxFault::MissingField(name)),
                "缺失 {name} 应报 MissingField"
            );
        }
    }

    /// 类型不符逐项拒收：`seq` 为零、负数或字符串均属 `BadSeq`
    /// （页码须为自 1 起的整数）；`is_succeeded` 非布尔、`description`
    /// 非字符串均属 `BadType`。
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

    /// 非映射输入（如列表）报 `NotMapping`。
    #[test]
    fn not_a_mapping_is_rejected() {
        assert_eq!(
            validate(&yaml(json!(["a", "b"]))),
            Err(SyntaxFault::NotMapping)
        );
    }
}
