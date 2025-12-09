use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::RwLock;
use serde_json;
use log::{info, warn, error};

use crate::config::Config;
use crate::search::SearchResult;

#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("初期化に失敗しました: {0}")]
    InitializationFailed(String),
    
    #[error("検索処理でエラーが発生しました: {0}")]
    SearchError(String),
    
    #[error("実行処理でエラーが発生しました: {0}")]
    ExecutionError(String),
    
    #[error("設定エラー: {0}")]
    ConfigurationError(String),
    
    #[error("I/Oエラー: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("その他のエラー: {0}")]
    Other(String),
}

impl From<&str> for PluginError {
    fn from(s: &str) -> Self {
        PluginError::Other(s.to_string())
    }
}

impl From<String> for PluginError {
    fn from(s: String) -> Self {
        PluginError::Other(s)
    }
}

#[async_trait]
pub trait Plugin: Send + Sync + std::fmt::Debug {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn description(&self) -> &str;
    
    async fn initialize(&self) -> std::result::Result<(), PluginError> {
        Ok(())
    }
    
    async fn shutdown(&self) -> std::result::Result<(), PluginError> {
        Ok(())
    }
    
    fn can_handle(&self, query: &str) -> bool;
    async fn search(&self, query: &str) -> std::result::Result<Vec<SearchResult>, PluginError>;
    async fn execute(&self, result: &SearchResult) -> std::result::Result<(), PluginError>;
    
    fn has_configuration(&self) -> bool {
        false
    }
    
    fn get_configuration_ui(&self) -> Option<serde_json::Value> {
        None
    }
}

#[derive(Debug, Clone)]
pub struct PluginContext {
    config: Arc<RwLock<Config>>,
}

impl PluginContext {
    pub fn new(config: Arc<RwLock<Config>>) -> Self {
        Self { config }
    }
    
    pub async fn get_config(&self) -> std::result::Result<Config, PluginError> {
        Ok(self.config.read().await.clone())
    }
    
    pub fn log(&self, level: LogLevel, message: &str) {
        match level {
            LogLevel::Error => error!("{}", message),
            LogLevel::Warn => warn!("{}", message),
            LogLevel::Info => info!("{}", message),
            LogLevel::Debug => log::debug!("{}", message),
        }
    }
    
    pub async fn read_file(&self, path: &std::path::Path) -> std::result::Result<Vec<u8>, PluginError> {
        tokio::fs::read(path).await.map_err(Into::into)
    }
    
    pub async fn http_get(&self, url: &str) -> std::result::Result<String, PluginError> {
        // HTTP client implementation would go here
        info!("HTTP GET request to: {}", url);
        Ok(String::new())
    }
    
    pub fn show_notification(&self, title: &str, message: &str) -> std::result::Result<(), PluginError> {
        info!("Plugin notification: {} - {}", title, message);
        // This would delegate to platform provider
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
}

pub struct PluginSystem {
    plugins: RwLock<Vec<Arc<dyn Plugin>>>,
    config: Arc<RwLock<Config>>,
    context: PluginContext,
}

impl PluginSystem {
    pub async fn new(config: Arc<RwLock<Config>>) -> std::result::Result<Self, PluginError> {
        info!("Initializing plugin system...");
        
        let context = PluginContext::new(config.clone());
        
        Ok(Self {
            plugins: RwLock::new(Vec::new()),
            config,
            context,
        })
    }
    
    pub async fn load_plugins(&self) -> std::result::Result<(), PluginError> {
        info!("Loading plugins...");
        
        // Load built-in plugins
        self.load_builtin_plugins().await?;
        
        // Load external plugins (would scan plugin directory)
        // self.load_external_plugins().await?;
        
        Ok(())
    }
    
    async fn load_builtin_plugins(&self) -> std::result::Result<(), PluginError> {
        let config = self.config.read().await;
        
        // Load calculator plugin if enabled
        if config.plugins.enabled.contains(&"calculator".to_string()) {
            let calculator_plugin = Arc::new(CalculatorPlugin::new(self.context.clone()));
            calculator_plugin.initialize().await?;
            self.register_plugin(calculator_plugin).await;
            info!("Loaded calculator plugin");
        }
        
        // Load translator plugin if enabled
        if config.plugins.enabled.contains(&"translator".to_string()) {
            let translator_plugin = Arc::new(TranslatorPlugin::new(self.context.clone()));
            translator_plugin.initialize().await?;
            self.register_plugin(translator_plugin).await;
            info!("Loaded translator plugin");
        }
        
        Ok(())
    }
    
    pub async fn register_plugin(&self, plugin: Arc<dyn Plugin>) {
        let mut plugins = self.plugins.write().await;
        info!("Registering plugin: {}", plugin.name());
        plugins.push(plugin);
    }
    
    pub async fn search_all(&self, query: &str) -> std::result::Result<Vec<SearchResult>, PluginError> {
        let plugins = self.plugins.read().await;
        let mut all_results = Vec::new();
        
        for plugin in plugins.iter() {
            if plugin.can_handle(query) {
                match plugin.search(query).await {
                    Ok(mut results) => {
                        all_results.append(&mut results);
                    }
                    Err(e) => {
                        warn!("Plugin '{}' search failed: {}", plugin.name(), e);
                    }
                }
            }
        }
        
        Ok(all_results)
    }
    
    pub async fn execute_plugin_action(&self, plugin_id: &str, result: &SearchResult) -> std::result::Result<(), PluginError> {
        let plugins = self.plugins.read().await;
        
        for plugin in plugins.iter() {
            if plugin.name() == plugin_id {
                return plugin.execute(result).await;
            }
        }
        
        Err(PluginError::Other(format!("Plugin '{}' not found", plugin_id)))
    }
}

// Built-in Calculator Plugin
#[derive(Debug)]
pub struct CalculatorPlugin {
    context: PluginContext,
}

impl CalculatorPlugin {
    pub fn new(context: PluginContext) -> Self {
        Self { context }
    }
    
    fn evaluate_expression(&self, expr: &str) -> Result<f64, String> {
        // Simple expression evaluation (placeholder)
        // In a real implementation, this would use a proper math parser
        match expr.parse::<f64>() {
            Ok(num) => Ok(num),
            Err(_) => {
                // Try simple operations
                if let Some(pos) = expr.find('+') {
                    let (left, right) = expr.split_at(pos);
                    let right = &right[1..];
                    let left_val: f64 = left.trim().parse().map_err(|_| "Invalid left operand")?;
                    let right_val: f64 = right.trim().parse().map_err(|_| "Invalid right operand")?;
                    Ok(left_val + right_val)
                } else if let Some(pos) = expr.find('-') {
                    let (left, right) = expr.split_at(pos);
                    let right = &right[1..];
                    let left_val: f64 = left.trim().parse().map_err(|_| "Invalid left operand")?;
                    let right_val: f64 = right.trim().parse().map_err(|_| "Invalid right operand")?;
                    Ok(left_val - right_val)
                } else {
                    Err("Unsupported expression".to_string())
                }
            }
        }
    }
}

#[async_trait]
impl Plugin for CalculatorPlugin {
    fn name(&self) -> &str {
        "Calculator"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn description(&self) -> &str {
        "Basic calculator for mathematical expressions"
    }
    
    fn can_handle(&self, query: &str) -> bool {
        // Check if query looks like a math expression
        query.chars().any(|c| "+-*/()0123456789.".contains(c)) && 
        query.chars().any(|c| c.is_ascii_digit())
    }
    
    async fn search(&self, query: &str) -> std::result::Result<Vec<SearchResult>, PluginError> {
        match self.evaluate_expression(query) {
            Ok(result) => {
                use crate::search::{Action, Category};
                
                let search_result = SearchResult::new(
                    format!("{} = {}", query, result),
                    "Mathematical calculation"
                )
                .with_action(Action::CopyToClipboard(result.to_string()))
                .with_category(Category::Plugin("Calculator".to_string()))
                .with_score(0.9);
                
                Ok(vec![search_result])
            }
            Err(_) => Ok(vec![])
        }
    }
    
    async fn execute(&self, result: &SearchResult) -> std::result::Result<(), PluginError> {
        use crate::search::Action;
        
        if let Action::CopyToClipboard(ref text) = result.action {
            self.context.show_notification("Calculator", "Result copied to clipboard")?;
            info!("Calculator result copied: {}", text);
        }
        Ok(())
    }
}

// Built-in Translator Plugin
#[derive(Debug)]
pub struct TranslatorPlugin {
    context: PluginContext,
}

impl TranslatorPlugin {
    pub fn new(context: PluginContext) -> Self {
        Self { context }
    }
}

#[async_trait]
impl Plugin for TranslatorPlugin {
    fn name(&self) -> &str {
        "Translator"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn description(&self) -> &str {
        "Text translation plugin"
    }
    
    fn can_handle(&self, query: &str) -> bool {
        query.starts_with("translate ") || query.starts_with("翻訳 ")
    }
    
    async fn search(&self, query: &str) -> std::result::Result<Vec<SearchResult>, PluginError> {
        let text = if let Some(text) = query.strip_prefix("translate ") {
            text
        } else if let Some(text) = query.strip_prefix("翻訳 ") {
            text
        } else {
            return Ok(vec![]);
        };
        
        // Placeholder translation (in real implementation, this would call translation API)
        let translated = format!("Translation of: {}", text);
        
        use crate::search::{Action, Category};
        
        let search_result = SearchResult::new(
            format!("Translation: {}", text),
            &translated
        )
        .with_action(Action::CopyToClipboard(translated))
        .with_category(Category::Plugin("Translator".to_string()))
        .with_score(0.8);
        
        Ok(vec![search_result])
    }
    
    async fn execute(&self, result: &SearchResult) -> std::result::Result<(), PluginError> {
        use crate::search::Action;
        
        if let Action::CopyToClipboard(ref text) = result.action {
            self.context.show_notification("Translator", "Translation copied to clipboard")?;
            info!("Translation copied: {}", text);
        }
        Ok(())
    }
}