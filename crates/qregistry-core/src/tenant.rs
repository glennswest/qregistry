use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// What a repo stores. All are OCI artifacts; the kind selects the
/// media-type set and default access pattern.
///
/// - `Image`  — container images (default).
/// - `Pvc`    — persistent-volume contents (data containers); see the
///   pvc-content-type enhancement. May be a read-only baseline (e.g. a
///   `default` PVC under a release group) or a host-specific repo that
///   snapshots are pushed back to.
/// - `Config` — config-data artifacts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ArtifactKind {
    #[default]
    Image,
    Pvc,
    Config,
}

impl ArtifactKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ArtifactKind::Image => "image",
            ArtifactKind::Pvc => "pvc",
            ArtifactKind::Config => "config",
        }
    }
}

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

/// A single registry repo (leaf), the unit of mount placement.
///
/// OCI repo names are hierarchical, so `name` is the full slash-path:
/// e.g. `4.18.2/kernel`, `4.18.2/system`, `4.18.2/general`. The leading
/// component(s) form a logical group; the leaf is what gets its own mount.
/// Each repo maps to its own root at `<data_dir>/repos/<tier>/<name>`
/// (per-repo placement onto tier mounts via rspace_registry#1).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Tenant {
    /// Full hierarchical repo path, e.g. `4.18.2/kernel`.
    pub name: String,

    #[serde(default)]
    pub description: String,

    /// What this repo stores (image / pvc / config). Defaults to `image`.
    #[serde(default)]
    pub kind: ArtifactKind,

    /// Storage tier (fast SSD vs slow archival). Defaults to `fast`.
    #[serde(default)]
    pub tier: StorageTier,

    /// Explicit mount point override. If unset, derived from tier + name:
    /// `<data_dir>/repos/<tier>/<name>`.
    #[serde(default)]
    pub mount_point: Option<PathBuf>,

    /// If true, anonymous pulls are permitted from this repo.
    #[serde(default)]
    pub public: bool,
}

impl Tenant {
    /// Logical group = everything before the last path component, or "" for
    /// a top-level repo. `4.18.2/kernel` -> `4.18.2`; `alpine` -> "".
    pub fn group(&self) -> &str {
        match self.name.rfind('/') {
            Some(i) => &self.name[..i],
            None => "",
        }
    }

    /// Mount point of this repo's rspacefs filesystem. Honors an explicit
    /// `mount_point`, otherwise derives `<data_dir>/repos/<tier>/<name>`.
    pub fn effective_mount_point(&self, data_dir: &Path) -> PathBuf {
        self.mount_point.clone().unwrap_or_else(|| {
            data_dir.join("repos").join(self.tier.dir()).join(&self.name)
        })
    }
}
