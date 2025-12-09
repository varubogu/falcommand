# 設定ファイル優先度設計書

## 概要

リリースビルド以外の場合に、デバッグ用設定ファイルをOSごとの分岐よりも高い優先度で適用する機能を実装する。

## 現在の優先度システム

現在の設定ファイル優先度（config.mdより）:
1. プラットフォームのローカル設定
2. 通常のローカル設定  
3. プラットフォームのユーザー設定
4. 通常のユーザー設定
5. デフォルト値

## 新しい優先度システム

### リリースビルドの場合
従来の優先度システムを維持:
1. プラットフォームのローカル設定
2. 通常のローカル設定
3. プラットフォームのユーザー設定
4. 通常のユーザー設定
5. デフォルト値

### デバッグビルド（非リリース）の場合
デバッグ設定を最優先に追加:
1. **デバッグ設定ファイル** ← 新規追加
2. プラットフォームのローカル設定
3. 通常のローカル設定
4. プラットフォームのユーザー設定
5. 通常のユーザー設定
6. デフォルト値

## デバッグ設定ファイルの仕様

### ファイル名規則
- 基本デバッグ設定: `config.debug.json`
- プラットフォーム固有デバッグ設定: `config.debug.windows.json`, `config.debug.macos.json`, `config.debug.linux.json`

### 配置場所
デバッグビルドの場合: プロジェクトルートの `.falcommand` フォルダ
- 例: `./falcommand/.falcommand/config.debug.json`

### 読み込み順序（デバッグビルド時）
1. `config.debug.{platform}.json` (最優先)
2. `config.debug.json`
3. 従来の優先度システム

## 実装方針

### 修正対象ファイル
- `crates/falcommand-config/src/config.rs`
  - 新しい`get_debug_config_path()`メソッドの追加
  - `load_default()`メソッドの修正（デバッグ設定の読み込み追加）
  - `get_platform_specific_config()`メソッドの修正

### 新規追加メソッド
- `get_debug_config_path() -> Result<PathBuf, ConfigError>`: デバッグ設定ファイルのパスを取得
- `get_platform_debug_config_path() -> Result<PathBuf, ConfigError>`: プラットフォーム固有デバッグ設定ファイルのパスを取得
- `load_debug_config() -> Result<Option<Config>, ConfigError>`: デバッグ設定の読み込み

### 修正メソッド
- `load_default()`: デバッグ設定の読み込みを追加
- `get_platform_specific_config()`: デバッグ設定との統合

## テスト戦略

1. デバッグビルドでデバッグ設定が最優先で読み込まれることを確認
2. リリースビルドで従来の優先度システムが維持されることを確認
3. プラットフォーム固有デバッグ設定が正しく読み込まれることを確認
4. 設定のマージが正しく動作することを確認

## 後方互換性

- 既存の設定ファイルは影響を受けない
- デバッグ設定ファイルが存在しない場合は従来通り動作
- リリースビルドでは完全に従来通り動作