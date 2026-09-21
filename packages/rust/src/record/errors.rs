//! 记录读不通时的错误。
//!
//! 只存位置（第 n 笔）与种类（[`Fault`]）；文件名不进错误，由端侧在渲染时给——
//! [`RecordError::message`] 出 canonical 文案。

use crate::fields::RECORD_FIELDS;

/// 记录里的位置。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Position {
    /// 账本里自上而下第 n 笔（从 1 数）。
    Record(usize),
}

impl Position {
    /// 位置的话头，比如「第 3 笔」。
    pub fn phrase(&self) -> String {
        match self {
            Position::Record(ordinal) => format!("第 {ordinal} 笔"),
        }
    }
}

/// 这一笔错在哪，带上渲染所需的取值。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fault {
    /// JSONL 行不是合法 JSON。
    NotJson,
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

impl Fault {
    /// 位置之后的那一句（canonical 文案的主体）。
    pub fn text(&self) -> String {
        match self {
            Fault::NotJson => "不是合法的 JSON 行".to_string(),
            Fault::NotMapping => "不是映射（工作记录是七个字段的账）".to_string(),
            Fault::UnknownFields(unknown) => format!(
                "有不认识的字段：{}（只认 {}）",
                unknown.join("、"),
                RECORD_FIELDS.join("、")
            ),
            Fault::MissingField(name) => format!("少了 {name}"),
            Fault::BadSeq => "的 seq 缺了，或不是自 1 起的整数".to_string(),
            Fault::BadType(name) => format!("的 {name} 类型不对"),
        }
    }
}

/// 一笔记录读不通。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordError {
    /// 在哪一笔读不通。
    pub position: Position,
    /// 哪一条不成立。
    pub fault: Fault,
}

impl RecordError {
    pub fn new(position: Position, fault: Fault) -> Self {
        RecordError { position, fault }
    }

    /// canonical 报错文字：`{file} {位置}{这一条}`；文件由端侧在渲染时给。
    pub fn message(&self, file: &str) -> String {
        format!("{file} {}{}", self.position.phrase(), self.fault.text())
    }
}

impl std::fmt::Display for RecordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.position.phrase(), self.fault.text())
    }
}

impl std::error::Error for RecordError {}

#[cfg(test)]
mod tests {
    use super::*;

    /// 渲染时文件在前、位置居中、毛病在后；不带文件时只出位置与毛病。
    #[test]
    fn message_prefixes_the_file_and_keeps_the_position() {
        let error = RecordError::new(Position::Record(3), Fault::BadSeq);
        assert_eq!(
            error.message("records.jsonl"),
            "records.jsonl 第 3 笔的 seq 缺了，或不是自 1 起的整数"
        );
        assert_eq!(error.to_string(), "第 3 笔的 seq 缺了，或不是自 1 起的整数");
    }

    /// 不认识的字段逐一列名，并带上字段表，便于对照改账。
    #[test]
    fn unknown_fields_are_named_in_the_text() {
        let error = RecordError::new(
            Position::Record(1),
            Fault::UnknownFields(vec!["step".into()]),
        );
        assert_eq!(
            error.message("records.jsonl"),
            "records.jsonl 第 1 笔有不认识的字段：step（只认 id、seq、created_at、order_id、step_id、description、is_succeeded）"
        );
    }
}
