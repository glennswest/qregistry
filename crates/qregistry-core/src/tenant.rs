use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A single registry tenant. In v0.1 each tenant maps to a directory under
/// the appliance data dir that is the mount point of its own rspacefs
/// filesystem (mount lifecycle managed by stormd).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Tenant {
    pub name: String,

    #[serde(default)]
    pub description: String,

    /// Mount point for this tenant's rspacefs filesystem.
    /// Defaults to `<server.data_dir>/repos/<name>` if unset.
    #[serde(default)]
    pub mount_point: Option<PathBuf>,

    /// If true, anonymous pulls are permitted from this tenant.
    #[serde(default)]
    pub public: bool,
}

impl Tenant {
    pub fn effective_mount_point(&self, data_dir: &std::path::Path) -> PathBuf {
        self.mount_point
            .clone()
            .unwrap_or_else(|| data_dir.join("repos").join(&self.name))
    }
}
