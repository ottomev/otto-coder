use axum::{
    Router,
    routing::{IntoMakeService, get},
};
use tower_http::cors::{CorsLayer, Any};

use crate::DeploymentImpl;

pub mod approvals;
pub mod auth;
pub mod config;
pub mod containers;
pub mod filesystem;
pub mod github_accounts;
// pub mod github;
pub mod events;
pub mod execution_processes;
pub mod frontend;
pub mod health;
pub mod images;
pub mod projects;
pub mod task_attempts;
pub mod task_templates;
pub mod tasks;
pub mod web_assist;

pub fn router(deployment: DeploymentImpl) -> IntoMakeService<Router> {
    // Configure CORS to allow requests from localhost:3000 and webassist.otto.lk
    let cors = CorsLayer::new()
        .allow_origin([
            "http://localhost:3000".parse::<axum::http::HeaderValue>().unwrap(),
            "https://webassist.otto.lk".parse::<axum::http::HeaderValue>().unwrap(),
        ])
        .allow_methods(Any)
        .allow_headers(Any);

    // Create routers with different middleware layers
    let base_routes = Router::new()
        .route("/health", get(health::health_check))
        .merge(config::router())
        .merge(containers::router(&deployment))
        .merge(projects::router(&deployment))
        .merge(tasks::router(&deployment))
        .merge(task_attempts::router(&deployment))
        .merge(execution_processes::router(&deployment))
        .merge(task_templates::router(&deployment))
        .merge(auth::router(&deployment))
        .merge(filesystem::router())
        .merge(events::router(&deployment))
        .merge(approvals::router())
        .nest("/images", images::routes())
        .nest("/github-accounts", github_accounts::router(&deployment))
        .nest("/web-assist", web_assist::router(&deployment))
        .layer(cors)
        .with_state(deployment);

    Router::new()
        .nest("/api", base_routes)
        .route("/", get(frontend::serve_frontend_root))
        .route("/{*path}", get(frontend::serve_frontend))
        .into_make_service()
}