# macOS スケジュール設定ガイド

## 概要

backup-suiteはmacOSの`launchd`を使用して自動バックアップをスケジューリングします。
優先度別に異なる頻度でバックアップを実行できます。

## セットアップ手順

### 1. スケジュール頻度の設定

```bash
# High優先度: 毎日、Medium優先度: 毎週、Low優先度: 毎月
backup-suite schedule setup --high daily --medium weekly --low monthly
```

### 2. スケジュールの有効化

```bash
# 全優先度を有効化
backup-suite schedule enable

# または特定優先度のみ有効化
backup-suite schedule enable --priority high
```

### 3. 状態確認

```bash
backup-suite schedule status
```

## スケジュール頻度オプション

- **daily**: 毎日午前2時に実行
- **weekly**: 毎週日曜午前2時に実行
- **monthly**: 毎月1日午前2時に実行
- **hourly**: 毎時0分に実行（テスト用）

## 生成されるファイル

スケジュールを有効化すると、以下のplistファイルが生成されます：

```
~/Library/LaunchAgents/
├── com.backup-suite.high.plist
├── com.backup-suite.medium.plist
└── com.backup-suite.low.plist
```

## plistファイルの例

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.backup-suite.high</string>

    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/backup-suite</string>
        <string>run</string>
        <string>--priority</string>
        <string>high</string>
    </array>

    <key>StartCalendarInterval</key>
    <dict>
        <key>Hour</key>
        <integer>2</integer>
        <key>Minute</key>
        <integer>0</integer>
    </dict>

    <key>RunAtLoad</key>
    <false/>

    <key>StandardOutPath</key>
    <string>/tmp/backup-suite-high.log</string>

    <key>StandardErrorPath</key>
    <string>/tmp/backup-suite-high.error.log</string>

    <key>EnvironmentVariables</key>
    <dict>
        <key>PATH</key>
        <string>/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin</string>
    </dict>
</dict>
</plist>
```

## ログファイル

バックアップ実行ログは以下に保存されます：

- 標準出力: `/tmp/backup-suite-{priority}.log`
- エラー出力: `/tmp/backup-suite-{priority}.error.log`

## トラブルシューティング

### スケジュールが実行されない

```bash
# launchctlで状態確認
launchctl list | grep backup-suite

# 手動でロード
launchctl load ~/Library/LaunchAgents/com.backup-suite.high.plist

# ログ確認
tail -f /tmp/backup-suite-high.log
tail -f /tmp/backup-suite-high.error.log
```

### スケジュールの削除

```bash
# 全優先度を無効化
backup-suite schedule disable

# または特定優先度のみ無効化
backup-suite schedule disable --priority high
```

### 手動アンロード（必要な場合のみ）

```bash
launchctl unload ~/Library/LaunchAgents/com.backup-suite.high.plist
rm ~/Library/LaunchAgents/com.backup-suite.high.plist
```

## セキュリティ上の注意

### パスワード管理

暗号化バックアップを使用する場合、パスワードをplistファイルに含めることは**推奨しません**。
代わりに以下の方法を使用してください：

1. **macOS Keychain統合**（将来実装予定）
2. **環境変数**（システムレベルの設定）
3. **パスワードなしバックアップ**（機密性の低いデータ）

## 実行時刻のカスタマイズ

デフォルトの午前2時を変更したい場合、手動でplistファイルを編集してください：

```xml
<key>StartCalendarInterval</key>
<dict>
    <key>Hour</key>
    <integer>3</integer>  <!-- 午前3時に変更 -->
    <key>Minute</key>
    <integer>30</integer>  <!-- 30分に変更 -->
</dict>
```

変更後、再ロードが必要です：

```bash
launchctl unload ~/Library/LaunchAgents/com.backup-suite.high.plist
launchctl load ~/Library/LaunchAgents/com.backup-suite.high.plist
```

## ベストプラクティス

1. **テスト実行**: 最初はdry-runで動作確認
   ```bash
   backup-suite run --priority high --dry-run
   ```

2. **ログ監視**: 定期的にログファイルを確認
   ```bash
   # 最新のバックアップログを確認
   tail -n 100 /tmp/backup-suite-high.log
   ```

3. **段階的導入**: Highから順に有効化
   ```bash
   backup-suite schedule enable --priority high
   # 1週間後、動作確認できたら
   backup-suite schedule enable --priority medium
   ```

4. **ディスク容量監視**: バックアップ先の容量を定期的に確認
   ```bash
   df -h ~/backup-suite/backups
   ```

## 設定ファイル例

`~/.config/backup-suite/config.toml`:

```toml
version = "1.0.0"

[backup]
destination = "/Volumes/ExternalHDD/backups"
auto_cleanup = true
keep_days = 30

[schedule]
enabled = true
high_frequency = "daily"
medium_frequency = "weekly"
low_frequency = "monthly"

[[targets]]
path = "/Users/username/Documents"
priority = "High"
category = "重要ドキュメント"
exclude_patterns = ["*.tmp", "*.log"]

[[targets]]
path = "/Users/username/Projects"
priority = "Medium"
category = "プロジェクト"
exclude_patterns = ["\\.git/.*", "node_modules/.*", "target/.*"]
```

## 参考リンク

- [launchd公式ドキュメント](https://developer.apple.com/library/archive/documentation/MacOSX/Conceptual/BPSystemStartup/Chapters/CreatingLaunchdJobs.html)
- [launchctl man page](https://ss64.com/osx/launchctl.html)
