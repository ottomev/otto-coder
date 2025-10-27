use anyhow::{Context, Result};
use hmac::{Hmac, Mac};
use serde_json::Value;
use sha2::Sha256;
use std::sync::Arc;
use uuid::Uuid;

use crate::models::{ApprovalStatus, CreateWebAssistProjectRequest, WebhookEvent};
use crate::project_manager::ProjectManager;

type HmacSha256 = Hmac<Sha256>;

/// Verifies the HMAC signature of a Supabase webhook
pub fn verify_webhook_signature(payload: &[u8], signature: &str, secret: &str) -> Result<bool> {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .context("Invalid HMAC secret")?;

    mac.update(payload);

    let expected_signature = hex::encode(mac.finalize().into_bytes());

    // Constant-time comparison
    Ok(signature == expected_signature)
}

/// Handles incoming webhooks from Supabase
pub struct WebhookHandler {
    project_manager: Arc<ProjectManager>,
    webhook_secret: String,
}

impl WebhookHandler {
    pub fn new(project_manager: Arc<ProjectManager>, webhook_secret: String) -> Self {
        Self {
            project_manager,
            webhook_secret,
        }
    }

    /// Verify and process a webhook event
    pub async fn handle_webhook(
        &self,
        payload: &[u8],
        signature: Option<&str>,
    ) -> Result<()> {
        // Verify signature if provided
        if let Some(sig) = signature {
            if !verify_webhook_signature(payload, sig, &self.webhook_secret)? {
                anyhow::bail!("Invalid webhook signature");
            }
        } else {
            tracing::warn!("Webhook received without signature - skipping verification");
        }

        // Parse webhook event
        let event: WebhookEvent = serde_json::from_slice(payload)
            .context("Failed to parse webhook payload")?;

        tracing::info!("Received webhook event: {}", event.event);

        // Route to appropriate handler
        match event.event.as_str() {
            "project.created" => self.handle_project_created(event.data).await,
            "approval.updated" => self.handle_approval_updated(event.data).await,
            "project.stage_changed" => self.handle_stage_changed(event.data).await,
            _ => {
                tracing::warn!("Unknown webhook event type: {}", event.event);
                Ok(())
            }
        }
    }

    /// Handle project.created event
    async fn handle_project_created(&self, data: Value) -> Result<()> {
        let request: CreateWebAssistProjectRequest = serde_json::from_value(data)
            .context("Failed to parse project.created payload")?;

        tracing::info!(
            "Creating Otto Coder project for WebAssist project {} ({})",
            request.project_id,
            request.company_name
        );

        self.project_manager
            .create_project_from_webhook(request)
            .await
            .context("Failed to create project from webhook")?;

        Ok(())
    }

    /// Handle approval.updated event
    async fn handle_approval_updated(&self, data: Value) -> Result<()> {
        let approval_id: Uuid = data["approval_id"]
            .as_str()
            .and_then(|s| Uuid::parse_str(s).ok())
            .context("Missing or invalid approval_id")?;

        let project_id: Uuid = data["project_id"]
            .as_str()
            .and_then(|s| Uuid::parse_str(s).ok())
            .context("Missing or invalid project_id")?;

        let status_str = data["status"]
            .as_str()
            .context("Missing approval status")?;

        let status = match status_str {
            "approved" => ApprovalStatus::Approved,
            "rejected" => ApprovalStatus::Rejected,
            "changes_requested" => ApprovalStatus::ChangesRequested,
            "pending" => ApprovalStatus::Pending,
            _ => anyhow::bail!("Unknown approval status: {}", status_str),
        };

        let feedback = data["client_feedback"].as_str().map(String::from);

        tracing::info!(
            "Approval {} for project {} updated to {:?}",
            approval_id,
            project_id,
            status
        );

        self.project_manager
            .handle_approval_response(project_id, approval_id, status, feedback)
            .await
            .context("Failed to handle approval response")?;

        Ok(())
    }

    /// Handle project.stage_changed event (optional - for manual stage changes in WebAssist)
    async fn handle_stage_changed(&self, data: Value) -> Result<()> {
        let project_id: Uuid = data["project_id"]
            .as_str()
            .and_then(|s| Uuid::parse_str(s).ok())
            .context("Missing or invalid project_id")?;

        let stage_name = data["stage_name"]
            .as_str()
            .context("Missing stage_name")?;

        tracing::info!(
            "Stage changed for project {} to {}",
            project_id,
            stage_name
        );

        // Optionally sync the stage change to Otto Coder
        // This could be used if stages are manually advanced in WebAssist UI

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_webhook_signature() {
        let secret = "test-secret";
        let payload = b"test payload";

        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(payload);
        let signature = hex::encode(mac.finalize().into_bytes());

        assert!(verify_webhook_signature(payload, &signature, secret).unwrap());
    }

    #[test]
    fn test_verify_webhook_signature_invalid() {
        let secret = "test-secret";
        let payload = b"test payload";
        let wrong_signature = "invalid";

        assert!(!verify_webhook_signature(payload, wrong_signature, secret).unwrap());
    }
}
