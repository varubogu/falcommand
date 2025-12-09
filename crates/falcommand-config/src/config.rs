use std::path::{Path, PathBuf};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use log::info;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Failed to load config file: {0}")]
    LoadError(#[from] std::io::Error),
    
    #[error("Failed to parse config: {0}")]
    ParseError(#[from] serde_json::Error),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("File system error: {0}")]
    FileSystemError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub appearance: AppearanceConfig,
    pub behavior: BehaviorConfig,
    pub search: SearchConfig,
    pub plugins: PluginConfig,
    pub sync: SyncConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceConfig {
    pub theme: Theme,
    pub transparency: f32,
    pub position: WindowPosition,
    pub show_window: ShowWindow,
    pub show_window_display_number: u32,
    pub font_size: u32,
    pub max_results: usize,
    pub enable_system_tray: bool,
    pub start_in_tray: bool,
    pub minimize_to_tray: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WindowPosition {
    Center,
    Cursor,
    Custom { x: i32, y: i32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShowWindow {
    Mouse,
    Display,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorConfig {
    pub hotkey: String,
    pub auto_hide: bool,
    pub max_results: usize,
    pub rebuild_index_on_startup: bool,
    pub save_search_history: bool,
    pub record_usage_stats: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    pub include_paths: HashMap<String, Vec<String>>,
    pub exclude_patterns: Vec<String>,
    pub fuzzy_threshold: f64,
    pub enable_file_search: bool,
    pub enable_app_search: bool,
    pub enable_web_search: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub enabled: Vec<String>,
    pub disabled: Vec<String>,
    pub plugin_settings: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub enabled: bool,
    pub provider: Option<String>,
    pub auto_sync_interval: u32,
    pub encrypt_data: bool,
}

impl Config {
    pub fn default() -> Self {
        Self {
            appearance: AppearanceConfig {
                theme: Theme::System,
                transparency: 0.95,
                position: WindowPosition::Center,
                show_window: ShowWindow::Mouse,
                show_window_display_number: 0,
                font_size: 14,
                max_results: 10,
                enable_system_tray: true,
                start_in_tray: false,
                minimize_to_tray: true,
            },
            behavior: BehaviorConfig {
                hotkey: "Ctrl+Space".to_string(),
                auto_hide: true,
                max_results: 10,
                rebuild_index_on_startup: true,
                save_search_history: true,
                record_usage_stats: true,
            },
            search: SearchConfig {
                include_paths: {
                    let mut paths = HashMap::new();
                    #[cfg(target_os = "windows")]
                    paths.insert("windows".to_string(), vec![
                        "C:\\Program Files".to_string(),
                        "C:\\Program Files (x86)".to_string(),
                    ]);
                    #[cfg(target_os = "macos")]
                    paths.insert("macos".to_string(), vec![
                        "~/Applications".to_string(),
                        "/Applications".to_string(),
                    ]);
                    #[cfg(target_os = "linux")]
                    paths.insert("linux".to_string(), vec![
                        "/usr/bin".to_string(),
                        "/usr/local/bin".to_string(),
                        "~/.local/share/applications".to_string(),
                    ]);
                    paths
                },
                exclude_patterns: vec!["*.tmp".to_string(), "*.log".to_string()],
                fuzzy_threshold: 0.6,
                enable_file_search: true,
                enable_app_search: true,
                enable_web_search: false,
            },
            plugins: PluginConfig {
                enabled: vec!["calculator".to_string(), "translator".to_string()],
                disabled: vec!["weather".to_string()],
                plugin_settings: HashMap::new(),
            },
            sync: SyncConfig {
                enabled: false,
                provider: None,
                auto_sync_interval: 3600, // 1 hour
                encrypt_data: true,
            },
        }
    }
    
    pub async fn load_default() -> Result<Self, ConfigError> {
        let config_path = Self::get_default_config_path()?;
        
        // ベース設定の読み込み
        let mut base_config = if config_path.exists() {
            Self::load_from_file(&config_path).await?
        } else {
            info!("Config file not found, creating default configuration");
            let config = Self::default();
            config.save_to_file(&config_path).await?;
            config
        };

        // プラットフォーム固有設定をマージ
        base_config = base_config.get_platform_specific_config().await;

        // デバッグビルドの場合、デバッグ設定を最優先でマージ
        if cfg!(debug_assertions) {
            if let Ok(Some(debug_config)) = Self::load_debug_config().await {
                info!("Applying debug configuration with highest priority");
                base_config = base_config.merge_with(debug_config);
            }
        }

        Ok(base_config)
    }
    
    pub async fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        info!("Loading config from: {:?}", path.as_ref());
        let content = tokio::fs::read_to_string(path).await?;
        let config: Config = serde_json::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }
    
    pub async fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), ConfigError> {
        info!("Saving config to: {:?}", path.as_ref());
        
        // Ensure directory exists
        if let Some(parent) = path.as_ref().parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        
        let content = serde_json::to_string_pretty(self)?;
        tokio::fs::write(path, content).await?;
        Ok(())
    }
    
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate transparency
        if self.appearance.transparency < 0.0 || self.appearance.transparency > 1.0 {
            return Err(ConfigError::ValidationError(
                "Transparency must be between 0.0 and 1.0".to_string()
            ));
        }
        
        // Validate fuzzy threshold
        if self.search.fuzzy_threshold < 0.0 || self.search.fuzzy_threshold > 1.0 {
            return Err(ConfigError::ValidationError(
                "Fuzzy threshold must be between 0.0 and 1.0".to_string()
            ));
        }
        
        // Validate max results
        if self.behavior.max_results == 0 || self.behavior.max_results > 100 {
            return Err(ConfigError::ValidationError(
                "Max results must be between 1 and 100".to_string()
            ));
        }
        
        Ok(())
    }
    
    fn get_default_config_path() -> Result<PathBuf, ConfigError> {
        if cfg!(debug_assertions) {
            // デバッグビルド（開発中）の場合はプロジェクトルートの.falcommandフォルダを使用
            let current_dir = std::env::current_dir()
                .map_err(|e| ConfigError::FileSystemError(format!("Cannot determine current directory: {}", e)))?;
            Ok(current_dir.join(".falcommand").join("config.json"))
        } else {
            // リリースビルドの場合は従来通りシステム設定ディレクトリを使用
            let config_dir = dirs::config_dir()
                .ok_or_else(|| ConfigError::FileSystemError("Cannot determine config directory".to_string()))?;
            
            Ok(config_dir.join("falcommand").join("config.json"))
        }
    }
    
    pub async fn get_platform_specific_config(&self) -> Config {
        let mut config = self.clone();
        
        // Load platform-specific overrides if they exist
        let platform_config_path = Self::get_platform_config_path();
        if let Ok(path) = platform_config_path {
            if path.exists() {
                if let Ok(platform_config) = Self::load_from_file(&path).await {
                    // Merge platform-specific settings
                    config = config.merge_with(platform_config);
                    info!("Applied platform-specific configuration from: {:?}", path);
                }
            }
        }
        
        config
    }
    
    fn get_platform_config_path() -> Result<PathBuf, ConfigError> {
        let config_dir = if cfg!(debug_assertions) {
            // デバッグビルドの場合はプロジェクトルートの.falcommandフォルダを使用
            let current_dir = std::env::current_dir()
                .map_err(|e| ConfigError::FileSystemError(format!("Cannot determine current directory: {}", e)))?;
            current_dir.join(".falcommand")
        } else {
            // リリースビルドの場合は従来通りシステム設定ディレクトリを使用
            dirs::config_dir()
                .ok_or_else(|| ConfigError::FileSystemError("Cannot determine config directory".to_string()))?
                .join("falcommand")
        };
        
        let filename = if cfg!(target_os = "windows") {
            "config.windows.json"
        } else if cfg!(target_os = "macos") {
            "config.macos.json"
        } else if cfg!(target_os = "linux") {
            "config.linux.json"
        } else {
            "config.json"
        };
        
        Ok(config_dir.join(filename))
    }
    
    fn merge_with(&self, other: Config) -> Config {
        // Simple merge - in real implementation, this would be more sophisticated
        other
    }

    /// デバッグ設定ファイルのパスを取得
    fn get_debug_config_path() -> Result<PathBuf, ConfigError> {
        if cfg!(debug_assertions) {
            let current_dir = std::env::current_dir()
                .map_err(|e| ConfigError::FileSystemError(format!("Cannot determine current directory: {}", e)))?;
            Ok(current_dir.join(".falcommand").join("config.debug.json"))
        } else {
            // リリースビルドの場合はデバッグ設定は使用しない
            Err(ConfigError::FileSystemError("Debug config not available in release builds".to_string()))
        }
    }

    /// プラットフォーム固有デバッグ設定ファイルのパスを取得
    fn get_platform_debug_config_path() -> Result<PathBuf, ConfigError> {
        if cfg!(debug_assertions) {
            let current_dir = std::env::current_dir()
                .map_err(|e| ConfigError::FileSystemError(format!("Cannot determine current directory: {}", e)))?;
            
            let filename = if cfg!(target_os = "windows") {
                "config.debug.windows.json"
            } else if cfg!(target_os = "macos") {
                "config.debug.macos.json"
            } else if cfg!(target_os = "linux") {
                "config.debug.linux.json"
            } else {
                "config.debug.json"
            };
            
            Ok(current_dir.join(".falcommand").join(filename))
        } else {
            // リリースビルドの場合はデバッグ設定は使用しない
            Err(ConfigError::FileSystemError("Platform debug config not available in release builds".to_string()))
        }
    }

    /// デバッグ設定を読み込み（存在する場合のみ）
    async fn load_debug_config() -> Result<Option<Config>, ConfigError> {
        if !cfg!(debug_assertions) {
            return Ok(None);
        }

        // プラットフォーム固有デバッグ設定を最優先で試行
        if let Ok(platform_debug_path) = Self::get_platform_debug_config_path() {
            if platform_debug_path.exists() {
                info!("Loading platform-specific debug config from: {:?}", platform_debug_path);
                return Self::load_from_file(&platform_debug_path).await.map(Some);
            }
        }

        // 一般デバッグ設定を次に試行
        if let Ok(debug_path) = Self::get_debug_config_path() {
            if debug_path.exists() {
                info!("Loading debug config from: {:?}", debug_path);
                return Self::load_from_file(&debug_path).await.map(Some);
            }
        }

        Ok(None)
    }
}