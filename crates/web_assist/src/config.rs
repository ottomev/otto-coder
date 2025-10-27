use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// WebAssist integration configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WebAssistConfig {
    /// Enable or disable WebAssist integration
    #[serde(default)]
    pub enabled: bool,

    /// Webhook secret for verifying Supabase webhooks (HMAC-SHA256)
    pub webhook_secret: Option<String>,

    /// Directory where WebAssist projects will be stored
    pub projects_directory: Option<PathBuf>,

    /// Supabase configuration
    #[serde(default)]
    pub supabase: SupabaseConfigSection,

    /// Executor configuration
    #[serde(default)]
    pub executor: ExecutorConfig,

    /// Approval workflow configuration
    #[serde(default)]
    pub approvals: ApprovalsConfig,

    /// Next.js project configuration
    #[serde(default)]
    pub nextjs: NextJsConfig,

    /// Monitoring and logging configuration
    #[serde(default)]
    pub monitoring: MonitoringConfig,

    /// Performance and optimization configuration
    #[serde(default)]
    pub performance: PerformanceConfig,

    /// File management configuration
    #[serde(default)]
    pub files: FilesConfig,

    /// Advanced settings
    #[serde(default)]
    pub advanced: AdvancedConfig,
}

impl Default for WebAssistConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            webhook_secret: None,
            projects_directory: None,
            supabase: SupabaseConfigSection::default(),
            executor: ExecutorConfig::default(),
            approvals: ApprovalsConfig::default(),
            nextjs: NextJsConfig::default(),
            monitoring: MonitoringConfig::default(),
            performance: PerformanceConfig::default(),
            files: FilesConfig::default(),
            advanced: AdvancedConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SupabaseConfigSection {
    /// Supabase project URL
    pub url: Option<String>,

    /// Supabase anonymous key
    pub anon_key: Option<String>,

    /// Supabase service role key (for admin operations)
    pub service_role_key: Option<String>,
}

impl Default for SupabaseConfigSection {
    fn default() -> Self {
        Self {
            url: None,
            anon_key: None,
            service_role_key: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExecutorConfig {
    /// Default executor profile for AI-powered stages
    #[serde(default = "default_executor_profile")]
    pub default_profile: String,

    /// Timeout for research stage (in minutes)
    #[serde(default = "default_research_timeout")]
    pub research_timeout_minutes: u32,

    /// Timeout for design stage (in minutes)
    #[serde(default = "default_design_timeout")]
    pub design_timeout_minutes: u32,

    /// Timeout for development stage (in minutes)
    #[serde(default = "default_development_timeout")]
    pub development_timeout_minutes: u32,

    /// Timeout for QA stage (in minutes)
    #[serde(default = "default_qa_timeout")]
    pub qa_timeout_minutes: u32,

    /// Timeout for deployment stage (in minutes)
    #[serde(default = "default_deployment_timeout")]
    pub deployment_timeout_minutes: u32,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            default_profile: default_executor_profile(),
            research_timeout_minutes: default_research_timeout(),
            design_timeout_minutes: default_design_timeout(),
            development_timeout_minutes: default_development_timeout(),
            qa_timeout_minutes: default_qa_timeout(),
            deployment_timeout_minutes: default_deployment_timeout(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApprovalsConfig {
    /// Automatically create approval requests in WebAssist Supabase
    #[serde(default = "default_true")]
    pub auto_create_in_webassist: bool,

    /// How often to sync approval status between systems (in seconds)
    #[serde(default = "default_sync_interval")]
    pub sync_interval_seconds: u64,

    /// Allow approvals from both Otto Coder and WebAssist UIs
    #[serde(default = "default_true")]
    pub bidirectional_approvals: bool,
}

impl Default for ApprovalsConfig {
    fn default() -> Self {
        Self {
            auto_create_in_webassist: default_true(),
            sync_interval_seconds: default_sync_interval(),
            bidirectional_approvals: default_true(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NextJsConfig {
    /// Next.js version to use for new projects
    #[serde(default = "default_nextjs_version")]
    pub version: String,

    /// Use TypeScript
    #[serde(default = "default_true")]
    pub typescript: bool,

    /// Use Tailwind CSS
    #[serde(default = "default_true")]
    pub tailwind: bool,

    /// Use App Router (vs Pages Router)
    #[serde(default = "default_true")]
    pub app_router: bool,

    /// Package manager: npm, pnpm, yarn, or bun
    #[serde(default = "default_package_manager")]
    pub package_manager: String,
}

impl Default for NextJsConfig {
    fn default() -> Self {
        Self {
            version: default_nextjs_version(),
            typescript: default_true(),
            tailwind: default_true(),
            app_router: default_true(),
            package_manager: default_package_manager(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MonitoringConfig {
    /// Log level for WebAssist operations
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// Enable detailed webhook logging
    #[serde(default = "default_true")]
    pub log_webhooks: bool,

    /// Enable detailed API call logging
    #[serde(default = "default_true")]
    pub log_api_calls: bool,

    /// Enable task execution logging
    #[serde(default = "default_true")]
    pub log_task_execution: bool,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            log_level: default_log_level(),
            log_webhooks: default_true(),
            log_api_calls: default_true(),
            log_task_execution: default_true(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PerformanceConfig {
    /// Maximum concurrent projects Otto Coder can handle
    #[serde(default = "default_max_concurrent_projects")]
    pub max_concurrent_projects: u32,

    /// Maximum concurrent tasks per project
    #[serde(default = "default_one")]
    pub max_concurrent_tasks_per_project: u32,

    /// Retry failed API calls (to Supabase)
    #[serde(default = "default_true")]
    pub retry_failed_api_calls: bool,

    /// Number of retries before giving up
    #[serde(default = "default_max_retries")]
    pub max_api_retries: u32,

    /// Delay between retries (in seconds)
    #[serde(default = "default_retry_delay")]
    pub retry_delay_seconds: u64,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_concurrent_projects: default_max_concurrent_projects(),
            max_concurrent_tasks_per_project: default_one(),
            retry_failed_api_calls: default_true(),
            max_api_retries: default_max_retries(),
            retry_delay_seconds: default_retry_delay(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FilesConfig {
    /// Maximum size for deliverable files (in MB)
    #[serde(default = "default_max_file_size")]
    pub max_deliverable_size_mb: u32,

    /// Allowed file types for deliverables
    #[serde(default = "default_allowed_file_types")]
    pub allowed_file_types: Vec<String>,

    /// Clean up project files after delivery (in days)
    #[serde(default = "default_cleanup_days")]
    pub cleanup_after_days: u32,

    /// Compress deliverables for storage
    #[serde(default = "default_true")]
    pub compress_deliverables: bool,
}

impl Default for FilesConfig {
    fn default() -> Self {
        Self {
            max_deliverable_size_mb: default_max_file_size(),
            allowed_file_types: default_allowed_file_types(),
            cleanup_after_days: default_cleanup_days(),
            compress_deliverables: default_true(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AdvancedConfig {
    /// Enable experimental features
    #[serde(default)]
    pub experimental_features: bool,

    /// Use git worktrees for isolation
    #[serde(default = "default_true")]
    pub use_git_worktrees: bool,

    /// Automatically clean up orphaned worktrees
    #[serde(default = "default_true")]
    pub auto_cleanup_worktrees: bool,

    /// Maximum age of orphaned worktrees before cleanup (in hours)
    #[serde(default = "default_worktree_max_age")]
    pub worktree_max_age_hours: u32,

    /// Enable parallel task execution within a project
    #[serde(default)]
    pub enable_parallel_tasks: bool,
}

impl Default for AdvancedConfig {
    fn default() -> Self {
        Self {
            experimental_features: false,
            use_git_worktrees: default_true(),
            auto_cleanup_worktrees: default_true(),
            worktree_max_age_hours: default_worktree_max_age(),
            enable_parallel_tasks: false,
        }
    }
}

// Default value functions
fn default_executor_profile() -> String {
    "claude/claude-code".to_string()
}

fn default_research_timeout() -> u32 {
    120 // 2 hours
}

fn default_design_timeout() -> u32 {
    480 // 8 hours
}

fn default_development_timeout() -> u32 {
    960 // 16 hours
}

fn default_qa_timeout() -> u32 {
    240 // 4 hours
}

fn default_deployment_timeout() -> u32 {
    240 // 4 hours
}

fn default_true() -> bool {
    true
}

fn default_sync_interval() -> u64 {
    30
}

fn default_nextjs_version() -> String {
    "latest".to_string()
}

fn default_package_manager() -> String {
    "npm".to_string()
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_max_concurrent_projects() -> u32 {
    10
}

fn default_one() -> u32 {
    1
}

fn default_max_retries() -> u32 {
    3
}

fn default_retry_delay() -> u64 {
    5
}

fn default_max_file_size() -> u32 {
    50
}

fn default_allowed_file_types() -> Vec<String> {
    vec![
        "pdf".to_string(),
        "png".to_string(),
        "jpg".to_string(),
        "jpeg".to_string(),
        "gif".to_string(),
        "svg".to_string(),
        "md".to_string(),
        "txt".to_string(),
        "json".to_string(),
    ]
}

fn default_cleanup_days() -> u32 {
    90
}

fn default_worktree_max_age() -> u32 {
    48
}

/// Load WebAssist configuration from TOML file
pub async fn load_web_assist_config(
    config_path: &std::path::Path,
) -> Result<WebAssistConfig, String> {
    if !config_path.exists() {
        tracing::debug!(
            "WebAssist config file not found at {:?}, using defaults (disabled)",
            config_path
        );
        return Ok(WebAssistConfig::default());
    }

    let contents = tokio::fs::read_to_string(config_path)
        .await
        .map_err(|e| format!("Failed to read WebAssist config: {}", e))?;

    let config: toml::Table = toml::from_str(&contents)
        .map_err(|e| format!("Failed to parse WebAssist config: {}", e))?;

    // Extract the web_assist section
    let web_assist_config = config
        .get("web_assist")
        .ok_or_else(|| "No [web_assist] section found in config".to_string())?;

    let parsed_config: WebAssistConfig = web_assist_config
        .clone()
        .try_into()
        .map_err(|e| format!("Failed to deserialize WebAssist config: {}", e))?;

    Ok(parsed_config)
}

impl WebAssistConfig {
    /// Check if configuration is valid and complete
    pub fn is_valid(&self) -> bool {
        if !self.enabled {
            return false;
        }

        // Check required fields
        self.webhook_secret.is_some()
            && self.projects_directory.is_some()
            && self.supabase.url.is_some()
            && self.supabase.service_role_key.is_some()
    }

    /// Get the webhook secret or panic
    pub fn webhook_secret(&self) -> &str {
        self.webhook_secret
            .as_ref()
            .expect("Webhook secret not configured")
    }

    /// Get the projects directory or panic
    pub fn projects_directory(&self) -> &PathBuf {
        self.projects_directory
            .as_ref()
            .expect("Projects directory not configured")
    }

    /// Get the Supabase URL or panic
    pub fn supabase_url(&self) -> &str {
        self.supabase
            .url
            .as_ref()
            .expect("Supabase URL not configured")
    }

    /// Get the Supabase service role key or panic
    pub fn supabase_service_role_key(&self) -> &str {
        self.supabase
            .service_role_key
            .as_ref()
            .expect("Supabase service role key not configured")
    }
}
