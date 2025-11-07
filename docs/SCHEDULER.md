# Scheduler機能ガイド

## 概要

backup-suiteのScheduler機能は、macOSとLinuxで自動バックアップを実現します。
優先度別に異なる頻度でバックアップを実行でき、システムの自動起動機能と統合されています。

## サポートされているプラットフォーム

- **macOS**: launchd（plist設定ファイル）
- **Linux**: systemd（service/timer ユニット）

## クイックスタート

### 1. 設定ファイルの確認

```bash
# 設定ファイルを開く
backup-suite config open

# または直接編集
nano ~/.config/backup-suite/config.toml
```

### 2. スケジュール頻度の設定

```bash
# High: 毎日、Medium: 毎週、Low: 毎月
backup-suite schedule setup --high daily --medium weekly --low monthly
```

### 3. スケジュールの有効化

```bash
# 全優先度を有効化
backup-suite schedule enable

# または特定優先度のみ
backup-suite schedule enable --priority high
```

### 4. 状態確認

```bash
backup-suite schedule status
```

## コマンドリファレンス

### `schedule setup`

スケジュール頻度を設定します。

```bash
backup-suite schedule setup [OPTIONS]

オプション:
  --high <FREQUENCY>     High優先度の頻度 [default: daily]
  --medium <FREQUENCY>   Medium優先度の頻度 [default: weekly]
  --low <FREQUENCY>      Low優先度の頻度 [default: monthly]
```

頻度オプション:
- `daily`: 毎日午前2時
- `weekly`: 毎週日曜午前2時
- `monthly`: 毎月1日午前2時
- `hourly`: 毎時0分（テスト用）

### `schedule enable`

スケジュールを有効化します。

```bash
backup-suite schedule enable [OPTIONS]

オプション:
  --priority <PRIORITY>  特定優先度のみ有効化 (high/medium/low)
```

### `schedule disable`

スケジュールを無効化します。

```bash
backup-suite schedule disable [OPTIONS]

オプション:
  --priority <PRIORITY>  特定優先度のみ無効化 (high/medium/low)
```

### `schedule status`

スケジュールの状態を表示します。

```bash
backup-suite schedule status
```

出力例:
```
📅 スケジュール設定
  有効: ✅
  高優先度: daily
  中優先度: weekly
  低優先度: monthly

📋 実際のスケジュール状態
  high: ✅ 有効
  medium: ✅ 有効
  low: ❌ 無効
```

## 使用例

### パターン1: 毎日のバックアップ（高優先度のみ）

```bash
# 設定
backup-suite schedule setup --high daily
backup-suite schedule enable --priority high

# 確認
backup-suite schedule status
```

### パターン2: 優先度別の頻度設定

```bash
# 重要データは毎日、通常データは毎週、アーカイブは毎月
backup-suite schedule setup \
  --high daily \
  --medium weekly \
  --low monthly

# 全て有効化
backup-suite schedule enable
```

### パターン3: 段階的導入

```bash
# まずHigh優先度のみ有効化
backup-suite schedule enable --priority high

# 1週間後、Medium優先度も有効化
backup-suite schedule enable --priority medium

# さらに1週間後、Low優先度も有効化
backup-suite schedule enable --priority low
```

### パターン4: テスト実行

```bash
# 毎時実行でテスト（本番環境では非推奨）
backup-suite schedule setup --high hourly
backup-suite schedule enable --priority high

# 動作確認後、日次に変更
backup-suite schedule setup --high daily
backup-suite schedule disable --priority high
backup-suite schedule enable --priority high
```

## プラットフォーム別の詳細

### macOS

- **設定ファイル**: `~/Library/LaunchAgents/com.backup-suite.{priority}.plist`
- **ログファイル**: `/tmp/backup-suite-{priority}.log`
- **管理コマンド**: `launchctl`

詳細: [schedule-setup-macos.md](./schedule-setup-macos.md)

### Linux

- **設定ファイル**: `~/.config/systemd/user/backup-suite-{priority}.{service,timer}`
- **ログ**: `journalctl --user -u backup-suite-{priority}.service`
- **管理コマンド**: `systemctl --user`

詳細: [schedule-setup-linux.md](./schedule-setup-linux.md)

## トラブルシューティング

### スケジュールが実行されない

#### macOS
```bash
# launchd の状態確認
launchctl list | grep backup-suite

# ログ確認
tail -f /tmp/backup-suite-high.log
tail -f /tmp/backup-suite-high.error.log
```

#### Linux
```bash
# systemd の状態確認
systemctl --user status backup-suite-high.timer

# ログ確認
journalctl --user -u backup-suite-high.service -f
```

### スケジュールの完全削除

```bash
# 全優先度を無効化・削除
backup-suite schedule disable
```

### パーミッションエラー

```bash
# macOS: LaunchAgents ディレクトリを作成
mkdir -p ~/Library/LaunchAgents

# Linux: systemd ユーザーディレクトリを作成
mkdir -p ~/.config/systemd/user
```

## セキュリティ推奨事項

### パスワード管理

暗号化バックアップを使用する場合:

1. **スケジュール設定ファイルにパスワードを含めない**
2. **将来実装予定のkeyring統合を使用**
3. **環境変数またはセキュアな設定ファイルを使用**

### ファイルパーミッション

```bash
# macOS
chmod 644 ~/Library/LaunchAgents/com.backup-suite.*.plist

# Linux
chmod 644 ~/.config/systemd/user/backup-suite-*
```

### ログのセキュリティ

ログファイルに機密情報が含まれないよう、エラーメッセージは適切にフィルタされます。

## ベストプラクティス

### 1. テスト実行

本番運用前に必ずdry-runでテスト:

```bash
backup-suite run --priority high --dry-run
```

### 2. ログ監視

定期的にログを確認:

```bash
# macOS
tail -n 100 /tmp/backup-suite-high.log

# Linux
journalctl --user -u backup-suite-high.service -n 100
```

### 3. ディスク容量監視

バックアップ先の容量を監視:

```bash
df -h ~/backup-suite/backups
```

### 4. 段階的導入

一度に全優先度を有効化せず、Highから順に導入:

```bash
# 1週目: High優先度のみ
backup-suite schedule enable --priority high

# 2週目: Medium優先度追加
backup-suite schedule enable --priority medium

# 3週目: Low優先度追加
backup-suite schedule enable --priority low
```

### 5. クリーンアップの自動化

古いバックアップの自動削除を有効化:

```toml
[backup]
auto_cleanup = true
keep_days = 30
```

## 設定ファイル例

`~/.config/backup-suite/config.toml`:

```toml
version = "1.0.0"

[backup]
destination = "/path/to/backup/storage"
auto_cleanup = true
keep_days = 30

[schedule]
enabled = true
high_frequency = "daily"
medium_frequency = "weekly"
low_frequency = "monthly"

[[targets]]
path = "/home/user/documents"
priority = "High"
category = "重要ドキュメント"
exclude_patterns = ["*.tmp", "*.log", "*.bak"]

[[targets]]
path = "/home/user/projects"
priority = "Medium"
category = "プロジェクト"
exclude_patterns = [
    "\\.git/.*",
    "node_modules/.*",
    "target/.*",
    "__pycache__/.*"
]

[[targets]]
path = "/home/user/archive"
priority = "Low"
category = "アーカイブ"
exclude_patterns = []
```

## APIリファレンス（Rustコード）

### Schedulerの使用例

```rust
use backup_suite::{Config, Scheduler};

fn main() -> anyhow::Result<()> {
    // 設定を読み込み
    let mut config = Config::load()?;

    // スケジュール設定
    config.schedule.enabled = true;
    config.schedule.high_frequency = "daily".to_string();
    config.schedule.medium_frequency = "weekly".to_string();
    config.schedule.low_frequency = "monthly".to_string();
    config.save()?;

    // Schedulerインスタンス作成
    let scheduler = Scheduler::new(config)?;

    // 全優先度をセットアップ
    scheduler.setup_all()?;

    // 全優先度を有効化
    scheduler.enable_all()?;

    // 状態確認
    let status = scheduler.check_status()?;
    println!("High: {}", status.high_enabled);
    println!("Medium: {}", status.medium_enabled);
    println!("Low: {}", status.low_enabled);

    Ok(())
}
```

## 関連ドキュメント

- [macOS詳細ガイド](./schedule-setup-macos.md)
- [Linux詳細ガイド](./schedule-setup-linux.md)
- [README.md](../README.md)
- [設定リファレンス](../README.md#設定)

## FAQ

### Q: 実行時刻を変更できますか？

A: はい。プラットフォーム固有の設定ファイルを直接編集してください。
- macOS: plistファイルの`Hour`/`Minute`キー
- Linux: timerファイルの`OnCalendar`ディレクティブ

### Q: 複数の実行時刻を設定できますか？

A: Linux systemdでは可能です（`OnCalendar`を複数指定）。
macOSでは複数のplistファイルを手動作成する必要があります。

### Q: バックアップが失敗した場合、通知されますか？

A: 現在は実装されていません。将来のバージョンで通知機能を追加予定です。
現在はログファイルで確認してください。

### Q: スケジュール実行時のパフォーマンスへの影響は？

A: バックアップはバックグラウンドで実行され、システムリソースを適切に使用します。
大量のファイルをバックアップする場合、深夜などの低負荷時間帯を推奨します。

### Q: cron との違いは？

A: launchd/systemd は以下の点でcronより優れています:
- システム起動時の実行遅延が可能（`Persistent=true`）
- より詳細なログ管理
- リソース制限の設定が可能
- より柔軟なスケジューリング（systemd）
