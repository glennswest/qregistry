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
pub struct RegistrySummary {
    pub name: String,
    pub tier: String,
    pub repos: usize,
    pub description: String,
}

pub async fn list_registries(State(state): State<Arc<AppState>>) -> Json<Vec<RegistrySummary>> {
    let cfg = state.config.read().await;
    let out = cfg
        .registries
        .iter()
        .map(|r| RegistrySummary {
            name: r.name.clone(),
            tier: r.tier.to_string(),
            repos: cfg.repos.iter().filter(|t| t.registry == r.name).count(),
            description: r.description.clone(),
        })
        .collect();
    Json(out)
}

#[derive(Serialize)]
pub struct RepoSummary {
    pub path: String,
    pub registry: String,
    pub kind: String,
    pub tier: String,
    pub public: bool,
    pub mount_point: String,
}

pub async fn list_repos(State(state): State<Arc<AppState>>) -> Json<Vec<RepoSummary>> {
    let cfg = state.config.read().await;
    let out = cfg
        .repos
        .iter()
        .map(|t| RepoSummary {
            path: t.full_path(),
            registry: t.registry.clone(),
            kind: t.kind.as_str().to_string(),
            tier: cfg.repo_tier(t).to_string(),
            public: t.public,
            mount_point: cfg.repo_mount(t).display().to_string(),
        })
        .collect();
    Json(out)
}

#[derive(Serialize)]
pub struct UserSummary {
    pub username: String,
    pub admin: bool,
    pub push_repos: Vec<String>,
}

pub async fn list_users(State(state): State<Arc<AppState>>) -> Json<Vec<UserSummary>> {
    let cfg = state.config.read().await;
    let out = cfg
        .users
        .iter()
        .map(|u| UserSummary {
            username: u.username.clone(),
            admin: u.admin,
            push_repos: u.push_tenants.clone(),
        })
        .collect();
    Json(out)
}

#[derive(Serialize)]
pub struct ReloadResponse {
    pub reloaded: bool,
}

pub async fn reload() -> Json<ReloadResponse> {
    // v0.2: re-read config and reconcile registries/repos + rspacefs mounts.
    Json(ReloadResponse { reloaded: false })
}
