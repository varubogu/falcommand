use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use log::{info, warn, error};

use crate::config::Theme;

#[derive(Debug, thiserror::Error)]
pub enum PlatformError {
    #[error("Failed to get installed applications: {0}")]
    ApplicationScanError(String),
    
    #[error("Failed to register hotkey: {0}")]
    HotkeyError(String),
    
    #[error("Failed to show notification: {0}")]
    NotificationError(String),
    
    #[error("File system error: {0}")]
    FileSystemError(String),
    
    #[error("Clipboard error: {0}")]
    ClipboardError(String),
    
    #[error("Other platform error: {0}")]
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppInfo {
    pub name: String,
    pub executable_path: PathBuf,
    pub icon_path: Option<PathBuf>,
    pub description: Option<String>,
    pub keywords: Vec<String>,
    pub usage_count: u32,
    pub last_used: Option<SystemTime>,
}

impl AppInfo {
    pub fn new(name: impl Into<String>, executable_path: PathBuf) -> Self {
        Self {
            name: name.into(),
            executable_path,
            icon_path: None,
            description: None,
            keywords: Vec::new(),
            usage_count: 0,
            last_used: None,
        }
    }
    
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
    
    pub fn with_icon(mut self, icon_path: PathBuf) -> Self {
        self.icon_path = Some(icon_path);
        self
    }
    
    pub fn with_keywords(mut self, keywords: Vec<String>) -> Self {
        self.keywords = keywords;
        self
    }
    
    pub fn increment_usage(&mut self) {
        self.usage_count += 1;
        self.last_used = Some(SystemTime::now());
    }
    
    pub fn to_search_result(&self) -> crate::search::SearchResult {
        use crate::search::{SearchResult, Action, Category};
        
        SearchResult::new(&self.name, self.description.as_deref().unwrap_or(""))
            .with_action(Action::ExecuteApplication {
                path: self.executable_path.clone(),
                args: Vec::new(),
            })
            .with_category(Category::Application)
            .with_path(self.executable_path.clone())
            .with_score(self.calculate_score())
    }
    
    fn calculate_score(&self) -> f64 {
        // Higher score for frequently used applications
        let usage_score = (self.usage_count as f64 * 0.1).min(0.5);
        
        // Recent usage bonus
        let recency_score = if let Some(last_used) = self.last_used {
            let elapsed = SystemTime::now().duration_since(last_used).unwrap_or_default();
            let days = elapsed.as_secs() / (24 * 3600);
            if days == 0 { 0.3 } else if days < 7 { 0.2 } else if days < 30 { 0.1 } else { 0.0 }
        } else {
            0.0
        };
        
        0.5 + usage_score + recency_score
    }
}

#[async_trait]
pub trait PlatformProvider: Send + Sync {
    async fn get_installed_applications(&self) -> Result<Vec<AppInfo>, PlatformError>;
    fn register_global_hotkey(&self, hotkey: &str, callback: Box<dyn Fn() + Send>) -> Result<(), PlatformError>;
    fn unregister_global_hotkey(&self, hotkey: &str) -> Result<(), PlatformError>;
    fn show_notification(&self, title: &str, message: &str) -> Result<(), PlatformError>;
    fn get_system_theme(&self) -> Theme;
    async fn open_with_default_app(&self, path: &std::path::Path) -> Result<(), PlatformError>;
    fn copy_to_clipboard(&self, text: &str) -> Result<(), PlatformError>;
    fn paste_from_clipboard(&self) -> Result<String, PlatformError>;
}

// Windows implementation
#[cfg(target_os = "windows")]
pub struct WindowsPlatform {
    app_cache: std::sync::RwLock<Vec<AppInfo>>,
}

#[cfg(target_os = "windows")]
impl WindowsPlatform {
    pub fn new() -> Self {
        Self {
            app_cache: std::sync::RwLock::new(Vec::new()),
        }
    }
    
    async fn scan_registry(&self) -> Result<Vec<AppInfo>, PlatformError> {
        // Registry scanning implementation would go here
        info!("Scanning Windows registry for applications");
        Ok(Vec::new())
    }
    
    async fn scan_start_menu(&self) -> Result<Vec<AppInfo>, PlatformError> {
        // Start menu scanning implementation would go here
        info!("Scanning Windows Start Menu");
        Ok(Vec::new())
    }
    
    async fn scan_program_files(&self) -> Result<Vec<AppInfo>, PlatformError> {
        // Program Files scanning implementation would go here
        info!("Scanning Program Files directories");
        Ok(Vec::new())
    }
}

#[cfg(target_os = "windows")]
#[async_trait]
impl PlatformProvider for WindowsPlatform {
    async fn get_installed_applications(&self) -> Result<Vec<AppInfo>, PlatformError> {
        let mut apps = Vec::new();
        
        apps.extend(self.scan_registry().await?);
        apps.extend(self.scan_start_menu().await?);
        apps.extend(self.scan_program_files().await?);
        
        Ok(apps)
    }
    
    fn register_global_hotkey(&self, hotkey: &str, callback: Box<dyn Fn() + Send>) -> Result<(), PlatformError> {
        info!("Registering Windows global hotkey: {}", hotkey);
        // Windows API implementation would go here
        Ok(())
    }
    
    fn unregister_global_hotkey(&self, hotkey: &str) -> Result<(), PlatformError> {
        info!("Unregistering Windows global hotkey: {}", hotkey);
        Ok(())
    }
    
    fn show_notification(&self, title: &str, message: &str) -> Result<(), PlatformError> {
        info!("Showing Windows notification: {} - {}", title, message);
        Ok(())
    }
    
    fn get_system_theme(&self) -> Theme {
        // Windows theme detection would go here
        Theme::System
    }
    
    async fn open_with_default_app(&self, path: &std::path::Path) -> Result<(), PlatformError> {
        info!("Opening file with default app on Windows: {:?}", path);
        Ok(())
    }
    
    fn copy_to_clipboard(&self, text: &str) -> Result<(), PlatformError> {
        info!("Copying to Windows clipboard: {}", text);
        Ok(())
    }
    
    fn paste_from_clipboard(&self) -> Result<String, PlatformError> {
        info!("Pasting from Windows clipboard");
        Ok(String::new())
    }
}

// macOS implementation
#[cfg(target_os = "macos")]
pub struct MacOSPlatform;

#[cfg(target_os = "macos")]
impl MacOSPlatform {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(target_os = "macos")]
#[async_trait]
impl PlatformProvider for MacOSPlatform {
    async fn get_installed_applications(&self) -> Result<Vec<AppInfo>, PlatformError> {
        info!("Scanning macOS applications");
        // macOS application scanning implementation would go here
        Ok(Vec::new())
    }
    
    fn register_global_hotkey(&self, hotkey: &str, callback: Box<dyn Fn() + Send>) -> Result<(), PlatformError> {
        info!("Registering macOS global hotkey: {}", hotkey);
        Ok(())
    }
    
    fn unregister_global_hotkey(&self, hotkey: &str) -> Result<(), PlatformError> {
        info!("Unregistering macOS global hotkey: {}", hotkey);
        Ok(())
    }
    
    fn show_notification(&self, title: &str, message: &str) -> Result<(), PlatformError> {
        info!("Showing macOS notification: {} - {}", title, message);
        Ok(())
    }
    
    fn get_system_theme(&self) -> Theme {
        Theme::System
    }
    
    async fn open_with_default_app(&self, path: &std::path::Path) -> Result<(), PlatformError> {
        info!("Opening file with default app on macOS: {:?}", path);
        Ok(())
    }
    
    fn copy_to_clipboard(&self, text: &str) -> Result<(), PlatformError> {
        info!("Copying to macOS clipboard: {}", text);
        Ok(())
    }
    
    fn paste_from_clipboard(&self) -> Result<String, PlatformError> {
        info!("Pasting from macOS clipboard");
        Ok(String::new())
    }
}

// Linux implementation
#[cfg(target_os = "linux")]
pub struct LinuxPlatform;

#[cfg(target_os = "linux")]
impl LinuxPlatform {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(target_os = "linux")]
#[async_trait]
impl PlatformProvider for LinuxPlatform {
    async fn get_installed_applications(&self) -> Result<Vec<AppInfo>, PlatformError> {
        info!("Scanning Linux applications");
        // Linux application scanning implementation would go here
        Ok(Vec::new())
    }
    
    fn register_global_hotkey(&self, hotkey: &str, callback: Box<dyn Fn() + Send>) -> Result<(), PlatformError> {
        info!("Registering Linux global hotkey: {}", hotkey);
        Ok(())
    }
    
    fn unregister_global_hotkey(&self, hotkey: &str) -> Result<(), PlatformError> {
        info!("Unregistering Linux global hotkey: {}", hotkey);
        Ok(())
    }
    
    fn show_notification(&self, title: &str, message: &str) -> Result<(), PlatformError> {
        info!("Showing Linux notification: {} - {}", title, message);
        Ok(())
    }
    
    fn get_system_theme(&self) -> Theme {
        Theme::System
    }
    
    async fn open_with_default_app(&self, path: &std::path::Path) -> Result<(), PlatformError> {
        info!("Opening file with default app on Linux: {:?}", path);
        Ok(())
    }
    
    fn copy_to_clipboard(&self, text: &str) -> Result<(), PlatformError> {
        info!("Copying to Linux clipboard: {}", text);
        Ok(())
    }
    
    fn paste_from_clipboard(&self) -> Result<String, PlatformError> {
        info!("Pasting from Linux clipboard");
        Ok(String::new())
    }
}

// Platform provider factory
pub fn create_platform_provider() -> Arc<dyn PlatformProvider> {
    #[cfg(target_os = "windows")]
    return Arc::new(WindowsPlatform::new());
    
    #[cfg(target_os = "macos")]
    return Arc::new(MacOSPlatform::new());
    
    #[cfg(target_os = "linux")]
    return Arc::new(LinuxPlatform::new());
    
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    compile_error!("Unsupported platform");
}