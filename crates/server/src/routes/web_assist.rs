use axum::{
    Extension, Json, Router,
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::Json as ResponseJson,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use ts_rs::TS;
use utils::response::ApiResponse;
use uuid::Uuid;
use web_assist::{
    models::{ApprovalDecision, ApprovalStatus, Deliverable, WebAssistApproval, WebAssistProject},
    ApprovalSync, ProjectManager, WebhookHandler,
};

use crate::{DeploymentImpl, error::ApiError};
use deployment::Deployment;

/// Response for WebAssist project status
#[derive(Debug, Serialize, TS)]
pub struct WebAssistProjectStatus {
    pub otto_project_id: Uuid,
    pub webassist_project_id: Uuid,
    pub current_stage: String,
    pub sync_status: String,
    pub tasks: Vec<TaskStatus>,
}

#[derive(Debug, Serialize, TS)]
pub struct TaskStatus {
    pub stage: String,
    pub task_id: Uuid,
    pub status: String,
    pub progress: Option<i32>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

/// Webhook endpoint - receives events from Supabase
pub async fn webhook_receiver(
    State(deployment): State<DeploymentImpl>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<ResponseJson<ApiResponse<()>>, StatusCode> {
    // Get webhook signature from header
    let signature = headers
        .get("X-Supabase-Signature")
        .and_then(|v| v.to_str().ok());

    // Get webhook handler from deployment
    let webhook_handler = deployment
        .web_assist_webhook_handler()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    // Process webhook
    match webhook_handler.handle_webhook(&body, signature).await {
        Ok(_) => {
            tracing::info!("Webhook processed successfully");
            Ok(ResponseJson(ApiResponse::success(())))
        }
        Err(e) => {
            tracing::error!("Webhook processing failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get WebAssist project status
pub async fn get_project_status(
    State(deployment): State<DeploymentImpl>,
    Path(webassist_project_id): Path<Uuid>,
) -> Result<ResponseJson<ApiResponse<WebAssistProjectStatus>>, ApiError> {
    // Find WebAssist project
    let wa_project =
        WebAssistProject::find_by_webassist_id(&deployment.db().pool, webassist_project_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("WebAssist project not found".to_string()))?;

    // Parse stage task mapping
    let stage_task_mapping: std::collections::HashMap<String, Uuid> =
        serde_json::from_str(&wa_project.stage_task_mapping)
            .map_err(|e| ApiError::Internal(format!("Failed to parse task mapping: {}", e)))?;

    // Get task statuses
    let mut tasks = Vec::new();
    for (stage, task_id) in stage_task_mapping.iter() {
        if let Ok(Some(task)) = db::models::task::Task::find_by_id(&deployment.db().pool, *task_id).await {
            tasks.push(TaskStatus {
                stage: stage.clone(),
                task_id: task.id,
                status: format!("{:?}", task.status),
                progress: None,
                started_at: Some(task.created_at.to_rfc3339()),
                completed_at: if matches!(task.status, db::models::task::TaskStatus::Done) {
                    Some(task.updated_at.to_rfc3339())
                } else {
                    None
                },
            });
        }
    }

    let status = WebAssistProjectStatus {
        otto_project_id: wa_project.otto_project_id,
        webassist_project_id: wa_project.webassist_project_id,
        current_stage: wa_project.current_stage.to_string(),
        sync_status: format!("{:?}", wa_project.sync_status),
        tasks,
    };

    Ok(ResponseJson(ApiResponse::success(status)))
}

/// Get deliverables for a specific stage
pub async fn get_stage_deliverables(
    State(deployment): State<DeploymentImpl>,
    Path((webassist_project_id, stage_name)): Path<(Uuid, String)>,
) -> Result<ResponseJson<ApiResponse<Vec<Deliverable>>>, ApiError> {
    // Find WebAssist project
    let _wa_project =
        WebAssistProject::find_by_webassist_id(&deployment.db().pool, webassist_project_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("WebAssist project not found".to_string()))?;

    // TODO: Implement deliverables file system
    // For now, return empty list
    let deliverables = Vec::new();

    Ok(ResponseJson(ApiResponse::success(deliverables)))
}

/// Submit approval decision (from Otto Coder UI)
pub async fn submit_approval(
    State(deployment): State<DeploymentImpl>,
    Path(approval_id): Path<Uuid>,
    Json(decision): Json<ApprovalDecision>,
) -> Result<ResponseJson<ApiResponse<()>>, ApiError> {
    // Get approval sync
    let approval_sync = deployment
        .web_assist_approval_sync()
        .ok_or_else(|| ApiError::Internal("WebAssist not configured".to_string()))?;

    // Find approval
    let approval = WebAssistApproval::find_by_id(&deployment.db().pool, approval_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Approval not found".to_string()))?;

    // Check if already responded
    if approval.status != ApprovalStatus::Pending {
        return Err(ApiError::Conflict("Approval already processed".to_string()));
    }

    // Sync to WebAssist
    approval_sync
        .sync_approval_to_webassist(approval_id, decision.status.clone(), decision.feedback)
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to sync approval: {}", e)))?;

    // Find WebAssist project to handle workflow continuation
    let wa_project =
        WebAssistProject::find_by_webassist_id(&deployment.db().pool, approval.web_assist_project_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("WebAssist project not found".to_string()))?;

    // Get project manager
    let project_manager = deployment
        .web_assist_project_manager()
        .ok_or_else(|| ApiError::Internal("WebAssist not configured".to_string()))?;

    // Handle approval response
    project_manager
        .handle_approval_response(
            wa_project.webassist_project_id,
            approval_id,
            decision.status,
            None, // Feedback already passed to sync
        )
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to handle approval: {}", e)))?;

    Ok(ResponseJson(ApiResponse::success(())))
}

/// Get all approvals for a project
pub async fn get_project_approvals(
    State(deployment): State<DeploymentImpl>,
    Path(webassist_project_id): Path<Uuid>,
) -> Result<ResponseJson<ApiResponse<Vec<WebAssistApproval>>>, ApiError> {
    // Find WebAssist project
    let wa_project =
        WebAssistProject::find_by_webassist_id(&deployment.db().pool, webassist_project_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("WebAssist project not found".to_string()))?;

    // Get all approvals for this project
    let approvals = sqlx::query_as!(
        WebAssistApproval,
        r#"SELECT
            id as "id!: Uuid",
            web_assist_project_id as "web_assist_project_id!: Uuid",
            stage_name as "stage_name!: web_assist::models::WebAssistStage",
            approval_id as "approval_id: Uuid",
            status as "status!: web_assist::models::ApprovalStatus",
            requested_at as "requested_at!: chrono::DateTime<chrono::Utc>",
            responded_at as "responded_at: chrono::DateTime<chrono::Utc>",
            client_feedback,
            preview_url,
            deliverables,
            created_at as "created_at!: chrono::DateTime<chrono::Utc>",
            updated_at as "updated_at!: chrono::DateTime<chrono::Utc>"
        FROM web_assist_approvals
        WHERE web_assist_project_id = $1
        ORDER BY created_at DESC"#,
        wa_project.id
    )
    .fetch_all(&deployment.db().pool)
    .await?;

    Ok(ResponseJson(ApiResponse::success(approvals)))
}

/// Manual sync trigger (admin/debug)
pub async fn manual_sync(
    State(deployment): State<DeploymentImpl>,
    Path(webassist_project_id): Path<Uuid>,
) -> Result<ResponseJson<ApiResponse<String>>, ApiError> {
    // Find WebAssist project
    let wa_project =
        WebAssistProject::find_by_webassist_id(&deployment.db().pool, webassist_project_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("WebAssist project not found".to_string()))?;

    tracing::info!("Manual sync triggered for project {}", webassist_project_id);

    // TODO: Implement full sync logic
    // For now, just mark as synced
    WebAssistProject::update_sync_status(
        &deployment.db().pool,
        wa_project.id,
        web_assist::models::SyncStatus::Active,
    )
    .await?;

    Ok(ResponseJson(ApiResponse::success(
        "Sync completed".to_string(),
    )))
}

/// Router for WebAssist endpoints
pub fn router(deployment: &DeploymentImpl) -> Router<DeploymentImpl> {
    Router::new()
        .route("/webhook", post(webhook_receiver))
        .route("/projects/:id", get(get_project_status))
        .route(
            "/projects/:id/stages/:stage/deliverables",
            get(get_stage_deliverables),
        )
        .route("/projects/:id/sync", post(manual_sync))
        .route("/approvals/:id", post(submit_approval))
        .route("/projects/:id/approvals", get(get_project_approvals))
}
