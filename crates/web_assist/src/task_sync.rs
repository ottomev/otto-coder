use anyhow::{Context, Result};
use db::models::execution_process::{ExecutionContext, ExecutionProcessStatus};
use db::models::task::TaskStatus;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::{models::WebAssistProject, supabase_client::SupabaseClient};

/// Service for synchronizing WebAssist task progress to Supabase
pub struct TaskSyncService {
    pool: SqlitePool,
    supabase_client: Arc<SupabaseClient>,
}

impl TaskSyncService {
    pub fn new(pool: SqlitePool, supabase_client: Arc<SupabaseClient>) -> Self {
        Self {
            pool,
            supabase_client,
        }
    }

    /// Called when a task execution completes (from local-deployment container)
    /// Updates Supabase otto_coder_tasks table with progress
    pub async fn on_execution_completed(&self, ctx: &ExecutionContext) -> Result<()> {
        // Check if this task belongs to a WebAssist project
        let wa_project = match WebAssistProject::find_by_otto_id(&self.pool, ctx.task.project_id).await? {
            Some(project) => project,
            None => {
                // Not a WebAssist project, skip
                return Ok(());
            }
        };

        tracing::debug!(
            "Task execution completed for WebAssist project {}: task_id={}, status={:?}",
            wa_project.webassist_project_id,
            ctx.task.id,
            ctx.execution_process.status
        );

        // Parse stage_task_mapping to find which stage this task belongs to
        let stage_task_mapping: HashMap<String, Uuid> =
            serde_json::from_str(&wa_project.stage_task_mapping)
                .context("Failed to parse stage_task_mapping")?;

        // Find the stage for this task
        let stage_name = stage_task_mapping
            .iter()
            .find(|(_, task_id)| **task_id == ctx.task.id)
            .map(|(stage, _)| stage.clone());

        let stage_name = match stage_name {
            Some(name) => name,
            None => {
                tracing::warn!(
                    "Task {} not found in stage_task_mapping for WebAssist project {}",
                    ctx.task.id,
                    wa_project.webassist_project_id
                );
                return Ok(());
            }
        };

        // Map execution status to task status and progress
        let (status, progress) = match ctx.execution_process.status {
            ExecutionProcessStatus::Running => {
                // Task is in progress
                ("InProgress", 50) // Assume 50% progress when running
            }
            ExecutionProcessStatus::Completed => {
                // Task completed successfully
                ("Done", 100)
            }
            ExecutionProcessStatus::Failed | ExecutionProcessStatus::Killed => {
                // Task failed - keep as InProgress but show 0% progress
                ("InProgress", 0)
            }
        };

        // Update Supabase otto_coder_tasks table
        self.supabase_client
            .update_otto_coder_task(ctx.task.id, progress, status)
            .await
            .context("Failed to update task progress in Supabase")?;

        // Calculate overall project progress (completed tasks / total tasks * 100)
        let completed_count = self.count_completed_tasks(&wa_project).await?;
        let total_tasks = stage_task_mapping.len() as i32;
        let overall_progress = (completed_count * 100) / total_tasks;

        // Update overall project progress
        self.supabase_client
            .update_otto_coder_project(
                wa_project.otto_project_id,
                &wa_project.current_stage.to_string(),
                overall_progress,
            )
            .await
            .context("Failed to update project progress in Supabase")?;

        tracing::info!(
            "Updated WebAssist task progress: project={}, stage={}, status={}, progress={}%, overall_progress={}%",
            wa_project.webassist_project_id,
            stage_name,
            status,
            progress,
            overall_progress
        );

        Ok(())
    }

    /// Count how many tasks are completed (Done status) for this project
    async fn count_completed_tasks(&self, wa_project: &WebAssistProject) -> Result<i32> {
        let stage_task_mapping: HashMap<String, Uuid> =
            serde_json::from_str(&wa_project.stage_task_mapping)
                .context("Failed to parse stage_task_mapping")?;

        let mut completed_count = 0;

        for task_id in stage_task_mapping.values() {
            if let Some(task) = db::models::task::Task::find_by_id(&self.pool, *task_id).await? {
                if matches!(task.status, TaskStatus::Done) {
                    completed_count += 1;
                }
            }
        }

        Ok(completed_count)
    }
}
