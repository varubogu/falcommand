# APIリファレンス

FalCommand の内部APIとプラグイン開発者向けAPIの詳細仕様を説明します。

## 概要

FalCommand は以下のAPIレイヤーを提供します：

1. **Core API**: システムコア機能へのアクセス
2. **Plugin API**: プラグイン開発者向けインターフェース
3. **Configuration API**: 設定管理機能
4. **Platform API**: プラットフォーム固有機能へのアクセス

## Core API

### SearchEngine

検索機能の中核を担うAPI。

```rust
pub struct SearchEngine {
    // 内部フィールドは非公開
}

impl SearchEngine {
    /// 新しい SearchEngine インスタンスを作成
    pub fn new(config: Config) -> Result<Self, SearchError>;
    
    /// 非同期検索を実行
    pub async fn search(&self, query: &str) -> Vec<SearchResult>;
    
    /// 検索結果をランキング順にソート
    pub fn sort_results(&self, results: &mut Vec<SearchResult>, query: &str);
    
    /// 検索履歴を追加
    pub fn add_to_history(&self, query: &str, selected_result: &SearchResult);
}
```

### SearchResult

検索結果を表現する構造体。

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct SearchResult {
    /// 表示タイトル
    pub title: String,
    
    /// 詳細説明
    pub description: String,
    
    /// 実行パス（アプリケーションの場合）
    pub path: Option<PathBuf>,
    
    /// アイコンパス
    pub icon: Option<PathBuf>,
    
    /// 実行アクション
    pub action: Action,
    
    /// 検索スコア（0.0-1.0）
    pub score: f64,
    
    /// カテゴリ
    pub category: Category,
}

impl SearchResult {
    /// 新しい検索結果を作成
    pub fn new(title: impl Into<String>, description: impl Into<String>) -> Self;
    
    /// アクションを設定
    pub fn with_action(mut self, action: Action) -> Self;
    
    /// スコアを設定
    pub fn with_score(mut self, score: f64) -> Self;
    
    /// カテゴリを設定
    pub fn with_category(mut self, category: Category) -> Self;
}
```

### Action

実行可能なアクションの種類。

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// アプリケーション実行
    ExecuteApplication { 
        path: PathBuf,
        args: Vec<String>,
    },
    
    /// ファイルを開く
    OpenFile(PathBuf),
    
    /// URLを開く
    OpenUrl(String),
    
    /// クリップボードにコピー
    CopyToClipboard(String),
    
    /// カスタムコマンド実行
    ExecuteCommand {
        command: String,
        args: Vec<String>,
    },
    
    /// プラグインアクション
    PluginAction {
        plugin_id: String,
        action_data: serde_json::Value,
    },
}

impl Action {
    /// アクションを実行
    pub async fn execute(&self) -> Result<(), ActionError>;
}
```

### Category

検索結果のカテゴリ。

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Category {
    /// システムアプリケーション
    Application,
    
    /// ファイル
    File,
    
    /// ウェブブックマーク
    Bookmark,
    
    /// プラグイン結果
    Plugin(String),
    
    /// システムコマンド
    SystemCommand,
    
    /// カスタムコマンド
    CustomCommand,
}
```

## Plugin API

### Plugin Trait

プラグインが実装すべきメインインターフェース。

```rust
#[async_trait::async_trait]
pub trait Plugin: Send + Sync + std::fmt::Debug {
    /// プラグイン名を返す
    fn name(&self) -> &str;
    
    /// プラグインバージョンを返す
    fn version(&self) -> &str;
    
    /// プラグインの説明を返す
    fn description(&self) -> &str;
    
    /// プラグインの初期化
    async fn initialize(&self) -> Result<(), PluginError> {
        Ok(())
    }
    
    /// プラグインの終了処理
    async fn shutdown(&self) -> Result<(), PluginError> {
        Ok(())
    }
    
    /// 検索クエリを処理可能かどうか判定
    fn can_handle(&self, query: &str) -> bool;
    
    /// 検索を実行
    async fn search(&self, query: &str) -> Result<Vec<SearchResult>, PluginError>;
    
    /// 検索結果の実行
    async fn execute(&self, result: &SearchResult) -> Result<(), PluginError>;
    
    /// 設定UIを提供するかどうか
    fn has_configuration(&self) -> bool {
        false
    }
    
    /// 設定UIのコンポーネントを返す
    fn get_configuration_ui(&self) -> Option<ConfigurationUI> {
        None
    }
}
```

### PluginError

プラグインで発生するエラーの種類。

```rust
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
```

### PluginContext

プラグインが利用可能なコンテキスト情報。

```rust
pub struct PluginContext {
    /// アプリケーション設定への読み取り専用アクセス
    config: Arc<RwLock<Config>>,
    
    /// ログ出力インターフェース
    logger: Arc<dyn Logger>,
    
    /// ファイルシステムアクセス
    fs: Arc<dyn FileSystem>,
    
    /// HTTP クライアント
    http_client: Arc<dyn HttpClient>,
}

impl PluginContext {
    /// 設定を読み取り
    pub fn get_config(&self) -> Result<Config, PluginError>;
    
    /// ログを出力
    pub fn log(&self, level: LogLevel, message: &str);
    
    /// ファイルを読み込み
    pub async fn read_file(&self, path: &Path) -> Result<Vec<u8>, PluginError>;
    
    /// HTTP GET リクエストを実行
    pub async fn http_get(&self, url: &str) -> Result<String, PluginError>;
    
    /// 通知を表示
    pub fn show_notification(&self, title: &str, message: &str) -> Result<(), PluginError>;
}
```

## 組み込みプラグイン例

### Calculator Plugin

```rust
#[derive(Debug)]
pub struct CalculatorPlugin {
    context: PluginContext,
}

#[async_trait::async_trait]
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
        // 数式パターンをチェック
        query.chars().any(|c| "+-*/()0123456789.".contains(c))
    }
    
    async fn search(&self, query: &str) -> Result<Vec<SearchResult>, PluginError> {
        match evaluate_expression(query) {
            Ok(result) => {
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
    
    async fn execute(&self, result: &SearchResult) -> Result<(), PluginError> {
        if let Action::CopyToClipboard(ref text) = result.action {
            copy_to_clipboard(text)?;
            self.context.show_notification("Calculator", "Result copied to clipboard")?;
        }
        Ok(())
    }
}

// プラグインのエクスポート関数
#[no_mangle]
pub fn create_plugin(context: PluginContext) -> Box<dyn Plugin> {
    Box::new(CalculatorPlugin { context })
}
```

## Configuration API

### Config

アプリケーション設定の管理。

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 外観設定
    pub appearance: AppearanceConfig,
    
    /// 動作設定  
    pub behavior: BehaviorConfig,
    
    /// 検索設定
    pub search: SearchConfig,
    
    /// プラグイン設定
    pub plugins: PluginConfig,
    
    /// 同期設定
    pub sync: SyncConfig,
}

impl Config {
    /// デフォルト設定を作成
    pub fn default() -> Self;
    
    /// 設定ファイルから読み込み
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError>;
    
    /// 設定ファイルに保存
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), ConfigError>;
    
    /// 設定を検証
    pub fn validate(&self) -> Result<(), ConfigError>;
}
```

### AppearanceConfig

外観に関する設定。

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceConfig {
    /// テーマ
    pub theme: Theme,
    
    /// ウィンドウの透明度 (0.0-1.0)
    pub transparency: f32,
    
    /// ウィンドウ位置
    pub position: WindowPosition,
    
    /// フォントサイズ
    pub font_size: u32,
    
    /// 最大結果表示数
    pub max_results: usize,
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
```

### BehaviorConfig

動作に関する設定。

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorConfig {
    /// グローバルホットキー
    pub hotkey: String,
    
    /// 自動非表示
    pub auto_hide: bool,
    
    /// 起動時にインデックス再構築
    pub rebuild_index_on_startup: bool,
    
    /// 検索履歴の保存
    pub save_search_history: bool,
    
    /// 使用統計の記録
    pub record_usage_stats: bool,
}
```

## Platform API

### PlatformProvider

プラットフォーム固有機能へのアクセス。

```rust
#[async_trait::async_trait]
pub trait PlatformProvider: Send + Sync {
    /// インストール済みアプリケーション一覧を取得
    async fn get_installed_applications(&self) -> Result<Vec<AppInfo>, PlatformError>;
    
    /// グローバルホットキーを登録
    fn register_global_hotkey(&self, hotkey: &str, callback: Box<dyn Fn() + Send>) -> Result<(), PlatformError>;
    
    /// グローバルホットキーを解除
    fn unregister_global_hotkey(&self, hotkey: &str) -> Result<(), PlatformError>;
    
    /// 通知を表示
    fn show_notification(&self, title: &str, message: &str) -> Result<(), PlatformError>;
    
    /// システムテーマを取得
    fn get_system_theme(&self) -> Theme;
    
    /// ファイル/アプリケーションを開く
    async fn open_with_default_app(&self, path: &Path) -> Result<(), PlatformError>;
    
    /// クリップボードにテキストをコピー
    fn copy_to_clipboard(&self, text: &str) -> Result<(), PlatformError>;
    
    /// クリップボードからテキストを取得
    fn paste_from_clipboard(&self) -> Result<String, PlatformError>;
}
```

### AppInfo

アプリケーション情報。

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct AppInfo {
    /// アプリケーション名
    pub name: String,
    
    /// 実行ファイルパス
    pub executable_path: PathBuf,
    
    /// アイコンパス
    pub icon_path: Option<PathBuf>,
    
    /// 説明
    pub description: Option<String>,
    
    /// キーワード
    pub keywords: Vec<String>,
    
    /// 使用回数
    pub usage_count: u32,
    
    /// 最終使用日時
    pub last_used: Option<SystemTime>,
}

impl AppInfo {
    /// 新しい AppInfo を作成
    pub fn new(name: impl Into<String>, executable_path: PathBuf) -> Self;
    
    /// 使用回数を増加
    pub fn increment_usage(&mut self);
    
    /// 検索結果に変換
    pub fn to_search_result(&self) -> SearchResult;
}
```

## エラーハンドリング

### 統一エラー型

```rust
#[derive(Debug, thiserror::Error)]
pub enum FalCommandError {
    #[error("検索エンジンエラー: {0}")]
    SearchEngine(#[from] SearchError),
    
    #[error("設定エラー: {0}")]
    Config(#[from] ConfigError),
    
    #[error("プラグインエラー: {0}")]
    Plugin(#[from] PluginError),
    
    #[error("プラットフォームエラー: {0}")]
    Platform(#[from] PlatformError),
    
    #[error("I/Oエラー: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("シリアライゼーションエラー: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, FalCommandError>;
```

## 使用例

### 基本的な検索実行

```rust
use falcommand::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // 設定を読み込み
    let config = Config::load_from_file("config.json")?;
    
    // 検索エンジンを初期化
    let search_engine = SearchEngine::new(config).await?;
    
    // 検索を実行
    let results = search_engine.search("notepad").await;
    
    // 結果を表示
    for result in results {
        println!("{}: {}", result.title, result.description);
    }
    
    Ok(())
}
```

### カスタムプラグインの作成

```rust
use falcommand::prelude::*;
use async_trait::async_trait;

#[derive(Debug)]
struct WeatherPlugin;

#[async_trait]
impl Plugin for WeatherPlugin {
    fn name(&self) -> &str { "Weather" }
    fn version(&self) -> &str { "1.0.0" }
    fn description(&self) -> &str { "Weather information plugin" }
    
    fn can_handle(&self, query: &str) -> bool {
        query.starts_with("weather ")
    }
    
    async fn search(&self, query: &str) -> Result<Vec<SearchResult>, PluginError> {
        let city = query.strip_prefix("weather ").unwrap_or("");
        
        // 天気情報を取得（実装は省略）
        let weather_info = fetch_weather(city).await?;
        
        let result = SearchResult::new(
            format!("Weather in {}", city),
            weather_info
        )
        .with_category(Category::Plugin("Weather".to_string()));
        
        Ok(vec![result])
    }
    
    async fn execute(&self, result: &SearchResult) -> Result<(), PluginError> {
        // 詳細な天気情報を表示
        Ok(())
    }
}
```

## ベストプラクティス

### プラグイン開発

1. **非同期処理**: 重い処理は `async/await` を使用
2. **エラーハンドリング**: 適切なエラー型を使用
3. **リソース管理**: ファイルハンドルやネットワーク接続の適切な管理
4. **テスト**: 単体テストを作成
5. **ドキュメント**: API使用例を含むドキュメントを作成

### パフォーマンス最適化

1. **キャッシュ**: 重い計算結果をキャッシュ
2. **並列処理**: 独立した処理は並列実行
3. **遅延読み込み**: 必要時まで初期化を遅延
4. **メモリ効率**: 大きなデータ構造の不要なコピーを避ける

## バージョニング

APIは [Semantic Versioning](https://semver.org/) に従います：

- **Major version**: 破壊的変更
- **Minor version**: 後方互換性のある機能追加
- **Patch version**: バグ修正

## サポート

APIに関する質問や問題：

1. [GitHub Issues](https://github.com/varubogu/falcommand/issues)でバグ報告
2. [GitHub Discussions](https://github.com/varubogu/falcommand/discussions)で質問・議論
3. 詳細なドキュメントは `cargo doc --open` で確認