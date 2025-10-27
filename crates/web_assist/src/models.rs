use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool, Type};
use ts_rs::TS;
use uuid::Uuid;

/// WebAssist project stages (matching backend-team-guide.md)
#[derive(Debug, Clone, Copy, Type, Serialize, Deserialize, PartialEq, Eq, TS)]
#[sqlx(type_name = "web_assist_stage", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum WebAssistStage {
    InitialReview,      // Human - 2h
    AiResearch,         // AI - 2h
    DesignMockup,       // AI + Approval - 8h
    ContentCollection,  // AI + Approval - 6h
    Development,        // AI - 16h
    QualityAssurance,   // Human - 4h
    ClientPreview,      // Human + Approval - 6h
    Deployment,         // AI - 4h
    Delivered,          // Complete - 0h
}

impl WebAssistStage {
    /// Returns true if this stage is executed by AI agents (not human)
    pub fn is_ai_stage(&self) -> bool {
        matches!(
            self,
            WebAssistStage::AiResearch
                | WebAssistStage::DesignMockup
                | WebAssistStage::ContentCollection
                | WebAssistStage::Development
                | WebAssistStage::Deployment
        )
    }

    /// Returns true if this stage requires client approval before proceeding
    pub fn requires_approval(&self) -> bool {
        matches!(
            self,
            WebAssistStage::DesignMockup
                | WebAssistStage::ContentCollection
                | WebAssistStage::ClientPreview
        )
    }

    /// Returns the expected duration in hours for this stage
    pub fn duration_hours(&self) -> u32 {
        match self {
            WebAssistStage::InitialReview => 2,
            WebAssistStage::AiResearch => 2,
            WebAssistStage::DesignMockup => 8,
            WebAssistStage::ContentCollection => 6,
            WebAssistStage::Development => 16,
            WebAssistStage::QualityAssurance => 4,
            WebAssistStage::ClientPreview => 6,
            WebAssistStage::Deployment => 4,
            WebAssistStage::Delivered => 0,
        }
    }

    /// Returns the next stage in the workflow, or None if this is the final stage
    pub fn next_stage(&self) -> Option<WebAssistStage> {
        match self {
            WebAssistStage::InitialReview => Some(WebAssistStage::AiResearch),
            WebAssistStage::AiResearch => Some(WebAssistStage::DesignMockup),
            WebAssistStage::DesignMockup => Some(WebAssistStage::ContentCollection),
            WebAssistStage::ContentCollection => Some(WebAssistStage::Development),
            WebAssistStage::Development => Some(WebAssistStage::QualityAssurance),
            WebAssistStage::QualityAssurance => Some(WebAssistStage::ClientPreview),
            WebAssistStage::ClientPreview => Some(WebAssistStage::Deployment),
            WebAssistStage::Deployment => Some(WebAssistStage::Delivered),
            WebAssistStage::Delivered => None,
        }
    }

    /// Returns all stages in order
    pub fn all_stages() -> Vec<WebAssistStage> {
        vec![
            WebAssistStage::InitialReview,
            WebAssistStage::AiResearch,
            WebAssistStage::DesignMockup,
            WebAssistStage::ContentCollection,
            WebAssistStage::Development,
            WebAssistStage::QualityAssurance,
            WebAssistStage::ClientPreview,
            WebAssistStage::Deployment,
            WebAssistStage::Delivered,
        ]
    }

    /// Returns the approval type for approval-required stages
    pub fn approval_type(&self) -> Option<&'static str> {
        match self {
            WebAssistStage::DesignMockup => Some("design_mockup"),
            WebAssistStage::ContentCollection => Some("content_review"),
            WebAssistStage::ClientPreview => Some("final_preview"),
            _ => None,
        }
    }
}

impl std::fmt::Display for WebAssistStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            WebAssistStage::InitialReview => "initial_review",
            WebAssistStage::AiResearch => "ai_research",
            WebAssistStage::DesignMockup => "design_mockup",
            WebAssistStage::ContentCollection => "content_collection",
            WebAssistStage::Development => "development",
            WebAssistStage::QualityAssurance => "quality_assurance",
            WebAssistStage::ClientPreview => "client_preview",
            WebAssistStage::Deployment => "deployment",
            WebAssistStage::Delivered => "delivered",
        };
        write!(f, "{}", s)
    }
}

/// Sync status for WebAssist projects
#[derive(Debug, Clone, Type, Serialize, Deserialize, PartialEq, Eq, TS)]
#[sqlx(type_name = "sync_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum SyncStatus {
    Active,
    Paused,
    Error,
    Completed,
}

/// Links WebAssist projects (from Supabase) to Otto Coder projects
#[derive(Debug, Clone, FromRow, Serialize, Deserialize, TS)]
pub struct WebAssistProject {
    pub id: Uuid,
    pub webassist_project_id: Uuid, // From Supabase projects table
    pub otto_project_id: Uuid,      // From Otto Coder projects table
    pub current_stage: WebAssistStage,
    pub stage_task_mapping: String, // JSONB: {"initial_review": "task_uuid", ...}
    pub sync_status: SyncStatus,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Approval status for client approvals
#[derive(Debug, Clone, Type, Serialize, Deserialize, PartialEq, Eq, TS)]
#[sqlx(type_name = "approval_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
    ChangesRequested,
}

/// Tracks approval states across both systems
#[derive(Debug, Clone, FromRow, Serialize, Deserialize, TS)]
pub struct WebAssistApproval {
    pub id: Uuid,
    pub web_assist_project_id: Uuid,
    pub stage_name: WebAssistStage,
    pub approval_id: Option<Uuid>, // WebAssist approval ID from Supabase
    pub status: ApprovalStatus,
    pub requested_at: DateTime<Utc>,
    pub responded_at: Option<DateTime<Utc>>,
    pub client_feedback: Option<String>,
    pub preview_url: Option<String>,
    pub deliverables: String, // JSONB: [{name, url, type}]
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to create a new WebAssist project (from webhook)
#[derive(Debug, Deserialize, Serialize, TS)]
pub struct CreateWebAssistProjectRequest {
    pub project_id: Uuid,           // WebAssist project ID
    pub project_number: String,     // e.g., "WA-2025-001"
    pub company_name: String,
    pub wizard_completion_id: Uuid,
    pub is_rush_delivery: bool,
}

/// Approval decision submitted from Otto Coder UI
#[derive(Debug, Deserialize, Serialize, TS)]
pub struct ApprovalDecision {
    pub status: ApprovalStatus,
    pub feedback: Option<String>,
}

/// Webhook event from Supabase
#[derive(Debug, Deserialize, Serialize, TS)]
pub struct WebhookEvent {
    pub event: String, // "project.created", "approval.updated", etc.
    #[serde(flatten)]
    #[ts(type = "Record<string, any>")]
    pub data: serde_json::Value,
}

/// Deliverable file/link for a stage
#[derive(Debug, Serialize, Deserialize, Clone, TS)]
pub struct Deliverable {
    pub id: Uuid,
    pub name: String,
    pub url: String,
    pub r#type: String, // "file", "link", "application/pdf", etc.
    pub size: Option<u64>,
    pub created_at: DateTime<Utc>,
}

impl WebAssistProject {
    /// Find by WebAssist project ID
    pub async fn find_by_webassist_id(
        pool: &SqlitePool,
        webassist_project_id: Uuid,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            WebAssistProject,
            r#"SELECT
                id as "id!: Uuid",
                webassist_project_id as "webassist_project_id!: Uuid",
                otto_project_id as "otto_project_id!: Uuid",
                current_stage as "current_stage!: WebAssistStage",
                stage_task_mapping,
                sync_status as "sync_status!: SyncStatus",
                last_synced_at as "last_synced_at: DateTime<Utc>",
                created_at as "created_at!: DateTime<Utc>",
                updated_at as "updated_at!: DateTime<Utc>"
            FROM web_assist_projects
            WHERE webassist_project_id = $1"#,
            webassist_project_id
        )
        .fetch_optional(pool)
        .await
    }

    /// Find by Otto Coder project ID
    pub async fn find_by_otto_id(
        pool: &SqlitePool,
        otto_project_id: Uuid,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            WebAssistProject,
            r#"SELECT
                id as "id!: Uuid",
                webassist_project_id as "webassist_project_id!: Uuid",
                otto_project_id as "otto_project_id!: Uuid",
                current_stage as "current_stage!: WebAssistStage",
                stage_task_mapping,
                sync_status as "sync_status!: SyncStatus",
                last_synced_at as "last_synced_at: DateTime<Utc>",
                created_at as "created_at!: DateTime<Utc>",
                updated_at as "updated_at!: DateTime<Utc>"
            FROM web_assist_projects
            WHERE otto_project_id = $1"#,
            otto_project_id
        )
        .fetch_optional(pool)
        .await
    }

    /// Create a new WebAssist project link
    pub async fn create(
        pool: &SqlitePool,
        webassist_project_id: Uuid,
        otto_project_id: Uuid,
        stage_task_mapping: String,
    ) -> Result<Self, sqlx::Error> {
        let id = Uuid::new_v4();
        sqlx::query_as!(
            WebAssistProject,
            r#"INSERT INTO web_assist_projects
                (id, webassist_project_id, otto_project_id, current_stage, stage_task_mapping, sync_status)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING
                id as "id!: Uuid",
                webassist_project_id as "webassist_project_id!: Uuid",
                otto_project_id as "otto_project_id!: Uuid",
                current_stage as "current_stage!: WebAssistStage",
                stage_task_mapping,
                sync_status as "sync_status!: SyncStatus",
                last_synced_at as "last_synced_at: DateTime<Utc>",
                created_at as "created_at!: DateTime<Utc>",
                updated_at as "updated_at!: DateTime<Utc>""#,
            id,
            webassist_project_id,
            otto_project_id,
            WebAssistStage::InitialReview as WebAssistStage,
            stage_task_mapping,
            SyncStatus::Active as SyncStatus
        )
        .fetch_one(pool)
        .await
    }

    /// Update current stage
    pub async fn update_stage(
        pool: &SqlitePool,
        id: Uuid,
        stage: WebAssistStage,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE web_assist_projects
            SET current_stage = $2, updated_at = CURRENT_TIMESTAMP
            WHERE id = $1",
            id,
            stage as WebAssistStage
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Update sync status
    pub async fn update_sync_status(
        pool: &SqlitePool,
        id: Uuid,
        status: SyncStatus,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE web_assist_projects
            SET sync_status = $2, last_synced_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP
            WHERE id = $1",
            id,
            status as SyncStatus
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}

impl WebAssistApproval {
    /// Find approval by ID
    pub async fn find_by_id(pool: &SqlitePool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            WebAssistApproval,
            r#"SELECT
                id as "id!: Uuid",
                web_assist_project_id as "web_assist_project_id!: Uuid",
                stage_name as "stage_name!: WebAssistStage",
                approval_id as "approval_id: Uuid",
                status as "status!: ApprovalStatus",
                requested_at as "requested_at!: DateTime<Utc>",
                responded_at as "responded_at: DateTime<Utc>",
                client_feedback,
                preview_url,
                deliverables,
                created_at as "created_at!: DateTime<Utc>",
                updated_at as "updated_at!: DateTime<Utc>"
            FROM web_assist_approvals
            WHERE id = $1"#,
            id
        )
        .fetch_optional(pool)
        .await
    }

    /// Find approval by project and stage
    pub async fn find_by_project_and_stage(
        pool: &SqlitePool,
        project_id: Uuid,
        stage: WebAssistStage,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            WebAssistApproval,
            r#"SELECT
                id as "id!: Uuid",
                web_assist_project_id as "web_assist_project_id!: Uuid",
                stage_name as "stage_name!: WebAssistStage",
                approval_id as "approval_id: Uuid",
                status as "status!: ApprovalStatus",
                requested_at as "requested_at!: DateTime<Utc>",
                responded_at as "responded_at: DateTime<Utc>",
                client_feedback,
                preview_url,
                deliverables,
                created_at as "created_at!: DateTime<Utc>",
                updated_at as "updated_at!: DateTime<Utc>"
            FROM web_assist_approvals
            WHERE web_assist_project_id = $1 AND stage_name = $2
            ORDER BY created_at DESC
            LIMIT 1"#,
            project_id,
            stage as WebAssistStage
        )
        .fetch_optional(pool)
        .await
    }

    /// Create a new approval request
    pub async fn create(
        pool: &SqlitePool,
        project_id: Uuid,
        stage: WebAssistStage,
        preview_url: Option<String>,
        deliverables: String,
    ) -> Result<Self, sqlx::Error> {
        let id = Uuid::new_v4();
        sqlx::query_as!(
            WebAssistApproval,
            r#"INSERT INTO web_assist_approvals
                (id, web_assist_project_id, stage_name, status, requested_at, preview_url, deliverables)
            VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP, $5, $6)
            RETURNING
                id as "id!: Uuid",
                web_assist_project_id as "web_assist_project_id!: Uuid",
                stage_name as "stage_name!: WebAssistStage",
                approval_id as "approval_id: Uuid",
                status as "status!: ApprovalStatus",
                requested_at as "requested_at!: DateTime<Utc>",
                responded_at as "responded_at: DateTime<Utc>",
                client_feedback,
                preview_url,
                deliverables,
                created_at as "created_at!: DateTime<Utc>",
                updated_at as "updated_at!: DateTime<Utc>""#,
            id,
            project_id,
            stage as WebAssistStage,
            ApprovalStatus::Pending as ApprovalStatus,
            preview_url,
            deliverables
        )
        .fetch_one(pool)
        .await
    }

    /// Update approval status
    pub async fn update_status(
        pool: &SqlitePool,
        id: Uuid,
        status: ApprovalStatus,
        feedback: Option<String>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE web_assist_approvals
            SET status = $2, client_feedback = $3, responded_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP
            WHERE id = $1",
            id,
            status as ApprovalStatus,
            feedback
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}
