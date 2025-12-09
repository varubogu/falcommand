use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use log::info;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub description: String,
    pub path: Option<PathBuf>,
    pub icon: Option<PathBuf>,
    pub action: Action,
    pub score: f64,
    pub category: Category,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Action {
    ExecuteApplication { 
        path: PathBuf,
        args: Vec<String>,
    },
    OpenFile(PathBuf),
    OpenUrl(String),
    CopyToClipboard(String),
    ExecuteCommand {
        command: String,
        args: Vec<String>,
    },
    PluginAction {
        plugin_id: String,
        action_data: serde_json::Value,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Category {
    Application,
    File,
    Bookmark,
    Plugin(String),
    SystemCommand,
    CustomCommand,
}

#[derive(Debug, thiserror::Error)]
pub enum ActionError {
    #[error("Platform error: {0}")]
    PlatformError(String),
    
    #[error("Other error: {0}")]
    Other(String),
}

impl SearchResult {
    pub fn new(title: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: description.into(),
            path: None,
            icon: None,
            action: Action::CopyToClipboard(String::new()),
            score: 0.0,
            category: Category::SystemCommand,
        }
    }
    
    pub fn with_action(mut self, action: Action) -> Self {
        self.action = action;
        self
    }
    
    pub fn with_score(mut self, score: f64) -> Self {
        self.score = score.clamp(0.0, 1.0);
        self
    }
    
    pub fn with_category(mut self, category: Category) -> Self {
        self.category = category;
        self
    }
    
    pub fn with_path(mut self, path: PathBuf) -> Self {
        self.path = Some(path);
        self
    }
    
    pub fn with_icon(mut self, icon: PathBuf) -> Self {
        self.icon = Some(icon);
        self
    }
}

impl Action {
    pub async fn execute(&self) -> Result<(), ActionError> {
        match self {
            Action::ExecuteApplication { path, args } => {
                info!("Executing application: {:?} with args: {:?}", path, args);
                let mut cmd = tokio::process::Command::new(path);
                cmd.args(args);
                let result = cmd.spawn();
                match result {
                    Ok(_) => Ok(()),
                    Err(e) => Err(ActionError::PlatformError(format!("Failed to execute application: {}", e))),
                }
            }
            Action::OpenFile(path) => {
                info!("Opening file: {:?}", path);
                #[cfg(target_os = "windows")]
                {
                    let result = tokio::process::Command::new("cmd")
                        .args(&["/C", "start", "", &path.to_string_lossy()])
                        .spawn();
                    match result {
                        Ok(_) => Ok(()),
                        Err(e) => Err(ActionError::PlatformError(format!("Failed to open file: {}", e))),
                    }
                }
                #[cfg(target_os = "macos")]
                {
                    let result = tokio::process::Command::new("open")
                        .arg(path)
                        .spawn();
                    match result {
                        Ok(_) => Ok(()),
                        Err(e) => Err(ActionError::PlatformError(format!("Failed to open file: {}", e))),
                    }
                }
                #[cfg(target_os = "linux")]
                {
                    let result = tokio::process::Command::new("xdg-open")
                        .arg(path)
                        .spawn();
                    match result {
                        Ok(_) => Ok(()),
                        Err(e) => Err(ActionError::PlatformError(format!("Failed to open file: {}", e))),
                    }
                }
            }
            Action::OpenUrl(url) => {
                info!("Opening URL: {}", url);
                // This would use platform-specific URL opening
                Ok(())
            }
            Action::CopyToClipboard(text) => {
                info!("Copying to clipboard: {}", text);
                // This would use platform-specific clipboard functionality
                Ok(())
            }
            Action::ExecuteCommand { command, args } => {
                info!("Executing command: {} with args: {:?}", command, args);
                let mut cmd = tokio::process::Command::new(command);
                cmd.args(args);
                let result = cmd.spawn();
                match result {
                    Ok(_) => Ok(()),
                    Err(e) => Err(ActionError::PlatformError(format!("Failed to execute command: {}", e))),
                }
            }
            Action::PluginAction { plugin_id, action_data } => {
                info!("Executing plugin action: {} with data: {}", plugin_id, action_data);
                // This would delegate to the plugin system
                Ok(())
            }
        }
    }
}