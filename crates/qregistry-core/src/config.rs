use crate::{StorageTier, Tenant, User};
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

    /// One OCI registry instance per storage tier (fast SSD, slow archive).
    /// Each is run by systemd as its own rspace-registry service.
    #[serde(default = "default_registries")]
    pub registries: Vec<RegistryEndpoint>,

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

/// A single OCI registry endpoint, bound to one storage tier. Run as a
/// dedicated rspace-registry systemd service inside the appliance.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegistryEndpoint {
    pub name: String,

    #[serde(default)]
    pub tier: StorageTier,

    /// Externally reachable URL, for display + client docs.
    pub url: String,

    /// Bind address the rspace-registry instance listens on.
    pub listen: String,
}

fn default_registries() -> Vec<RegistryEndpoint> {
    vec![
        RegistryEndpoint {
            name: "fast".into(),
            tier: StorageTier::Fast,
            url: "http://qregistry.g8.lo:5000".into(),
            listen: "0.0.0.0:5000".into(),
        },
        RegistryEndpoint {
            name: "archive".into(),
            tier: StorageTier::Archive,
            url: "http://qregistry.g8.lo:5001".into(),
            listen: "0.0.0.0:5001".into(),
        },
    ]
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
