use std::collections::{HashMap, BTreeMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use log::{info, warn, error, debug};

use crate::config::Config;
use crate::platform::{PlatformProvider, AppInfo};
use crate::search::{SearchResult, Action, Category};

#[derive(Debug, thiserror::Error)]
pub enum IndexError {
    #[error("Failed to build index: {0}")]
    BuildError(String),
    
    #[error("Search error: {0}")]
    SearchError(String),
    
    #[error("File system error: {0}")]
    FileSystemError(#[from] std::io::Error),
    
    #[error("Platform error: {0}")]
    PlatformError(String),
    
    #[error("Other index error: {0}")]
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub path: PathBuf,
    pub extension: Option<String>,
    pub size: u64,
    pub modified: SystemTime,
    pub keywords: Vec<String>,
}

impl FileInfo {
    pub fn new(path: PathBuf) -> std::io::Result<Self> {
        let metadata = std::fs::metadata(&path)?;
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_string());
        
        Ok(Self {
            name,
            path: path.clone(),
            extension,
            size: metadata.len(),
            modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            keywords: Vec::new(),
        })
    }
    
    pub fn to_search_result(&self) -> SearchResult {
        SearchResult::new(&self.name, &format!("File: {}", self.path.display()))
            .with_action(Action::OpenFile(self.path.clone()))
            .with_category(Category::File)
            .with_path(self.path.clone())
            .with_score(0.5)
    }
}

pub struct IndexManager {
    config: Arc<RwLock<Config>>,
    app_index: RwLock<HashMap<String, AppInfo>>,
    file_index: RwLock<BTreeMap<String, FileInfo>>,
    last_rebuild: RwLock<Option<SystemTime>>,
}

impl IndexManager {
    pub async fn new(config: Arc<RwLock<Config>>) -> Result<Self, IndexError> {
        info!("Initializing index manager...");
        
        Ok(Self {
            config,
            app_index: RwLock::new(HashMap::new()),
            file_index: RwLock::new(BTreeMap::new()),
            last_rebuild: RwLock::new(None),
        })
    }
    
    pub async fn rebuild_index(&self, platform_provider: Arc<dyn PlatformProvider>) -> Result<(), IndexError> {
        info!("Starting index rebuild...");
        let start_time = SystemTime::now();
        
        // Rebuild in parallel
        let (app_result, file_result) = tokio::join!(
            self.rebuild_app_index(platform_provider),
            self.rebuild_file_index()
        );
        
        if let Err(e) = app_result {
            error!("Failed to rebuild app index: {}", e);
        }
        
        if let Err(e) = file_result {
            error!("Failed to rebuild file index: {}", e);
        }
        
        // Update last rebuild time
        *self.last_rebuild.write().await = Some(start_time);
        
        if let Ok(elapsed) = start_time.elapsed() {
            info!("Index rebuild completed in {:?}", elapsed);
        }
        
        Ok(())
    }
    
    async fn rebuild_app_index(&self, platform_provider: Arc<dyn PlatformProvider>) -> Result<(), IndexError> {
        info!("Rebuilding application index...");
        
        let apps = platform_provider.get_installed_applications().await
            .map_err(|e| IndexError::PlatformError(e.to_string()))?;
        
        let mut app_index = self.app_index.write().await;
        app_index.clear();
        
        for app in apps {
            let key = app.name.to_lowercase();
            app_index.insert(key, app);
        }
        
        info!("Application index rebuilt with {} entries", app_index.len());
        Ok(())
    }
    
    async fn rebuild_file_index(&self) -> Result<(), IndexError> {
        info!("Rebuilding file index...");
        
        let config = self.config.read().await;
        let mut file_index = self.file_index.write().await;
        file_index.clear();
        
        // Get platform-specific include paths
        let current_os = if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "macos") {
            "macos"
        } else {
            "linux"
        };
        
        if let Some(paths) = config.search.include_paths.get(current_os) {
            for path_str in paths {
                let path = PathBuf::from(path_str);
                if let Err(e) = self.scan_directory(&path, &mut file_index, &config.search.exclude_patterns).await {
                    warn!("Failed to scan directory {}: {}", path.display(), e);
                }
            }
        }
        
        info!("File index rebuilt with {} entries", file_index.len());
        Ok(())
    }
    
    async fn scan_directory(
        &self,
        dir: &Path,
        file_index: &mut BTreeMap<String, FileInfo>,
        exclude_patterns: &[String],
    ) -> Result<(), IndexError> {
        if !dir.exists() {
            debug!("Directory does not exist: {}", dir.display());
            return Ok(());
        }
        
        let mut entries = match tokio::fs::read_dir(dir).await {
            Ok(entries) => entries,
            Err(e) => {
                warn!("Cannot read directory {}: {}", dir.display(), e);
                return Ok(());
            }
        };
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            // Skip if matches exclude patterns
            if self.should_exclude(&path, exclude_patterns) {
                continue;
            }
            
            if entry.file_type().await?.is_file() {
                if let Ok(file_info) = FileInfo::new(path.clone()) {
                    let key = file_info.name.to_lowercase();
                    file_index.insert(key, file_info);
                }
            }
        }
        
        Ok(())
    }
    
    fn should_exclude(&self, path: &Path, exclude_patterns: &[String]) -> bool {
        let path_str = path.to_string_lossy();
        
        for pattern in exclude_patterns {
            // Simple glob-like matching
            if pattern.contains('*') {
                let pattern_parts: Vec<&str> = pattern.split('*').collect();
                if pattern_parts.len() == 2 {
                    let prefix = pattern_parts[0];
                    let suffix = pattern_parts[1];
                    
                    if path_str.starts_with(prefix) && path_str.ends_with(suffix) {
                        return true;
                    }
                }
            } else if path_str.contains(pattern) {
                return true;
            }
        }
        
        false
    }
    
    pub async fn search_applications(&self, query: &str) -> Result<Vec<SearchResult>, IndexError> {
        let app_index = self.app_index.read().await;
        let mut results = Vec::new();
        
        let query_lower = query.to_lowercase();
        
        for (key, app_info) in app_index.iter() {
            let score = if key.contains(&query_lower) {
                self.calculate_app_match_score(key, &query_lower, app_info)
            } else if app_info.keywords.iter().any(|k| k.to_lowercase().contains(&query_lower)) {
                0.6 // Lower score for keyword matches
            } else {
                continue;
            };
            
            let mut result = app_info.to_search_result();
            result.score = score;
            results.push(result);
        }
        
        // Sort by score
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        Ok(results)
    }
    
    pub async fn search_files(&self, query: &str) -> Result<Vec<SearchResult>, IndexError> {
        let file_index = self.file_index.read().await;
        let mut results = Vec::new();
        
        let query_lower = query.to_lowercase();
        
        for (key, file_info) in file_index.iter() {
            if key.contains(&query_lower) {
                let score = self.calculate_file_match_score(key, &query_lower);
                let mut result = file_info.to_search_result();
                result.score = score;
                results.push(result);
            }
        }
        
        // Sort by score
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        // Limit to reasonable number for file results
        results.truncate(20);
        
        Ok(results)
    }
    
    fn calculate_app_match_score(&self, app_name: &str, query: &str, app_info: &AppInfo) -> f64 {
        let mut score = 0.5;
        
        // Exact match gets highest score
        if app_name == query {
            score += 0.4;
        } else if app_name.starts_with(query) {
            score += 0.3;
        } else if app_name.contains(query) {
            score += 0.2;
        }
        
        // Usage frequency bonus
        let usage_bonus = (app_info.usage_count as f64 * 0.01).min(0.2);
        score += usage_bonus;
        
        // Recent usage bonus
        if let Some(last_used) = app_info.last_used {
            if let Ok(elapsed) = SystemTime::now().duration_since(last_used) {
                let hours = elapsed.as_secs() / 3600;
                if hours < 24 {
                    score += 0.1;
                }
            }
        }
        
        score.min(1.0)
    }
    
    fn calculate_file_match_score(&self, file_name: &str, query: &str) -> f64 {
        let mut score = 0.3;
        
        if file_name == query {
            score += 0.4;
        } else if file_name.starts_with(query) {
            score += 0.3;
        } else if file_name.contains(query) {
            score += 0.2;
        }
        
        score.min(1.0)
    }
    
    pub async fn get_app_info(&self, app_name: &str) -> Option<AppInfo> {
        let app_index = self.app_index.read().await;
        app_index.get(&app_name.to_lowercase()).cloned()
    }
    
    pub async fn update_app_usage(&self, app_name: &str) {
        let mut app_index = self.app_index.write().await;
        if let Some(app_info) = app_index.get_mut(&app_name.to_lowercase()) {
            app_info.increment_usage();
            info!("Updated usage for app: {}", app_name);
        }
    }
    
    pub async fn get_index_stats(&self) -> IndexStats {
        let app_index = self.app_index.read().await;
        let file_index = self.file_index.read().await;
        let last_rebuild = *self.last_rebuild.read().await;
        
        IndexStats {
            app_count: app_index.len(),
            file_count: file_index.len(),
            last_rebuild,
        }
    }
}

#[derive(Debug, Clone)]
pub struct IndexStats {
    pub app_count: usize,
    pub file_count: usize,
    pub last_rebuild: Option<SystemTime>,
}

impl IndexStats {
    pub fn is_stale(&self) -> bool {
        match self.last_rebuild {
            Some(last_rebuild) => {
                if let Ok(elapsed) = SystemTime::now().duration_since(last_rebuild) {
                    elapsed.as_secs() > 86400 // 24 hours
                } else {
                    true
                }
            }
            None => true,
        }
    }
}

pub type Result<T> = std::result::Result<T, IndexError>;