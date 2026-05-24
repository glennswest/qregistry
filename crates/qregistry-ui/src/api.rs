use crate::AppState;
use axum::extract::State;
use axum::Json;
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub version: &'static str,
}

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    })
}

#[derive(Serialize)]
pub struct TenantSummary {
    pub name: String,
    pub tier: String,
    pub public: bool,
    pub mount_point: String,
    pub description: String,
}

pub async fn list_tenants(State(state): State<Arc<AppState>>) -> Json<Vec<TenantSummary>> {
    let cfg = state.config.read().await;
    let data_dir = cfg.server.data_dir.clone();
    let tenants = cfg
        .tenants
        .iter()
        .map(|t| TenantSummary {
            name: t.name.clone(),
            tier: t.tier.to_string(),
            public: t.public,
            mount_point: t.effective_mount_point(&data_dir).display().to_string(),
            description: t.description.clone(),
        })
        .collect();
    Json(tenants)
}

#[derive(Serialize)]
pub struct UserSummary {
    pub username: String,
    pub admin: bool,
    pub push_tenants: Vec<String>,
}

pub async fn list_users(State(state): State<Arc<AppState>>) -> Json<Vec<UserSummary>> {
    let cfg = state.config.read().await;
    let users = cfg
        .users
        .iter()
        .map(|u| UserSummary {
            username: u.username.clone(),
            admin: u.admin,
            push_tenants: u.push_tenants.clone(),
        })
        .collect();
    Json(users)
}

#[derive(Serialize)]
pub struct ReloadResponse {
    pub reloaded: bool,
}

pub async fn reload() -> Json<ReloadResponse> {
    // v0.1: no-op. v0.2 will re-read config from disk and rewrite stormd
    // config to add/remove rspacefs-mount processes per the new tenant set.
    Json(ReloadResponse { reloaded: false })
}
