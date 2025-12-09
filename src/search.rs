use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use log::{info, warn, error};

use crate::config::Config;
use crate::index::IndexManager;
use crate::plugins::PluginSystem;

#[derive(Debug, thiserror::Error)]
pub enum SearchError {
    #[error("Index error: {0}")]
    IndexError(String),
    
    #[error("Plugin error: {0}")]
    PluginError(String),
    
    #[error("Platform error: {0}")]
    PlatformError(String),
    
    #[error("Other error: {0}")]
    Other(String),
}

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
    pub async fn execute(&self) -> Result<(), SearchError> {
        match self {
            Action::ExecuteApplication { path, args } => {
                info!("Executing application: {:?} with args: {:?}", path, args);
                let mut cmd = tokio::process::Command::new(path);
                cmd.args(args);
                let result = cmd.spawn();
                match result {
                    Ok(_) => Ok(()),
                    Err(e) => Err(SearchError::PlatformError(format!("Failed to execute application: {}", e))),
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
                        Err(e) => Err(SearchError::PlatformError(format!("Failed to open file: {}", e))),
                    }
                }
                #[cfg(target_os = "macos")]
                {
                    let result = tokio::process::Command::new("open")
                        .arg(path)
                        .spawn();
                    match result {
                        Ok(_) => Ok(()),
                        Err(e) => Err(SearchError::PlatformError(format!("Failed to open file: {}", e))),
                    }
                }
                #[cfg(target_os = "linux")]
                {
                    let result = tokio::process::Command::new("xdg-open")
                        .arg(path)
                        .spawn();
                    match result {
                        Ok(_) => Ok(()),
                        Err(e) => Err(SearchError::PlatformError(format!("Failed to open file: {}", e))),
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
                    Err(e) => Err(SearchError::PlatformError(format!("Failed to execute command: {}", e))),
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

pub struct SearchEngine {
    config: Arc<RwLock<Config>>,
    index_manager: Arc<IndexManager>,
    plugin_system: Arc<PluginSystem>,
    matcher: SkimMatcherV2,
}

impl SearchEngine {
    pub async fn new(
        config: Arc<RwLock<Config>>,
        index_manager: Arc<IndexManager>,
        plugin_system: Arc<PluginSystem>,
    ) -> Result<Self, SearchError> {
        info!("Initializing search engine...");
        
        Ok(Self {
            config,
            index_manager,
            plugin_system,
            matcher: SkimMatcherV2::default(),
        })
    }
    
    pub async fn search(&self, query: &str) -> Vec<SearchResult> {
        if query.trim().is_empty() {
            return Vec::new();
        }
        
        info!("Searching for: '{}'", query);
        let mut all_results = Vec::new();
        
        // Search in parallel
        let (app_results, file_results, plugin_results) = tokio::join!(
            self.search_applications(query),
            self.search_files(query),
            self.search_plugins(query)
        );
        
        all_results.extend(app_results);
        all_results.extend(file_results);
        all_results.extend(plugin_results);
        
        // Sort by score and limit results
        self.sort_and_limit_results(all_results, query).await
    }
    
    async fn search_applications(&self, query: &str) -> Vec<SearchResult> {
        match self.index_manager.search_applications(query).await {
            Ok(results) => results,
            Err(e) => {
                error!("Application search failed: {}", e);
                Vec::new()
            }
        }
    }
    
    async fn search_files(&self, query: &str) -> Vec<SearchResult> {
        let config = self.config.read().await;
        if !config.search.enable_file_search {
            return Vec::new();
        }
        
        match self.index_manager.search_files(query).await {
            Ok(results) => results,
            Err(e) => {
                error!("File search failed: {}", e);
                Vec::new()
            }
        }
    }
    
    async fn search_plugins(&self, query: &str) -> Vec<SearchResult> {
        match self.plugin_system.search_all(query).await {
            Ok(results) => results,
            Err(e) => {
                error!("Plugin search failed: {}", e);
                Vec::new()
            }
        }
    }
    
    async fn sort_and_limit_results(&self, mut results: Vec<SearchResult>, query: &str) -> Vec<SearchResult> {
        let config = self.config.read().await;
        let max_results = config.behavior.max_results;
        
        // Sort by score (descending)
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        // Apply fuzzy matching boost for better matches
        for result in &mut results {
            if let Some((score, _)) = self.matcher.fuzzy_match(&result.title, query) {
                let normalized_score = score as f64 / 100.0; // Normalize to 0.0-1.0
                result.score = (result.score + normalized_score) / 2.0; // Combine scores
            }
        }
        
        // Re-sort after fuzzy boost
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        // Limit results
        results.truncate(max_results);
        
        results
    }
    
    pub fn add_to_history(&self, query: &str, selected_result: &SearchResult) {
        info!("Adding to search history: '{}' -> '{}'", query, selected_result.title);
        // This would store search history for learning user preferences
    }
}

pub type Result<T> = std::result::Result<T, SearchError>;