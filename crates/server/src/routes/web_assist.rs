use axum::{
    Json, Router,
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{Json as ResponseJson, sse::{Event, KeepAlive, Sse}},
    routing::{get, post},
};
use futures::stream::Stream;
use serde::Serialize;
use std::{convert::Infallible, time::Duration};
use tokio::time::interval;
use ts_rs::TS;
use utils::response::ApiResponse;
use uuid::Uuid;
use web_assist::{
    models::{ApprovalDecision, ApprovalStatus, Deliverable, WebAssistApproval, WebAssistProject},
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

/// SSE event types for WebAssist
#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WebAssistEvent {
    StageChanged {
        project_id: Uuid,
        old_stage: String,
        new_stage: String,
    },
    ApprovalRequested {
        project_id: Uuid,
        approval_id: Uuid,
        stage: String,
    },
    ApprovalResponded {
        project_id: Uuid,
        approval_id: Uuid,
        status: String,
    },
    TaskStarted {
        project_id: Uuid,
        task_id: Uuid,
        stage: String,
    },
    TaskCompleted {
        project_id: Uuid,
        task_id: Uuid,
        stage: String,
    },
    SyncStatusChanged {
        project_id: Uuid,
        old_status: String,
        new_status: String,
    },
}

/// Summary response for project list
#[derive(Debug, Serialize, TS)]
pub struct WebAssistProjectSummary {
    pub id: Uuid,
    pub webassist_project_id: Uuid,
    pub otto_project_id: Uuid,
    pub company_name: String,
    pub current_stage: String,
    pub sync_status: String,
    pub pending_approvals_count: i32,
    pub created_at: String,
    pub updated_at: String,
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
/// DEPRECATED: Frontend reads from Supabase directly now
#[allow(dead_code)]
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
/// DEPRECATED: Frontend reads from Supabase directly now
#[allow(dead_code)]
pub async fn get_stage_deliverables(
    State(deployment): State<DeploymentImpl>,
    Path((webassist_project_id, _stage_name)): Path<(Uuid, String)>,
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
/// DEPRECATED: Frontend reads from Supabase directly now
#[allow(dead_code)]
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

/// List all WebAssist projects
pub async fn list_projects(
    State(deployment): State<DeploymentImpl>,
) -> Result<ResponseJson<ApiResponse<Vec<WebAssistProjectSummary>>>, ApiError> {
    // Get all WebAssist projects
    let wa_projects = WebAssistProject::find_all(&deployment.db().pool).await?;

    // Get project names and pending approval counts
    let mut summaries = Vec::new();
    for wa_project in wa_projects {
        // Get Otto project for company name
        let otto_project = db::models::project::Project::find_by_id(
            &deployment.db().pool,
            wa_project.otto_project_id,
        )
        .await?
        .ok_or_else(|| ApiError::NotFound("Otto project not found".to_string()))?;

        // Count pending approvals
        let pending_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM web_assist_approvals WHERE web_assist_project_id = $1 AND status = 'pending'"
        )
        .bind(wa_project.id)
        .fetch_one(&deployment.db().pool)
        .await
        .unwrap_or(0);

        summaries.push(WebAssistProjectSummary {
            id: wa_project.id,
            webassist_project_id: wa_project.webassist_project_id,
            otto_project_id: wa_project.otto_project_id,
            company_name: otto_project.name,
            current_stage: wa_project.current_stage.to_string(),
            sync_status: format!("{:?}", wa_project.sync_status),
            pending_approvals_count: pending_count as i32,
            created_at: wa_project.created_at.to_rfc3339(),
            updated_at: wa_project.updated_at.to_rfc3339(),
        });
    }

    Ok(ResponseJson(ApiResponse::success(summaries)))
}

/// SSE endpoint for WebAssist project events
pub async fn project_events(
    State(deployment): State<DeploymentImpl>,
    Path(webassist_project_id): Path<Uuid>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // Find the WebAssist project to ensure it exists
    let wa_project_result = WebAssistProject::find_by_webassist_id(
        &deployment.db().pool,
        webassist_project_id,
    )
    .await;

    // Clone pool for use in stream
    let pool = deployment.db().pool.clone();

    let stream = async_stream::stream! {
        // If project doesn't exist, send error and end stream
        if wa_project_result.is_err() || wa_project_result.as_ref().ok().and_then(|p| p.as_ref()).is_none() {
            let error_event = serde_json::json!({
                "type": "error",
                "message": "WebAssist project not found"
            });
            yield Ok(Event::default().json_data(error_event).unwrap());
            return;
        }

        let mut ticker = interval(Duration::from_secs(5));
        let mut last_stage = String::new();
        let mut last_sync_status = String::new();

        loop {
            ticker.tick().await;

            // Query current state
            match WebAssistProject::find_by_webassist_id(&pool, webassist_project_id).await {
                Ok(Some(project)) => {
                    let current_stage = project.current_stage.to_string();
                    let current_sync = format!("{:?}", project.sync_status);

                    // Check for stage change
                    if !last_stage.is_empty() && last_stage != current_stage {
                        let event = WebAssistEvent::StageChanged {
                            project_id: webassist_project_id,
                            old_stage: last_stage.clone(),
                            new_stage: current_stage.clone(),
                        };
                        if let Ok(data) = serde_json::to_value(&event) {
                            yield Ok(Event::default().json_data(data).unwrap());
                        }
                    }

                    // Check for sync status change
                    if !last_sync_status.is_empty() && last_sync_status != current_sync {
                        let event = WebAssistEvent::SyncStatusChanged {
                            project_id: webassist_project_id,
                            old_status: last_sync_status.clone(),
                            new_status: current_sync.clone(),
                        };
                        if let Ok(data) = serde_json::to_value(&event) {
                            yield Ok(Event::default().json_data(data).unwrap());
                        }
                    }

                    last_stage = current_stage;
                    last_sync_status = current_sync;
                }
                Ok(None) => {
                    // Project was deleted
                    break;
                }
                Err(e) => {
                    tracing::error!("Error fetching WebAssist project: {}", e);
                    break;
                }
            }
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// Router for WebAssist endpoints
pub fn router(_deployment: &DeploymentImpl) -> Router<DeploymentImpl> {
    Router::new()
        .route("/webhook", post(webhook_receiver))
        .route("/projects", get(list_projects))
        // Removed: GET /projects/{id} - Frontend reads from Supabase directly
        .route("/projects/{id}/events", get(project_events))
        // Removed: GET /projects/{id}/stages/{stage}/deliverables - Not needed
        .route("/projects/{id}/sync", post(manual_sync))
        .route("/approvals/{id}", post(submit_approval))
        // Removed: GET /projects/{id}/approvals - Not needed
}
