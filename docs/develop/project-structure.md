# プロジェクト構造

FalCommand プロジェクトのクレート構造とフォルダ構造について詳細に説明します。

## プロジェクト全体構造

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
├── .junie/                  # Junie AI 関連ファイル
│   └── guideline.md         # 開発ガイドライン
├── Cargo.toml              # ワークスペース設定
└── Cargo.lock              # 依存関係ロック
```

## ワークスペース設定

プロジェクトは Rust のワークスペース機能を使用して管理されており、`Cargo.toml` で以下のように定義されています：

```toml
[workspace]
members = [
    "crates/falcommand-config",
    "crates/falcommand-platform", 
    "crates/falcommand-plugins",
    "crates/falcommand-core",
    "crates/falcommand-ui",
]
```

## クレート詳細構造

### 1. falcommand-config

**目的**: 設定管理を担当するクレート

**構造**:
```
crates/falcommand-config/
├── Cargo.toml              # クレート設定
└── src/
    ├── lib.rs              # クレートエントリーポイント
    ├── config.rs           # 設定構造体とロジック
    └── types.rs            # 設定関連の型定義
```

**主な機能**:
- アプリケーション設定の読み込み・保存
- プラットフォーム固有設定の管理
- デバッグ設定のマージ
- 設定値のバリデーション

### 2. falcommand-core  

**目的**: コア検索・インデックスエンジンを担当するクレート

**構造**:
```
crates/falcommand-core/
├── Cargo.toml              # クレート設定
└── src/
    ├── lib.rs              # クレートエントリーポイント
    ├── search.rs           # 検索エンジン実装
    ├── index.rs            # インデックス管理
    └── sync.rs             # データ同期機能
```

**主な機能**:
- アプリケーション・ファイル検索
- インデックスの構築・更新
- あいまい検索アルゴリズム
- データ同期管理

### 3. falcommand-platform

**目的**: プラットフォーム固有実装を担当するクレート

**構造**:
```
crates/falcommand-platform/
├── Cargo.toml              # クレート設定
└── src/
    ├── lib.rs              # クレートエントリーポイント
    └── platform.rs         # プラットフォーム固有機能
```

**主な機能**:
- Windows/macOS/Linux 固有の実装
- システムトレイ機能
- ホットキー管理
- OS 固有のアプリケーション検索

### 4. falcommand-plugins

**目的**: プラグインシステムを担当するクレート

**構造**:
```
crates/falcommand-plugins/
├── Cargo.toml              # クレート設定
└── src/
    ├── lib.rs              # クレートエントリーポイント
    └── plugins.rs          # プラグイン管理・実行
```

**主な機能**:
- プラグインの動的ロード
- プラグイン API の提供
- プラグインの実行・管理
- 拡張機能の統合

### 5. falcommand-ui

**目的**: ユーザーインターフェースを担当するクレート

**構造**:
```
crates/falcommand-ui/
├── Cargo.toml              # クレート設定
└── src/
    ├── lib.rs              # クレートエントリーポイント
    └── ui.rs               # UI コンポーネント・ロジック
```

**主な機能**:
- メインウィンドウの管理
- 検索結果の表示
- ユーザー操作の処理
- テーマ・外観の管理

## メインアプリケーション

**構造**:
```
src/
├── main.rs                 # アプリケーションエントリーポイント
└── app.rs                  # メインアプリケーションロジック
```

**役割**:
- 各クレートの統合
- アプリケーションライフサイクルの管理
- 初期化・終了処理

## ドキュメント構造

```
docs/
├── develop/                # 開発者向けドキュメント
│   ├── README.md           # 開発ドキュメントの概要
│   ├── architecture.md     # アーキテクチャ設計書
│   ├── api-reference.md    # API リファレンス
│   ├── build-deploy.md     # ビルド・デプロイガイド
│   ├── config.md           # 設定仕様書
│   ├── config-priority-design.md # 設定優先順位設計
│   ├── plugin-system.md    # プラグインシステム仕様
│   ├── project-structure.md # プロジェクト構造（本ドキュメント）
│   └── setup.md            # セットアップガイド
└── user/                   # ユーザー向けドキュメント
    └── (ユーザーマニュアル等)
```

## 依存関係

### 外部依存関係
- **tokio**: 非同期ランタイム
- **serde**: シリアライゼーション
- **log**: ロギング
- **anyhow/thiserror**: エラーハンドリング

### 内部依存関係
```
falcommand (メイン)
├── falcommand-config
├── falcommand-platform
├── falcommand-plugins
├── falcommand-core
└── falcommand-ui
```

各クレートは独立性を保ちながら、必要に応じて他のクレートを依存関係として使用します。

## ビルド成果物

```
target/
├── debug/                  # デバッグビルド
│   ├── deps/              # 依存関係の中間ファイル
│   ├── examples/          # サンプル実行ファイル
│   └── incremental/       # インクリメンタルコンパイルキャッシュ
└── release/               # リリースビルド
    ├── deps/              # 依存関係の中間ファイル
    ├── examples/          # サンプル実行ファイル
    └── incremental/       # インクリメンタルコンパイルキャッシュ
```

## 開発時の注意点

1. **モジュラー設計**: 各クレートは明確な責任を持ち、疎結合を維持する
2. **クロスプラットフォーム対応**: プラットフォーム固有のコードは falcommand-platform に集約
3. **軽量性**: 不要な依存関係を避け、遅延読み込みを活用
4. **拡張性**: プラグインシステムを通じた機能拡張をサポート