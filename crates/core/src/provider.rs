//! Core 自持的 provider 配置：校验、快照与原子读写。
//!
//! 配置文件属于 Core，不读宿主配置；里面存的是凭据**引用**（环境变量名或密钥文件路径），
//! 不是密钥值。出站白名单等于已配置 provider 的 endpoint 集合。

use std::fmt;
use std::path::{Path, PathBuf};

use crate::protocol::{
    Capability, CredentialKind, ProviderConfig, ProviderConfigSnapshot, ProviderEntry, Role,
};

/// 读取、校验或写入 provider 配置时的错误。
#[derive(Debug)]
pub enum ConfigError {
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    Parse {
        path: PathBuf,
        source: serde_json::Error,
    },
    Serialize {
        path: PathBuf,
        source: serde_json::Error,
    },
    Write {
        path: PathBuf,
        source: std::io::Error,
    },
    Invalid(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Read { path, source } => {
                write!(f, "无法读取 provider 配置 {}：{source}。", path.display())
            }
            ConfigError::Parse { path, source } => write!(
                f,
                "provider 配置 {} 不是合法 JSON：{source}。字段名用 camelCase，能力值是 llm、embedding、crossEncoder、decision，角色值是 extraction、queryRewrite、ranking、abstention、consolidation、insight。",
                path.display()
            ),
            ConfigError::Write { path, source } => {
                write!(f, "无法写入 provider 配置 {}：{source}。", path.display())
            }
            ConfigError::Serialize { path, source } => write!(
                f,
                "无法把 provider 配置序列化成 JSON（{}）：{source}。",
                path.display()
            ),
            ConfigError::Invalid(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for ConfigError {}

/// 读取配置；文件不存在时返回空配置（所有角色都未配置）。
pub fn load(path: &Path) -> Result<ProviderConfig, ConfigError> {
    match std::fs::read_to_string(path) {
        Ok(raw) => serde_json::from_str(&raw).map_err(|source| ConfigError::Parse {
            path: path.to_path_buf(),
            source,
        }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(ProviderConfig::default()),
        Err(source) => Err(ConfigError::Read {
            path: path.to_path_buf(),
            source,
        }),
    }
}

/// 原子写入配置：先写同目录临时文件，再改名覆盖。
pub fn save(path: &Path, config: &ProviderConfig) -> Result<(), ConfigError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|source| ConfigError::Write {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    let body = serde_json::to_string_pretty(config).map_err(|source| ConfigError::Serialize {
        path: path.to_path_buf(),
        source,
    })?;
    let temporary = path.with_extension("json.tmp");
    std::fs::write(&temporary, format!("{body}\n")).map_err(|source| ConfigError::Write {
        path: temporary.clone(),
        source,
    })?;
    std::fs::rename(&temporary, path).map_err(|source| ConfigError::Write {
        path: path.to_path_buf(),
        source,
    })
}

/// 校验配置结构自身是否自洽，不看凭据此刻能否取到。
fn validate_structure(config: &ProviderConfig) -> Result<(), ConfigError> {
    let mut seen_names: Vec<&str> = Vec::new();
    for (index, provider) in config.providers.iter().enumerate() {
        let name = provider.name.trim();
        if name.is_empty() {
            return Err(ConfigError::Invalid(format!(
                "第 {} 个 provider 缺少 name。",
                index + 1
            )));
        }
        if seen_names.contains(&name) {
            return Err(ConfigError::Invalid(format!(
                "provider 名字 {name} 重复；每个 provider 只能登记一次。"
            )));
        }
        seen_names.push(name);

        if !(provider.endpoint.starts_with("http://") || provider.endpoint.starts_with("https://")) {
            return Err(ConfigError::Invalid(format!(
                "provider {name} 的 endpoint 必须是 http:// 或 https:// 开头的地址。"
            )));
        }
        if provider.capabilities.is_empty() {
            return Err(ConfigError::Invalid(format!(
                "provider {name} 没有声明任何能力；可选：llm、embedding、crossEncoder、decision。"
            )));
        }
        let mut seen_capabilities: Vec<Capability> = Vec::new();
        for capability in &provider.capabilities {
            if seen_capabilities.contains(capability) {
                return Err(ConfigError::Invalid(format!(
                    "provider {name} 重复声明了能力 {}。",
                    capability_name(*capability)
                )));
            }
            seen_capabilities.push(*capability);
        }

        if provider.credential.reference.trim().is_empty() {
            return Err(ConfigError::Invalid(format!(
                "provider {name} 的凭据引用为空；请写环境变量名或密钥文件路径。"
            )));
        }
    }

    let mut seen_roles: Vec<Role> = Vec::new();
    for role_chain in &config.roles {
        if seen_roles.contains(&role_chain.role) {
            return Err(ConfigError::Invalid(format!(
                "角色 {} 重复配置；每个角色只能有一条 fallback 链。",
                role_name(role_chain.role)
            )));
        }
        seen_roles.push(role_chain.role);

        if role_chain.chain.is_empty() {
            return Err(ConfigError::Invalid(format!(
                "角色 {} 的 fallback 链为空；要么绑定至少一个能力，要么不要写这个角色。",
                role_name(role_chain.role)
            )));
        }
        for binding in &role_chain.chain {
            let provider = config
                .providers
                .iter()
                .find(|provider| provider.name == binding.provider)
                .ok_or_else(|| {
                    ConfigError::Invalid(format!(
                        "角色 {} 绑定了不存在的 provider {}。",
                        role_name(role_chain.role),
                        binding.provider
                    ))
                })?;
            if !provider.capabilities.contains(&binding.capability) {
                return Err(ConfigError::Invalid(format!(
                    "provider {} 没有声明能力 {}，角色 {} 不能绑定它。",
                    binding.provider,
                    capability_name(binding.capability),
                    role_name(role_chain.role)
                )));
            }
            let required = role_chain.role.required_capability();
            if binding.capability != required {
                return Err(ConfigError::Invalid(format!(
                    "角色 {} 需要能力 {}，不能绑定 {}。",
                    role_name(role_chain.role),
                    capability_name(required),
                    capability_name(binding.capability)
                )));
            }
        }
    }
    Ok(())
}

/// 凭据引用当前取不到时返回它的引用值；取得到时返回 `None`。
fn missing_credential(provider: &ProviderEntry) -> Option<&str> {
    let reference = provider.credential.reference.trim();
    let present = match provider.credential.kind {
        CredentialKind::Env => std::env::var_os(reference).is_some(),
        CredentialKind::File => Path::new(reference).is_file(),
    };
    (!present).then_some(reference)
}

/// 启动时的校验：只看配置自身是否自洽，不要求凭据此刻取得到。
///
/// 凭据缺失按 ADR-0003 降级处理（记入快照的 `degradedProviders`），不阻止 Core 启动。
pub fn validate_at_startup(config: &ProviderConfig) -> Result<(), ConfigError> {
    validate_structure(config)
}

/// 写入配置时的校验：结构自洽之外，还要求凭据引用此刻可取到。
pub fn validate_for_write(config: &ProviderConfig) -> Result<(), ConfigError> {
    validate_structure(config)?;
    for provider in &config.providers {
        if provider.credential.reference.trim().is_empty() {
            return Err(ConfigError::Invalid(format!(
                "provider {} 的凭据引用为空；请写环境变量名或密钥文件路径。",
                provider.name
            )));
        }
        if let Some(reference) = missing_credential(provider) {
            return Err(ConfigError::Invalid(match provider.credential.kind {
                CredentialKind::Env => format!(
                    "provider {} 引用的环境变量 {reference} 未设置。",
                    provider.name
                ),
                CredentialKind::File => format!(
                    "provider {} 引用的密钥文件 {reference} 不存在。",
                    provider.name
                ),
            }));
        }
    }
    Ok(())
}

/// 生成给读取方的配置快照：补上未配置的角色、凭据取不到的 provider 与出站白名单。
pub fn snapshot(config: &ProviderConfig) -> ProviderConfigSnapshot {
    let unconfigured_roles = Role::ALL
        .iter()
        .copied()
        .filter(|role| !config.roles.iter().any(|chain| chain.role == *role))
        .collect();

    let degraded_providers = config
        .providers
        .iter()
        .filter(|provider| missing_credential(provider).is_some())
        .map(|provider| provider.name.clone())
        .collect();

    let mut outbound_whitelist: Vec<String> = Vec::new();
    for provider in &config.providers {
        if !outbound_whitelist.contains(&provider.endpoint) {
            outbound_whitelist.push(provider.endpoint.clone());
        }
    }

    ProviderConfigSnapshot {
        providers: config.providers.clone(),
        roles: config.roles.clone(),
        unconfigured_roles,
        degraded_providers,
        outbound_whitelist,
    }
}

fn capability_name(capability: Capability) -> &'static str {
    match capability {
        Capability::Llm => "llm",
        Capability::Embedding => "embedding",
        Capability::CrossEncoder => "crossEncoder",
        Capability::Decision => "decision",
    }
}

fn role_name(role: Role) -> &'static str {
    match role {
        Role::Extraction => "extraction",
        Role::QueryRewrite => "queryRewrite",
        Role::Ranking => "ranking",
        Role::Abstention => "abstention",
        Role::Consolidation => "consolidation",
        Role::Insight => "insight",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{CredentialRef, ProviderEntry, RoleBinding, RoleChain};

    fn provider(name: &str, capabilities: Vec<Capability>) -> ProviderEntry {
        ProviderEntry {
            name: name.to_owned(),
            endpoint: format!("http://127.0.0.1:1234/{name}"),
            capabilities,
            credential: CredentialRef {
                kind: CredentialKind::Env,
                reference: "PATH".to_owned(),
            },
        }
    }

    fn config(providers: Vec<ProviderEntry>, roles: Vec<RoleChain>) -> ProviderConfig {
        ProviderConfig { providers, roles }
    }

    #[test]
    fn empty_config_is_valid() {
        validate_for_write(&ProviderConfig::default()).expect("空配置应合法");
    }

    #[test]
    fn duplicate_provider_name_is_rejected() {
        let config = config(
            vec![
                provider("a", vec![Capability::Llm]),
                provider("a", vec![Capability::Embedding]),
            ],
            Vec::new(),
        );
        let error = validate_for_write(&config).expect_err("应拒绝重名");
        assert!(error.to_string().contains("重复"));
    }

    #[test]
    fn unknown_capability_is_rejected_at_parse_time() {
        let raw = r#"{"providers":[{"name":"a","endpoint":"http://x","capabilities":["telepathy"],"credential":{"kind":"env","reference":"PATH"}}],"roles":[]}"#;
        let error = serde_json::from_str::<ProviderConfig>(raw).expect_err("应拒绝未知能力");
        assert!(error.to_string().contains("telepathy"), "报错应点名未知能力：{error}");
    }

    #[test]
    fn role_bound_to_missing_provider_is_rejected() {
        let config = config(
            vec![provider("a", vec![Capability::Llm])],
            vec![RoleChain {
                role: Role::Extraction,
                chain: vec![RoleBinding {
                    provider: "b".to_owned(),
                    capability: Capability::Llm,
                }],
            }],
        );
        let error = validate_for_write(&config).expect_err("应拒绝悬空引用");
        assert!(error.to_string().contains("provider b"), "报错应点名 provider：{error}");
    }

    #[test]
    fn role_bound_to_undeclared_capability_is_rejected() {
        let config = config(
            vec![provider("a", vec![Capability::Llm])],
            vec![RoleChain {
                role: Role::Ranking,
                chain: vec![RoleBinding {
                    provider: "a".to_owned(),
                    capability: Capability::CrossEncoder,
                }],
            }],
        );
        let error = validate_for_write(&config).expect_err("应拒绝未声明能力");
        assert!(error.to_string().contains("crossEncoder"));
    }

    #[test]
    fn snapshot_lists_unconfigured_roles_and_whitelist() {
        let config = config(
            vec![
                provider("a", vec![Capability::Llm]),
                provider("b", vec![Capability::Llm]),
            ],
            vec![RoleChain {
                role: Role::Extraction,
                chain: vec![
                    RoleBinding {
                        provider: "a".to_owned(),
                        capability: Capability::Llm,
                    },
                    RoleBinding {
                        provider: "b".to_owned(),
                        capability: Capability::Llm,
                    },
                ],
            }],
        );
        let snapshot = snapshot(&config);

        assert_eq!(snapshot.roles.len(), 1);
        assert_eq!(snapshot.unconfigured_roles.len(), 5, "五个未配置角色应显式列出");
        assert!(!snapshot.unconfigured_roles.contains(&Role::Extraction));
        assert!(
            snapshot.unconfigured_roles.contains(&Role::Consolidation)
                && snapshot.unconfigured_roles.contains(&Role::Insight),
            "新增的整理与提炼角色应出现在未配置列表里",
        );
        assert_eq!(snapshot.outbound_whitelist.len(), 2);
        assert_eq!(snapshot.roles[0].chain[1].provider, "b", "fallback 顺序应保持");
    }

    #[test]
    fn role_bound_to_wrong_capability_family_is_rejected() {
        let config = config(
            vec![provider("a", vec![Capability::Embedding])],
            vec![RoleChain {
                role: Role::Extraction,
                chain: vec![RoleBinding {
                    provider: "a".to_owned(),
                    capability: Capability::Embedding,
                }],
            }],
        );
        let error = validate_for_write(&config).expect_err("抽取不能绑向量能力");
        assert!(
            error.to_string().contains("需要能力 llm"),
            "报错应说清角色需要哪条能力：{error}"
        );
    }

    #[test]
    fn missing_credential_is_degraded_at_startup_but_rejected_on_write() {
        let config = config(
            vec![provider("a", vec![Capability::Llm])],
            Vec::new(),
        );
        let mut missing = config.clone();
        missing.providers[0].credential.reference = "OMW_TEST_NO_SUCH_ENV".to_owned();

        validate_at_startup(&missing).expect("启动时凭据缺失应降级而不是拒绝启动");
        let snapshot = snapshot(&missing);
        assert_eq!(snapshot.degraded_providers, vec!["a".to_owned()]);

        let error = validate_for_write(&missing).expect_err("写入时应拒绝取不到的凭据");
        assert!(error.to_string().contains("OMW_TEST_NO_SUCH_ENV"));
    }

    #[test]
    fn save_then_load_round_trips() {
        let dir = std::env::temp_dir().join(format!(
            "omw-provider-test-{}-{}",
            std::process::id(),
            time::OffsetDateTime::now_utc().unix_timestamp_nanos()
        ));
        let path = dir.join("config.json");
        let config = config(
            vec![provider("a", vec![Capability::Llm])],
            vec![RoleChain {
                role: Role::QueryRewrite,
                chain: vec![RoleBinding {
                    provider: "a".to_owned(),
                    capability: Capability::Llm,
                }],
            }],
        );

        save(&path, &config).expect("写入配置");
        let loaded = load(&path).expect("读回配置");
        assert_eq!(loaded, config);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_file_loads_as_empty_config() {
        let path = std::env::temp_dir().join("omw-provider-config-missing.json");
        let _ = std::fs::remove_file(&path);
        assert_eq!(load(&path).expect("缺文件应返回空配置"), ProviderConfig::default());
    }

    #[test]
    fn all_six_roles_are_listed_and_bindable() {
        assert_eq!(Role::ALL.len(), 6, "角色闭集应有六个成员");
        for role in Role::ALL {
            assert_eq!(
                role_name(role),
                serde_json::to_value(role).expect("角色应可序列化")
                    .as_str()
                    .expect("角色应序列化成字符串"),
                "报错里的角色名应与 wire 值一致"
            );
        }

        // 六个角色都能绑定，也都能解绑（未绑定时进入 unconfigured_roles）。
        let providers = vec![
            provider("llm", vec![Capability::Llm]),
            provider("rerank", vec![Capability::CrossEncoder]),
            provider("judge", vec![Capability::Decision]),
        ];
        let mut roles = Vec::new();
        for role in Role::ALL {
            let capability = role.required_capability();
            let name = match capability {
                Capability::Llm => "llm",
                Capability::CrossEncoder => "rerank",
                Capability::Decision => "judge",
                Capability::Embedding => unreachable!("没有角色绑定 embedding"),
            };
            roles.push(RoleChain {
                role,
                chain: vec![RoleBinding {
                    provider: name.to_owned(),
                    capability,
                }],
            });
        }
        let bound = config(providers.clone(), roles);
        validate_for_write(&bound).expect("六个角色都应可绑定");
        let bound_snapshot = snapshot(&bound);
        assert!(
            bound_snapshot.unconfigured_roles.is_empty(),
            "六个角色都绑定后不应有未配置项"
        );

        let unbound = config(providers, Vec::new());
        let unbound_snapshot = snapshot(&unbound);
        assert_eq!(unbound_snapshot.unconfigured_roles.len(), 6, "全部解绑后六个角色都应显式报告");
    }

    #[test]
    fn old_four_role_config_loads_and_new_roles_are_reported_unconfigured() {
        // 旧用户的配置里只有四个角色；新增的整理与提炼必须能正常加载，
        // 出现在 unconfigured_roles 里，而不是让 Core 启动失败。
        let raw = r#"{
            "providers":[{"name":"p","endpoint":"http://x","capabilities":["llm","embedding","crossEncoder","decision"],"credential":{"kind":"env","reference":"PATH"}}],
            "roles":[
                {"role":"extraction","chain":[{"provider":"p","capability":"llm"}]},
                {"role":"queryRewrite","chain":[{"provider":"p","capability":"llm"}]},
                {"role":"ranking","chain":[{"provider":"p","capability":"crossEncoder"}]},
                {"role":"abstention","chain":[{"provider":"p","capability":"decision"}]}
            ]
        }"#;
        let loaded: ProviderConfig = serde_json::from_str(raw).expect("旧四角色配置应能加载");
        validate_at_startup(&loaded).expect("旧配置应通过启动校验");

        let snapshot = snapshot(&loaded);
        assert_eq!(
            snapshot.unconfigured_roles,
            vec![Role::Consolidation, Role::Insight],
            "只有新增的两个角色应报告未配置",
        );
    }

    #[test]
    fn unknown_role_error_lists_all_six_legal_values() {
        let raw = r#"{"providers":[],"roles":[{"role":"telepathy","chain":[]}]}"#;
        let error = serde_json::from_str::<ProviderConfig>(raw).expect_err("应拒绝未知角色");
        let message = error.to_string();
        for legal in [
            "extraction",
            "queryRewrite",
            "ranking",
            "abstention",
            "consolidation",
            "insight",
        ] {
            assert!(
                message.contains(legal),
                "未知角色的报错应列出合法值 {legal}：{message}"
            );
        }
    }
}
