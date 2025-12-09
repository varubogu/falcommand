# ビルド・デプロイ手順

FalCommand のビルドとデプロイに関する詳細な手順を説明します。

## ビルド環境の準備

### 前提条件
- [開発環境セットアップ](./setup.md)が完了していること
- Git がインストールされていること
- 各プラットフォームの開発ツールが利用可能であること

## 1. 開発ビルド

### 基本的なビルド
```bash
# デバッグビルド（最も高速）
cargo build

# 実行
cargo run

# 引数付きで実行
cargo run -- --help
```

### 最適化オプション
```bash
# リリースビルド（最適化あり）
cargo build --release

# 特定のターゲット向けビルド
cargo build --target x86_64-pc-windows-msvc

# 全ターゲット向けビルド確認
cargo build --target x86_64-pc-windows-msvc
cargo build --target x86_64-apple-darwin
cargo build --target x86_64-unknown-linux-gnu
```

## 2. テストとコード品質チェック

### テスト実行
```bash
# 全テスト実行
cargo test

# 特定のテストのみ実行
cargo test search_engine

# テストカバレッジ表示（tarpaulinが必要）
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

### コード品質チェック
```bash
# Linter実行
cargo clippy

# 警告をエラーとして扱う
cargo clippy -- -D warnings

# フォーマットチェック
cargo fmt --check

# フォーマット実行
cargo fmt
```

### セキュリティ監査
```bash
# セキュリティ脆弱性チェック
cargo audit

# 依存関係の更新確認
cargo outdated
```

## 3. クロスプラットフォームビルド

### Windows向けビルド
```bash
# Windows（Intel/AMD 64bit）
cargo build --release --target x86_64-pc-windows-msvc

# 出力場所: target/x86_64-pc-windows-msvc/release/falcommand.exe
```

### macOS向けビルド
```bash
# macOS Intel
cargo build --release --target x86_64-apple-darwin

# macOS Apple Silicon
cargo build --release --target aarch64-apple-darwin

# 出力場所: target/{target}/release/falcommand
```

### Linux向けビルド
```bash
# Linux（Intel/AMD 64bit）
cargo build --release --target x86_64-unknown-linux-gnu

# Linux ARM64
cargo build --release --target aarch64-unknown-linux-gnu

# 出力場所: target/{target}/release/falcommand
```

### 全プラットフォーム向け一括ビルド
```bash
#!/bin/bash
# build-all.sh

set -e

echo "Building for all platforms..."

# Windows
echo "Building for Windows..."
cargo build --release --target x86_64-pc-windows-msvc

# macOS Intel
echo "Building for macOS Intel..."
cargo build --release --target x86_64-apple-darwin

# macOS Apple Silicon
echo "Building for macOS Apple Silicon..."
cargo build --release --target aarch64-apple-darwin

# Linux
echo "Building for Linux x86_64..."
cargo build --release --target x86_64-unknown-linux-gnu

# Linux ARM64
echo "Building for Linux ARM64..."
cargo build --release --target aarch64-unknown-linux-gnu

echo "All builds completed!"
```

## 4. パッケージング

### アーティファクトの準備
```bash
#!/bin/bash
# package.sh

VERSION=$(cargo pkgid | cut -d# -f2)
DIST_DIR="dist"

rm -rf $DIST_DIR
mkdir -p $DIST_DIR

# Windows
cp target/x86_64-pc-windows-msvc/release/falcommand.exe $DIST_DIR/falcommand-windows-x86_64.exe

# macOS Intel
cp target/x86_64-apple-darwin/release/falcommand $DIST_DIR/falcommand-macos-x86_64

# macOS Apple Silicon
cp target/aarch64-apple-darwin/release/falcommand $DIST_DIR/falcommand-macos-aarch64

# Linux
cp target/x86_64-unknown-linux-gnu/release/falcommand $DIST_DIR/falcommand-linux-x86_64
cp target/aarch64-unknown-linux-gnu/release/falcommand $DIST_DIR/falcommand-linux-aarch64

# 圧縮
cd $DIST_DIR
tar -czf falcommand-${VERSION}-windows-x86_64.tar.gz falcommand-windows-x86_64.exe
tar -czf falcommand-${VERSION}-macos-x86_64.tar.gz falcommand-macos-x86_64
tar -czf falcommand-${VERSION}-macos-aarch64.tar.gz falcommand-macos-aarch64
tar -czf falcommand-${VERSION}-linux-x86_64.tar.gz falcommand-linux-x86_64
tar -czf falcommand-${VERSION}-linux-aarch64.tar.gz falcommand-linux-aarch64

echo "Packaging completed for version $VERSION"
```

### インストーラーの作成

#### Windows（Inno Setup使用）
```pascal
; falcommand.iss
[Setup]
AppName=FalCommand
AppVersion=0.1.0
DefaultDirName={pf}\FalCommand
DefaultGroupName=FalCommand
OutputBaseFilename=falcommand-installer-windows

[Files]
Source: "target\x86_64-pc-windows-msvc\release\falcommand.exe"; DestDir: "{app}"

[Icons]
Name: "{group}\FalCommand"; Filename: "{app}\falcommand.exe"
```

#### macOS（.app バンドル）
```bash
#!/bin/bash
# create-macos-app.sh

APP_NAME="FalCommand"
BUNDLE_DIR="${APP_NAME}.app"
CONTENTS_DIR="${BUNDLE_DIR}/Contents"
MACOS_DIR="${CONTENTS_DIR}/MacOS"
RESOURCES_DIR="${CONTENTS_DIR}/Resources"

rm -rf "${BUNDLE_DIR}"
mkdir -p "${MACOS_DIR}"
mkdir -p "${RESOURCES_DIR}"

# バイナリをコピー
cp "target/x86_64-apple-darwin/release/falcommand" "${MACOS_DIR}/"

# Info.plist を作成
cat > "${CONTENTS_DIR}/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>falcommand</string>
    <key>CFBundleIdentifier</key>
    <string>com.example.falcommand</string>
    <key>CFBundleName</key>
    <string>FalCommand</string>
    <key>CFBundleVersion</key>
    <string>0.1.0</string>
</dict>
</plist>
EOF

# DMGファイル作成
hdiutil create -volname "FalCommand" -srcfolder "${BUNDLE_DIR}" -ov -format UDZO "${APP_NAME}.dmg"
```

#### Linux（.deb パッケージ）
```bash
#!/bin/bash
# create-deb-package.sh

PACKAGE_NAME="falcommand"
VERSION="0.1.0"
ARCH="amd64"
DEB_DIR="${PACKAGE_NAME}_${VERSION}_${ARCH}"

rm -rf "$DEB_DIR"
mkdir -p "$DEB_DIR/DEBIAN"
mkdir -p "$DEB_DIR/usr/bin"
mkdir -p "$DEB_DIR/usr/share/applications"

# バイナリをコピー
cp "target/x86_64-unknown-linux-gnu/release/falcommand" "$DEB_DIR/usr/bin/"

# control ファイル作成
cat > "$DEB_DIR/DEBIAN/control" << EOF
Package: $PACKAGE_NAME
Version: $VERSION
Architecture: $ARCH
Maintainer: FalCommand Team <team@falcommand.example>
Description: Lightweight cross-platform command launcher
 A fast and lightweight command launcher for desktop environments.
EOF

# .desktop ファイル作成
cat > "$DEB_DIR/usr/share/applications/falcommand.desktop" << EOF
[Desktop Entry]
Name=FalCommand
Comment=Command Launcher
Exec=/usr/bin/falcommand
Icon=falcommand
Type=Application
Categories=Utility;
EOF

# パッケージ作成
dpkg-deb --build "$DEB_DIR"
```

## 5. CI/CD パイプライン

### GitHub Actions設定
```yaml
# .github/workflows/build.yml
name: Build and Release

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]
  release:
    types: [ published ]

jobs:
  build:
    strategy:
      matrix:
        include:
        - os: windows-latest
          target: x86_64-pc-windows-msvc
          artifact_name: falcommand.exe
          asset_name: falcommand-windows-x86_64.exe
        - os: macos-latest
          target: x86_64-apple-darwin
          artifact_name: falcommand
          asset_name: falcommand-macos-x86_64
        - os: ubuntu-latest
          target: x86_64-unknown-linux-gnu
          artifact_name: falcommand
          asset_name: falcommand-linux-x86_64
    
    runs-on: ${{ matrix.os }}
    
    steps:
    - uses: actions/checkout@v3
    
    - uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        target: ${{ matrix.target }}
        override: true
    
    - name: Install Linux dependencies
      if: runner.os == 'Linux'
      run: |
        sudo apt-get update
        sudo apt-get install -y libfontconfig1-dev libxcb-shape0-dev libxcb-xfixes0-dev
    
    - name: Build
      run: cargo build --release --target ${{ matrix.target }}
    
    - name: Upload artifacts
      uses: actions/upload-artifact@v3
      with:
        name: ${{ matrix.asset_name }}
        path: target/${{ matrix.target }}/release/${{ matrix.artifact_name }}
```

## 6. デプロイメント

### GitHub Releases
```bash
# GitHub CLI を使用したリリース作成
gh release create v0.1.0 \
  dist/falcommand-0.1.0-windows-x86_64.tar.gz \
  dist/falcommand-0.1.0-macos-x86_64.tar.gz \
  dist/falcommand-0.1.0-linux-x86_64.tar.gz \
  --title "FalCommand v0.1.0" \
  --notes "Initial release"
```

### パッケージマネージャー対応

#### Homebrew（macOS）
```ruby
# falcommand.rb
class Falcommand < Formula
  desc "Lightweight cross-platform command launcher"
  homepage "https://github.com/varubogu/falcommand"
  url "https://github.com/varubogu/falcommand/archive/v0.1.0.tar.gz"
  sha256 "..."
  
  depends_on "rust" => :build
  
  def install
    system "cargo", "install", *std_cargo_args
  end
  
  test do
    system "#{bin}/falcommand", "--version"
  end
end
```

#### Chocolatey（Windows）
```xml
<!-- falcommand.nuspec -->
<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://schemas.microsoft.com/packaging/2015/06/nuspec.xsd">
  <metadata>
    <id>falcommand</id>
    <version>0.1.0</version>
    <title>FalCommand</title>
    <authors>FalCommand Team</authors>
    <description>Lightweight cross-platform command launcher</description>
  </metadata>
</package>
```

## 7. バージョン管理

### セマンティックバージョニング
- **メジャー**: 破壊的変更
- **マイナー**: 新機能追加（後方互換）
- **パッチ**: バグ修正

### リリース手順
1. バージョン番号を Cargo.toml で更新
2. CHANGELOG.md を更新
3. Git タグを作成
4. GitHub Actions でビルド・リリース自動実行

```bash
# リリース例
git tag v0.1.0
git push origin v0.1.0
```

## トラブルシューティング

### よくある問題

#### ビルドエラー
```bash
# 依存関係の問題
cargo clean
cargo update
cargo build

# ターゲットが見つからない
rustup target list
rustup target add <target-name>
```

#### クロスコンパイルの問題
```bash
# リンカーの設定（~/.cargo/config.toml）
[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
```

#### メモリ不足
```bash
# 並列ビルド数を制限
export CARGO_BUILD_JOBS=1
cargo build --release
```

## パフォーマンス最適化

### 実行ファイルサイズの最適化
```toml
# Cargo.toml
[profile.release]
opt-level = "z"  # サイズ最適化
lto = true       # Link Time Optimization
codegen-units = 1
panic = "abort"
strip = true     # デバッグシンボルを削除
```

### ビルド時間の短縮
```toml
# Cargo.toml
[profile.dev]
opt-level = 1    # 軽い最適化でビルド高速化

# 並列ビルド設定
[build]
jobs = 4
```

## 次のステップ

ビルドが成功したら：

1. [テストガイド](./testing.md)で品質確認
2. [パフォーマンス最適化](./performance.md)でチューニング
3. [デバッグ・診断](./debugging.md)で問題解決手法を学習