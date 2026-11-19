//! loopback HTTP 边界：协议握手、Observation 读写与 provider 配置。
//!
//! Core 只绑 127.0.0.1；所有错误共用 `{"error":{...}}` 信封。

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use axum::body::Bytes;
use axum::extract::rejection::BytesRejection;
use axum::extract::{DefaultBodyLimit, Query, State};
use axum::http::{Method, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::de::DeserializeOwned;
use serde_json::json;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::protocol::{
    CODE_INTERNAL_ERROR, CODE_INVALID_CONFIG, CODE_INVALID_REQUEST, CODE_METHOD_NOT_ALLOWED,
    CODE_NOT_FOUND, CODE_PROTOCOL_VERSION_MISMATCH, CoreIdentity, CreateObservationRequest,
    ErrorBody, ErrorPayload, HandshakeRequest, HandshakeResponse, ObservationList, PROTOCOL_VERSION,
    ProviderConfig, is_compatible,
};
use crate::provider;
use crate::state::{DEFAULT_OBSERVATION_LIMIT, MAX_OBSERVATION_LIMIT, SCHEMA_VERSION, StateStore};

/// 全部可用接口，用于 404 与 405 的提示，只在这里写一遍。
const AVAILABLE_ENDPOINTS: &str =
    "GET /v1/config、PUT /v1/config、GET /v1/observations、POST /v1/observations、POST /v1/handshake";

/// 一次进程启动的共享状态。
pub struct AppState {
    store: StateStore,
    config_path: PathBuf,
    provider_config: RwLock<ProviderConfig>,
}

impl AppState {
    pub fn new(
        store: StateStore,
        config_path: PathBuf,
        provider_config: ProviderConfig,
    ) -> Arc<Self> {
        Arc::new(Self {
            store,
            config_path,
            provider_config: RwLock::new(provider_config),
        })
    }
}

/// 单个请求体的上限。超过时也要走统一错误信封，不能由 axum 返回纯文本。
const MAX_BODY_BYTES: usize = 1024 * 1024;

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/v1/handshake", post(handshake))
        .route(
            "/v1/observations",
            get(list_observations).post(create_observation),
        )
        .route("/v1/config", get(read_config).put(write_config))
        .method_not_allowed_fallback(method_not_allowed)
        .fallback(not_found)
        .layer(DefaultBodyLimit::max(MAX_BODY_BYTES))
        .with_state(state)
}

/// 在已绑定的 loopback 监听器上提供服务，收到 Ctrl-C 后优雅退出。
pub async fn listen(
    state: Arc<AppState>,
    listener: tokio::net::TcpListener,
) -> std::io::Result<()> {
    axum::serve(listener, router(state))
        .with_graceful_shutdown(shutdown_signal())
        .await
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}

async fn handshake(State(state): State<Arc<AppState>>, request: Result<Bytes, BytesRejection>) -> Response {
    let body = match request {
        Ok(body) => body,
        Err(rejection) => return body_rejection(rejection),
    };
    let request: HandshakeRequest = match parse_body(
        &body,
        "握手请求",
        r#"{"protocolVersion":"1.0.0"}"#,
    ) {
        Ok(request) => request,
        Err(response) => return *response,
    };

    if !is_compatible(&request.protocol_version, PROTOCOL_VERSION) {
        return error_response(
            StatusCode::CONFLICT,
            CODE_PROTOCOL_VERSION_MISMATCH,
            &format!(
                "协议版本不匹配：Core 支持 {PROTOCOL_VERSION}，Adapter 请求的是 {}。请把版本较低的一侧升级后重试。",
                request.protocol_version
            ),
            Some(json!({
                "supported": PROTOCOL_VERSION,
                "requested": request.protocol_version,
            })),
        );
    }

    (
        StatusCode::OK,
        Json(HandshakeResponse {
            identity: identity(&state),
        }),
    )
        .into_response()
}

async fn create_observation(
    State(state): State<Arc<AppState>>,
    request: Result<Bytes, BytesRejection>,
) -> Response {
    let body = match request {
        Ok(body) => body,
        Err(rejection) => return body_rejection(rejection),
    };
    let request: CreateObservationRequest = match parse_body(
        &body,
        "Observation 请求",
        r#"{"sessionId":"s1","source":"host.message","speaker":"user","content":"……"}"#,
    ) {
        Ok(request) => request,
        Err(response) => return *response,
    };

    if let Some(occurred_at) = request.occurred_at.as_deref()
        && OffsetDateTime::parse(occurred_at, &Rfc3339).is_err()
    {
        return error_response(
            StatusCode::BAD_REQUEST,
            CODE_INVALID_REQUEST,
            &format!("occurredAt 不是合法的 RFC3339 时刻：{occurred_at}。"),
            None,
        );
    }

    match state.store.insert_observation(request) {
        Ok(observation) => (StatusCode::CREATED, Json(observation)).into_response(),
        Err(error) => internal_error(&error),
    }
}

async fn list_observations(
    State(state): State<Arc<AppState>>,
    Query(query): Query<HashMap<String, String>>,
) -> Response {
    let session_id = query.get("sessionId").map(String::as_str);
    let limit = match query.get("limit") {
        None => DEFAULT_OBSERVATION_LIMIT,
        Some(raw) => match raw.parse::<u32>() {
            Ok(value) if (1..=MAX_OBSERVATION_LIMIT).contains(&value) => value,
            _ => {
                return error_response(
                    StatusCode::BAD_REQUEST,
                    CODE_INVALID_REQUEST,
                    &format!("limit 必须是 1–{MAX_OBSERVATION_LIMIT} 之间的整数。"),
                    None,
                );
            }
        },
    };

    match state.store.observations(session_id, limit) {
        Ok(observations) => Json(ObservationList { observations }).into_response(),
        Err(error) => internal_error(&error),
    }
}

async fn read_config(State(state): State<Arc<AppState>>) -> Response {
    let config = state
        .provider_config
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    Json(provider::snapshot(&config)).into_response()
}

async fn write_config(
    State(state): State<Arc<AppState>>,
    request: Result<Bytes, BytesRejection>,
) -> Response {
    let body = match request {
        Ok(body) => body,
        Err(rejection) => return body_rejection(rejection),
    };
    let config: ProviderConfig = match parse_body(
        &body,
        "provider 配置",
        r#"{"providers":[],"roles":[]}"#,
    ) {
        Ok(config) => config,
        Err(response) => return *response,
    };

    if let Err(error) = provider::validate_for_write(&config) {
        return error_response(
            StatusCode::BAD_REQUEST,
            CODE_INVALID_CONFIG,
            &error.to_string(),
            None,
        );
    }

    let mut current = state
        .provider_config
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Err(error) = provider::save(&state.config_path, &config) {
        return internal_error(&error);
    }
    *current = config;
    Json(provider::snapshot(&current)).into_response()
}

async fn not_found(method: Method, uri: Uri) -> Response {
    error_response(
        StatusCode::NOT_FOUND,
        CODE_NOT_FOUND,
        &format!("没有这个接口：{method} {uri}。可用接口：{AVAILABLE_ENDPOINTS}。"),
        None,
    )
}

async fn method_not_allowed(method: Method, uri: Uri) -> Response {
    error_response(
        StatusCode::METHOD_NOT_ALLOWED,
        CODE_METHOD_NOT_ALLOWED,
        &format!("接口 {uri} 不接受 {method}。可用接口：{AVAILABLE_ENDPOINTS}。"),
        None,
    )
}

fn identity(state: &AppState) -> CoreIdentity {
    CoreIdentity {
        protocol_version: PROTOCOL_VERSION.to_owned(),
        core_version: core_version(),
        schema_version: SCHEMA_VERSION,
        started_at: state.store.started_at().to_owned(),
    }
}

fn body_rejection(rejection: BytesRejection) -> Response {
    let status = rejection.status();
    let message = if status == StatusCode::PAYLOAD_TOO_LARGE {
        format!(
            "请求体超过上限（{MAX_BODY_BYTES} 字节）。请把一条消息拆小，或减少一次提交的条目数。"
        )
    } else {
        format!("无法读取请求体：{rejection}。")
    };
    error_response(status, CODE_INVALID_REQUEST, &message, None)
}

/// 解析请求体；失败时返回一个已渲染的错误响应（装箱以缩小 `Result` 的错误分支）。
fn parse_body<T: DeserializeOwned>(
    body: &Bytes,
    what: &str,
    example: &str,
) -> Result<T, Box<Response>> {
    serde_json::from_slice(body).map_err(|error| {
        Box::new(error_response(
            StatusCode::BAD_REQUEST,
            CODE_INVALID_REQUEST,
            &format!("{what}不是合法 JSON（{error}）。请求体应为 {example}。"),
            None,
        ))
    })
}

fn internal_error(error: &impl std::fmt::Display) -> Response {
    error_response(
        StatusCode::INTERNAL_SERVER_ERROR,
        CODE_INTERNAL_ERROR,
        &format!("Core 内部错误：{error}"),
        None,
    )
}

fn error_response(
    status: StatusCode,
    code: &str,
    message: &str,
    details: Option<serde_json::Value>,
) -> Response {
    (
        status,
        Json(ErrorBody {
            error: ErrorPayload {
                code: code.to_owned(),
                message: message.to_owned(),
                details,
            },
        }),
    )
        .into_response()
}

fn core_version() -> String {
    env!("CARGO_PKG_VERSION").to_owned()
}
