use crate::{Registry, StorageTier, Tenant, User};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Top-level appliance configuration loaded from `qregistry.toml`.
///
/// Hierarchy: the appliance runs **one** OCI service ([`OciEndpoint`]) that
/// hosts multiple [`Registry`]s (release/layer groups, each on a tier), each
/// holding multiple repos ([`Tenant`], an rspacefs filesystem).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    #[serde(default)]
    pub server: ServerConfig,

    /// The single OCI service endpoint (where clients push/pull).
    #[serde(default)]
    pub oci: OciEndpoint,

    /// Release/layer registries (e.g. `4.18.4`), each pinned to a tier.
    #[serde(default)]
    pub registries: Vec<Registry>,

    /// Repos (rspacefs filesystems), each belonging to a registry.
    #[serde(default)]
    pub repos: Vec<Tenant>,

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

/// The single OCI registry service. One `rspace-registry` instance serves
/// all registries/repos; per-repo placement onto tier mounts is delivered
/// by rspace_registry per-repo storage roots (rspace_registry#1).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OciEndpoint {
    /// Externally reachable URL, for display + client docs.
    pub url: String,
    /// Bind address the rspace-registry instance listens on.
    pub listen: String,
}

impl Default for OciEndpoint {
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

    /// Tier of the registry a repo belongs to (defaults to `fast` if the
    /// registry isn't declared).
    pub fn repo_tier(&self, repo: &Tenant) -> StorageTier {
        self.registries
            .iter()
            .find(|r| r.name == repo.registry)
            .map(|r| r.tier)
            .unwrap_or_default()
    }

    /// Resolved rspacefs mount point for a repo.
    pub fn repo_mount(&self, repo: &Tenant) -> PathBuf {
        repo.effective_mount_point(&self.server.data_dir, self.repo_tier(repo))
    }
}
