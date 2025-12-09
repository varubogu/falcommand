# クロスプラットフォーム軽量コマンドランチャー

## 概要

このドキュメントは、クロスプラットフォーム対応で軽量な
GUIコマンドランチャーアプリケーション「falcommand」の開発ガイドラインを定義します。

## 技術要件

### プラットフォーム対応
- **対象OS**: Windows, macOS, Linux

## アーキテクチャ原則

### 1. 軽量性優先設計
- **最小依存**: 外部ライブラリは必要最小限に
- **遅延読み込み**: 機能の必要時読み込み
- **メモリ効率**: オブジェクトプールやキャッシュ戦略の活用
- **バンドルサイズ**: 実行ファイル10MB以下

### 2. モジュラー（クレート）設計

#### プロジェクト全体構造
```
falcommand/
├── src/                      # メインアプリケーション
│   ├── main.rs              # エントリーポイント
│   └── app.rs               # アプリケーションロジック
├── crates/                  # ワークスペースクレート
│   ├── falcommand-config/   # 設定管理
│   ├── falcommand-core/     # コア検索・インデックスエンジン
│   ├── falcommand-platform/ # プラットフォーム固有実装
│   ├── falcommand-plugins/  # プラグインシステム
│   └── falcommand-ui/       # ユーザーインターフェース
├── docs/                    # ドキュメント
│   ├── develop/             # 開発者向けドキュメント
│   └── user/                # ユーザー向けドキュメント
├── target/                  # ビルド成果物
├── Cargo.toml              # ワークスペース設定
└── Cargo.lock              # 依存関係ロック
```

#### クレート詳細構造

##### falcommand-config
設定管理を担当するクレート
```
crates/falcommand-config/
├── Cargo.toml
└── src/
    ├── lib.rs      # クレートエントリーポイント
    ├── config.rs   # 設定構造体とロジック
    └── types.rs    # 設定関連の型定義
```

##### falcommand-core  
コア検索・インデックスエンジンを担当するクレート
```
crates/falcommand-core/
├── Cargo.toml
└── src/
    ├── lib.rs      # クレートエントリーポイント
    ├── search.rs   # 検索エンジン実装
    ├── index.rs    # インデックス管理
    └── sync.rs     # データ同期機能
```

##### falcommand-platform
プラットフォーム固有実装を担当するクレート
```
crates/falcommand-platform/
├── Cargo.toml
└── src/
    ├── lib.rs       # クレートエントリーポイント
    └── platform.rs  # プラットフォーム固有機能
```

##### falcommand-plugins
プラグインシステムを担当するクレート
```
crates/falcommand-plugins/
├── Cargo.toml
└── src/
    ├── lib.rs      # クレートエントリーポイント
    └── plugins.rs  # プラグイン管理・実行
```

##### falcommand-ui
ユーザーインターフェースを担当するクレート
```
crates/falcommand-ui/
├── Cargo.toml
└── src/
    ├── lib.rs  # クレートエントリーポイント
    └── ui.rs   # UI コンポーネント・ロジック
```

### 3. プラグインアーキテクチャ
- **コアとプラグインの分離**: 基本機能とオプション機能を明確に区別
- **動的プラグインロード**: 必要な機能のみメモリに読み込み
- **API統一**: プラットフォーム間で共通のプラグインAPI

## 実装ガイドライン

### 技術スタック推奨

#### 採用フレームワーク: Slint + Rust
```toml
[dependencies]
slint = "1.0"
tokio = { version = "1.0", features = ["macros", "rt-multi-thread"] }
serde = { version = "1.0", features = ["derive"] }
fuzzy-matcher = "0.3"
```

### UI/UX設計原則

#### 1. ミニマルインターフェース
```
┌─────────────────────────────┐
│ [🔍] Search command...      │
├─────────────────────────────┤
│ > notepad                   │
│   📝 Notepad                │
├─────────────────────────────┤
│ > calc                      │
│   🧮 Calculator             │
└─────────────────────────────┘
```

#### 2. キーボード中心操作
- **起動**: Ctrl+Space（カスタマイズ可能）
- **ナビゲーション**: 矢印キー、Tab、Emacsキーバインド、vimモード
- **実行**: Enter
- **終了**: Escape

#### 3. 視覚的フィードバック
- **検索結果ハイライト**: 一致テキストの強調表示
- **アイコン表示**: アプリケーション/ファイル種別アイコン
- **プレビュー**: 選択項目の詳細情報

### 検索エンジン仕様

#### 1. 高速インデックス
```rust
pub struct SearchIndex {
    applications: HashMap<String, AppInfo>,
    files: BTreeMap<String, FileInfo>,
    commands: Vec<Command>,
}

impl SearchIndex {
    pub fn search(&self, query: &str) -> Vec<SearchResult> {
        // Fuzzy search implementation
        // Priority: exact match > prefix match > fuzzy match
    }
}
```

#### 2. 検索対象
- **システムアプリケーション**: インストール済みプログラム
- **ファイル**: 指定フォルダ内のファイル（ホワイトリスト、ブラックリスト対応）
- **WEBブックマーク**: ブラウザブックマーク（Chromium系、Safari、Firefox系）
- **カスタムコマンド**: ユーザー定義ショートカット

#### 3. 検索アルゴリズム
- **優先順位**: 使用頻度 > 完全一致 > 前方一致 > あいまい一致
- **学習機能**: 使用履歴による結果の最適化
- **キャッシュ戦略**: 検索結果の一時保存

### 設定管理

設定ファイルは次のレベルで用意する。
- ユーザー設定（共通的な設定で同期される）
- ローカル設定（プラットフォーム固有設定で動悸されない）

また、それぞれのレベルでプラットフォーム固有設定を書くこともできる。
ファイル名規則は以下の通り

- 通常のユーザー設定: config.json
- Windowsのユーザー設定: config.windows.json
- MacOsのユーザー設定: config.macos.json
- Linuxのユーザー設定: config.linux.json
- Ubuntuのユーザー設定: config.ubuntu.json
- 通常のローカル設定: config.local.json
- Windowsのローカル設定: config.windows.local.json
- MacOsのローカル設定: config.macos.local.json
- Linuxのローカル設定: config.linux.local.json
- Ubuntuのユーザー設定: config.ubuntu.local.json

同じキーが複数レベルで見つかった場合、優先度は

1. プラットフォームのローカル設定
2. 通常のローカル設定
3. プラットフォームのユーザー設定
4. 通常のユーザー設定
5. デフォルト値
 
となる

#### 設定ファイル構造
```json
{
  "appearance": {
    "theme": "dark|light|system",
    "transparency": 0.95,
    "position": "center|cursor",
    "showWindow": "mouse|display",
    "showWindowDisplayNumber": "[0-9]+"
  },
  "behavior": {
    "hotkey": "Ctrl+Space",
    "autoHide": true,
    "maxResults": 10
  },
  "search": {
    "includePaths": {
      "windows": [
        "C:\\Program Files",
        "C:\\Program Files (x86)"
      ],
      "macos": [
        "~/Applications"
      ]
    },
    "excludePatterns": ["*.tmp", "*.log"],
    "fuzzyThreshold": 0.6
  },
  "plugins": {
    "enabled": ["calculator", "translator"],
    "disabled": ["weather"]
  }
}
```

### プラットフォーム固有実装

#### Windows
```rust
// Windows特有の実装
impl PlatformImpl for WindowsImpl {
    fn get_installed_apps(&self) -> Vec<AppInfo> {
        // レジストリから取得
        // Start Menu解析
    }
    
    fn register_hotkey(&self, hotkey: &str) -> Result<()> {
        // Windows API使用
    }
}
```

#### macOS
```rust
impl PlatformImpl for MacOSImpl {
    fn get_installed_apps(&self) -> Vec<AppInfo> {
        // /Applications解析
        // Spotlight メタデータ活用
    }
}
```

#### Linux
```rust
impl PlatformImpl for LinuxImpl {
    fn get_installed_apps(&self) -> Vec<AppInfo> {
        // .desktop ファイル解析
        // PATH環境変数からコマンド取得
    }
}
```

## 開発ワークフロー

### 1. テスト戦略
- **単体テスト**: コア機能の動作確認
- **統合テスト**: プラットフォーム間の互換性
- **パフォーマンステスト**: 起動時間、メモリ使用量
- **UI テスト**: キーボードナビゲーション

## パフォーマンス最適化

### 1. 起動時間最適化
```rust
// 遅延初期化パターン
lazy_static! {
    static ref SEARCH_INDEX: Mutex<SearchIndex> = Mutex::new(SearchIndex::new());
}

// バックグラウンドでインデックス構築
tokio::spawn(async {
    let mut index = SEARCH_INDEX.lock().unwrap();
    index.build_async().await;
});
```

### 2. メモリ使用量最適化
- **オブジェクトプール**: 頻繁に作成/破棄されるオブジェクトの再利用
- **弱参照**: 循環参照の回避
- **データ圧縮**: アイコン画像の効率的な格納

### 3. 応答性最適化
```rust
// 非同期検索実装
pub async fn search_async(&self, query: String) -> Vec<SearchResult> {
    let results = tokio::task::spawn_blocking(move || {
        // CPU集約的な検索処理
        perform_search(&query)
    }).await.unwrap();
    
    results
}
```

## セキュリティ考慮事項

### 1. 実行権限
- **最小権限の原則**: 必要最小限の権限で動作
- **サンドボックス**: 可能な限りサンドボックス内で実行
- **署名検証**: 実行ファイルのデジタル署名確認

### 2. データ保護
- **設定暗号化**: 機密情報を含む設定の暗号化保存
- **一時ファイル**: セキュアな一時ファイル作成
- **メモリクリア**: 機密データのメモリからの確実な消去

## 配布・更新戦略

### 1. パッケージング
- **インストーラー**: プラットフォーム固有のインストーラー作成
- **ポータブル版**: インストール不要のスタンドアロン実行ファイル
- **パッケージマネージャー**: winget, Chocolatey, Homebrew, APT対応

### 2. 自動更新
```rust
pub struct UpdateChecker {
    current_version: Version,
    update_url: String,
}

impl UpdateChecker {
    pub async fn check_updates(&self) -> Result<Option<UpdateInfo>> {
        // GitHub Releases APIチェック
        // バックグラウンドでダウンロード
    }
}
```

## プラグインシステム

### 1. プラグインAPI
```rust
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn search(&self, query: &str) -> Vec<SearchResult>;
    fn execute(&self, result: &SearchResult) -> Result<()>;
}

// 計算機プラグイン例
pub struct CalculatorPlugin;

impl Plugin for CalculatorPlugin {
    fn search(&self, query: &str) -> Vec<SearchResult> {
        if let Ok(result) = eval_math(query) {
            vec![SearchResult {
                title: format!("{} = {}", query, result),
                action: Action::CopyToClipboard(result.to_string()),
                icon: Icon::Calculator,
            }]
        } else {
            vec![]
        }
    }
}
```

### 2. 組み込みプラグイン
- **電卓**: 数式の計算
- **単位変換**: 長さ、重量、温度などの変換
- **翻訳**: 簡単な翻訳機能
- **天気**: 天気予報の表示
- **時計**: 世界時計、タイマー

## 品質保証

### 1. コード品質
- **静的解析**: Clippy, ESLint等の使用
- **コードカバレッジ**: 80%以上の目標
- **文書化**: 全パブリック関数のドキュメント

### 2. ユーザビリティ
- **アクセシビリティ**: キーボードのみでの完全操作
- **国際化**: 主要言語対応（日本語、英語、中国語など）
- **カスタマイズ性**: テーマ、ホットキー等の柔軟な設定

## 詳細アーキテクチャ仕様

### システム全体図
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

## ドキュメント修正の指針

### ドキュメント更新ルール

ドキュメントを修正する場合は、以下のルールに従ってください：

1. **両方への反映が必須**
   - `guideline.md` の修正
   - `docs/` 配下の対応するドキュメントファイルの修正

2. **修正の優先順位**
   - まず `docs/` 配下の専門ドキュメントを修正
   - その後、`guideline.md` に同じ内容を反映
   - 両方のドキュメントの内容を一致させる

3**確認事項**
   - 技術仕様の整合性チェック
   - コード例とAPI定義の一致確認
   - バージョン情報の統一
   - リンクとクロスリファレンスの正確性

4**修正プロセス**
   ```
   1. 変更が必要な内容を特定
   2. docs/ 配下の該当ファイルを修正
   3. guideline.md の対応セクションを更新
   4. 両方のドキュメントで内容の一致を確認
   5. コミット時に両方のファイルを含める
   ```

### 注意事項

- **単独修正の禁止**: `guideline.md` のみ、または `docs/` のファイルのみを修正することは避ける
- **内容の重複管理**: 同じ技術仕様が複数箇所に記載されることを前提とした管理
- **定期的な整合性確認**: 大きな機能追加時は全ドキュメントの整合性を再確認

## エラーハンドリング戦略

### 統一エラー型
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FalCommandError {
    #[error("検索インデックスエラー: {0}")]
    SearchIndexError(String),
    
    #[error("設定エラー: {0}")]
    ConfigurationError(String),
    
    #[error("プラットフォームエラー: {0}")]
    PlatformError(String),
    
    #[error("プラグインエラー: {0}")]
    PluginError(String),
    
    #[error("同期エラー: {0}")]
    SyncError(String),
}
```

### ログ出力システム
```rust
use tracing::{info, warn, error, debug};

pub struct Logger {
    level: LogLevel,
    output: LogOutput,
}

impl Logger {
    pub fn init() -> Result<(), FalCommandError> {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
        Ok(())
    }
}
```

## 国際化・多言語対応

### 翻訳リソース管理
```rust
// src/i18n/mod.rs
use fluent::{FluentBundle, FluentResource};
use std::collections::HashMap;

pub struct I18nManager {
    bundles: HashMap<String, FluentBundle<FluentResource>>,
    current_locale: String,
}

impl I18nManager {
    pub fn new() -> Self {
        let mut manager = Self {
            bundles: HashMap::new(),
            current_locale: "en-US".to_string(),
        };
        manager.load_locales();
        manager
    }
    
    pub fn t(&self, key: &str) -> String {
        // 翻訳の実装
    }
}
```

### 多言語設定ファイル
```json
{
  "localization": {
    "defaultLocale": "en-US",
    "supportedLocales": ["en-US", "ja-JP", "zh-CN", "ko-KR"],
    "fallbackLocale": "en-US",
    "rtlSupport": true
  }
}
```

## アクセシビリティ対応

### スクリーンリーダー対応

設定ファイルでON/OFFできるようにする

### キーボードショートカット設定

設定ファイルで設定できる（ローカル設定が望ましい）

## ローカルファーストデータ同期

### クラウドストレージ同期設計

任意のクラウドストレージプロバイダを指定可能。有名どころは対応する。

- Google Drive
- Dropbox
- OneDrive
- iCloud Drive
- Box
- pCloud
- Custom

### 同期設定
```json
{
  "sync": {
    "enabled": false,
    "provider": "googledrive|dropbox|onedrive|custom",
    "syncInterval": 300,
    "conflictResolution": "local|remote|prompt",
    "encryptData": true,
    "dataToSync": {
      "settings": true,
      "searchHistory": false,
      "customCommands": true,
      "plugins": true
    }
  }
}
```

## CI/CDパイプライン

GitHub Actionのワークフローファイルを参照

## ドキュメンテーション

### API リファレンス生成
```rust
//! # FalCommand API リファレンス
//! 
//! このクレートは軽量なクロスプラットフォームコマンドランチャーを提供します。
//! 
//! ## 基本的な使用方法
//! 
//! ```rust
//! use falcommand::{SearchEngine, Config};
//! 
//! let config = Config::load()?;
//! let mut engine = SearchEngine::new(config);
//! let results = engine.search("notepad").await?;
//! ```

/// 検索エンジンのメイン実装
/// 
/// # Examples
/// 
/// ```rust
/// let engine = SearchEngine::new(config);
/// let results = engine.search("calculator").await?;
/// ```
pub struct SearchEngine {
    // フィールドの実装
}
```

## ライセンス・法的事項

アプリ自体: MIT
第三者プラグイン: 任意


## 災害復旧計画

### 設定バックアップ・復元

起動時にバックアップを保持する（10回分まで）
ただし起動時にエラーが起きた場合は新しいバックアップは作成しない（古いバックアップは削除されない）

## 最終リリース計画の更新

### フェーズ1: MVP（最小実行可能製品）
- [x] 基本的な検索・実行機能
- [x] シンプルなUI
- [x] Windows/macOS/Linux対応

### フェーズ2: 機能拡張
- [ ] プラグインシステム
- [ ] 設定画面
- [ ] 使用統計・学習機能
- [ ] 国際化・多言語対応
- [ ] アクセシビリティ対応

### フェーズ3: 最適化・エンタープライズ対応
- [ ] パフォーマンス向上
- [ ] UI/UX改善
- [ ] 高度な検索機能
- [ ] ローカルファーストデータ同期
- [ ] 監視・テレメトリシステム
- [ ] デバッグ・診断機能
- [ ] 災害復旧機能
