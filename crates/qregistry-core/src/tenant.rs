use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Storage tier a repo lives on. Each tier is a separate physical drive
/// mounted into the appliance at its own base path, so a repo's rspacefs
/// filesystem inherits the speed/durability characteristics of its tier.
///
/// - `Fast`    — SSD/NVMe-backed, for actively pushed/pulled images.
/// - `Archive` — slow rotating/ZFS-backed, for cold images kept long-term.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum StorageTier {
    #[default]
    Fast,
    Archive,
}

impl StorageTier {
    /// Subdirectory name under the appliance repos root for this tier.
    pub fn dir(self) -> &'static str {
        match self {
            StorageTier::Fast => "fast",
            StorageTier::Archive => "archive",
        }
    }
}

impl std::fmt::Display for StorageTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.dir())
    }
}

/// A single registry tenant (repo). Each maps to its own rspacefs
/// filesystem mounted at `<data_dir>/repos/<tier>/<name>`, where `<tier>`
/// selects the backing physical drive (see [`StorageTier`]).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Tenant {
    pub name: String,

    #[serde(default)]
    pub description: String,

    /// Storage tier (fast SSD vs slow archival). Defaults to `fast`.
    #[serde(default)]
    pub tier: StorageTier,

    /// Explicit mount point override. If unset, derived from tier + name:
    /// `<data_dir>/repos/<tier>/<name>`.
    #[serde(default)]
    pub mount_point: Option<PathBuf>,

    /// If true, anonymous pulls are permitted from this tenant.
    #[serde(default)]
    pub public: bool,
}

impl Tenant {
    /// Mount point of this tenant's rspacefs filesystem. Honors an explicit
    /// `mount_point`, otherwise derives `<data_dir>/repos/<tier>/<name>`.
    pub fn effective_mount_point(&self, data_dir: &Path) -> PathBuf {
        self.mount_point.clone().unwrap_or_else(|| {
            data_dir.join("repos").join(self.tier.dir()).join(&self.name)
        })
    }
}
