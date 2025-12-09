# プラグインシステム

FalCommand のプラグインシステムの設計と実装について説明します。

## 概要

FalCommand のプラグインシステムは、アプリケーションの機能を動的に拡張するためのフレームワークです。プラグインを使用することで、サードパーティ開発者や上級ユーザーが独自の機能を追加できます。

### 特徴
- **動的ロード**: 実行時にプラグインを読み込み
- **安全性**: サンドボックス環境での実行
- **パフォーマンス**: 軽量で高速な実行
- **API統一**: 標準化されたプラグインAPI
- **設定管理**: 個別の設定とカスタマイズ

## アーキテクチャ

### プラグインライフサイクル

```
1. 発見 (Discovery)
   ↓
2. 読み込み (Loading)
   ↓
3. 初期化 (Initialization)
   ↓
4. 実行 (Execution)
   ↓
5. 終了 (Shutdown)
```

### プラグインマネージャー

```rust
pub struct PluginManager {
    plugins: RwLock<HashMap<String, Arc<dyn Plugin>>>,
    plugin_configs: RwLock<HashMap<String, PluginConfig>>,
    runtime: Arc<PluginRuntime>,
    registry: Arc<PluginRegistry>,
}

impl PluginManager {
    pub async fn discover_plugins(&self) -> Result<Vec<PluginInfo>, PluginError> {
        // プラグインディレクトリをスキャン
        let mut plugins = Vec::new();
        
        for entry in fs::read_dir(&self.plugin_dir)? {
            if let Ok(plugin_info) = self.load_plugin_info(&entry.path()).await {
                plugins.push(plugin_info);
            }
        }
        
        Ok(plugins)
    }
    
    pub async fn load_plugin(&self, path: &Path) -> Result<Arc<dyn Plugin>, PluginError> {
        // プラグインバイナリを読み込み
        let library = unsafe { libloading::Library::new(path)? };
        
        // エントリーポイント関数を取得
        let create_plugin: Symbol<fn() -> Box<dyn Plugin>> = 
            unsafe { library.get(b"create_plugin")? };
        
        // プラグインインスタンスを作成
        let plugin = create_plugin();
        
        // 初期化
        plugin.initialize().await?;
        
        Ok(Arc::from(plugin))
    }
    
    pub async fn search_all(&self, query: &str) -> Vec<SearchResult> {
        let plugins = self.plugins.read().unwrap();
        let futures: Vec<_> = plugins
            .values()
            .filter(|p| p.can_handle(query))
            .map(|p| p.search(query))
            .collect();
        
        let results = join_all(futures).await;
        results.into_iter()
            .filter_map(|r| r.ok())
            .flatten()
            .collect()
    }
}
```

## プラグインAPI

### Plugin Trait

すべてのプラグインが実装すべき基本インターフェース。

```rust
#[async_trait::async_trait]
pub trait Plugin: Send + Sync + std::fmt::Debug {
    /// プラグインのメタデータ
    fn metadata(&self) -> &PluginMetadata;
    
    /// プラグインの初期化
    async fn initialize(&self) -> Result<(), PluginError>;
    
    /// プラグインの終了処理
    async fn shutdown(&self) -> Result<(), PluginError>;
    
    /// クエリを処理可能かどうかの判定
    fn can_handle(&self, query: &str) -> bool;
    
    /// 検索の実行
    async fn search(&self, query: &str) -> Result<Vec<SearchResult>, PluginError>;
    
    /// アクションの実行
    async fn execute(&self, result: &SearchResult) -> Result<(), PluginError>;
    
    /// プラグイン固有の設定
    fn get_settings_schema(&self) -> Option<serde_json::Value> {
        None
    }
    
    /// 設定の更新
    async fn update_settings(&self, settings: serde_json::Value) -> Result<(), PluginError> {
        Ok(())
    }
}
```

### PluginMetadata

プラグインのメタ情報。

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// プラグインID（一意識別子）
    pub id: String,
    
    /// 表示名
    pub name: String,
    
    /// バージョン
    pub version: semver::Version,
    
    /// 説明
    pub description: String,
    
    /// 作成者
    pub author: String,
    
    /// ライセンス
    pub license: String,
    
    /// ホームページURL
    pub homepage: Option<String>,
    
    /// 依存関係
    pub dependencies: Vec<PluginDependency>,
    
    /// 権限要求
    pub permissions: Vec<Permission>,
    
    /// 対象プラットフォーム
    pub platforms: Vec<Platform>,
    
    /// 最小要求バージョン
    pub min_falcommand_version: semver::Version,
}
```

### PluginContext

プラグインが利用可能なコンテキスト。

```rust
#[derive(Clone)]
pub struct PluginContext {
    config: Arc<RwLock<Config>>,
    logger: Arc<dyn Logger>,
    file_system: Arc<dyn FileSystem>,
    http_client: Arc<dyn HttpClient>,
    clipboard: Arc<dyn ClipboardProvider>,
    notification: Arc<dyn NotificationProvider>,
}

impl PluginContext {
    /// アプリケーション設定の読み取り
    pub fn get_app_config(&self) -> Result<Config, PluginError> {
        Ok(self.config.read().unwrap().clone())
    }
    
    /// ログ出力
    pub fn log(&self, level: LogLevel, message: &str) {
        self.logger.log(level, &format!("[Plugin] {}", message));
    }
    
    /// ファイルの読み書き（サンドボックス内）
    pub async fn read_file(&self, path: &Path) -> Result<Vec<u8>, PluginError> {
        if !self.is_path_allowed(path) {
            return Err(PluginError::PermissionDenied);
        }
        self.file_system.read(path).await
    }
    
    /// HTTP リクエスト
    pub async fn http_request(&self, request: HttpRequest) -> Result<HttpResponse, PluginError> {
        self.http_client.execute(request).await
    }
    
    /// クリップボード操作
    pub fn set_clipboard(&self, text: &str) -> Result<(), PluginError> {
        self.clipboard.set_text(text)
    }
    
    /// 通知表示
    pub fn show_notification(&self, title: &str, message: &str) -> Result<(), PluginError> {
        self.notification.show(title, message)
    }
}
```

## 組み込みプラグイン

### Calculator Plugin

数式計算プラグイン。

```rust
#[derive(Debug)]
pub struct CalculatorPlugin {
    metadata: PluginMetadata,
    evaluator: MathEvaluator,
}

impl CalculatorPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: "org.falcommand.calculator".to_string(),
                name: "Calculator".to_string(),
                version: semver::Version::new(1, 0, 0),
                description: "Mathematical expression evaluator".to_string(),
                author: "FalCommand Team".to_string(),
                license: "MIT".to_string(),
                homepage: None,
                dependencies: vec![],
                permissions: vec![Permission::Clipboard],
                platforms: vec![Platform::All],
                min_falcommand_version: semver::Version::new(0, 1, 0),
            },
            evaluator: MathEvaluator::new(),
        }
    }
}

#[async_trait::async_trait]
impl Plugin for CalculatorPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    async fn initialize(&self) -> Result<(), PluginError> {
        // 数学ライブラリの初期化
        Ok(())
    }
    
    async fn shutdown(&self) -> Result<(), PluginError> {
        Ok(())
    }
    
    fn can_handle(&self, query: &str) -> bool {
        // 数式パターンの検証
        self.evaluator.is_valid_expression(query)
    }
    
    async fn search(&self, query: &str) -> Result<Vec<SearchResult>, PluginError> {
        match self.evaluator.evaluate(query) {
            Ok(result) => {
                let search_result = SearchResult {
                    title: format!("{} = {}", query, result),
                    description: "Mathematical calculation".to_string(),
                    action: Action::CopyToClipboard(result.to_string()),
                    icon: Some("calculator.png".into()),
                    score: 0.95,
                    category: Category::Plugin("Calculator".to_string()),
                };
                
                Ok(vec![search_result])
            }
            Err(e) => {
                // 無効な式の場合は空の結果を返す
                Ok(vec![])
            }
        }
    }
    
    async fn execute(&self, result: &SearchResult) -> Result<(), PluginError> {
        match &result.action {
            Action::CopyToClipboard(text) => {
                // クリップボードにコピー
                copy_to_clipboard(text)?;
                Ok(())
            }
            _ => Err(PluginError::UnsupportedAction),
        }
    }
}
```

### Unit Converter Plugin

単位変換プラグイン。

```rust
#[derive(Debug)]
pub struct UnitConverterPlugin {
    metadata: PluginMetadata,
    conversions: HashMap<String, ConversionSet>,
}

impl UnitConverterPlugin {
    pub fn new() -> Self {
        let mut conversions = HashMap::new();
        
        // 長さの単位変換
        conversions.insert("length".to_string(), ConversionSet {
            base_unit: "meter".to_string(),
            conversions: vec![
                ("mm", 0.001),
                ("cm", 0.01),
                ("m", 1.0),
                ("km", 1000.0),
                ("inch", 0.0254),
                ("ft", 0.3048),
                ("yard", 0.9144),
                ("mile", 1609.344),
            ].into_iter().map(|(u, f)| (u.to_string(), f)).collect(),
        });
        
        // 重量の単位変換
        conversions.insert("weight".to_string(), ConversionSet {
            base_unit: "kilogram".to_string(),
            conversions: vec![
                ("mg", 0.000001),
                ("g", 0.001),
                ("kg", 1.0),
                ("lb", 0.453592),
                ("oz", 0.0283495),
            ].into_iter().map(|(u, f)| (u.to_string(), f)).collect(),
        });
        
        Self {
            metadata: PluginMetadata {
                id: "org.falcommand.unit_converter".to_string(),
                name: "Unit Converter".to_string(),
                version: semver::Version::new(1, 0, 0),
                description: "Convert between different units of measurement".to_string(),
                author: "FalCommand Team".to_string(),
                license: "MIT".to_string(),
                homepage: None,
                dependencies: vec![],
                permissions: vec![Permission::Clipboard],
                platforms: vec![Platform::All],
                min_falcommand_version: semver::Version::new(0, 1, 0),
            },
            conversions,
        }
    }
}

#[async_trait::async_trait]
impl Plugin for UnitConverterPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    async fn initialize(&self) -> Result<(), PluginError> {
        Ok(())
    }
    
    async fn shutdown(&self) -> Result<(), PluginError> {
        Ok(())
    }
    
    fn can_handle(&self, query: &str) -> bool {
        // "5 km to mile" のようなパターンをチェック
        lazy_static! {
            static ref CONVERT_REGEX: Regex = Regex::new(
                r"(?i)^(\d+(?:\.\d+)?)\s*(\w+)\s+to\s+(\w+)$"
            ).unwrap();
        }
        
        CONVERT_REGEX.is_match(query)
    }
    
    async fn search(&self, query: &str) -> Result<Vec<SearchResult>, PluginError> {
        if let Some(captures) = CONVERT_REGEX.captures(query) {
            let value: f64 = captures[1].parse().map_err(|_| PluginError::ParseError)?;
            let from_unit = &captures[2];
            let to_unit = &captures[3];
            
            if let Some(result) = self.convert(value, from_unit, to_unit) {
                let search_result = SearchResult {
                    title: format!("{} {} = {} {}", value, from_unit, result, to_unit),
                    description: "Unit conversion".to_string(),
                    action: Action::CopyToClipboard(result.to_string()),
                    icon: Some("converter.png".into()),
                    score: 0.9,
                    category: Category::Plugin("Unit Converter".to_string()),
                };
                
                return Ok(vec![search_result]);
            }
        }
        
        Ok(vec![])
    }
    
    async fn execute(&self, result: &SearchResult) -> Result<(), PluginError> {
        match &result.action {
            Action::CopyToClipboard(text) => {
                copy_to_clipboard(text)?;
                Ok(())
            }
            _ => Err(PluginError::UnsupportedAction),
        }
    }
}
```

### Weather Plugin

天気予報プラグイン。

```rust
#[derive(Debug)]
pub struct WeatherPlugin {
    metadata: PluginMetadata,
    api_client: WeatherApiClient,
    cache: Arc<Mutex<LruCache<String, WeatherData>>>,
}

impl WeatherPlugin {
    pub fn new(api_key: String) -> Self {
        Self {
            metadata: PluginMetadata {
                id: "org.falcommand.weather".to_string(),
                name: "Weather".to_string(),
                version: semver::Version::new(1, 0, 0),
                description: "Current weather and forecast information".to_string(),
                author: "FalCommand Team".to_string(),
                license: "MIT".to_string(),
                homepage: None,
                dependencies: vec![],
                permissions: vec![Permission::Network, Permission::Location],
                platforms: vec![Platform::All],
                min_falcommand_version: semver::Version::new(0, 1, 0),
            },
            api_client: WeatherApiClient::new(api_key),
            cache: Arc::new(Mutex::new(LruCache::new(100))),
        }
    }
}

#[async_trait::async_trait]
impl Plugin for WeatherPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    async fn initialize(&self) -> Result<(), PluginError> {
        // API接続テスト
        self.api_client.test_connection().await?;
        Ok(())
    }
    
    async fn shutdown(&self) -> Result<(), PluginError> {
        Ok(())
    }
    
    fn can_handle(&self, query: &str) -> bool {
        query.starts_with("weather ") || query == "weather"
    }
    
    async fn search(&self, query: &str) -> Result<Vec<SearchResult>, PluginError> {
        let location = if query == "weather" {
            "current_location".to_string()
        } else {
            query.strip_prefix("weather ").unwrap_or("").to_string()
        };
        
        // キャッシュをチェック
        {
            let mut cache = self.cache.lock().unwrap();
            if let Some(cached_data) = cache.get(&location) {
                if cached_data.is_fresh(Duration::from_secs(600)) { // 10分間有効
                    return Ok(vec![cached_data.to_search_result()]);
                }
            }
        }
        
        // APIから天気データを取得
        let weather_data = self.api_client.get_weather(&location).await?;
        
        // キャッシュに保存
        {
            let mut cache = self.cache.lock().unwrap();
            cache.put(location, weather_data.clone());
        }
        
        Ok(vec![weather_data.to_search_result()])
    }
    
    async fn execute(&self, result: &SearchResult) -> Result<(), PluginError> {
        // 詳細な天気情報を表示
        match &result.action {
            Action::ShowDetails(data) => {
                // 詳細ウィンドウを表示
                show_weather_details(data)?;
                Ok(())
            }
            _ => Err(PluginError::UnsupportedAction),
        }
    }
    
    fn get_settings_schema(&self) -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "type": "object",
            "properties": {
                "api_key": {
                    "type": "string",
                    "title": "Weather API Key",
                    "description": "API key for weather service"
                },
                "default_location": {
                    "type": "string",
                    "title": "Default Location",
                    "description": "Default location for weather queries"
                },
                "units": {
                    "type": "string",
                    "enum": ["metric", "imperial"],
                    "title": "Units",
                    "description": "Temperature and speed units"
                }
            }
        }))
    }
}
```

## プラグインの配布とインストール

### プラグインパッケージ形式

プラグインは `.fcp` (FalCommand Plugin) ファイルとして配布されます。

```
plugin.fcp
├── plugin.toml          # メタデータ
├── plugin.dll/.so/.dylib # バイナリ
├── assets/              # リソース
│   ├── icon.png
│   └── config-schema.json
└── README.md            # ドキュメント
```

### plugin.toml

```toml
[package]
id = "org.falcommand.calculator"
name = "Calculator"
version = "1.0.0"
description = "Mathematical expression evaluator"
author = "FalCommand Team"
license = "MIT"
homepage = "https://github.com/falcommand/plugins/calculator"

[dependencies]
falcommand = "0.1.0"

[permissions]
clipboard = true
network = false
filesystem = false

[targets]
windows = "plugin.dll"
macos = "libplugin.dylib"
linux = "libplugin.so"

[assets]
icon = "assets/icon.png"
config_schema = "assets/config-schema.json"
```

### インストール

```bash
# プラグインのインストール
falcommand plugin install calculator.fcp

# プラグインの一覧表示
falcommand plugin list

# プラグインの有効/無効化
falcommand plugin enable calculator
falcommand plugin disable calculator

# プラグインのアンインストール
falcommand plugin uninstall calculator
```

## セキュリティとサンドボックス

### 権限システム

プラグインは明示的に権限を要求する必要があります：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Permission {
    /// ファイルシステムアクセス
    FileSystem { paths: Vec<PathBuf> },
    
    /// ネットワークアクセス
    Network { hosts: Vec<String> },
    
    /// クリップボードアクセス
    Clipboard,
    
    /// 通知表示
    Notifications,
    
    /// システムコマンド実行
    SystemCommands,
    
    /// 位置情報アクセス
    Location,
}
```

### サンドボックス実行

プラグインは制限された環境で実行されます：

- ファイルシステムアクセスの制限
- ネットワークアクセスの制限
- システムリソースの制限
- メモリ使用量の制限

## デバッグとテスト

### プラグインのデバッグ

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;
    
    #[tokio::test]
    async fn test_calculator_plugin() {
        let plugin = CalculatorPlugin::new();
        
        // 初期化テスト
        assert!(plugin.initialize().await.is_ok());
        
        // 検索テスト
        let results = plugin.search("2 + 2").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "2 + 2 = 4");
        
        // 実行テスト
        assert!(plugin.execute(&results[0]).await.is_ok());
        
        // 終了テスト
        assert!(plugin.shutdown().await.is_ok());
    }
}
```

### プラグイン開発ツール

```bash
# プラグインプロジェクトの作成
falcommand plugin new my_plugin --template=basic

# プラグインのビルド
falcommand plugin build

# プラグインのテスト
falcommand plugin test

# プラグインのパッケージ化
falcommand plugin package
```

## ベストプラクティス

### パフォーマンス
1. **キャッシュ活用**: 重い計算結果はキャッシュする
2. **非同期処理**: ブロッキング操作は避ける
3. **リソース管理**: メモリリークを防ぐ
4. **遅延初期化**: 必要時まで初期化を延期

### セキュリティ
1. **最小権限**: 必要最小限の権限のみ要求
2. **入力検証**: すべての入力を検証
3. **エラーハンドリング**: 適切なエラー処理
4. **ログ出力**: セキュリティ関連のイベントをログ

### ユーザビリティ
1. **直感的なクエリ**: 自然な検索クエリサポート
2. **適切なアイコン**: 分かりやすいアイコンを提供
3. **設定の提供**: ユーザーがカスタマイズ可能
4. **ヘルプ情報**: 使用方法の説明を含める

## まとめ

FalCommand のプラグインシステムは、安全で高性能な機能拡張を可能にします。標準化されたAPIと強力なサンドボックス機能により、サードパーティ開発者が安心してプラグインを作成できる環境を提供します。