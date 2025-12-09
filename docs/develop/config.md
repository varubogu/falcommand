# 設定管理

## 設定ファイルのスコープ

設定ファイルは次のレベルで用意する。
- ユーザー設定（共通的な設定で同期される）
- ローカル設定（プラットフォーム固有設定で動悸されない）

### プラットフォームごとの設定ファイルのスコープ

それぞれのレベルでプラットフォーム固有設定を書くこともできる。
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

### デバッグビルド専用設定ファイル

**デバッグビルド時のみ利用可能：**
- デバッグ設定（共通）: config.debug.json
- Windowsのデバッグ設定: config.debug.windows.json
- MacOsのデバッグ設定: config.debug.macos.json
- Linuxのデバッグ設定: config.debug.linux.json

**配置場所：** プロジェクトルートの `.falcommand` フォルダ
- 例: `./falcommand/.falcommand/config.debug.json`

**注意：** デバッグ設定ファイルはリリースビルドでは読み込まれません。

同じキーが複数レベルで見つかった場合、優先度は

### リリースビルドの場合
1. プラットフォームのローカル設定
2. 通常のローカル設定
3. プラットフォームのユーザー設定
4. 通常のユーザー設定
5. デフォルト値

### デバッグビルド（非リリース）の場合
**デバッグ設定が最優先となります：**

1. **デバッグ設定ファイル（プラットフォーム固有）**
2. **デバッグ設定ファイル（共通）**
3. プラットフォームのローカル設定
4. 通常のローカル設定
5. プラットフォームのユーザー設定
6. 通常のユーザー設定
7. デフォルト値

となる

## 設定ファイルのスキーマ

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

## 設定バックアップ・復元

起動時にバックアップを保持する（10回分まで）
ただし起動時にエラーが起きた場合は新しいバックアップは作成しない（古いバックアップは削除されない）
