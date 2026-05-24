//! qregistry-core — config types and persistence.
//!
//! Defines [`AppConfig`] (top-level TOML), [`Tenant`] (a single registry repo
//! backed by its own rspacefs filesystem), and [`User`] (htpasswd-style
//! credentials). Loaders are sync — config is read once at startup.

pub mod config;
pub mod tenant;
pub mod user;

pub use config::AppConfig;
pub use tenant::{ArtifactKind, StorageTier, Tenant};
pub use user::User;
