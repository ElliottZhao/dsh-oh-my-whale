//! `omw-core` 二进制入口：解析参数、打开数据文件、绑定 loopback、打印 ready 行。

use std::io::Write;
use std::net::{Ipv4Addr, SocketAddr};
use std::process::ExitCode;

use omw_core::config::{self, ConfigOutcome, USAGE};
use omw_core::loopback::{self, AppState};
use omw_core::protocol::{CoreIdentity, PROTOCOL_VERSION, ReadyLine};
use omw_core::provider;
use omw_core::state::{SCHEMA_VERSION, StateStore};

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("omw-core: {message}");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<(), String> {
    let outcome = config::from_args_env(std::env::args().skip(1), std::env::vars())
        .map_err(|error| format!("{error}\n\n{USAGE}"))?;
    let config = match outcome {
        ConfigOutcome::Help => {
            println!("{USAGE}");
            return Ok(());
        }
        ConfigOutcome::Run(config) => config,
    };

    // 先抢端口再动数据文件：端口起不来时不能留下一次「假启动」。
    let listener = tokio::net::TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, config.port)))
        .await
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::AddrInUse {
                format!(
                    "无法监听 127.0.0.1:{}：端口已被占用。请换一个端口（--port <端口>），或用 --port 0 让系统分配。",
                    config.port
                )
            } else {
                format!("无法监听 127.0.0.1:{}：{error}。", config.port)
            }
        })?;
    let port = listener
        .local_addr()
        .map_err(|error| format!("无法读取监听端口：{error}"))?
        .port();

    let store = StateStore::open(&config.data_dir).map_err(|error| error.to_string())?;
    let provider_config = provider::load(&config.config_path).map_err(|error| error.to_string())?;
    // 凭据此刻取不到不算启动失败：按 ADR-0003 降级，由快照的 degradedProviders 报告。
    provider::validate_at_startup(&provider_config).map_err(|error| error.to_string())?;

    // 这一行是 Adapter 与测试脚本唯一的端口来源；日志一律走 stderr。
    let ready = ReadyLine {
        event: "core.ready".to_owned(),
        endpoint: format!("http://127.0.0.1:{port}"),
        data_file: store.path().display().to_string(),
        pid: std::process::id(),
        identity: CoreIdentity {
            protocol_version: PROTOCOL_VERSION.to_owned(),
            core_version: env!("CARGO_PKG_VERSION").to_owned(),
            schema_version: SCHEMA_VERSION,
            started_at: store.started_at().to_owned(),
        },
    };
    println!(
        "{}",
        serde_json::to_string(&ready).map_err(|error| format!("无法序列化 ready 行：{error}"))?
    );
    std::io::stdout()
        .flush()
        .map_err(|error| format!("无法写出 ready 行：{error}"))?;

    loopback::listen(
        AppState::new(store, config.config_path, provider_config),
        listener,
    )
    .await
    .map_err(|error| format!("Core 服务异常退出：{error}"))
}
