//! oh-my-whale 的本机常驻内核。
//!
//! 边界切分：`protocol` 是与 Adapter 共享的 wire 契约（也是 TypeScript 类型的唯一来源），
//! `provider` 是 Core 自持的 provider 配置，`state` 是单文件持久化，`loopback` 是
//! loopback HTTP 边界，`config` 是启动参数。Core 不监听外网。

pub mod config;
pub mod degrade;
pub mod loopback;
pub mod protocol;
pub mod provider;
pub mod state;
pub mod tokens;
