use crate::{Tenant, User};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Top-level appliance configuration loaded from `qregistry.toml`.
///
/// The matching `stormd.toml` declares the supervised processes — see
/// `config/stormd.toml`. These two files are co-designed: every tenant in
/// `tenants` should have a corresponding `rspacefs-mount.<name>` process in
/// the stormd config so its data dir is a real rspacefs mount.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    #[serde(default)]
    pub server: ServerConfig,

    #[serde(default)]
    pub registry: RegistryConfig,

    #[serde(default)]
    pub tenants: Vec<Tenant>,

    #[serde(default)]
    pub users: Vec<User>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub bind: String,
    pub data_dir: PathBuf,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind: "127.0.0.1:8081".to_string(),
            data_dir: PathBuf::from("/var/lib/qregistry"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegistryConfig {
    pub url: String,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            url: "http://127.0.0.1:5000".to_string(),
        }
    }
}

impl AppConfig {
    pub fn load(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("read config {}", path.display()))?;
        let cfg: AppConfig =
            toml::from_str(&text).with_context(|| format!("parse config {}", path.display()))?;
        Ok(cfg)
    }
}
