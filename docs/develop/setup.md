# 開発環境セットアップ

FalCommand の開発環境を構築するための手順を説明します。

## 前提条件

### システム要件
- **OS**: Windows 10+, macOS 10.14+, Ubuntu 18.04+
- **メモリ**: 4GB以上推奨
- **ディスク容量**: 5GB以上の空き容量

## 1. Rust 開発環境の構築

### Rust のインストール
```bash
# Rust 公式インストーラーを使用
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# インストール後、環境変数を読み込み
source $HOME/.cargo/env

# バージョン確認
rustc --version
cargo --version
```

### クロスコンパイル用ターゲットの追加
```bash
# Windows 用
rustup target add x86_64-pc-windows-msvc

# macOS 用  
rustup target add x86_64-apple-darwin

# Linux 用
rustup target add x86_64-unknown-linux-gnu

# ARM64 対応（Apple Silicon Mac等）
rustup target add aarch64-apple-darwin
rustup target add aarch64-unknown-linux-gnu
```

## 2. プラットフォーム固有の依存関係

### Windows
```powershell
# Visual Studio Build Tools または Visual Studio Community をインストール
# https://visualstudio.microsoft.com/downloads/

# 追加の依存関係は不要（Slintが自動処理）
```

### macOS
```bash
# Xcode Command Line Tools のインストール
xcode-select --install

# 追加の依存関係は不要（Slintが自動処理）
```

### Linux (Ubuntu/Debian)
```bash
# 必須依存関係のインストール
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    libfontconfig1-dev \
    libxcb-shape0-dev \
    libxcb-xfixes0-dev \
    libxcb1-dev \
    libxcb-keysyms1-dev \
    libxcb-image0-dev \
    libxcb-shm0-dev \
    libxcb-util0-dev \
    libxcb-render-util0-dev \
    libxcb-render0-dev \
    libxcb-randr0-dev \
    libxcb-sync-dev \
    libxcb-xfixes0-dev \
    libxcb-icccm4-dev \
    libxcb-shape0-dev \
    libxcb-xkb-dev
```

### Linux (Fedora/CentOS)
```bash
# 必須依存関係のインストール
sudo dnf install -y \
    gcc \
    gcc-c++ \
    fontconfig-devel \
    libxcb-devel \
    libX11-devel
```

## 3. 開発ツールの設定

### VS Code の設定（推奨）
```bash
# VS Code のインストール（各OS固有の方法でインストール）

# 推奨拡張機能をインストール
code --install-extension rust-lang.rust-analyzer
code --install-extension serayuzgur.crates
code --install-extension vadimcn.vscode-lldb
```

VS Code の設定ファイル（`.vscode/settings.json`）:
```json
{
    "rust-analyzer.cargo.target": "x86_64-unknown-linux-gnu",
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.rustfmt.rangeFormatting.enable": true
}
```

## 4. プロジェクトのクローンと初期セットアップ

```bash
# プロジェクトをクローン
git clone https://github.com/varubogu/falcommand.git
cd falcommand

# 依存関係をダウンロード
cargo build

# テスト実行で環境確認
cargo test

# 開発モードで実行
cargo run
```

## 5. 開発用便利ツールのインストール

```bash
# コード品質チェックツール
cargo install cargo-clippy

# セキュリティ監査ツール
cargo install cargo-audit

# ベンチマークツール
cargo install cargo-criterion

# ライセンスチェックツール
cargo install cargo-license

# ドキュメント生成用
cargo install cargo-doc
```

## 6. 環境変数の設定

開発時に使用する環境変数を設定します：

```bash
# .envファイルを作成（プロジェクトルート）
echo "RUST_LOG=debug" > .env
echo "FALCOMMAND_ENV=development" >> .env
```

## 7. 開発環境の確認

以下のコマンドで環境が正しく構築されているか確認します：

```bash
# コンパイルテスト
cargo check

# Lintチェック
cargo clippy

# フォーマットチェック
cargo fmt --check

# テスト実行
cargo test

# ドキュメント生成
cargo doc --open
```

## トラブルシューティング

### よくある問題と解決方法

#### Linux: フォントが表示されない
```bash
# フォント関連パッケージをインストール
sudo apt-get install fonts-liberation fonts-dejavu
```

#### macOS: リンカーエラー
```bash
# Command Line Tools を再インストール
sudo xcode-select --install
```

#### Windows: MSVC not found エラー
```powershell
# Visual Studio Installer から「C++ によるデスクトップ開発」をインストール
```

#### 全般: 依存関係の問題
```bash
# Cargoキャッシュをクリア
cargo clean
rm -rf ~/.cargo/registry
cargo build
```

## 次のステップ

環境構築が完了したら：

1. [アーキテクチャ設計書](./architecture.md)でシステム構造を理解
2. [コーディング規約](./coding-standards.md)に従って開発開始
3. [テストガイド](./testing.md)でテストの書き方を確認

## サポート

セットアップで問題が発生した場合：

1. [トラブルシューティング](../user/troubleshooting.md)を確認
2. [GitHub Issues](https://github.com/varubogu/falcommand/issues)で質問を投稿
3. [GitHub Discussions](https://github.com/varubogu/falcommand/discussions)で議論に参加