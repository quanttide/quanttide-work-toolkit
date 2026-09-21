//! 读一行读不通时的错误。
//!
//! 只存第几笔与为什么；文件名不进错误，由端侧渲染时拼（`format!("{file} {error}")`）。

/// 读不通的一笔：第几笔 + 为什么。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordError {
    /// 账本里自上而下第几笔（从 1 数）。
    pub ordinal: usize,
    /// 为什么读不通。
    pub reason: String,
}

impl RecordError {
    pub fn new(ordinal: usize, reason: impl Into<String>) -> Self {
        RecordError {
            ordinal,
            reason: reason.into(),
        }
    }
}

impl std::fmt::Display for RecordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "第 {} 笔{}", self.ordinal, self.reason)
    }
}

impl std::error::Error for RecordError {}

#[cfg(test)]
mod tests {
    use super::*;

    /// 错误只装第几笔与为什么；渲染时拼「第 n 笔{这一条}」，文件名由端侧加在前头。
    #[test]
    fn error_carries_ordinal_and_reason_only() {
        let error = RecordError::new(7, "不是合法的 JSON 行");
        assert_eq!(error.ordinal, 7);
        assert_eq!(error.reason, "不是合法的 JSON 行");
        assert_eq!(error.to_string(), "第 7 笔不是合法的 JSON 行");
        assert_eq!(
            format!("records.jsonl {error}"),
            "records.jsonl 第 7 笔不是合法的 JSON 行"
        );
    }
}
