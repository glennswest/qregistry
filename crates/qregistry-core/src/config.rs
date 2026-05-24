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

    /// The single OCI registry instance. It serves all repos; per-repo
    /// placement onto different tier mounts is delivered by rspace_registry
    /// per-repo storage roots (rspace_registry#1) and expressed via each
    /// tenant's `tier`/`mount_point`.
    #[serde(default)]
    pub registry: RegistryEndpoint,

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
            bind: "0.0.0.0:8081".to_string(),
            data_dir: PathBuf::from("/var/lib/qregistry"),
        }
    }
}

/// The OCI registry endpoint. A single rspace-registry instance serves all
/// repos; per-repo placement onto tier mounts is handled by rspace_registry
/// per-repo storage roots (rspace_registry#1).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegistryEndpoint {
    /// Externally reachable URL, for display + client docs.
    pub url: String,

    /// Bind address the rspace-registry instance listens on.
    pub listen: String,
}

impl Default for RegistryEndpoint {
    fn default() -> Self {
        Self {
            url: "http://qregistry.g8.lo:5000".into(),
            listen: "0.0.0.0:5000".into(),
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
