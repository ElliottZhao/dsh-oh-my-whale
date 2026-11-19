//! 降级编码：全仓唯一的一张稳定编码表。
//!
//! 七张票都会「记降级」，但降级长什么样只能在这里定义一次。各模块不得自造字符串，
//! 只能引用这里的常量；编码全集有测试断言，新增编码必须同时改这张表。
//!
//! 编码与 [`crate::protocol::LedgerKind`] 的映射也在这里：降级一律记 `Degradation`，
//! 注入解释记 `Injection`。写入与查询由 Ledger 实现（#5），本模块只定契约。

use crate::protocol::LedgerKind;

/// 没有配置语言模型：只记录 Observation，不做抽取。
pub const LLM_NOT_CONFIGURED: &str = "llm_not_configured";
/// 某个能力角色没有绑定 provider。
pub const ROLE_UNCONFIGURED: &str = "role_unconfigured";
/// 召回超过预算。
pub const RECALL_TIMEOUT: &str = "recall_timeout";
/// 抽取调用失败或输出不合 schema（粗粒度编码；细分见下）。
pub const EXTRACTION_FAILED: &str = "extraction_failed";
/// 向量或重排能力不可用，退回基线路径。
pub const INDEX_UNAVAILABLE: &str = "index_unavailable";
/// Adapter 连不上 Core。
pub const CORE_UNREACHABLE: &str = "core_unreachable";
/// 目标不在出站白名单，出站被拒绝。
pub const OUTBOUND_BLOCKED: &str = "outbound_blocked";
/// 注入超过了 Adapter 自建的 deadline。
pub const INJECTION_DEADLINE: &str = "injection_deadline";

/// 抽取调用超过 Core 自建的 deadline（#10 的细分编码）。
pub const EXTRACTION_TIMEOUT: &str = "extraction_timeout";
/// 抽取 provider 返回连接错误、5xx 或 429（#10 的细分编码）。
pub const EXTRACTION_PROVIDER_ERROR: &str = "extraction_provider_error";
/// 抽取输出不是合法 JSON（含被截断）（#10 的细分编码）。
pub const EXTRACTION_INVALID_JSON: &str = "extraction_invalid_json";
/// 抽取输出是合法 JSON 但结构、类型或枚举不符（#10 的细分编码）。
pub const EXTRACTION_SCHEMA_MISMATCH: &str = "extraction_schema_mismatch";
/// 证据摘录不是来源 Observation 的逐字子串（#10 的细分编码）。
pub const EVIDENCE_NOT_FOUND: &str = "evidence_not_found";
/// 抽取返回空内容或只有思考字段（#10 的细分编码）。
pub const EXTRACTION_EMPTY_RESPONSE: &str = "extraction_empty_response";
/// 抽取角色未配置，或 fallback 链全部失败（#10 的细分编码）。
pub const EXTRACTION_UNAVAILABLE: &str = "extraction_unavailable";

/// 全部降级编码。测试断言这张表的全集，新增编码必须同时改这里。
///
/// 顺序即文档顺序：先八个粗粒度编码，再七个抽取细分编码。
pub const ALL: &[&str] = &[
    LLM_NOT_CONFIGURED,
    ROLE_UNCONFIGURED,
    RECALL_TIMEOUT,
    EXTRACTION_FAILED,
    INDEX_UNAVAILABLE,
    CORE_UNREACHABLE,
    OUTBOUND_BLOCKED,
    INJECTION_DEADLINE,
    EXTRACTION_TIMEOUT,
    EXTRACTION_PROVIDER_ERROR,
    EXTRACTION_INVALID_JSON,
    EXTRACTION_SCHEMA_MISMATCH,
    EVIDENCE_NOT_FOUND,
    EXTRACTION_EMPTY_RESPONSE,
    EXTRACTION_UNAVAILABLE,
];

/// 注入解释里「为什么没注入这一条」的稳定原因。
pub const SKIP_BUDGET_EXHAUSTED: &str = "budget_exhausted";
/// 这一条的内容与上一轮注入过的内容相同，按去重纪律跳过。
pub const SKIP_DUPLICATE: &str = "duplicate";
/// 这一条的分数低于本轮入选阈值。
pub const SKIP_LOW_SCORE: &str = "low_score";
/// 弃权判定认为本轮不该注入这条。
pub const SKIP_ABSTAINED: &str = "abstained";

/// 全部跳过原因。与降级编码分开：跳过是正常行为，不是故障。
pub const SKIP_ALL: &[&str] = &[SKIP_BUDGET_EXHAUSTED, SKIP_DUPLICATE, SKIP_LOW_SCORE, SKIP_ABSTAINED];

/// 这个字符串是否是已知降级编码。
pub fn is_known(code: &str) -> bool {
    ALL.contains(&code)
}

/// 降级编码对应的可读文案；未知编码返回一句可读的兜底说明，不 panic。
///
/// 兜底文案里带上原编码，便于账本里出现新编码时仍能定位。
pub fn message(code: &str) -> String {
    match code {
        LLM_NOT_CONFIGURED => "没有配置语言模型：这一轮只记录观察，不做理解。".to_owned(),
        ROLE_UNCONFIGURED => "所需的能力角色没有绑定 provider：这一步被跳过。".to_owned(),
        RECALL_TIMEOUT => "召回超过了预算：本轮按无记忆继续，不阻塞回合。".to_owned(),
        EXTRACTION_FAILED => "抽取失败：这批观察没有被理解，原样保留以便重抽。".to_owned(),
        INDEX_UNAVAILABLE => "向量或重排能力不可用：已退回全文检索基线。".to_owned(),
        CORE_UNREACHABLE => "连不上本机内核：本轮没有记忆，对话照常进行。".to_owned(),
        OUTBOUND_BLOCKED => "出站目标不在白名单：请求被拒绝，未发出。".to_owned(),
        INJECTION_DEADLINE => "注入超过了自建 deadline：本轮放弃注入，回合继续。".to_owned(),
        EXTRACTION_TIMEOUT => "抽取调用超时：这批观察没有被理解。".to_owned(),
        EXTRACTION_PROVIDER_ERROR => "抽取 provider 报错：这批观察没有被理解。".to_owned(),
        EXTRACTION_INVALID_JSON => "抽取输出不是合法 JSON：这批观察没有被理解。".to_owned(),
        EXTRACTION_SCHEMA_MISMATCH => "抽取输出结构不符：不合格的条目被拒绝，合格的条目照常写入。".to_owned(),
        EVIDENCE_NOT_FOUND => "证据摘录在来源观察里找不到：整条拒绝写入。".to_owned(),
        EXTRACTION_EMPTY_RESPONSE => "抽取返回了空内容：这批观察没有被理解。".to_owned(),
        EXTRACTION_UNAVAILABLE => "抽取角色不可用：这一轮只记录观察，不做理解。".to_owned(),
        other => format!("未知降级编码 {other}：请检查写入方是否改了编码表。"),
    }
}

/// 每条降级在 Ledger 里的类别。降级全是 `Degradation`；保留成函数是为了让映射显式、
/// 可测试，也方便将来把某条编码改归到别的类别时只改一处。
pub fn ledger_kind(_code: &str) -> LedgerKind {
    LedgerKind::Degradation
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 全集的显式清单：新增编码必须同时改这里，否则本测试红。
    const EXPECTED: &[&str] = &[
        "llm_not_configured",
        "role_unconfigured",
        "recall_timeout",
        "extraction_failed",
        "index_unavailable",
        "core_unreachable",
        "outbound_blocked",
        "injection_deadline",
        "extraction_timeout",
        "extraction_provider_error",
        "extraction_invalid_json",
        "extraction_schema_mismatch",
        "evidence_not_found",
        "extraction_empty_response",
        "extraction_unavailable",
    ];

    #[test]
    fn code_table_matches_the_expected_full_set() {
        let mut actual: Vec<&str> = ALL.to_vec();
        actual.sort_unstable();
        let mut expected: Vec<&str> = EXPECTED.to_vec();
        expected.sort_unstable();
        assert_eq!(actual, expected, "降级编码全集变了；请同步更新常量表与这张清单");
    }

    #[test]
    fn every_code_has_the_snake_case_shape_and_a_readable_message() {
        for code in ALL {
            assert!(
                code.chars().all(|ch| ch.is_ascii_lowercase() || ch == '_'),
                "编码 {code} 只允许小写字母与下划线"
            );
            let message = message(code);
            assert!(!message.trim().is_empty(), "编码 {code} 缺少可读文案");
            assert!(
                message.contains('：') || message.contains('。'),
                "编码 {code} 的文案应是给人读的完整句子：{message}"
            );
        }
    }

    #[test]
    fn unknown_code_gets_a_readable_fallback_instead_of_panicking() {
        assert!(!is_known("not_a_code"));
        let message = message("not_a_code");
        assert!(message.contains("not_a_code"), "兜底文案应点名未知编码：{message}");
    }

    #[test]
    fn every_degradation_maps_to_the_degradation_ledger_kind() {
        for code in ALL {
            assert_eq!(ledger_kind(code), LedgerKind::Degradation);
        }
    }

    #[test]
    fn skip_codes_are_distinct_from_degradations() {
        for code in SKIP_ALL {
            assert!(!is_known(code), "跳过原因 {code} 不是降级编码");
        }
        assert!(SKIP_ALL.contains(&SKIP_DUPLICATE));
        assert!(SKIP_ALL.contains(&SKIP_BUDGET_EXHAUSTED));
    }
}
