use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// What a repo stores. All are OCI artifacts; the kind selects the
/// media-type set and default access pattern.
///
/// - `Image`  — container images (default).
/// - `Pvc`    — persistent-volume contents (data containers). May be a
///   read-only baseline (e.g. a `pvc` repo under a release registry) or a
///   host-specific repo that snapshots are pushed back to (via
///   `rspacefs-pvc` capture on the host).
/// - `Config` — config-data artifacts (incl. keys-as-data-containers, TBD).
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

/// Storage tier — two tiers, set at the registry (release/layer) level.
/// Each is a separate physical drive mounted into the appliance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum StorageTier {
    #[default]
    Fast,
    Archive,
}

impl StorageTier {
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

/// A **registry** — a named release/layer group (e.g. `4.18.4`) that holds
/// multiple repos and lives on one storage tier. Migration between tiers is
/// performed locally by rspacefs (capture/pivot); qregistry coordinates and
/// repoints. The tier here is the default for the registry's repos.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Registry {
    pub name: String,

    #[serde(default)]
    pub description: String,

    /// Storage tier for this registry's repos. Defaults to `fast`.
    #[serde(default)]
    pub tier: StorageTier,
}

/// A **repo** (an rspacefs filesystem) within a registry. Full OCI repo
/// path is `<registry>/<name>` — e.g. registry `4.18.4`, repo `system`
/// → `4.18.4/system`. Each repo is its own rspacefs.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Tenant {
    /// Parent registry name (e.g. `4.18.4`).
    pub registry: String,

    /// Repo name within the registry (e.g. `system`, `user`, `pvc`).
    pub name: String,

    #[serde(default)]
    pub description: String,

    /// What this repo stores (image / pvc / config). Defaults to `image`.
    #[serde(default)]
    pub kind: ArtifactKind,

    /// Explicit mount-point override; otherwise derived from the registry's
    /// tier and the full path.
    #[serde(default)]
    pub mount_point: Option<PathBuf>,

    /// If true, anonymous pulls are permitted from this repo.
    #[serde(default)]
    pub public: bool,
}

impl Tenant {
    /// Full hierarchical OCI repo path, `<registry>/<name>`.
    pub fn full_path(&self) -> String {
        format!("{}/{}", self.registry, self.name)
    }

    /// rspacefs mount point for this repo. Honors an explicit override,
    /// otherwise `<data_dir>/repos/<tier>/<registry>/<name>`.
    pub fn effective_mount_point(&self, data_dir: &Path, tier: StorageTier) -> PathBuf {
        self.mount_point.clone().unwrap_or_else(|| {
            data_dir
                .join("repos")
                .join(tier.dir())
                .join(&self.registry)
                .join(&self.name)
        })
    }
}
