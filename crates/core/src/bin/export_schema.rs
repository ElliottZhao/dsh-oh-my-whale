//! 把协议类型导出成 JSON Schema，供 Adapter 侧生成 TypeScript 类型与运行时校验器。
//!
//! 用法：`cargo run -p oh-my-whale-core --bin export-schema [输出目录]`
//! 输出目录默认 `packages/adapter/src/generated`。文件由 `pnpm run gen:protocol` 调用生成。

use std::path::PathBuf;
use std::process::ExitCode;

use omw_core::degrade;
use omw_core::protocol::{PROTOCOL_VERSION, WireContract};
use schemars::generate::SchemaSettings;
use serde_json::{Map, Value};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("export-schema: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let out_dir = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("packages/adapter/src/generated"));
    std::fs::create_dir_all(&out_dir)
        .map_err(|error| format!("无法创建输出目录 {}：{error}", out_dir.display()))?;

    let generator = SchemaSettings::draft07().into_generator();
    let schema = generator.into_root_schema_for::<WireContract>();
    let schema_json =
        serde_json::to_value(&schema).map_err(|error| format!("无法序列化 schema：{error}"))?;

    // 降级编码与跳过原因不是 wire 类型，但同样是 Adapter 要照抄的契约；
    // 一起导出，生成脚本才会把它们写进 TypeScript 常量表。
    let mut messages = Map::new();
    for code in degrade::ALL {
        messages.insert((*code).to_owned(), Value::String(degrade::message(code)));
    }

    let bundle = serde_json::json!({
        "protocolVersion": PROTOCOL_VERSION,
        "schema": schema_json,
        "degradationCodes": degrade::ALL,
        "degradationMessages": messages,
        "injectionSkipCodes": degrade::SKIP_ALL,
    });
    let path = out_dir.join("protocol.json");
    let body = serde_json::to_string_pretty(&bundle)
        .map_err(|error| format!("无法格式化 schema：{error}"))?;
    std::fs::write(&path, format!("{body}\n"))
        .map_err(|error| format!("无法写入 {}：{error}", path.display()))?;

    println!("已写出 {}", path.display());
    Ok(())
}
