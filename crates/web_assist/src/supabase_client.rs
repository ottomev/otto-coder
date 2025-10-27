use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;
use uuid::Uuid;

use crate::models::{ApprovalStatus, WebAssistStage};

/// Configuration for Supabase client
#[derive(Debug, Clone)]
pub struct SupabaseConfig {
    pub url: String,
    pub anon_key: String,
    pub service_role_key: Option<String>,
}

/// Client for interacting with WebAssist's Supabase backend
#[derive(Clone)]
pub struct SupabaseClient {
    client: Client,
    config: SupabaseConfig,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProjectUpdate {
    project_id: Uuid,
    update_type: String,
    title: String,
    message: String,
    created_by: String,
    is_visible_to_client: bool,
    metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProjectStageUpdate {
    project_id: Uuid,
    status: String,
    stage_progress: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApprovalUpdate {
    approval_id: Uuid,
    status: String,
    responded_at: String,
    client_feedback: Option<String>,
}

impl SupabaseClient {
    /// Create a new Supabase client
    pub fn new(config: SupabaseConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self { client, config })
    }

    /// Get authorization header (prefer service role key for admin operations)
    fn auth_header(&self) -> String {
        if let Some(service_key) = &self.config.service_role_key {
            format!("Bearer {}", service_key)
        } else {
            format!("Bearer {}", self.config.anon_key)
        }
    }

    /// Create a project update in WebAssist's activity feed
    pub async fn create_project_update(
        &self,
        project_id: Uuid,
        update_type: &str,
        title: &str,
        message: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        let url = format!("{}/rest/v1/project_updates", self.config.url);

        let update = ProjectUpdate {
            project_id,
            update_type: update_type.to_string(),
            title: title.to_string(),
            message: message.to_string(),
            created_by: "team:otto-coder".to_string(),
            is_visible_to_client: true,
            metadata,
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .header("apikey", &self.config.anon_key)
            .header("Content-Type", "application/json")
            .header("Prefer", "return=minimal")
            .json(&update)
            .send()
            .await
            .context("Failed to send project update request")?;

        if response.status().is_success() {
            tracing::info!(
                "Created project update for project {}: {}",
                project_id,
                title
            );
            Ok(())
        } else {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!(
                "Failed to create project update (status {}): {}",
                status,
                error_text
            );
        }
    }

    /// Update project stage and progress in WebAssist
    pub async fn update_project_stage(
        &self,
        project_id: Uuid,
        current_stage: WebAssistStage,
        stage_progress: i32,
    ) -> Result<()> {
        let url = format!(
            "{}/rest/v1/projects?id=eq.{}",
            self.config.url, project_id
        );

        let update = json!({
            "current_stage": current_stage.to_string(),
            "stage_progress": stage_progress,
            "updated_at": chrono::Utc::now().to_rfc3339()
        });

        let response = self
            .client
            .patch(&url)
            .header("Authorization", self.auth_header())
            .header("apikey", &self.config.anon_key)
            .header("Content-Type", "application/json")
            .header("Prefer", "return=minimal")
            .json(&update)
            .send()
            .await
            .context("Failed to send project stage update request")?;

        if response.status().is_success() {
            tracing::info!(
                "Updated project {} stage to {} ({}%)",
                project_id,
                current_stage,
                stage_progress
            );
            Ok(())
        } else {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!(
                "Failed to update project stage (status {}): {}",
                status,
                error_text
            );
        }
    }

    /// Create an approval request in WebAssist
    pub async fn create_approval_request(
        &self,
        project_id: Uuid,
        stage_id: Uuid,
        approval_type: &str,
        preview_url: Option<&str>,
        attachments: Option<serde_json::Value>,
    ) -> Result<Uuid> {
        let url = format!("{}/rest/v1/project_approvals", self.config.url);

        let approval = json!({
            "project_id": project_id,
            "stage_id": stage_id,
            "approval_type": approval_type,
            "status": "pending",
            "requested_by": "team:otto-coder",
            "preview_url": preview_url,
            "attachments": attachments.unwrap_or(json!([])),
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .header("apikey", &self.config.anon_key)
            .header("Content-Type", "application/json")
            .header("Prefer", "return=representation")
            .json(&approval)
            .send()
            .await
            .context("Failed to send approval request")?;

        if response.status().is_success() {
            let approval_response: Vec<serde_json::Value> = response
                .json()
                .await
                .context("Failed to parse approval response")?;

            if let Some(approval_data) = approval_response.first() {
                let approval_id = approval_data["id"]
                    .as_str()
                    .and_then(|s| Uuid::parse_str(s).ok())
                    .context("Failed to extract approval ID from response")?;

                tracing::info!(
                    "Created approval request {} for project {}",
                    approval_id,
                    project_id
                );
                Ok(approval_id)
            } else {
                anyhow::bail!("Empty response when creating approval request");
            }
        } else {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!(
                "Failed to create approval request (status {}): {}",
                status,
                error_text
            );
        }
    }

    /// Update an existing approval in WebAssist
    pub async fn update_approval(
        &self,
        approval_id: Uuid,
        status: ApprovalStatus,
        feedback: Option<&str>,
    ) -> Result<()> {
        let url = format!(
            "{}/rest/v1/project_approvals?id=eq.{}",
            self.config.url, approval_id
        );

        let status_str = match status {
            ApprovalStatus::Approved => "approved",
            ApprovalStatus::Rejected => "rejected",
            ApprovalStatus::ChangesRequested => "changes_requested",
            ApprovalStatus::Pending => "pending",
        };

        let update = json!({
            "status": status_str,
            "responded_at": chrono::Utc::now().to_rfc3339(),
            "client_feedback": feedback,
        });

        let response = self
            .client
            .patch(&url)
            .header("Authorization", self.auth_header())
            .header("apikey", &self.config.anon_key)
            .header("Content-Type", "application/json")
            .header("Prefer", "return=minimal")
            .json(&update)
            .send()
            .await
            .context("Failed to send approval update request")?;

        if response.status().is_success() {
            tracing::info!("Updated approval {} to status {:?}", approval_id, status);
            Ok(())
        } else {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!(
                "Failed to update approval (status {}): {}",
                status,
                error_text
            );
        }
    }

    /// Mark a stage as completed in WebAssist
    pub async fn complete_stage(
        &self,
        project_id: Uuid,
        stage_id: Uuid,
        deliverables: Option<serde_json::Value>,
    ) -> Result<()> {
        // Update the stage status
        let url = format!(
            "{}/rest/v1/project_stages?id=eq.{}",
            self.config.url, stage_id
        );

        let mut update = json!({
            "status": "completed",
            "completed_at": chrono::Utc::now().to_rfc3339(),
        });

        if let Some(deliverables_data) = deliverables {
            update["deliverables"] = deliverables_data;
        }

        let response = self
            .client
            .patch(&url)
            .header("Authorization", self.auth_header())
            .header("apikey", &self.config.anon_key)
            .header("Content-Type", "application/json")
            .header("Prefer", "return=minimal")
            .json(&update)
            .send()
            .await
            .context("Failed to send stage completion request")?;

        if response.status().is_success() {
            tracing::info!("Marked stage {} as completed", stage_id);

            // Also create an activity update
            self.create_project_update(
                project_id,
                "stage_completed",
                "Stage Completed",
                "Stage has been completed by AI agent",
                None,
            )
            .await?;

            Ok(())
        } else {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!(
                "Failed to complete stage (status {}): {}",
                status,
                error_text
            );
        }
    }

    /// Fetch project details from WebAssist
    pub async fn get_project(&self, project_id: Uuid) -> Result<serde_json::Value> {
        let url = format!("{}/rest/v1/projects?id=eq.{}", self.config.url, project_id);

        let response = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .header("apikey", &self.config.anon_key)
            .send()
            .await
            .context("Failed to fetch project from WebAssist")?;

        if response.status().is_success() {
            let projects: Vec<serde_json::Value> = response
                .json()
                .await
                .context("Failed to parse project response")?;

            projects
                .into_iter()
                .next()
                .context("Project not found in WebAssist")
        } else {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Failed to fetch project (status {}): {}", status, error_text);
        }
    }

    /// Fetch wizard completion details
    pub async fn get_wizard_completion(
        &self,
        wizard_completion_id: Uuid,
    ) -> Result<serde_json::Value> {
        let url = format!(
            "{}/rest/v1/wizard_completions?id=eq.{}",
            self.config.url, wizard_completion_id
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .header("apikey", &self.config.anon_key)
            .send()
            .await
            .context("Failed to fetch wizard completion from WebAssist")?;

        if response.status().is_success() {
            let completions: Vec<serde_json::Value> = response
                .json()
                .await
                .context("Failed to parse wizard completion response")?;

            completions
                .into_iter()
                .next()
                .context("Wizard completion not found in WebAssist")
        } else {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!(
                "Failed to fetch wizard completion (status {}): {}",
                status,
                error_text
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supabase_client_creation() {
        let config = SupabaseConfig {
            url: "https://example.supabase.co".to_string(),
            anon_key: "test-anon-key".to_string(),
            service_role_key: Some("test-service-key".to_string()),
        };

        let client = SupabaseClient::new(config);
        assert!(client.is_ok());
    }
}
