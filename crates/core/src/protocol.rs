//! 两端的 wire 契约：协议版本协商、统一错误信封、Observation 与 Provider 配置。
//!
//! 这个模块是协议结构的**唯一定义处**。Adapter 的 TypeScript 类型由 `export-schema`
//! 导出 JSON Schema 后生成，不要手写第二份；改这里、重新生成、再跑类型检查。

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Core 实现的协议版本。
///
/// 任何请求/响应字段或语义的变更都必须改这里；主版本号不同的 Adapter 会被拒绝。
/// Adapter 侧的常量由这里生成，不再各写一份。
pub const PROTOCOL_VERSION: &str = "1.0.0";

pub const CODE_PROTOCOL_VERSION_MISMATCH: &str = "protocol_version_mismatch";
pub const CODE_INVALID_REQUEST: &str = "invalid_request";
pub const CODE_NOT_FOUND: &str = "not_found";
pub const CODE_METHOD_NOT_ALLOWED: &str = "method_not_allowed";
pub const CODE_INVALID_CONFIG: &str = "invalid_config";
pub const CODE_INTERNAL_ERROR: &str = "internal_error";

/// 取出版本号的主版本段。`1.2.3` → `1`；无法解析时返回 `None`。
pub fn major(version: &str) -> Option<u64> {
    version.trim().split('.').next()?.parse().ok()
}

/// 主版本号一致即可沟通，次版本号差异不拒绝：新 Adapter 连旧 Core 只会少用新字段。
pub fn is_compatible(requested: &str, supported: &str) -> bool {
    match (major(requested), major(supported)) {
        (Some(requested_major), Some(supported_major)) => requested_major == supported_major,
        _ => false,
    }
}

/// 两端共享的版本与进程信息。握手响应与 ready 行都平铺这几个字段。
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CoreIdentity {
    pub protocol_version: String,
    pub core_version: String,
    pub schema_version: u32,
    /// 本次进程启动的时间；重启会变。
    pub started_at: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HandshakeRequest {
    pub protocol_version: String,
    #[serde(default)]
    pub adapter_version: Option<String>,
}

/// 握手成功时返回的 Core 身份。
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HandshakeResponse {
    #[serde(flatten)]
    pub identity: CoreIdentity,
}

/// Core 就绪后写到 stdout 的那一行。Adapter 与测试脚本都从它取端口。
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReadyLine {
    pub event: String,
    pub endpoint: String,
    pub data_file: String,
    pub pid: u32,
    #[serde(flatten)]
    pub identity: CoreIdentity,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ErrorBody {
    pub error: ErrorPayload,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ErrorPayload {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// 一条待写入的 Observation；未给出事件时间时，Core 用摄入时刻。
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateObservationRequest {
    /// 会话标识；同一会话重复摄入由此归并（去重在 #8 完整实现）。
    pub session_id: String,
    /// 来源，例如 `host.message`、`tool.call`。
    pub source: String,
    /// 说话者或产生者；工具事件可省略。
    #[serde(default)]
    pub speaker: Option<String>,
    pub content: String,
    /// 事件发生的时刻（RFC3339）。省略时 Core 用摄入时刻，两者不混为一谈。
    #[serde(default)]
    pub occurred_at: Option<String>,
}

/// 已落盘的 Observation。`occurredAt` 是事件时刻，由请求给出或用摄入时刻补。
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Observation {
    pub id: i64,
    pub session_id: String,
    pub source: String,
    pub speaker: Option<String>,
    pub content: String,
    pub occurred_at: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ObservationList {
    pub observations: Vec<Observation>,
}

/// 估算一段文本的 token 数。口径由 Core 的 `tokens` 模块唯一实现。
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EstimateTokensRequest {
    pub text: String,
}

/// 估算结果。`wHan` 是本次采用的汉字倍率，便于调用方知道预算口径。
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EstimateTokensResponse {
    /// 文本密度 token 数，预算判定用这个值。
    pub tokens: u32,
    /// 按 DSH `estimateMessage()` 口径的整条消息 token 数（文本密度 + 8）。
    pub message_tokens: u32,
    pub w_han: f64,
}

/// Ledger 条目的类别：每次出站、每次注入、每次降级、每次写入都各记一类。
///
/// 降级编码表见 [`crate::degrade`]；本类型只定义分类，不定义具体编码。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum LedgerKind {
    /// 一次真实出站调用：用途、provider、模型、时间。
    Outbound,
    /// 一轮记忆注入：注入了什么、为什么、占了哪些预算。
    Injection,
    /// 一次降级：编码见 `degrade` 模块。
    Degradation,
    /// 一次状态变更：写入条目、置失效、确认删除等。
    Change,
}

/// 注入解释里的一条入选条目：id、分数、路径与预算占用。
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InjectedItem {
    pub item_id: String,
    pub score: f64,
    /// 这条的贡献来自哪条路径：`lexical` / `semantic` / `rerank`。
    pub path: String,
    /// 这条在本轮 View 里占用的 token 数，口径由 #29 的 Core 实现给出。
    pub token_count: u32,
}

/// 注入解释里「为什么没注入那一条」：候选 id 与稳定的跳过原因。
///
/// 跳过原因是正常行为（去重、低分、弃权、预算），不是降级；降级另见
/// [`crate::degrade`]。
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SkippedRecall {
    pub item_id: String,
    /// 稳定编码，取值见 `degrade::SKIP_ALL`。
    pub reason: String,
}

/// 一轮注入的完整解释，可直接落成一张表。
///
/// 字段足以回答「本轮为什么注入这些、为什么没注入那些」：入选条目的 id、分数、
/// 路径与 token 占用，被跳过条目的原因，以及本轮整体预算与发生过的降级编码。
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InjectionExplanation {
    pub turn: i64,
    pub session_id: String,
    pub items: Vec<InjectedItem>,
    pub skipped: Vec<SkippedRecall>,
    /// 本轮注入消耗的 token 数。
    pub token_count: u32,
    /// 本轮允许的 token 预算。
    pub budget_tokens: u32,
    /// 本轮发生过的降级编码；正常轮为空。
    pub degraded: Vec<String>,
}

/// Provider 能提供的模型能力类别，按能力而非业务划分。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Capability {
    Llm,
    Embedding,
    CrossEncoder,
    Decision,
}

/// 一条能力角色对能力的硬性要求：抽取与查询改写要语言模型，排序要重排模型，
/// 弃权判定要能判「答不了」的决策能力。角色绑错了能力，配置就是错的。
impl Role {
    pub fn required_capability(self) -> Capability {
        match self {
            Role::Extraction | Role::QueryRewrite => Capability::Llm,
            Role::Ranking => Capability::CrossEncoder,
            Role::Abstention => Capability::Decision,
        }
    }
}

/// Core 内部对某类工作的具名需求。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Role {
    Extraction,
    QueryRewrite,
    Ranking,
    Abstention,
}

impl Role {
    pub const ALL: [Role; 4] = [
        Role::Extraction,
        Role::QueryRewrite,
        Role::Ranking,
        Role::Abstention,
    ];
}

/// 凭据引用的种类：只存引用，不存密钥值。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum CredentialKind {
    /// 环境变量的名字。
    Env,
    /// 只属于 Core 的密钥文件路径。
    File,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CredentialRef {
    pub kind: CredentialKind,
    /// `env` 时是环境变量名，`file` 时是文件路径；都不含密钥值。
    pub reference: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProviderEntry {
    pub name: String,
    pub endpoint: String,
    pub capabilities: Vec<Capability>,
    pub credential: CredentialRef,
}

/// 能力角色的一项绑定：某 provider 的某条能力。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RoleBinding {
    pub provider: String,
    pub capability: Capability,
}

/// 一个角色的有序 fallback 链。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RoleChain {
    pub role: Role,
    pub chain: Vec<RoleBinding>,
}

/// Core 自持的 provider 配置；出站白名单等于这里的 endpoint 集合。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProviderConfig {
    #[serde(default)]
    pub providers: Vec<ProviderEntry>,
    #[serde(default)]
    pub roles: Vec<RoleChain>,
}

/// 读取或写入配置后的视图：补上未配置的角色、凭据取不到的 provider 与派生的出站白名单。
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProviderConfigSnapshot {
    pub providers: Vec<ProviderEntry>,
    pub roles: Vec<RoleChain>,
    /// 没有任何角色绑定的能力角色；显式列出来，不隐式挑一个 provider。
    pub unconfigured_roles: Vec<Role>,
    /// 凭据引用此刻取不到的 provider；它们仍在白名单里，但用到时按 ADR-0003 降级。
    pub degraded_providers: Vec<String>,
    pub outbound_whitelist: Vec<String>,
}

/// 导出 JSON Schema 时把全部 wire 类型收进来的根对象。
///
/// 它只服务代码生成，不出现在任何请求或响应里。
#[derive(JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct WireContract {
    pub core_identity: CoreIdentity,
    pub handshake_request: HandshakeRequest,
    pub handshake_response: HandshakeResponse,
    pub ready_line: ReadyLine,
    pub error_body: ErrorBody,
    pub error_payload: ErrorPayload,
    pub create_observation_request: CreateObservationRequest,
    pub observation: Observation,
    pub observation_list: ObservationList,
    pub estimate_tokens_request: EstimateTokensRequest,
    pub estimate_tokens_response: EstimateTokensResponse,
    pub ledger_kind: LedgerKind,
    pub injected_item: InjectedItem,
    pub skipped_recall: SkippedRecall,
    pub injection_explanation: InjectionExplanation,
    pub capability: Capability,
    pub role: Role,
    pub credential_kind: CredentialKind,
    pub credential_ref: CredentialRef,
    pub provider_entry: ProviderEntry,
    pub role_binding: RoleBinding,
    pub role_chain: RoleChain,
    pub provider_config: ProviderConfig,
    pub provider_config_snapshot: ProviderConfigSnapshot,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_major_version_is_compatible() {
        assert!(is_compatible("1.0.0", "1.0.0"));
        assert!(is_compatible("1.4.2", "1.0.0"));
    }

    #[test]
    fn different_major_version_is_rejected() {
        assert!(!is_compatible("2.0.0", "1.0.0"));
        assert!(!is_compatible("0.9.0", "1.0.0"));
    }

    #[test]
    fn unparsable_version_is_rejected() {
        assert!(!is_compatible("garbage", "1.0.0"));
        assert!(!is_compatible("", "1.0.0"));
    }

    #[test]
    fn version_major_parses_numeric_prefix() {
        assert_eq!(major("1.0.0"), Some(1));
        assert_eq!(major("10.2"), Some(10));
        assert_eq!(major(" 2.1.0 "), Some(2));
        assert_eq!(major("v2.0.0"), None);
    }
}
