use std::sync::Arc;
use tokio::sync::RwLock;
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use log::{info, error};

use falcommand_config::{Config, SearchResult};
use crate::index::IndexManager;

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

pub struct SearchEngine {
    config: Arc<RwLock<Config>>,
    index_manager: Arc<IndexManager>,
    matcher: SkimMatcherV2,
}

impl std::fmt::Debug for SearchEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SearchEngine")
            .field("config", &self.config)
            .field("index_manager", &self.index_manager)
            .field("matcher", &"SkimMatcherV2")
            .finish()
    }
}

impl SearchEngine {
    pub async fn new(
        config: Arc<RwLock<Config>>,
        index_manager: Arc<IndexManager>,
    ) -> std::result::Result<Self, SearchError> {
        info!("Initializing search engine...");
        
        Ok(Self {
            config,
            index_manager,
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
        let (app_results, file_results) = tokio::join!(
            self.search_applications(query),
            self.search_files(query)
        );
        
        all_results.extend(app_results);
        all_results.extend(file_results);
        
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
    
    
    async fn sort_and_limit_results(&self, mut results: Vec<SearchResult>, query: &str) -> Vec<SearchResult> {
        let config = self.config.read().await;
        let max_results = config.behavior.max_results;
        
        // Sort by score (descending)
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        // Apply fuzzy matching boost for better matches
        for result in &mut results {
            if let Some(score) = self.matcher.fuzzy_match(&result.title, query) {
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
