use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum TlsBackend {
    Rustls,
    NativeTls,
}

impl Default for TlsBackend {
    fn default() -> Self {
        Self::Rustls
    }
}

/// KNA 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(default = "default_host")]
    pub host: String,

    #[serde(default = "default_port")]
    pub port: u16,

    #[serde(default = "default_region")]
    pub region: String,

    #[serde(default = "default_kiro_version")]
    pub kiro_version: String,

    #[serde(default)]
    pub machine_id: Option<String>,

    #[serde(default = "default_api_key")]
    pub api_key: Option<String>,

    #[serde(default = "default_system_version")]
    pub system_version: String,

    #[serde(default = "default_node_version")]
    pub node_version: String,

    #[serde(default = "default_tls_backend")]
    pub tls_backend: TlsBackend,

    /// 外部 count_tokens API 地址（可选）
    #[serde(default)]
    pub count_tokens_api_url: Option<String>,

    /// count_tokens API 密钥（可选）
    #[serde(default)]
    pub count_tokens_api_key: Option<String>,

    /// count_tokens API 认证类型（可选，"x-api-key" 或 "bearer"，默认 "x-api-key"）
    #[serde(default = "default_count_tokens_auth_type")]
    pub count_tokens_auth_type: String,

    /// HTTP 代理地址（可选）
    /// 支持格式: http://host:port, https://host:port, socks5://host:port
    #[serde(default)]
    pub proxy_url: Option<String>,

    /// 代理认证用户名（可选）
    #[serde(default)]
    pub proxy_username: Option<String>,

    /// 代理认证密码（可选）
    #[serde(default)]
    pub proxy_password: Option<String>,

    /// Admin API 密钥（可选，启用 Admin API 功能）
    #[serde(default)]
    pub admin_api_key: Option<String>,

    /// PostgreSQL 数据库连接地址（可选，启用数据库存储）
    #[serde(default)]
    pub database_url: Option<String>,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    8990
}

fn default_region() -> String {
    "us-east-1".to_string()
}

fn default_kiro_version() -> String {
    "0.8.0".to_string()
}

fn default_system_version() -> String {
    const SYSTEM_VERSIONS: &[&str] = &["darwin#24.6.0", "win32#10.0.22631"];
    SYSTEM_VERSIONS[fastrand::usize(..SYSTEM_VERSIONS.len())].to_string()
}

fn default_node_version() -> String {
    "22.21.1".to_string()
}

fn default_count_tokens_auth_type() -> String {
    "x-api-key".to_string()
}

fn default_tls_backend() -> TlsBackend {
    TlsBackend::Rustls
}

fn default_api_key() -> Option<String> {
    Some("sk-kiro-rs-default-key".to_string())
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            region: default_region(),
            kiro_version: default_kiro_version(),
            machine_id: None,
            api_key: default_api_key(),
            system_version: default_system_version(),
            node_version: default_node_version(),
            tls_backend: default_tls_backend(),
            count_tokens_api_url: None,
            count_tokens_api_key: None,
            count_tokens_auth_type: default_count_tokens_auth_type(),
            proxy_url: None,
            proxy_username: None,
            proxy_password: None,
            admin_api_key: None,
            database_url: None,
        }
    }
}

impl Config {
    /// 获取默认配置文件路径
    pub fn default_config_path() -> &'static str {
        "config.json"
    }

    /// 从文件加载配置，环境变量优先覆盖
    pub fn load<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let path = path.as_ref();
        let mut config = if path.exists() {
            let content = fs::read_to_string(path)?;
            serde_json::from_str::<Config>(&content)?
        } else {
            Self::default()
        };

        // 环境变量覆盖（KIRO_ 前缀）
        macro_rules! env_override {
            ($field:expr, $key:expr) => {
                if let Ok(v) = env::var($key) {
                    $field = v;
                }
            };
            (opt $field:expr, $key:expr) => {
                if let Ok(v) = env::var($key) {
                    $field = Some(v);
                }
            };
            (parse $field:expr, $key:expr) => {
                if let Ok(v) = env::var($key) {
                    if let Ok(parsed) = v.parse() {
                        $field = parsed;
                    }
                }
            };
        }

        env_override!(config.host, "KIRO_HOST");
        env_override!(parse config.port, "KIRO_PORT");
        env_override!(config.region, "KIRO_REGION");
        env_override!(config.kiro_version, "KIRO_VERSION");
        env_override!(opt config.machine_id, "KIRO_MACHINE_ID");
        env_override!(opt config.api_key, "KIRO_API_KEY");
        env_override!(config.system_version, "KIRO_SYSTEM_VERSION");
        env_override!(config.node_version, "KIRO_NODE_VERSION");
        env_override!(opt config.count_tokens_api_url, "KIRO_COUNT_TOKENS_API_URL");
        env_override!(opt config.count_tokens_api_key, "KIRO_COUNT_TOKENS_API_KEY");
        env_override!(config.count_tokens_auth_type, "KIRO_COUNT_TOKENS_AUTH_TYPE");
        env_override!(opt config.proxy_url, "KIRO_PROXY_URL");
        env_override!(opt config.proxy_username, "KIRO_PROXY_USERNAME");
        env_override!(opt config.proxy_password, "KIRO_PROXY_PASSWORD");
        env_override!(opt config.admin_api_key, "KIRO_ADMIN_API_KEY");
        env_override!(opt config.database_url, "KIRO_DATABASE_URL");

        // 兼容通用 DATABASE_URL
        if config.database_url.is_none() {
            if let Ok(v) = env::var("DATABASE_URL") {
                config.database_url = Some(v);
            }
        }

        if let Ok(v) = env::var("KIRO_TLS_BACKEND") {
            match v.as_str() {
                "rustls" => config.tls_backend = TlsBackend::Rustls,
                "native-tls" => config.tls_backend = TlsBackend::NativeTls,
                _ => {}
            }
        }

        Ok(config)
    }
}
