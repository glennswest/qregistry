//! qregistry-ui — axum HTTP server.
//!
//! Renders admin pages **without** outer nav chrome — stormd wraps this
//! UI in an iframe inside its own dashboard nav, so we render only the
//! page body. CSS palette matches stormd so the embed looks native.

use axum::routing::{get, post};
use axum::Router;
use qregistry_core::AppConfig;
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod api;
pub mod pages;
pub mod style;

pub struct AppState {
    pub config: RwLock<AppConfig>,
}

pub fn build_router(config: AppConfig) -> Router {
    let state = Arc::new(AppState {
        config: RwLock::new(config),
    });

    Router::new()
        .route("/", get(pages::overview))
        .route("/registries", get(pages::registries))
        .route("/repos", get(pages::repos))
        .route("/users", get(pages::users))
        .route("/system", get(pages::system))
        .route("/api/v1/registries", get(api::list_registries))
        .route("/api/v1/repos", get(api::list_repos))
        .route("/api/v1/users", get(api::list_users))
        .route("/api/v1/health", get(api::health))
        .route("/api/v1/reload", post(api::reload))
        .with_state(state)
}
