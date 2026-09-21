//! 工作记录：工单流水中的单条记录。
//!
//! 一条记录记一件事的发生：发生时刻（`created_at`）、所执行步骤的锚点
//! （`step_id`）、简要描述（`description`）与判定结果（`is_succeeded`）。
//! 跨边界引用以 `id` 为准，排序以 `seq` 为准。记录不独立落盘，内嵌于
//! 工单的 `records` 字段；只增不改，落笔后原样保留，改结论的唯一方式
//! 是追加一条新记录。
//!
//! 读出即校验：记录只从 JSONL 行读入——[`WorkRecord::from_jsonl`] 解析
//! 单行 JSON，语法校验与构造一次完成（字段表之外、必选缺失、类型不符均
//! 报错），读出的一笔必是语法合法的，非法记录不存在已构造的形态。落形为
//! [`WorkRecord::to_jsonl`] 的单行 JSON。单条记录在整本流水中是否有效——
//! 凭证唯一、页码连续、时序递增——属账本级约束，由 [`crate::record`]
//! 的账本侧对账执行。

use super::errors::Fault;
use crate::fields::RECORD_FIELDS;
use serde_json::Value as Json;

/// 必选的文本字段：缺失或取值为空白均视为未提供。
const REQUIRED_TEXT: [&str; 4] = ["id", "created_at", "order_id", "step_id"];

/// 工作记录实体：凭证、页码、发生时刻、工单与步骤锚点、简要描述、判定结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkRecord {
    /// 凭证号：追加方生成，落笔后永不改变；跨边界引用一律以它为准。
    pub id: String,
    /// 页码：账本方分配，自 1 起严格递增且连续；排序一律以它为准。
    pub seq: u64,
    /// 步骤发生时刻，而非记录录入时刻——流水是证据链，答「何时发生」。
    pub created_at: String,
    /// 所属工单的凭证。
    pub order_id: String,
    /// 所执行步骤的机器锚点，与所引工作流中的步骤同一指向；落笔后不可失配。
    pub step_id: String,
    /// 对已发生事实的简要描述，记录中唯一的自由文本字段；默认为空。
    pub description: String,
    /// 判定结果；缺省 `false`——未记录「通过」即视为未通过。
    pub is_succeeded: bool,
}

impl WorkRecord {
    /// 从 JSONL 行读出：解析单行 JSON、语法校验、构造一次完成。字段表
    /// 之外、必选缺失、类型不符均报错；行不是合法 JSON 时报
    /// [`Fault::NotJson`]，`is_succeeded` 缺省为 `false`。
    pub fn from_jsonl(line: &str) -> Result<WorkRecord, Fault> {
        let value: Json = serde_json::from_str(line).map_err(|_| Fault::NotJson)?;
        let map = value.as_object().ok_or(Fault::NotMapping)?;
        // 取文本字段：去掉两侧空白，取值不是字符串时视为未提供。
        let text = |key: &str| {
            value
                .get(key)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string()
        };
        let unknown: Vec<String> = map
            .keys()
            .filter(|key| !RECORD_FIELDS.contains(&key.as_str()))
            .cloned()
            .collect();
        if !unknown.is_empty() {
            return Err(Fault::UnknownFields(unknown));
        }
        for name in REQUIRED_TEXT {
            if text(name).is_empty() {
                return Err(Fault::MissingField(name));
            }
        }
        let seq = match value.get("seq").and_then(|v| v.as_u64()) {
            Some(seq) if seq >= 1 => seq,
            _ => return Err(Fault::BadSeq),
        };
        if value
            .get("description")
            .is_some_and(|v| v.as_str().is_none())
        {
            return Err(Fault::BadType("description"));
        }
        if value
            .get("is_succeeded")
            .is_some_and(|v| v.as_bool().is_none())
        {
            return Err(Fault::BadType("is_succeeded"));
        }
        Ok(WorkRecord {
            id: text("id"),
            seq,
            created_at: text("created_at"),
            order_id: text("order_id"),
            step_id: text("step_id"),
            description: text("description"),
            is_succeeded: value
                .get("is_succeeded")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        })
    }

    /// 落形为 JSONL 行：单行紧凑 JSON，七个字段全写，不产生字段表之外
    /// 的键；行与行之间的分隔符由写入方负责。
    pub fn to_jsonl(&self) -> String {
        serde_json::json!({
            "id": self.id,
            "seq": self.seq,
            "created_at": self.created_at,
            "order_id": self.order_id,
            "step_id": self.step_id,
            "description": self.description,
            "is_succeeded": self.is_succeeded,
        })
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// 把 JSON 字面量装成一行，供 [`WorkRecord::from_jsonl`] 测试使用。
    fn line(value: serde_json::Value) -> String {
        value.to_string()
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

    /// 合法记录读出成功，取值逐一相符；且「`to_jsonl` 落形 →
    /// `from_jsonl` 读出」往返后保持不变。
    #[test]
    fn valid_record_round_trips() {
        let record = WorkRecord::from_jsonl(&line(sample())).expect("合法记录应读出成功");
        assert_eq!(record.seq, 1);
        assert!(record.is_succeeded);
        let again =
            WorkRecord::from_jsonl(&record.to_jsonl()).expect("落形后的 JSONL 行应是合法输入");
        assert_eq!(record, again, "往返不变性");
    }

    /// 非法 JSON 行报 `NotJson`；合法 JSON 但不是映射（如数组）报
    /// `NotMapping`——解析与校验在同一次读出内完成。
    #[test]
    fn malformed_lines_are_rejected() {
        assert_eq!(WorkRecord::from_jsonl("{oops"), Err(Fault::NotJson));
        assert_eq!(WorkRecord::from_jsonl("[1, 2]"), Err(Fault::NotMapping));
    }

    /// 推荐字段可省略：`description` 缺省为空串，`is_succeeded` 缺省为
    /// `false`（规格约束：未记录「通过」即视为未通过）。
    #[test]
    fn optional_fields_fill_in() {
        let mut bare = sample();
        bare.as_object_mut().unwrap().remove("description");
        bare.as_object_mut().unwrap().remove("is_succeeded");
        let record = WorkRecord::from_jsonl(&line(bare)).expect("推荐字段省略不应报错");
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
            WorkRecord::from_jsonl(&line(odd)),
            Err(Fault::UnknownFields(vec!["step".into()])),
        );
    }

    /// 四个必选文本字段逐一缺失时，均报 `MissingField` 并指名缺失字段。
    #[test]
    fn missing_required_is_rejected() {
        for name in REQUIRED_TEXT {
            let mut bare = sample();
            bare.as_object_mut().unwrap().remove(name);
            assert_eq!(
                WorkRecord::from_jsonl(&line(bare)),
                Err(Fault::MissingField(name)),
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
            assert_eq!(WorkRecord::from_jsonl(&line(odd)), Err(Fault::BadSeq));
        }
        let mut odd = sample();
        odd.as_object_mut()
            .unwrap()
            .insert("is_succeeded".into(), json!("yes"));
        assert_eq!(
            WorkRecord::from_jsonl(&line(odd)),
            Err(Fault::BadType("is_succeeded"))
        );
        let mut odd = sample();
        odd.as_object_mut()
            .unwrap()
            .insert("description".into(), json!(3));
        assert_eq!(
            WorkRecord::from_jsonl(&line(odd)),
            Err(Fault::BadType("description"))
        );
    }
}
