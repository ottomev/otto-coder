use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::Json as ResponseJson,
    routing::{get, post},
};
use db::models::github_account::{
    CreateGitHubAccount, GitHubAccount, GitHubAccountError, GitHubAccountSafe, UpdateGitHubAccount,
};
use deployment::Deployment;
use services::services::github_service::{GitHubService, GitHubServiceError};
use utils::response::ApiResponse;
use uuid::Uuid;

use crate::{DeploymentImpl, error::ApiError};

/// GET /api/github-accounts
pub async fn list_accounts(
    State(deployment): State<DeploymentImpl>,
) -> Result<ResponseJson<ApiResponse<Vec<GitHubAccountSafe>>>, ApiError> {
    let accounts = GitHubAccount::find_all(&deployment.db().pool).await?;
    let safe_accounts: Vec<GitHubAccountSafe> = accounts.into_iter().map(Into::into).collect();
    Ok(ResponseJson(ApiResponse::success(safe_accounts)))
}

/// GET /api/github-accounts/:id
pub async fn get_account(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<Uuid>,
) -> Result<ResponseJson<ApiResponse<GitHubAccountSafe>>, StatusCode> {
    match GitHubAccount::find_by_id(&deployment.db().pool, id).await {
        Ok(Some(account)) => Ok(ResponseJson(ApiResponse::success(account.into()))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to get GitHub account: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// POST /api/github-accounts
pub async fn create_account(
    State(deployment): State<DeploymentImpl>,
    Json(payload): Json<CreateGitHubAccount>,
) -> Result<ResponseJson<ApiResponse<GitHubAccountSafe>>, ApiError> {
    // Validate the token if provided
    if let Some(ref token) = payload.oauth_token.as_ref().or(payload.pat.as_ref()) {
        match GitHubService::new(token) {
            Ok(gh_service) => {
                if let Err(e) = gh_service.check_token().await {
                    return match e {
                        GitHubServiceError::TokenInvalid => {
                            Ok(ResponseJson(ApiResponse::error("GitHub token is invalid")))
                        }
                        GitHubServiceError::InsufficientPermissions => Ok(ResponseJson(
                            ApiResponse::error("Insufficient GitHub permissions"),
                        )),
                        _ => {
                            tracing::error!("Failed to validate GitHub token: {}", e);
                            Ok(ResponseJson(ApiResponse::error(
                                "Failed to validate GitHub token",
                            )))
                        }
                    };
                }
            }
            Err(e) => {
                tracing::error!("Failed to create GitHub service: {}", e);
                return Ok(ResponseJson(ApiResponse::error(
                    "Failed to validate GitHub token",
                )));
            }
        }
    }

    match GitHubAccount::create(&deployment.db().pool, &payload).await {
        Ok(account) => {
            // Track account creation event
            deployment
                .track_if_analytics_allowed(
                    "github_account_created",
                    serde_json::json!({
                        "account_id": account.id.to_string(),
                        "username": account.username,
                        "has_oauth": account.oauth_token.is_some(),
                        "has_pat": account.pat.is_some(),
                    }),
                )
                .await;

            Ok(ResponseJson(ApiResponse::success(account.into())))
        }
        Err(GitHubAccountError::UsernameExists(username)) => Ok(ResponseJson(
            ApiResponse::error(&format!("GitHub account '{}' already exists", username)),
        )),
        Err(GitHubAccountError::NoTokenProvided) => Ok(ResponseJson(ApiResponse::error(
            "At least one authentication token (oauth_token or pat) is required",
        ))),
        Err(e) => {
            tracing::error!("Failed to create GitHub account: {}", e);
            Err(e.into())
        }
    }
}

/// PUT /api/github-accounts/:id
pub async fn update_account(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateGitHubAccount>,
) -> Result<ResponseJson<ApiResponse<GitHubAccountSafe>>, ApiError> {
    // Validate the token if being updated
    if let Some(ref token) = payload.oauth_token.as_ref().or(payload.pat.as_ref()) {
        match GitHubService::new(token) {
            Ok(gh_service) => {
                if let Err(e) = gh_service.check_token().await {
                    return match e {
                        GitHubServiceError::TokenInvalid => {
                            Ok(ResponseJson(ApiResponse::error("GitHub token is invalid")))
                        }
                        GitHubServiceError::InsufficientPermissions => Ok(ResponseJson(
                            ApiResponse::error("Insufficient GitHub permissions"),
                        )),
                        _ => {
                            tracing::error!("Failed to validate GitHub token: {}", e);
                            Ok(ResponseJson(ApiResponse::error(
                                "Failed to validate GitHub token",
                            )))
                        }
                    };
                }
            }
            Err(e) => {
                tracing::error!("Failed to create GitHub service: {}", e);
                return Ok(ResponseJson(ApiResponse::error(
                    "Failed to validate GitHub token",
                )));
            }
        }
    }

    match GitHubAccount::update(&deployment.db().pool, id, &payload).await {
        Ok(account) => Ok(ResponseJson(ApiResponse::success(account.into()))),
        Err(GitHubAccountError::AccountNotFound) => {
            Ok(ResponseJson(ApiResponse::error("GitHub account not found")))
        }
        Err(GitHubAccountError::UsernameExists(username)) => Ok(ResponseJson(
            ApiResponse::error(&format!("GitHub account '{}' already exists", username)),
        )),
        Err(e) => {
            tracing::error!("Failed to update GitHub account: {}", e);
            Err(e.into())
        }
    }
}

/// DELETE /api/github-accounts/:id
pub async fn delete_account(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<Uuid>,
) -> Result<ResponseJson<ApiResponse<()>>, StatusCode> {
    // Check if any projects are using this account
    let projects_using_account = sqlx::query!(
        r#"SELECT COUNT(*) as "count!: i64" FROM projects WHERE github_account_id = $1"#,
        id
    )
    .fetch_one(&deployment.db().pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to check projects using account: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if projects_using_account.count > 0 {
        return Ok(ResponseJson(ApiResponse::error(&format!(
            "Cannot delete account: {} project(s) are using this account",
            projects_using_account.count
        ))));
    }

    match GitHubAccount::delete(&deployment.db().pool, id).await {
        Ok(rows_affected) => {
            if rows_affected == 0 {
                Err(StatusCode::NOT_FOUND)
            } else {
                Ok(ResponseJson(ApiResponse::success(())))
            }
        }
        Err(e) => {
            tracing::error!("Failed to delete GitHub account: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// POST /api/github-accounts/:id/validate
pub async fn validate_account_token(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<Uuid>,
) -> Result<ResponseJson<ApiResponse<serde_json::Value>>, StatusCode> {
    let account = match GitHubAccount::find_by_id(&deployment.db().pool, id).await {
        Ok(Some(account)) => account,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to get GitHub account: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let token = match account.token() {
        Some(token) => token,
        None => {
            return Ok(ResponseJson(ApiResponse::success(serde_json::json!({
                "valid": false,
                "error": "No token configured for this account"
            }))));
        }
    };

    match GitHubService::new(&token) {
        Ok(gh_service) => match gh_service.check_token().await {
            Ok(()) => Ok(ResponseJson(ApiResponse::success(serde_json::json!({
                "valid": true
            })))),
            Err(GitHubServiceError::TokenInvalid) => {
                Ok(ResponseJson(ApiResponse::success(serde_json::json!({
                    "valid": false,
                    "error": "Token is invalid or expired"
                }))))
            }
            Err(GitHubServiceError::InsufficientPermissions) => {
                Ok(ResponseJson(ApiResponse::success(serde_json::json!({
                    "valid": false,
                    "error": "Insufficient permissions"
                }))))
            }
            Err(e) => {
                tracing::error!("Failed to validate token: {}", e);
                Ok(ResponseJson(ApiResponse::success(serde_json::json!({
                    "valid": false,
                    "error": "Failed to validate token"
                }))))
            }
        },
        Err(e) => {
            tracing::error!("Failed to create GitHub service: {}", e);
            Ok(ResponseJson(ApiResponse::success(serde_json::json!({
                "valid": false,
                "error": "Failed to create GitHub service"
            }))))
        }
    }
}

pub fn router(_deployment: &DeploymentImpl) -> Router<DeploymentImpl> {
    Router::new()
        .route("/", get(list_accounts).post(create_account))
        .route(
            "/{id}",
            get(get_account).put(update_account).delete(delete_account),
        )
        .route("/{id}/validate", post(validate_account_token))
}
