use anyhow::{Context, Result};
use db::models::{
    project::{CreateProject, Project},
    task::{CreateTask, Task},
};
use serde_json::json;
use sqlx::SqlitePool;
use std::{collections::HashMap, path::PathBuf};
use uuid::Uuid;

use crate::{
    models::*,
    supabase_client::SupabaseClient,
};

/// Manages WebAssist project creation and lifecycle
pub struct ProjectManager {
    pool: SqlitePool,
    supabase_client: SupabaseClient,
    projects_directory: PathBuf,
}

impl ProjectManager {
    pub fn new(
        pool: SqlitePool,
        supabase_client: SupabaseClient,
        projects_directory: PathBuf,
    ) -> Self {
        Self {
            pool,
            supabase_client,
            projects_directory,
        }
    }

    /// Create an Otto Coder project from a WebAssist webhook
    pub async fn create_project_from_webhook(
        &self,
        request: CreateWebAssistProjectRequest,
    ) -> Result<WebAssistProject> {
        tracing::info!(
            "Creating Otto Coder project for WebAssist project {} ({})",
            request.project_id,
            request.company_name
        );

        // 1. Fetch wizard completion data from Supabase to get full requirements
        let wizard_data = self
            .supabase_client
            .get_wizard_completion(request.wizard_completion_id)
            .await
            .context("Failed to fetch wizard completion data")?;

        tracing::debug!("Wizard completion data: {:?}", wizard_data);

        // 2. Create project directory
        let project_dir = self
            .projects_directory
            .join(request.project_id.to_string());

        if !project_dir.exists() {
            std::fs::create_dir_all(&project_dir)
                .context("Failed to create project directory")?;
        }

        // 3. Create Otto Coder project
        let otto_project = self.create_otto_project(&request, &project_dir).await?;

        // 4. Initialize Next.js project
        self.initialize_nextjs_project(&project_dir).await?;

        // 5. Create 9 Otto Coder tasks (one per stage)
        let stage_task_mapping = self
            .create_stage_tasks(otto_project.id, &request, &wizard_data)
            .await?;

        // 6. Create WebAssistProject link
        let wa_project = WebAssistProject::create(
            &self.pool,
            request.project_id,
            otto_project.id,
            serde_json::to_string(&stage_task_mapping)?,
        )
        .await?;

        // 7. Notify WebAssist of project creation
        self.supabase_client
            .create_project_update(
                request.project_id,
                "project_created",
                "Otto Coder Project Created",
                &format!(
                    "AI agents are now setting up your project. Initial review is starting..."
                ),
                Some(json!({
                    "otto_project_id": otto_project.id.to_string()
                })),
            )
            .await?;

        // 8. Start first task (Initial Review - AI Research)
        self.start_first_task(wa_project.id, &stage_task_mapping)
            .await?;

        tracing::info!(
            "Successfully created Otto Coder project {} for WebAssist project {}",
            otto_project.id,
            request.project_id
        );

        Ok(wa_project)
    }

    /// Create Otto Coder project
    async fn create_otto_project(
        &self,
        request: &CreateWebAssistProjectRequest,
        project_dir: &PathBuf,
    ) -> Result<Project> {
        let project_data = CreateProject {
            name: format!("{} ({})", request.company_name, request.project_number),
            git_repo_path: project_dir.join("project").to_string_lossy().to_string(),
            use_existing_repo: false,
            setup_script: Some("npm install".to_string()),
            dev_script: Some("npm run dev".to_string()),
            cleanup_script: None,
            copy_files: None,
            github_account_id: None,
        };

        Project::create(&self.pool, &project_data, Uuid::new_v4())
            .await
            .context("Failed to create Otto Coder project")
    }

    /// Initialize Next.js project in the project directory
    async fn initialize_nextjs_project(&self, project_dir: &PathBuf) -> Result<()> {
        let nextjs_dir = project_dir.join("project");

        if nextjs_dir.exists() {
            tracing::warn!("Next.js project directory already exists, skipping initialization");
            return Ok(());
        }

        tracing::info!("Initializing Next.js project at {:?}", nextjs_dir);

        // Create Next.js project using create-next-app
        let output = tokio::process::Command::new("npx")
            .args([
                "create-next-app@latest",
                nextjs_dir.to_str().unwrap(),
                "--typescript",
                "--tailwind",
                "--app",
                "--no-src-dir",
                "--import-alias",
                "@/*",
                "--use-npm",
            ])
            .output()
            .await
            .context("Failed to execute create-next-app")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to create Next.js project: {}", stderr);
        }

        tracing::info!("Next.js project initialized successfully");
        Ok(())
    }

    /// Create 9 tasks (one per WebAssist stage)
    async fn create_stage_tasks(
        &self,
        project_id: Uuid,
        request: &CreateWebAssistProjectRequest,
        wizard_data: &serde_json::Value,
    ) -> Result<HashMap<String, Uuid>> {
        let mut mapping = HashMap::new();
        let stages = WebAssistStage::all_stages();

        for (index, stage) in stages.iter().enumerate() {
            let task_data = CreateTask {
                project_id,
                title: format!("Stage {}: {}", index + 1, self.stage_display_name(stage)),
                description: Some(self.stage_description(stage, request, wizard_data)),
                parent_task_attempt: None,
                image_ids: None,
            };

            let task = Task::create(&self.pool, &task_data, Uuid::new_v4()).await?;

            mapping.insert(stage.to_string(), task.id);

            tracing::debug!(
                "Created task {} for stage {}",
                task.id,
                stage.to_string()
            );
        }

        Ok(mapping)
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

    /// Generate task description for a stage
    fn stage_description(
        &self,
        stage: &WebAssistStage,
        request: &CreateWebAssistProjectRequest,
        wizard_data: &serde_json::Value,
    ) -> String {
        // Extract key info from wizard data
        let industry = wizard_data["industry"]
            .as_str()
            .unwrap_or("general business");
        let target_audience = wizard_data["targetAudience"]
            .as_str()
            .unwrap_or("general audience");
        let requirements = wizard_data["requirements"]
            .as_str()
            .unwrap_or("See wizard completion for details");

        let base_context = format!(
            "**Project:** {}\n\n\
            **Company:** {}\n\
            **Industry:** {}\n\
            **Target Audience:** {}\n\
            **Rush Delivery:** {}\n\n\
            **Requirements:**\n{}\n\n",
            request.project_number,
            request.company_name,
            industry,
            target_audience,
            if request.is_rush_delivery { "Yes (24h)" } else { "No (48h)" },
            requirements
        );

        let stage_specific = match stage {
            WebAssistStage::InitialReview => {
                "# Initial Review & Research Setup\n\n\
                Your task is to review the project requirements and prepare the foundation.\n\n\
                ## Objectives:\n\
                - Analyze the requirements thoroughly\n\
                - Create a project strategy document\n\
                - Set up the development environment\n\
                - Prepare research questions for the next stage\n\n\
                ## Deliverables:\n\
                - `deliverables/01_initial_review/strategy.md` - Project strategy\n\
                - `deliverables/01_initial_review/research_plan.md` - Research plan for next stage\n"
            }
            WebAssistStage::AiResearch => {
                "# AI Research & Analysis (THOROUGH - 2 HOURS)\n\n\
                This is a CRITICAL stage. Take the FULL 2 hours to conduct comprehensive research.\n\n\
                ## Research Areas (ALL REQUIRED):\n\n\
                ### 1. Industry Analysis (60 minutes)\n\
                - Research current trends in the industry\n\
                - Identify top 10-15 competitor websites\n\
                - Analyze design patterns and UX conventions\n\
                - Document technology stacks used by industry leaders\n\
                - Screenshot and analyze competitor homepages\n\n\
                ### 2. Target Audience Research (30 minutes)\n\
                - Define detailed user personas\n\
                - Research user pain points and expectations\n\
                - Analyze user journey patterns\n\
                - Identify key conversion points\n\n\
                ### 3. Technical Requirements (30 minutes)\n\
                - Define performance targets (Core Web Vitals)\n\
                - Plan SEO strategy\n\
                - Identify required integrations\n\
                - Plan accessibility requirements (WCAG)\n\n\
                ## Deliverables (ALL REQUIRED):\n\
                - `deliverables/02_research/market_analysis.md` - Comprehensive findings\n\
                - `deliverables/02_research/competitor_analysis.md` - Detailed competitor breakdown\n\
                - `deliverables/02_research/technical_requirements.md` - Full tech spec\n\
                - `deliverables/02_research/recommendations.md` - Strategic recommendations\n\
                - `deliverables/02_research/screenshots/` - Competitor screenshots\n\n\
                **IMPORTANT:** Use all available time. Be thorough. This research guides ALL subsequent stages.\n"
            }
            WebAssistStage::DesignMockup => {
                "# Design Mockup Creation\n\n\
                Create professional, responsive design mockups based on research.\n\n\
                ## Objectives:\n\
                - Design homepage, about, services/products, contact pages\n\
                - Create responsive layouts (desktop, tablet, mobile)\n\
                - Define color scheme and typography\n\
                - Create design system/style guide\n\n\
                ## Deliverables:\n\
                - `deliverables/03_design/mockups/*.png` - Page mockups\n\
                - `deliverables/03_design/design_system.md` - Design system documentation\n\
                - `deliverables/03_design/figma_link.txt` - Figma/design tool link (if used)\n\n\
                **NOTE:** This stage requires CLIENT APPROVAL before proceeding.\n"
            }
            WebAssistStage::ContentCollection => {
                "# Content Collection & SEO\n\n\
                Create all website content optimized for SEO.\n\n\
                ## Objectives:\n\
                - Write homepage copy\n\
                - Create page content for all sections\n\
                - Optimize for SEO (meta titles, descriptions, keywords)\n\
                - Prepare/optimize images\n\n\
                ## Deliverables:\n\
                - `deliverables/04_content/*.md` - Page content\n\
                - `deliverables/04_content/seo_meta.json` - SEO metadata\n\
                - `deliverables/04_content/images/` - Optimized images\n\n\
                **NOTE:** This stage requires CLIENT APPROVAL before proceeding.\n"
            }
            WebAssistStage::Development => {
                "# Full-Stack Development\n\n\
                Build the complete Next.js application.\n\n\
                ## Objectives:\n\
                - Implement all pages with approved designs\n\
                - Add all features and functionality\n\
                - Integrate CMS (if required)\n\
                - Set up analytics\n\
                - Optimize performance\n\n\
                ## Technical Stack:\n\
                - Next.js 15+ with App Router\n\
                - TypeScript\n\
                - Tailwind CSS\n\
                - Responsive design (mobile-first)\n\n\
                The Next.js project is already initialized at `project/`.\n"
            }
            WebAssistStage::QualityAssurance => {
                "# Quality Assurance & Testing\n\n\
                Test thoroughly and optimize the website.\n\n\
                ## Objectives:\n\
                - Test all functionality\n\
                - Cross-browser testing (Chrome, Firefox, Safari, Edge)\n\
                - Cross-device testing (desktop, tablet, mobile)\n\
                - Performance optimization\n\
                - Accessibility testing\n\
                - Fix all bugs\n\n\
                ## Deliverables:\n\
                - `deliverables/06_qa/test_report.md` - Test results\n\
                - `deliverables/06_qa/performance_report.md` - Performance metrics\n"
            }
            WebAssistStage::ClientPreview => {
                "# Client Preview & Final Review\n\n\
                Deploy to staging and prepare for client review.\n\n\
                ## Objectives:\n\
                - Deploy to staging environment\n\
                - Create preview URL\n\
                - Prepare handoff documentation\n\
                - Final polish and adjustments\n\n\
                ## Deliverables:\n\
                - `deliverables/07_preview/staging_url.txt` - Staging URL\n\
                - `deliverables/07_preview/handoff_docs.md` - Handoff documentation\n\n\
                **NOTE:** This stage requires CLIENT APPROVAL before deployment.\n"
            }
            WebAssistStage::Deployment => {
                "# Production Deployment\n\n\
                Deploy the website to production.\n\n\
                ## Objectives:\n\
                - Deploy to production environment (Vercel recommended)\n\
                - Configure custom domain\n\
                - Set up SSL certificate\n\
                - Final production checks\n\
                - Go live!\n\n\
                ## Deliverables:\n\
                - `deliverables/08_deployment/production_url.txt` - Live URL\n\
                - `deliverables/08_deployment/dns_records.md` - DNS configuration\n\
                - `deliverables/08_deployment/deployment_docs.md` - Deployment documentation\n"
            }
            WebAssistStage::Delivered => {
                "# Project Delivered\n\n\
                Project is complete! The website is live and delivered to the client.\n\n\
                30-day support period begins now.\n"
            }
        };

        format!("{}\n\n{}", base_context, stage_specific)
    }

    /// Start the first task (Initial Review)
    async fn start_first_task(
        &self,
        wa_project_id: Uuid,
        stage_task_mapping: &HashMap<String, Uuid>,
    ) -> Result<()> {
        // Find Initial Review task
        let task_id = stage_task_mapping
            .get("initial_review")
            .context("Initial review task not found")?;

        tracing::info!("Starting first task (Initial Review): {}", task_id);

        // Update task status to in_progress
        db::models::task::Task::update_status(
            &self.pool,
            *task_id,
            db::models::task::TaskStatus::InProgress,
        )
        .await?;

        // Note: Actual task execution will be triggered by the API route
        // when creating a task attempt. This just marks it ready to start.

        Ok(())
    }

    /// Handle approval response from client (via webhook or API)
    pub async fn handle_approval_response(
        &self,
        webassist_project_id: Uuid,
        approval_id: Uuid,
        status: ApprovalStatus,
        feedback: Option<String>,
    ) -> Result<()> {
        tracing::info!(
            "Handling approval {} response: {:?}",
            approval_id,
            status
        );

        // Find WebAssist project
        let wa_project = WebAssistProject::find_by_webassist_id(&self.pool, webassist_project_id)
            .await?
            .context("WebAssist project not found")?;

        // Find the approval
        let approval = WebAssistApproval::find_by_id(&self.pool, approval_id)
            .await?
            .context("Approval not found")?;

        // Update approval status
        WebAssistApproval::update_status(&self.pool, approval_id, status.clone(), feedback.clone())
            .await?;

        // Handle based on status
        match status {
            ApprovalStatus::Approved => {
                // Resume workflow - start next stage
                if let Some(next_stage) = approval.stage_name.next_stage() {
                    self.start_next_stage(wa_project.id, next_stage).await?;
                }
            }
            ApprovalStatus::ChangesRequested | ApprovalStatus::Rejected => {
                // Pause workflow, notify team
                WebAssistProject::update_sync_status(&self.pool, wa_project.id, SyncStatus::Paused)
                    .await?;

                self.supabase_client
                    .create_project_update(
                        webassist_project_id,
                        "approval_rejected",
                        "Changes Requested",
                        &format!(
                            "Client requested changes: {}",
                            feedback.as_deref().unwrap_or("No feedback provided")
                        ),
                        None,
                    )
                    .await?;
            }
            ApprovalStatus::Pending => {
                // No action needed
            }
        }

        Ok(())
    }

    /// Start the next stage in the workflow
    async fn start_next_stage(&self, wa_project_id: Uuid, next_stage: WebAssistStage) -> Result<()> {
        tracing::info!("Starting next stage: {}", next_stage);

        // Update project stage
        WebAssistProject::update_stage(&self.pool, wa_project_id, next_stage).await?;

        // Find and start the task for this stage
        // (Task execution logic will be handled by StageExecutor)

        Ok(())
    }
}
