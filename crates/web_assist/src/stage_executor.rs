use anyhow::{Context, Result};
use sqlx::SqlitePool;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    models::{WebAssistProject, WebAssistStage},
    supabase_client::SupabaseClient,
};

/// Executes WebAssist stages and manages task transitions
pub struct StageExecutor {
    pool: SqlitePool,
    supabase_client: Arc<SupabaseClient>,
}

impl StageExecutor {
    pub fn new(pool: SqlitePool, supabase_client: Arc<SupabaseClient>) -> Self {
        Self {
            pool,
            supabase_client,
        }
    }

    /// Called when a task completes - determines if we should advance to next stage
    pub async fn on_task_completed(
        &self,
        wa_project_id: Uuid,
        completed_stage: WebAssistStage,
    ) -> Result<()> {
        tracing::info!(
            "Task completed for stage {} in project {}",
            completed_stage,
            wa_project_id
        );

        let wa_project = WebAssistProject::find_by_webassist_id(&self.pool, wa_project_id)
            .await?
            .context("WebAssist project not found")?;

        // Verify this is the current stage
        if wa_project.current_stage != completed_stage {
            tracing::warn!(
                "Completed stage {} doesn't match current stage {}",
                completed_stage,
                wa_project.current_stage
            );
            return Ok(());
        }

        // Handle based on stage type
        if completed_stage.requires_approval() {
            self.handle_approval_required_stage(wa_project_id, completed_stage)
                .await?;
        } else {
            self.advance_to_next_stage(wa_project_id, completed_stage)
                .await?;
        }

        Ok(())
    }

    /// Handle stages that require client approval
    async fn handle_approval_required_stage(
        &self,
        wa_project_id: Uuid,
        stage: WebAssistStage,
    ) -> Result<()> {
        tracing::info!(
            "Stage {} requires approval, creating approval request",
            stage
        );

        // Create approval request in Otto Coder database
        // (handled by approval_sync module)

        // Notify WebAssist that approval is needed
        self.supabase_client
            .create_project_update(
                wa_project_id,
                "approval_requested",
                "Your Approval Needed",
                &format!(
                    "Stage '{}' is complete and ready for your review.",
                    self.stage_display_name(&stage)
                ),
                None,
            )
            .await?;

        Ok(())
    }

    /// Advance to the next stage automatically
    async fn advance_to_next_stage(
        &self,
        wa_project_id: Uuid,
        current_stage: WebAssistStage,
    ) -> Result<()> {
        if let Some(next_stage) = current_stage.next_stage() {
            tracing::info!(
                "Advancing project {} from {} to {}",
                wa_project_id,
                current_stage,
                next_stage
            );

            // Update WebAssist project stage
            WebAssistProject::update_stage(&self.pool, wa_project_id, next_stage).await?;

            // Notify WebAssist
            self.supabase_client
                .create_project_update(
                    wa_project_id,
                    "stage_started",
                    &format!("Stage Started: {}", self.stage_display_name(&next_stage)),
                    &format!(
                        "AI agents are now working on {}",
                        self.stage_display_name(&next_stage)
                    ),
                    None,
                )
                .await?;

            self.supabase_client
                .update_project_stage(wa_project_id, next_stage, 0)
                .await?;

            // Start the next task
            // (Task starting logic handled by Otto Coder's existing task orchestration)
        } else {
            // Project complete!
            self.handle_project_completion(wa_project_id).await?;
        }

        Ok(())
    }

    /// Handle project completion
    async fn handle_project_completion(&self, wa_project_id: Uuid) -> Result<()> {
        tracing::info!("Project {} completed!", wa_project_id);

        self.supabase_client
            .create_project_update(
                wa_project_id,
                "project_completed",
                "🎉 Project Delivered!",
                "Your website is complete and has been delivered. Thank you!",
                None,
            )
            .await?;

        self.supabase_client
            .update_project_stage(wa_project_id, WebAssistStage::Delivered, 100)
            .await?;

        Ok(())
    }

    /// Register a deliverable for a stage (writes to Supabase)
    /// Call this when AI agents create files/assets during stage execution
    pub async fn register_deliverable(
        &self,
        otto_project_id: Uuid,
        stage: WebAssistStage,
        name: &str,
        url: &str,
        file_type: &str, // "file", "link", "preview"
        description: Option<&str>,
        mime_type: Option<&str>,
        size_bytes: Option<i64>,
    ) -> Result<()> {
        tracing::info!(
            "Registering deliverable for stage {}: {}",
            stage,
            name
        );

        self.supabase_client
            .create_otto_coder_deliverable(
                otto_project_id,
                &stage.to_string(),
                name,
                url,
                file_type,
                description,
                mime_type,
                size_bytes,
            )
            .await?;

        Ok(())
    }

    /// Get human-readable stage name
    fn stage_display_name(&self, stage: &WebAssistStage) -> &'static str {
        match stage {
            WebAssistStage::InitialReview => "Initial Review & Research Setup",
            WebAssistStage::AiResearch => "AI Research & Analysis",
            WebAssistStage::DesignMockup => "Design Mockup Creation",
            WebAssistStage::ContentCollection => "Content Collection & SEO",
            WebAssistStage::Development => "Full-Stack Development",
            WebAssistStage::QualityAssurance => "Quality Assurance & Testing",
            WebAssistStage::ClientPreview => "Client Preview & Final Review",
            WebAssistStage::Deployment => "Production Deployment",
            WebAssistStage::Delivered => "Project Delivered",
        }
    }
}
