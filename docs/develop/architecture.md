# アーキテクチャ設計書

FalCommand の全体的なシステム設計と実装アーキテクチャについて説明します。

## システム概要

### 設計原則
1. **軽量性**: 最小限のリソース使用量
2. **応答性**: 瞬時の検索とUIレスポンス
3. **モジュラー性**: 機能別の明確な責任分離
4. **拡張性**: プラグインシステムによる機能拡張
5. **クロスプラットフォーム**: 統一されたコードベース

### 技術スタック
- **言語**: Rust (安全性、パフォーマンス、メモリ効率)
- **UI フレームワーク**: Slint (軽量、ネイティブ性能)
- **非同期処理**: Tokio (高性能非同期ランタイム)
- **シリアライゼーション**: Serde (設定・データ処理)
- **検索**: Fuzzy Matcher (あいまい検索)

## システム全体図

```
┌─────────────────────────────────────────────────────────┐
│                    FalCommand                           │
├─────────────────────────────────────────────────────────┤
│                  UI Layer (Slint)                      │
├─────────────────────────────────────────────────────────┤
│               Application Layer                         │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐      │
│  │   Search    │ │   Config    │ │  Hotkeys    │      │
│  │   Engine    │ │  Manager    │ │  Manager    │      │
│  └─────────────┘ └─────────────┘ └─────────────┘      │
├─────────────────────────────────────────────────────────┤
│                 Core Layer                              │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐      │
│  │   Index     │ │   Plugin    │ │   Sync      │      │
│  │   Manager   │ │   System    │ │   Manager   │      │
│  └─────────────┘ └─────────────┘ └─────────────┘      │
├─────────────────────────────────────────────────────────┤
│                Platform Layer                           │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐      │
│  │   Windows   │ │    macOS    │ │    Linux    │      │
│  │    Impl     │ │    Impl     │ │    Impl     │      │
│  └─────────────┘ └─────────────┘ └─────────────┘      │
└─────────────────────────────────────────────────────────┘
```

## 詳細アーキテクチャ

### 1. UI Layer (プレゼンテーション層)

#### 役割
- ユーザーインターフェース
- ユーザー入力の処理
- 検索結果の表示
- 設定画面の管理

#### 主要コンポーネント
```rust
// src/ui/mod.rs
pub struct MainWindow {
    slint_ui: MainWindowHandle,
    search_engine: Arc<SearchEngine>,
    config: Arc<Mutex<Config>>,
}

impl MainWindow {
    pub fn new() -> Result<Self, UiError> {
        // UI初期化
    }
    
    pub fn show(&self) -> Result<(), UiError> {
        // ウィンドウ表示
    }
    
    pub fn hide(&self) -> Result<(), UiError> {
        // ウィンドウ非表示
    }
    
    fn setup_callbacks(&self) {
        // 検索、実行、設定のコールバック設定
    }
}
```

#### Slint UI定義
```slint
// ui/main_window.slint
import { Button, LineEdit, ListView } from "std-widgets.slint";

export component MainWindow inherits Window {
    width: 600px;
    height: 400px;
    title: "FalCommand";
    
    property <[SearchResult]> search_results;
    property <string> search_query;
    callback search_changed(string);
    callback execute_result(SearchResult);
    
    VerticalBox {
        SearchInput { query <=> search_query; }
        ResultList { 
            results <=> search_results;
            on_execute(result) => { execute_result(result); }
        }
    }
}
```

### 2. Application Layer (アプリケーション層)

#### Search Engine
```rust
// src/search/engine.rs
pub struct SearchEngine {
    index_manager: Arc<IndexManager>,
    plugin_system: Arc<PluginSystem>,
    config: Arc<RwLock<Config>>,
}

impl SearchEngine {
    pub async fn search(&self, query: &str) -> Vec<SearchResult> {
        let mut results = Vec::new();
        
        // 1. アプリケーション検索
        results.extend(self.search_applications(query).await);
        
        // 2. ファイル検索
        results.extend(self.search_files(query).await);
        
        // 3. プラグイン検索
        results.extend(self.search_plugins(query).await);
        
        // 4. 結果のソートと優先順位付け
        self.sort_results(&mut results, query);
        
        results
    }
    
    async fn search_applications(&self, query: &str) -> Vec<SearchResult> {
        // アプリケーション検索の実装
    }
}
```

#### Config Manager
```rust
// src/config/manager.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub appearance: AppearanceConfig,
    pub behavior: BehaviorConfig,
    pub search: SearchConfig,
    pub plugins: PluginConfig,
    pub sync: SyncConfig,
}

pub struct ConfigManager {
    config: Arc<RwLock<Config>>,
    config_path: PathBuf,
    watchers: Vec<notify::RecommendedWatcher>,
}

impl ConfigManager {
    pub fn load() -> Result<Self, ConfigError> {
        // 設定ファイル読み込み
    }
    
    pub async fn save(&self) -> Result<(), ConfigError> {
        // 設定ファイル保存
    }
    
    pub fn watch_changes<F>(&mut self, callback: F) 
    where F: Fn(&Config) + Send + 'static {
        // 設定変更監視
    }
}
```

#### Hotkey Manager
```rust
// src/hotkeys/manager.rs
pub struct HotkeyManager {
    bindings: HashMap<String, HotkeyBinding>,
    platform_handler: Box<dyn PlatformHotkeyHandler>,
}

pub trait PlatformHotkeyHandler: Send + Sync {
    fn register(&self, hotkey: &str, callback: Box<dyn Fn() + Send>) -> Result<(), HotkeyError>;
    fn unregister(&self, hotkey: &str) -> Result<(), HotkeyError>;
}

impl HotkeyManager {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            platform_handler: create_platform_handler(),
        }
    }
    
    pub fn register_hotkey<F>(&mut self, hotkey: &str, callback: F) -> Result<(), HotkeyError>
    where F: Fn() + Send + 'static {
        // ホットキー登録
    }
}
```

### 3. Core Layer (コア層)

#### Index Manager
```rust
// src/index/manager.rs
pub struct IndexManager {
    app_index: RwLock<HashMap<String, AppInfo>>,
    file_index: RwLock<BTreeMap<String, FileInfo>>,
    rebuild_scheduler: Arc<RebuildScheduler>,
}

#[derive(Debug, Clone)]
pub struct AppInfo {
    pub name: String,
    pub path: PathBuf,
    pub icon: Option<PathBuf>,
    pub keywords: Vec<String>,
    pub usage_count: u32,
    pub last_used: Option<SystemTime>,
}

impl IndexManager {
    pub async fn rebuild_index(&self) -> Result<(), IndexError> {
        tokio::join!(
            self.rebuild_app_index(),
            self.rebuild_file_index()
        );
        Ok(())
    }
    
    pub fn search_fuzzy(&self, query: &str) -> Vec<SearchResult> {
        // ファジー検索実装
    }
    
    async fn rebuild_app_index(&self) {
        // アプリケーションインデックス再構築
    }
}
```

#### Plugin System
```rust
// src/plugins/system.rs
pub trait Plugin: Send + Sync + Debug {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn search(&self, query: &str) -> BoxFuture<Vec<SearchResult>>;
    fn execute(&self, result: &SearchResult) -> BoxFuture<Result<(), PluginError>>;
    fn can_handle(&self, query: &str) -> bool;
}

pub struct PluginSystem {
    plugins: RwLock<Vec<Arc<dyn Plugin>>>,
    plugin_config: Arc<RwLock<PluginConfig>>,
}

impl PluginSystem {
    pub fn register_plugin(&self, plugin: Arc<dyn Plugin>) {
        let mut plugins = self.plugins.write().unwrap();
        plugins.push(plugin);
    }
    
    pub async fn search_all(&self, query: &str) -> Vec<SearchResult> {
        let plugins = self.plugins.read().unwrap();
        let futures: Vec<_> = plugins.iter()
            .filter(|p| p.can_handle(query))
            .map(|p| p.search(query))
            .collect();
        
        let results = join_all(futures).await;
        results.into_iter().flatten().collect()
    }
}
```

#### 組み込みプラグイン例
```rust
// src/plugins/calculator.rs
#[derive(Debug)]
pub struct CalculatorPlugin;

impl Plugin for CalculatorPlugin {
    fn name(&self) -> &str { "Calculator" }
    fn version(&self) -> &str { "1.0.0" }
    
    fn search(&self, query: &str) -> BoxFuture<Vec<SearchResult>> {
        let query = query.to_string();
        Box::pin(async move {
            if let Ok(result) = evaluate_math_expression(&query) {
                vec![SearchResult {
                    title: format!("{} = {}", query, result),
                    description: "数式の計算結果".to_string(),
                    action: Action::CopyToClipboard(result.to_string()),
                    icon: Icon::Calculator,
                    score: 1.0,
                }]
            } else {
                vec![]
            }
        })
    }
    
    fn execute(&self, result: &SearchResult) -> BoxFuture<Result<(), PluginError>> {
        // クリップボードへのコピー実装
    }
    
    fn can_handle(&self, query: &str) -> bool {
        // 数式かどうかの判定
        MATH_REGEX.is_match(query)
    }
}
```

#### Sync Manager
```rust
// src/sync/manager.rs
pub struct SyncManager {
    local_storage: Arc<LocalStorage>,
    cloud_providers: Vec<Arc<dyn CloudProvider>>,
    sync_config: Arc<RwLock<SyncConfig>>,
}

pub trait CloudProvider: Send + Sync {
    fn name(&self) -> &str;
    async fn upload(&self, data: &[u8]) -> Result<(), SyncError>;
    async fn download(&self) -> Result<Vec<u8>, SyncError>;
    async fn is_available(&self) -> bool;
}

impl SyncManager {
    pub async fn sync_up(&self) -> Result<(), SyncError> {
        let data = self.local_storage.export_data().await?;
        let encrypted_data = self.encrypt_data(&data)?;
        
        for provider in &self.cloud_providers {
            if provider.is_available().await {
                provider.upload(&encrypted_data).await?;
                break;
            }
        }
        Ok(())
    }
    
    pub async fn sync_down(&self) -> Result<(), SyncError> {
        // 同期ダウンロード実装
    }
}
```

### 4. Platform Layer (プラットフォーム層)

#### プラットフォーム抽象化
```rust
// src/platform/mod.rs
pub trait PlatformProvider: Send + Sync {
    fn get_installed_applications(&self) -> BoxFuture<Vec<AppInfo>>;
    fn register_global_hotkey(&self, hotkey: &str, callback: Arc<dyn Fn() + Send + Sync>) -> Result<(), PlatformError>;
    fn show_notification(&self, title: &str, message: &str) -> Result<(), PlatformError>;
    fn get_system_theme(&self) -> Theme;
}

#[cfg(target_os = "windows")]
pub fn create_platform_provider() -> Arc<dyn PlatformProvider> {
    Arc::new(WindowsPlatform::new())
}

#[cfg(target_os = "macos")]
pub fn create_platform_provider() -> Arc<dyn PlatformProvider> {
    Arc::new(MacOSPlatform::new())
}

#[cfg(target_os = "linux")]
pub fn create_platform_provider() -> Arc<dyn PlatformProvider> {
    Arc::new(LinuxPlatform::new())
}
```

#### Windows
```rust
// src/platform/windows.rs
pub struct WindowsPlatform {
    app_cache: RwLock<Vec<AppInfo>>,
}

impl PlatformProvider for WindowsPlatform {
    fn get_installed_applications(&self) -> BoxFuture<Vec<AppInfo>> {
        Box::pin(async move {
            let mut apps = Vec::new();
            
            // レジストリから取得
            apps.extend(self.scan_registry().await?);
            
            // Start Menu から取得
            apps.extend(self.scan_start_menu().await?);
            
            // Program Files から取得
            apps.extend(self.scan_program_files().await?);
            
            Ok(apps)
        })
    }
    
    fn register_global_hotkey(&self, hotkey: &str, callback: Arc<dyn Fn() + Send + Sync>) -> Result<(), PlatformError> {
        // Windows API使用してグローバルホットキー登録
    }
}
```

## データフロー

### 1. 起動フロー
```
1. main() 
   ↓
2. プラットフォーム初期化
   ↓
3. 設定読み込み
   ↓
4. インデックス構築（バックグラウンド）
   ↓
5. プラグインロード
   ↓
6. UI初期化
   ↓
7. ホットキー登録
   ↓
8. イベントループ開始
```

### 2. 検索フロー
```
1. ユーザー入力 (UI Layer)
   ↓
2. 検索クエリ処理 (Application Layer)
   ↓
3. 並列検索実行
   ├─ アプリケーション検索 (Index Manager)
   ├─ ファイル検索 (Index Manager)  
   └─ プラグイン検索 (Plugin System)
   ↓
4. 結果統合・ソート (Search Engine)
   ↓
5. UI更新 (UI Layer)
```

### 3. 実行フロー
```
1. ユーザー選択 (UI Layer)
   ↓
2. 実行要求処理 (Application Layer)
   ↓
3. アクション判定
   ├─ アプリケーション起動 (Platform Layer)
   ├─ ファイル操作 (Platform Layer)
   └─ プラグイン実行 (Plugin System)
   ↓
4. 使用統計更新 (Index Manager)
   ↓
5. UI非表示 (UI Layer)
```

## パフォーマンス設計

### 1. メモリ効率
- **オブジェクトプール**: 頻繁に作成されるオブジェクトの再利用
- **遅延読み込み**: 必要時のみデータを読み込み
- **弱参照**: 循環参照の回避

### 2. 検索最適化
- **インクリメンタル検索**: 入力中のリアルタイム検索
- **結果キャッシュ**: 最近の検索結果をメモリに保持
- **優先度付きキューイング**: 重要度の高い結果を優先表示

### 3. 非同期処理
```rust
// 非同期検索実装例
pub async fn search_async(&self, query: String) -> Vec<SearchResult> {
    let (app_results, file_results, plugin_results) = tokio::join!(
        self.search_applications(&query),
        self.search_files(&query),
        self.search_plugins(&query)
    );
    
    let mut all_results = Vec::new();
    all_results.extend(app_results);
    all_results.extend(file_results);
    all_results.extend(plugin_results);
    
    self.sort_and_limit_results(all_results, &query)
}
```

## セキュリティ設計

### 1. データ保護
- **設定暗号化**: 機密情報の暗号化保存
- **メモリクリア**: 機密データの安全な消去
- **サンドボックス**: プラグインの実行制限

### 2. 権限管理
- **最小権限の原則**: 必要最小限の権限で動作
- **実行検証**: 危険な実行ファイルの警告表示

## 拡張性設計

### 1. プラグインAPI
```rust
// プラグイン開発者向けAPI
pub trait Plugin: Send + Sync + Debug {
    // 必須メソッド
    fn name(&self) -> &str;
    fn search(&self, query: &str) -> BoxFuture<Vec<SearchResult>>;
    fn execute(&self, result: &SearchResult) -> BoxFuture<Result<(), PluginError>>;
    
    // オプションメソッド
    fn initialize(&self) -> BoxFuture<Result<(), PluginError>> {
        Box::pin(async { Ok(()) })
    }
    
    fn configuration_ui(&self) -> Option<ConfigurationUI> {
        None
    }
}
```

### 2. 設定拡張
```rust
// プラグイン固有設定
#[derive(Serialize, Deserialize)]
pub struct PluginConfig {
    pub enabled_plugins: HashSet<String>,
    pub plugin_settings: HashMap<String, serde_json::Value>,
}
```

## モニタリング・診断

### 1. メトリクス収集
```rust
pub struct Metrics {
    pub search_latency: Histogram,
    pub memory_usage: Gauge,
    pub active_plugins: Gauge,
    pub search_requests: Counter,
}
```

### 2. ヘルスチェック
```rust
pub struct HealthChecker {
    pub fn check_system_health(&self) -> HealthStatus {
        // システム健康状態チェック
    }
    
    pub fn collect_diagnostic_info(&self) -> DiagnosticInfo {
        // 診断情報収集
    }
}
```

## まとめ

この設計により以下を実現：

1. **高性能**: 非同期処理とインデックスによる高速検索
2. **軽量**: 効率的なメモリ使用とSlintによる軽量UI
3. **拡張性**: プラグインシステムによる機能拡張
4. **保守性**: 明確な層分離とモジュール設計
5. **クロスプラットフォーム**: 統一されたAPIと実装の分離

このアーキテクチャに基づいて、段階的に機能を実装していくことで、高品質なコマンドランチャーを構築できます。