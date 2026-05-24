use serde::{Deserialize, Serialize};

/// A registry user. Password stored as a bcrypt hash; matches the htpasswd
/// format rspace_registry already accepts via `--auth-file`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct User {
    pub username: String,

    /// bcrypt hash of the password.
    pub password_hash: String,

    #[serde(default)]
    pub admin: bool,

    /// Tenants this user is allowed to push to. Pull is controlled by
    /// [`crate::Tenant::public`] and (later) per-tenant ACLs.
    #[serde(default)]
    pub push_tenants: Vec<String>,
}

impl User {
    pub fn verify(&self, password: &str) -> bool {
        bcrypt::verify(password, &self.password_hash).unwrap_or(false)
    }

    pub fn hash(password: &str) -> anyhow::Result<String> {
        Ok(bcrypt::hash(password, bcrypt::DEFAULT_COST)?)
    }
}
