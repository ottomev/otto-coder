use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use db::DBService;
use deployment::{Deployment, DeploymentError};
use executors::profile::ExecutorConfigs;
use services::services::{
    analytics::{AnalyticsConfig, AnalyticsContext, AnalyticsService, generate_user_id},
    approvals::Approvals,
    auth::AuthService,
    config::{Config, load_config_from_file, save_config_to_file},
    container::ContainerService,
    events::EventService,
    file_search_cache::FileSearchCache,
    filesystem::FilesystemService,
    git::GitService,
    image::ImageService,
    sentry::SentryService,
};
use tokio::sync::RwLock;
use utils::{assets::config_path, msg_store::MsgStore};
use uuid::Uuid;

use crate::container::LocalContainerService;

mod command;
pub mod container;

#[derive(Clone)]
pub struct LocalDeployment {
    config: Arc<RwLock<Config>>,
    sentry: SentryService,
    user_id: String,
    db: DBService,
    analytics: Option<AnalyticsService>,
    msg_stores: Arc<RwLock<HashMap<Uuid, Arc<MsgStore>>>>,
    container: LocalContainerService,
    git: GitService,
    auth: AuthService,
    image: ImageService,
    filesystem: FilesystemService,
    events: EventService,
    file_search_cache: Arc<FileSearchCache>,
    approvals: Approvals,
    // WebAssist integration (optional)
    web_assist_webhook_handler: Option<Arc<web_assist::WebhookHandler>>,
    web_assist_project_manager: Option<Arc<web_assist::ProjectManager>>,
    web_assist_approval_sync: Option<Arc<web_assist::ApprovalSync>>,
}

#[async_trait]
impl Deployment for LocalDeployment {
    async fn new() -> Result<Self, DeploymentError> {
        let mut raw_config = load_config_from_file(&config_path()).await;

        let profiles = ExecutorConfigs::get_cached();
        if !raw_config.onboarding_acknowledged
            && let Ok(recommended_executor) = profiles.get_recommended_executor_profile().await
        {
            raw_config.executor_profile = recommended_executor;
        }

        // Check if app version has changed and set release notes flag
        {
            let current_version = utils::version::APP_VERSION;
            let stored_version = raw_config.last_app_version.as_deref();

            if stored_version != Some(current_version) {
                // Show release notes only if this is an upgrade (not first install)
                raw_config.show_release_notes = stored_version.is_some();
                raw_config.last_app_version = Some(current_version.to_string());
            }
        }

        // Always save config (may have been migrated or version updated)
        save_config_to_file(&raw_config, &config_path()).await?;

        let config = Arc::new(RwLock::new(raw_config));
        let sentry = SentryService::new();
        let user_id = generate_user_id();
        let analytics = AnalyticsConfig::new().map(AnalyticsService::new);
        let git = GitService::new();
        let msg_stores = Arc::new(RwLock::new(HashMap::new()));
        let auth = AuthService::new();
        let filesystem = FilesystemService::new();

        // Create shared components for EventService
        let events_msg_store = Arc::new(MsgStore::new());
        let events_entry_count = Arc::new(RwLock::new(0));

        // Create DB with event hooks
        let db = {
            let hook = EventService::create_hook(
                events_msg_store.clone(),
                events_entry_count.clone(),
                DBService::new().await?, // Temporary DB service for the hook
            );
            DBService::new_with_after_connect(hook).await?
        };

        let image = ImageService::new(db.clone().pool)?;
        {
            let image_service = image.clone();
            tokio::spawn(async move {
                tracing::info!("Starting orphaned image cleanup...");
                if let Err(e) = image_service.delete_orphaned_images().await {
                    tracing::error!("Failed to clean up orphaned images: {}", e);
                }
            });
        }

        let approvals = Approvals::new(db.pool.clone(), msg_stores.clone());

        // We need to make analytics accessible to the ContainerService
        // TODO: Handle this more gracefully
        let analytics_ctx = analytics.as_ref().map(|s| AnalyticsContext {
            user_id: user_id.clone(),
            analytics_service: s.clone(),
        });
        let container = LocalContainerService::new(
            db.clone(),
            msg_stores.clone(),
            config.clone(),
            git.clone(),
            image.clone(),
            analytics_ctx,
        );
        container.spawn_worktree_cleanup().await;

        let events = EventService::new(db.clone(), events_msg_store, events_entry_count);
        let file_search_cache = Arc::new(FileSearchCache::new());

        // Initialize WebAssist integration (if enabled)
        let (web_assist_webhook_handler, web_assist_project_manager, web_assist_approval_sync) = {
            match Self::initialize_web_assist(&db).await {
                Ok(components) => components,
                Err(e) => {
                    tracing::warn!("WebAssist integration disabled: {}", e);
                    (None, None, None)
                }
            }
        };

        Ok(Self {
            config,
            sentry,
            user_id,
            db,
            analytics,
            msg_stores,
            container,
            git,
            auth,
            image,
            filesystem,
            events,
            file_search_cache,
            approvals,
            web_assist_webhook_handler,
            web_assist_project_manager,
            web_assist_approval_sync,
        })
    }

    fn user_id(&self) -> &str {
        &self.user_id
    }

    fn shared_types() -> Vec<String> {
        vec![]
    }

    fn config(&self) -> &Arc<RwLock<Config>> {
        &self.config
    }

    fn sentry(&self) -> &SentryService {
        &self.sentry
    }

    fn db(&self) -> &DBService {
        &self.db
    }

    fn analytics(&self) -> &Option<AnalyticsService> {
        &self.analytics
    }

    fn container(&self) -> &impl ContainerService {
        &self.container
    }
    fn auth(&self) -> &AuthService {
        &self.auth
    }

    fn git(&self) -> &GitService {
        &self.git
    }

    fn image(&self) -> &ImageService {
        &self.image
    }

    fn filesystem(&self) -> &FilesystemService {
        &self.filesystem
    }

    fn msg_stores(&self) -> &Arc<RwLock<HashMap<Uuid, Arc<MsgStore>>>> {
        &self.msg_stores
    }

    fn events(&self) -> &EventService {
        &self.events
    }

    fn file_search_cache(&self) -> &Arc<FileSearchCache> {
        &self.file_search_cache
    }

    fn approvals(&self) -> &Approvals {
        &self.approvals
    }
}

impl LocalDeployment {
    /// Initialize WebAssist integration components
    /// Returns (webhook_handler, project_manager, approval_sync) wrapped in Options
    async fn initialize_web_assist(
        db: &DBService,
    ) -> Result<(
        Option<Arc<web_assist::WebhookHandler>>,
        Option<Arc<web_assist::ProjectManager>>,
        Option<Arc<web_assist::ApprovalSync>>,
    ), String> {
        // Load WebAssist configuration
        let config_path = utils::assets::config_dir().join("web-assist.toml");
        let wa_config = web_assist::load_web_assist_config(&config_path).await?;

        // Check if WebAssist is enabled
        if !wa_config.enabled {
            return Ok((None, None, None));
        }

        // Validate configuration
        if !wa_config.is_valid() {
            return Err("WebAssist configuration is incomplete. Check webhook_secret, projects_directory, and supabase settings.".to_string());
        }

        tracing::info!("Initializing WebAssist integration...");

        // Create Supabase client
        let supabase_config = web_assist::SupabaseConfig {
            url: wa_config.supabase_url().to_string(),
            anon_key: wa_config.supabase.anon_key.clone().unwrap_or_default(),
            service_role_key: Some(wa_config.supabase_service_role_key().to_string()),
        };
        let supabase_client = Arc::new(web_assist::SupabaseClient::new(supabase_config));

        // Create ProjectManager
        let project_manager = Arc::new(
            web_assist::ProjectManager::new(
                db.pool.clone(),
                supabase_client.clone(),
                wa_config.projects_directory().clone(),
                wa_config.executor.default_profile.clone(),
            )
        );

        // Create ApprovalSync
        let approval_sync = Arc::new(
            web_assist::ApprovalSync::new(
                db.pool.clone(),
                supabase_client.clone(),
            )
        );

        // Create WebhookHandler
        let webhook_handler = Arc::new(
            web_assist::WebhookHandler::new(
                wa_config.webhook_secret().to_string(),
                project_manager.clone(),
                approval_sync.clone(),
            )
        );

        tracing::info!("WebAssist integration initialized successfully");

        Ok((
            Some(webhook_handler),
            Some(project_manager),
            Some(approval_sync),
        ))
    }

    /// Get the WebAssist webhook handler (if enabled)
    pub fn web_assist_webhook_handler(&self) -> Option<Arc<web_assist::WebhookHandler>> {
        self.web_assist_webhook_handler.clone()
    }

    /// Get the WebAssist project manager (if enabled)
    pub fn web_assist_project_manager(&self) -> Option<Arc<web_assist::ProjectManager>> {
        self.web_assist_project_manager.clone()
    }

    /// Get the WebAssist approval sync (if enabled)
    pub fn web_assist_approval_sync(&self) -> Option<Arc<web_assist::ApprovalSync>> {
        self.web_assist_approval_sync.clone()
    }
}
