use anyhow::{Context, Result};
use serde_json::json;
use sqlx::SqlitePool;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    models::{ApprovalStatus, Deliverable, WebAssistApproval, WebAssistProject, WebAssistStage},
    supabase_client::SupabaseClient,
};

/// Manages bidirectional synchronization of approval states
pub struct ApprovalSync {
    pool: SqlitePool,
    supabase_client: Arc<SupabaseClient>,
}

impl ApprovalSync {
    pub fn new(pool: SqlitePool, supabase_client: Arc<SupabaseClient>) -> Self {
        Self {
            pool,
            supabase_client,
        }
    }

    /// Create approval request in both Otto Coder and WebAssist
    pub async fn create_approval_request(
        &self,
        wa_project_id: Uuid,
        stage: WebAssistStage,
        preview_url: Option<String>,
        deliverables: Vec<Deliverable>,
    ) -> Result<WebAssistApproval> {
        tracing::info!(
            "Creating approval request for project {} stage {}",
            wa_project_id,
            stage
        );

        // Find WebAssist project
        let wa_project = WebAssistProject::find_by_webassist_id(&self.pool, wa_project_id)
            .await?
            .context("WebAssist project not found")?;

        // Serialize deliverables
        let deliverables_json = serde_json::to_string(&deliverables)?;

        // Create in Otto Coder database
        let approval = WebAssistApproval::create(
            &self.pool,
            wa_project.id,
            stage,
            preview_url.clone(),
            deliverables_json.clone(),
        )
        .await?;

        // Convert deliverables to Supabase format
        let attachments = json!(
            deliverables
                .iter()
                .map(|d| {
                    json!({
                        "id": d.id,
                        "name": d.name,
                        "url": d.url,
                        "type": d.r#type,
                        "size": d.size
                    })
                })
                .collect::<Vec<_>>()
        );

        // Create in WebAssist Supabase
        let approval_type = stage
            .approval_type()
            .context("Stage does not require approval")?;

        // Get stage_id from WebAssist (we need to query this)
        // For now, we'll use a placeholder - in production, this should be fetched
        let stage_id = Uuid::new_v4(); // TODO: Fetch actual stage_id from Supabase

        let wa_approval_id = self
            .supabase_client
            .create_approval_request(
                wa_project_id,
                stage_id,
                approval_type,
                preview_url.as_deref(),
                Some(attachments),
            )
            .await?;

        // Update Otto Coder approval with WebAssist approval ID
        sqlx::query!(
            "UPDATE web_assist_approvals SET approval_id = $2 WHERE id = $1",
            approval.id,
            wa_approval_id
        )
        .execute(&self.pool)
        .await?;

        tracing::info!(
            "Created approval request {} (WebAssist: {})",
            approval.id,
            wa_approval_id
        );

        Ok(approval)
    }

    /// Sync approval response from Otto Coder to WebAssist
    pub async fn sync_approval_to_webassist(
        &self,
        approval_id: Uuid,
        status: ApprovalStatus,
        feedback: Option<String>,
    ) -> Result<()> {
        tracing::info!(
            "Syncing approval {} to WebAssist with status {:?}",
            approval_id,
            status
        );

        // Find approval
        let approval = WebAssistApproval::find_by_id(&self.pool, approval_id)
            .await?
            .context("Approval not found")?;

        // Update in Otto Coder
        WebAssistApproval::update_status(&self.pool, approval_id, status.clone(), feedback.clone())
            .await?;

        // Sync to WebAssist if we have a WebAssist approval ID
        if let Some(wa_approval_id) = approval.approval_id {
            self.supabase_client
                .update_approval(wa_approval_id, status, feedback.as_deref())
                .await?;
        } else {
            tracing::warn!(
                "Approval {} has no WebAssist approval ID, cannot sync",
                approval_id
            );
        }

        Ok(())
    }

    /// Sync approval response from WebAssist to Otto Coder
    pub async fn sync_approval_from_webassist(
        &self,
        wa_approval_id: Uuid,
        status: ApprovalStatus,
        feedback: Option<String>,
    ) -> Result<()> {
        tracing::info!(
            "Syncing approval {} from WebAssist with status {:?}",
            wa_approval_id,
            status
        );

        // Find approval by WebAssist approval ID
        let approval = sqlx::query!(
            r#"SELECT id as "id!: Uuid" FROM web_assist_approvals WHERE approval_id = $1"#,
            wa_approval_id
        )
        .fetch_optional(&self.pool)
        .await?
        .context("Approval not found by WebAssist approval ID")?;

        // Update in Otto Coder
        WebAssistApproval::update_status(&self.pool, approval.id, status, feedback).await?;

        Ok(())
    }

    /// Check for approval conflicts (if both systems were updated independently)
    pub async fn resolve_conflicts(&self) -> Result<()> {
        tracing::debug!("Checking for approval conflicts...");

        // Query approvals that might be out of sync
        let approvals = sqlx::query!(
            r#"SELECT
                id as "id!: Uuid",
                approval_id as "approval_id: Uuid",
                status as "status!: ApprovalStatus",
                updated_at as "updated_at: chrono::DateTime<chrono::Utc>"
            FROM web_assist_approvals
            WHERE status = 'pending' AND approval_id IS NOT NULL"#
        )
        .fetch_all(&self.pool)
        .await?;

        for approval in approvals {
            // In production, fetch from WebAssist and compare timestamps
            // For now, we log the potential conflict
            tracing::debug!("Checking approval {} for conflicts", approval.id);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_approval_sync_creation() {
        // Test placeholder
        assert!(true);
    }
}
