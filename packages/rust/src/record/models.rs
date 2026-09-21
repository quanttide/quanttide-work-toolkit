//! 工作记录：工单流水中的单条记录。
//!
//! 一条记录记一件事的发生：发生时刻（`created_at`）、所执行步骤的锚点
//! （`step_id`）、简要描述（`description`）与判定结果（`is_succeeded`）。
//! 跨边界引用以 `id` 为准，排序以 `seq` 为准。记录不独立落盘，内嵌于
//! 工单的 `records` 字段；只增不改，落笔后原样保留，改结论的唯一方式
//! 是追加一条新记录。
//!
//! 读出即校验：[`read_line`] 读一行 JSONL，要么产出一条记录，要么给出
//! [`RecordError`]（第几笔 + 为什么）。行级读不通（不是合法 JSON、不是
//! 映射）与模型级读不通（字段表之外、必选缺失、类型不符）都在这一个入口
//! 报出。单条记录在整本流水中是否有效——凭证唯一、页码连续、时序递增——
//! 属账本级约束，由账本侧对账执行。

use crate::record::errors::RecordError;
use serde::Serialize;
use serde_json::Value as Json;

/// 字段表与模型同源：字段顺序即声明顺序，抄错字段的硬伤由测试守着。
const FIELDS: [&str; 7] = [
    "id",
    "seq",
    "created_at",
    "order_id",
    "step_id",
    "description",
    "is_succeeded",
];

/// 必选的文本字段：缺失或取值为空白均视为未提供；读出时按这个顺序报缺字段。
const REQUIRED_TEXT: [&str; 4] = ["id", "created_at", "order_id", "step_id"];

/// 工作记录实体：凭证、页码、发生时刻、工单与步骤锚点、简要描述、判定结果。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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
    /// 落形为 JSONL 行：单行紧凑 JSON，字段全写，不产生字段表之外的键。
    pub fn to_jsonl(&self) -> String {
        serde_json::to_string(self).expect("工作记录的字段都是可序列化的简单取值")
    }
}

/// 读一行 JSONL：读不通就给出 [`RecordError`]（第几笔 + 为什么）。
///
/// 检查顺序即报错顺序——行是不是合法 JSON、是不是映射、字段表之外、
/// 必选缺失、页码、两个推荐字段的类型。文案里的「七个字段」由字段表测试守着。
pub fn read_line(line: &str, ordinal: usize) -> Result<WorkRecord, RecordError> {
    let fault = |reason: &str| RecordError::new(ordinal, reason);
    let value: Json = match serde_json::from_str(line) {
        Ok(value) => value,
        Err(_) => return Err(fault("不是合法的 JSON 行")),
    };
    let Some(map) = value.as_object() else {
        return Err(fault("不是映射（工作记录是七个字段的账）"));
    };
    let text = |key: &str| {
        // 取文本字段：去掉两侧空白，取值不是字符串时视为未提供。
        value
            .get(key)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string()
    };
    let mut unknown: Vec<String> = map
        .keys()
        .filter(|key| !FIELDS.contains(&key.as_str()))
        .cloned()
        .collect();
    unknown.sort();
    if !unknown.is_empty() {
        return Err(fault(&format!(
            "有不认识的字段：{}（只认 {}）",
            unknown.join("、"),
            FIELDS.join("、")
        )));
    }
    for name in REQUIRED_TEXT {
        if text(name).is_empty() {
            return Err(fault(&format!("少了 {name}")));
        }
    }
    let Some(seq) = value
        .get("seq")
        .and_then(|v| v.as_u64())
        .filter(|n| *n >= 1)
    else {
        return Err(fault("的 seq 缺了，或不是自 1 起的整数"));
    };
    if value.get("description").is_some_and(|v| !v.is_string()) {
        return Err(fault("的 description 类型不对"));
    }
    if value.get("is_succeeded").is_some_and(|v| !v.is_boolean()) {
        return Err(fault("的 is_succeeded 类型不对"));
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// 合法记录的落形（也是契约：Python 侧逐字相同）。
    const SAMPLE_LINE: &str = concat!(
        r#"{"id":"0f0e5b1a-9d0c-4c7e-8d1e-2b6a5f4e3d2c","seq":1,"#,
        r#""created_at":"2026-02-11T10:00:00Z","#,
        r#""order_id":"3b8a2c1d-4e5f-4a6b-8c9d-0e1f2a3b4c5d","#,
        r#""step_id":"8c7d6e5f-4a3b-4c2d-9e1f-0a2b3c4d5e6f","#,
        r#""description":"草拟了提纲","is_succeeded":true}"#
    );

    /// 合法记录样本：七个字段齐备，`seq` 为 1，判定为通过。
    fn sample() -> Json {
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

    /// 字段表从模型声明来，且仍是规格的七个字段——抄错一个、数错一个就红。
    #[test]
    fn field_table_matches_the_record() {
        assert_eq!(
            FIELDS,
            [
                "id",
                "seq",
                "created_at",
                "order_id",
                "step_id",
                "description",
                "is_succeeded"
            ]
        );
    }

    /// 合法行读出记录，取值逐一相符。
    #[test]
    fn valid_line_reads_a_record() {
        let record = read_line(SAMPLE_LINE, 3).expect("合法行应读出成功");
        assert_eq!(record.seq, 1);
        assert_eq!(record.description, "草拟了提纲");
        assert!(record.is_succeeded);
    }

    /// 落形是单行紧凑 JSON，与契约样本逐字相同；往返不失真。
    #[test]
    fn to_jsonl_is_one_compact_line() {
        let record = read_line(SAMPLE_LINE, 3).expect("合法行应读出成功");
        let got = record.to_jsonl();
        assert_eq!(got, SAMPLE_LINE);
        assert!(!got.contains('\n'), "单行");
        assert_eq!(read_line(&got, 3).expect("往返"), record);
    }

    /// 行级读不通：不是合法 JSON、不是映射，各报一句。
    #[test]
    fn line_level_reasons() {
        let error = read_line("{oops", 3).expect_err("不是合法 JSON");
        assert_eq!(error.to_string(), "第 3 笔不是合法的 JSON 行");
        let error = read_line("[1, 2]", 3).expect_err("不是映射");
        assert_eq!(
            error.to_string(),
            "第 3 笔不是映射（工作记录是七个字段的账）"
        );
    }

    /// 推荐字段可省略：`description` 缺省为空串，`is_succeeded` 缺省为
    /// `false`（规格约束：未记录「通过」即视为未通过）。
    #[test]
    fn optional_fields_fill_in() {
        let mut bare = sample();
        bare.as_object_mut()
            .expect("样本是映射")
            .remove("description");
        bare.as_object_mut()
            .expect("样本是映射")
            .remove("is_succeeded");
        let record = read_line(&bare.to_string(), 3).expect("推荐字段省略不应报错");
        assert_eq!(record.description, "");
        assert!(!record.is_succeeded);
    }

    /// 字段表之外的字段被拒收，报错里列名并带上字段表。以 `step` 为反例：
    /// 站名不落账，记录仅以 `step_id`（机器锚点）指认步骤。
    #[test]
    fn unknown_fields_are_rejected() {
        let odd = json!({"step": "草拟"});
        let mut value = sample();
        value
            .as_object_mut()
            .expect("样本是映射")
            .extend(odd.as_object().expect("映射").clone());
        let error = read_line(&value.to_string(), 3).expect_err("多字段");
        assert_eq!(
            error.reason,
            format!("有不认识的字段：step（只认 {}）", FIELDS.join("、"))
        );
    }

    /// 多个不认识的字段按名字排序，报错不随输入顺序摇摆。
    #[test]
    fn unknown_fields_are_sorted() {
        let mut value = sample();
        value
            .as_object_mut()
            .expect("样本是映射")
            .insert("b".to_string(), json!(1));
        value
            .as_object_mut()
            .expect("样本是映射")
            .insert("a".to_string(), json!(2));
        let error = read_line(&value.to_string(), 3).expect_err("多字段");
        assert!(error.reason.starts_with("有不认识的字段：a、b（"));
    }

    /// 四个必选文本字段逐一缺失时，均报「少了 <字段>」。
    #[test]
    fn missing_required_is_rejected() {
        for name in REQUIRED_TEXT {
            let mut bare = sample();
            bare.as_object_mut().expect("样本是映射").remove(name);
            let error = read_line(&bare.to_string(), 3).expect_err("缺必选");
            assert_eq!(error.reason, format!("少了 {name}"));
        }
    }

    /// 类型不符逐项拒收：`seq` 为零、负数、字符串或布尔均属坏页码
    /// （页码须为自 1 起的整数）；`description` 非字符串、`is_succeeded`
    /// 非布尔均属类型不符。
    #[test]
    fn bad_values_are_rejected() {
        for seq in [json!(0), json!(-1), json!("1"), json!(true)] {
            let mut odd = sample();
            odd.as_object_mut()
                .expect("样本是映射")
                .insert("seq".to_string(), seq);
            let error = read_line(&odd.to_string(), 3).expect_err("坏页码");
            assert_eq!(error.reason, "的 seq 缺了，或不是自 1 起的整数");
        }
        for (name, bad) in [("description", json!(3)), ("is_succeeded", json!("yes"))] {
            let mut odd = sample();
            odd.as_object_mut()
                .expect("样本是映射")
                .insert(name.to_string(), bad);
            let error = read_line(&odd.to_string(), 3).expect_err("坏类型");
            assert_eq!(error.reason, format!("的 {name} 类型不对"));
        }
    }
}
