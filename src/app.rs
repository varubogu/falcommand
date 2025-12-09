use std::sync::Arc;
use tokio::sync::RwLock;
use log::{info, error};

use falcommand_config::{Config, ConfigError};
use falcommand_platform::PlatformProvider;
use falcommand_core::{SearchEngine, IndexManager, SyncManager, IndexError, SearchError, SyncError};
use falcommand_plugins::{PluginSystem, PluginError};
use falcommand_ui::MainWindow;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
    
    #[error("Index error: {0}")]
    Index(#[from] IndexError),
    
    #[error("Search engine error: {0}")]
    SearchEngine(#[from] SearchError),
    
    #[error("Sync error: {0}")]
    Sync(#[from] SyncError),
    
    #[error("Plugin system error: {0}")]
    Plugin(#[from] PluginError),
    
    #[error("UI error: {0}")]
    Ui(String),
    
    #[error("Platform error: {0}")]
    Platform(String),
    
    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

pub struct App {
    config: Arc<RwLock<Config>>,
    platform_provider: Arc<dyn PlatformProvider>,
    search_engine: Arc<SearchEngine>,
    plugin_system: Arc<PluginSystem>,
    index_manager: Arc<IndexManager>,
    sync_manager: Arc<SyncManager>,
    ui: Option<MainWindow>,
}

impl App {
    pub async fn new(
        config: Config,
        platform_provider: Arc<dyn PlatformProvider>
    ) -> Result<Self> {
        info!("Initializing application...");
        
        let config = Arc::new(RwLock::new(config));
        
        // Initialize core components
        let index_manager = Arc::new(IndexManager::new(config.clone()).await?);
        let plugin_system = Arc::new(PluginSystem::new(config.clone()).await?);
        let sync_manager = Arc::new(SyncManager::new(config.clone()).await?);
        
        let search_engine = Arc::new(
            SearchEngine::new(
                config.clone(),
                index_manager.clone(),
            ).await?
        );
        
        Ok(Self {
            config,
            platform_provider,
            search_engine,
            plugin_system,
            index_manager,
            sync_manager,
            ui: None,
        })
    }
    
    pub async fn run(&mut self) -> Result<()> {
        info!("Starting application...");
        
        // Initialize index in background
        let index_manager = self.index_manager.clone();
        let platform_provider = self.platform_provider.clone();
        tokio::spawn(async move {
            if let Err(e) = index_manager.rebuild_index(platform_provider).await {
                error!("Failed to build index: {}", e);
            }
        });
        
        // Initialize plugins
        self.plugin_system.load_plugins().await?;
        
        // Initialize UI
        let ui = MainWindow::new(
            self.search_engine.clone(),
            self.config.clone(),
        ).await.map_err(|e| AppError::Ui(e.to_string()))?;
        
        self.ui = Some(ui);
        
        // Initialize system tray if enabled (after UI is created)
        let config = self.config.read().await;
        if config.appearance.enable_system_tray {
            // Try to initialize system tray, but don't fail if it's not available
            if let Err(e) = self.initialize_system_tray().await {
                error!("Failed to initialize system tray: {}. Continuing without system tray.", e);
            }
        }
        drop(config);
        
        // Register global hotkey
        if let Err(e) = self.register_global_hotkey().await {
            error!("Failed to register global hotkey: {}. Continuing without global hotkey.", e);
        }
        
        // Show window on startup unless configured to start in tray
        let config = self.config.read().await;
        if !config.appearance.start_in_tray {
            if let Some(ref ui) = self.ui {
                ui.show().map_err(|e| AppError::Ui(e.to_string()))?;
            }
        }
        drop(config);
        
        // Start UI event loop
        if let Some(ref ui) = self.ui {
            ui.run().await.map_err(|e| AppError::Ui(e.to_string()))?;
        }
        
        Ok(())
    }
    
    async fn initialize_system_tray(&self) -> Result<()> {
        info!("Initializing system tray...");
        
        // Create system tray icon (simple 32x32 RGBA icon)
        let icon_data = Self::create_default_icon();
        
        self.platform_provider
            .create_system_tray("FalCommand", "FalCommand - Fast Application Launcher", Some(&icon_data))
            .map_err(|e| AppError::Platform(e.to_string()))?;
        
        // Show the system tray
        self.platform_provider
            .show_system_tray()
            .map_err(|e| AppError::Platform(e.to_string()))?;
        
        // Setup system tray menu callbacks (no direct UI handle capture to keep things simple)
        let show_callback = Box::new(move || {
            info!("Show requested from system tray (UI handle not captured in this build)");
        });
        
        let quit_callback = Box::new(|| {
            info!("Quit requested from system tray");
            std::process::exit(0);
        });
        
        self.platform_provider
            .update_system_tray_menu(show_callback, quit_callback)
            .map_err(|e| AppError::Platform(e.to_string()))?;
        
        info!("System tray initialized successfully");
        Ok(())
    }
    
    fn create_default_icon() -> Vec<u8> {
        // Create a simple 32x32 RGBA icon (blue square)
        let mut icon_data = Vec::with_capacity(32 * 32 * 4);
        for _y in 0..32 {
            for _x in 0..32 {
                icon_data.extend_from_slice(&[0, 100, 200, 255]); // RGBA: Blue with full opacity
            }
        }
        icon_data
    }

    async fn register_global_hotkey(&self) -> Result<()> {
        let config = self.config.read().await;
        let hotkey = &config.behavior.hotkey;

        self.platform_provider
            .register_global_hotkey(hotkey, Box::new(move || {
                info!("Global hotkey triggered (toggle visibility not wired in this build)");
            }))
            .map_err(|e| AppError::Platform(e.to_string()))?;
        
        info!("Registered global hotkey: {}", hotkey);
        Ok(())
    }
}

pub type Result<T> = std::result::Result<T, AppError>;