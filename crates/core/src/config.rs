//! 启动参数与环境变量解析。
//!
//! 优先级：命令行 > 环境变量 > 平台默认值。监听地址不可配置：Core 只绑 loopback。

use std::fmt;
use std::path::PathBuf;

/// Core 的默认监听端口。同一个数据文件只应有一个 Core 在本端口上服务。
pub const DEFAULT_PORT: u16 = 7419;

pub const ENV_DATA_DIR: &str = "OMW_DATA_DIR";
pub const ENV_PORT: &str = "OMW_PORT";
pub const ENV_CONFIG: &str = "OMW_CONFIG";

pub const USAGE: &str = "\
用法：omw-core [--data-dir <目录>] [--config <文件>] [--port <端口>]

  --data-dir <目录>  数据目录；里面只有一个数据文件 core.db
                     默认：平台数据目录/oh-my-whale
  --config <文件>    Core 自持的 provider 配置文件
                     默认：平台配置目录/oh-my-whale/config.json
  --port <端口>      监听端口，只绑 127.0.0.1；0 表示由系统分配
                     默认：7419
  -h, --help         显示这段说明

环境变量：OMW_DATA_DIR、OMW_CONFIG、OMW_PORT";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub data_dir: PathBuf,
    pub config_path: PathBuf,
    pub port: u16,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ConfigOutcome {
    Run(Config),
    Help,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ConfigError {
    UnknownArgument(String),
    MissingValue(&'static str),
    InvalidPort(String),
    NoHomeDirectory,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::UnknownArgument(arg) => write!(
                f,
                "未知参数 {arg}。可用参数：--data-dir、--config、--port、--help。"
            ),
            ConfigError::MissingValue(flag) => write!(f, "参数 {flag} 后面缺少取值。"),
            ConfigError::InvalidPort(raw) => write!(
                f,
                "端口 {raw} 不是 0–65535 之间的整数。请用 --port 指定端口，或 --port 0 让系统分配。"
            ),
            ConfigError::NoHomeDirectory => write!(
                f,
                "找不到用户主目录，无法确定默认数据目录。请用 --data-dir 显式指定。"
            ),
        }
    }
}

impl std::error::Error for ConfigError {}

/// 解析命令行参数与环境变量。`args` 不含程序名。
pub fn from_args_env<I, K, V>(args: I, env: K) -> Result<ConfigOutcome, ConfigError>
where
    I: IntoIterator<Item = String>,
    K: IntoIterator<Item = (String, V)>,
    V: Into<String>,
{
    let env: std::collections::HashMap<String, String> =
        env.into_iter().map(|(key, value)| (key, value.into())).collect();

    let mut data_dir = env.get(ENV_DATA_DIR).map(PathBuf::from);
    let mut config_path = env.get(ENV_CONFIG).map(PathBuf::from);
    let mut port = match env.get(ENV_PORT) {
        Some(raw) => Some(parse_port(raw)?),
        None => None,
    };

    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => return Ok(ConfigOutcome::Help),
            "--data-dir" => {
                let value = args.next().ok_or(ConfigError::MissingValue("--data-dir"))?;
                data_dir = Some(PathBuf::from(value));
            }
            "--config" => {
                let value = args.next().ok_or(ConfigError::MissingValue("--config"))?;
                config_path = Some(PathBuf::from(value));
            }
            "--port" => {
                let value = args.next().ok_or(ConfigError::MissingValue("--port"))?;
                port = Some(parse_port(&value)?);
            }
            other => return Err(ConfigError::UnknownArgument(other.to_owned())),
        }
    }

    let data_dir = match data_dir {
        Some(dir) => dir,
        None => default_data_dir(env.get("LOCALAPPDATA"), env.get("HOME"), env.get("XDG_DATA_HOME"))?,
    };
    let config_path = match config_path {
        Some(path) => path,
        None => default_config_path(env.get("APPDATA"), env.get("HOME"), env.get("XDG_CONFIG_HOME"))?,
    };

    Ok(ConfigOutcome::Run(Config {
        data_dir,
        config_path,
        port: port.unwrap_or(DEFAULT_PORT),
    }))
}

fn parse_port(raw: &str) -> Result<u16, ConfigError> {
    raw.trim()
        .parse::<u16>()
        .map_err(|_| ConfigError::InvalidPort(raw.to_owned()))
}

/// 平台默认数据目录，只依赖环境变量，便于测试。
pub fn default_data_dir(
    local_app_data: Option<&String>,
    home: Option<&String>,
    xdg_data_home: Option<&String>,
) -> Result<PathBuf, ConfigError> {
    let base = if cfg!(windows) {
        local_app_data
            .map(PathBuf::from)
            .or_else(|| home.map(|home| PathBuf::from(home).join("AppData").join("Local")))
    } else if cfg!(target_os = "macos") {
        home.map(|home| PathBuf::from(home).join("Library").join("Application Support"))
    } else {
        xdg_data_home
            .map(PathBuf::from)
            .or_else(|| home.map(|home| PathBuf::from(home).join(".local").join("share")))
    };

    base.map(|base| base.join("oh-my-whale"))
        .ok_or(ConfigError::NoHomeDirectory)
}

/// 平台默认配置目录，只依赖环境变量，便于测试。
pub fn default_config_path(
    app_data: Option<&String>,
    home: Option<&String>,
    xdg_config_home: Option<&String>,
) -> Result<PathBuf, ConfigError> {
    let base = if cfg!(windows) {
        app_data
            .map(PathBuf::from)
            .or_else(|| home.map(|home| PathBuf::from(home).join("AppData").join("Roaming")))
    } else if cfg!(target_os = "macos") {
        home.map(|home| PathBuf::from(home).join("Library").join("Application Support"))
    } else {
        xdg_config_home
            .map(PathBuf::from)
            .or_else(|| home.map(|home| PathBuf::from(home).join(".config")))
    };

    base.map(|base| base.join("oh-my-whale").join("config.json"))
        .ok_or(ConfigError::NoHomeDirectory)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_env() -> Vec<(String, String)> {
        Vec::new()
    }

    fn env(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        pairs
            .iter()
            .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
            .collect()
    }

    fn run(args: &[&str], env_pairs: &[(&str, &str)]) -> Config {
        let args: Vec<String> = args.iter().map(|arg| (*arg).to_owned()).collect();
        match from_args_env(args, env(env_pairs)).expect("解析参数") {
            ConfigOutcome::Run(config) => config,
            ConfigOutcome::Help => panic!("不应是 --help"),
        }
    }

    #[test]
    fn cli_flags_override_env_and_defaults() {
        let config = run(
            &[
                "--data-dir",
                "C:/omw-data",
                "--config",
                "C:/omw-config.json",
                "--port",
                "7000",
            ],
            &[
                (ENV_DATA_DIR, "C:/from-env"),
                (ENV_CONFIG, "C:/from-env-config.json"),
                (ENV_PORT, "7100"),
            ],
        );
        assert_eq!(config.data_dir, PathBuf::from("C:/omw-data"));
        assert_eq!(config.config_path, PathBuf::from("C:/omw-config.json"));
        assert_eq!(config.port, 7000);
    }

    #[test]
    fn env_is_used_when_flags_are_absent() {
        let config = run(
            &[],
            &[
                (ENV_DATA_DIR, "C:/from-env"),
                (ENV_CONFIG, "C:/from-env-config.json"),
                (ENV_PORT, "7100"),
            ],
        );
        assert_eq!(config.data_dir, PathBuf::from("C:/from-env"));
        assert_eq!(config.config_path, PathBuf::from("C:/from-env-config.json"));
        assert_eq!(config.port, 7100);
    }

    #[test]
    fn help_short_circuits() {
        let outcome = from_args_env(vec!["--help".to_owned()], no_env()).expect("解析 --help");
        assert_eq!(outcome, ConfigOutcome::Help);
    }

    #[test]
    fn unknown_argument_is_reported_readably() {
        let error = from_args_env(vec!["--nope".to_owned()], no_env()).expect_err("应拒绝未知参数");
        assert_eq!(error, ConfigError::UnknownArgument("--nope".to_owned()));
        assert!(error.to_string().contains("--nope"));
    }

    #[test]
    fn invalid_port_is_reported_readably() {
        let error =
            from_args_env(vec!["--port".to_owned(), "99999".to_owned()], no_env()).expect_err("应拒绝非法端口");
        assert_eq!(error, ConfigError::InvalidPort("99999".to_owned()));
        assert!(error.to_string().contains("99999"));
    }

    #[test]
    fn missing_flag_value_is_reported_readably() {
        let error = from_args_env(vec!["--data-dir".to_owned()], no_env()).expect_err("应拒绝缺值");
        assert_eq!(error, ConfigError::MissingValue("--data-dir"));
    }

    #[test]
    fn port_zero_means_system_assigned() {
        let config = run(
            &["--port", "0"],
            &[(ENV_DATA_DIR, "C:/omw-data"), (ENV_CONFIG, "C:/omw-config.json")],
        );
        assert_eq!(config.port, 0);
    }
}
