# Scheduler機能 - 実装完了報告

## 実装概要

backup-suiteにスケジューリング機能を実装しました。macOS (launchd) と Linux (systemd) の両方をサポートし、優先度別に異なる頻度でバックアップを自動実行できます。

## 実装ファイル

### 1. コアモジュール

#### `src/core/scheduler.rs`（新規作成）

スケジューリングのコアロジックを実装:

**主要な構造体:**
- `Scheduler`: スケジューラのメインAPI
- `Platform`: サポートされているプラットフォーム（macOS/Linux）
- `Frequency`: スケジュール頻度（daily/weekly/monthly/hourly）
- `ScheduleStatus`: スケジュールの状態

**主要なメソッド:**
- `new(config)`: Schedulerインスタンス作成
- `setup_all()`: 全優先度のスケジュールをセットアップ
- `setup_priority(priority)`: 特定優先度のスケジュールをセットアップ
- `enable_all()`: 全優先度のスケジュールを有効化
- `enable_priority(priority)`: 特定優先度のスケジュールを有効化
- `disable_all()`: 全優先度のスケジュールを無効化
- `disable_priority(priority)`: 特定優先度のスケジュールを無効化
- `check_status()`: スケジュールの状態を確認

**プラットフォーム別実装:**

##### macOS (launchd)
- plist設定ファイルの生成（`~/Library/LaunchAgents/com.backup-suite.{priority}.plist`）
- `launchctl load/unload` による有効化/無効化
- `launchctl list` による状態確認
- ログファイル: `/tmp/backup-suite-{priority}.log`

##### Linux (systemd)
- service/timer ユニットファイルの生成（`~/.config/systemd/user/`）
- `systemctl --user enable/disable/start/stop` による管理
- `systemctl --user is-enabled` による状態確認
- ログ: `journalctl --user -u backup-suite-{priority}.service`

### 2. 既存ファイルの更新

#### `src/core/mod.rs`
- `pub mod scheduler;` 追加
- `Scheduler`, `Frequency`, `Platform`, `ScheduleStatus` のエクスポート

#### `src/lib.rs`
- 上記の型を再エクスポート

#### `src/main.rs`
- `use backup_suite::core::Scheduler;` 追加
- 古いlaunchd関数（`setup_launchd_schedule`等）を削除
- `Commands::Schedule` の処理を新しい`Scheduler` APIで書き直し

#### `src/core/config.rs`（既存）
- `ScheduleConfig` 構造体は既に実装済み
- 変更なし

## CLIコマンド

### `schedule setup`
スケジュール頻度を設定:
```bash
backup-suite schedule setup --high daily --medium weekly --low monthly
```

### `schedule enable`
スケジュールを有効化:
```bash
# 全優先度を有効化
backup-suite schedule enable

# 特定優先度のみ
backup-suite schedule enable --priority high
```

### `schedule disable`
スケジュールを無効化:
```bash
# 全優先度を無効化
backup-suite schedule disable

# 特定優先度のみ
backup-suite schedule disable --priority high
```

### `schedule status`
スケジュールの状態を表示:
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

## 設定ファイル

`~/.config/backup-suite/config.toml`:

```toml
[schedule]
enabled = true
high_frequency = "daily"
medium_frequency = "weekly"
low_frequency = "monthly"
```

## ドキュメント

以下のドキュメントを作成しました:

### 1. `docs/SCHEDULER.md`
スケジューラ機能の総合ガイド:
- 概要
- クイックスタート
- コマンドリファレンス
- 使用例
- トラブルシューティング
- FAQ

### 2. `docs/schedule-setup-macos.md`
macOS固有の詳細ガイド:
- launchd統合
- plistファイル例
- ログ確認
- トラブルシューティング
- 実行時刻のカスタマイズ
- セキュリティ推奨事項

### 3. `docs/schedule-setup-linux.md`
Linux固有の詳細ガイド:
- systemd統合
- service/timerファイル例
- journalctlログ確認
- OnCalendar形式の例
- タイマー管理
- セキュリティ推奨事項

### 4. `docs/example-config.toml`
設定ファイルの完全な例:
- バックアップ対象の定義例
- 除外パターンの例
- よく使うパターン集

## セキュリティ考慮事項

### パスワード管理
暗号化バックアップを使用する場合:
- ⚠️ スケジュール設定ファイルにパスワードを**含めない**
- 将来実装予定: macOS Keychain / Linux keyring 統合
- 現在の推奨: 環境変数またはセキュアな設定ファイル

### ファイルパーミッション
設定ファイルは適切なパーミッションで保護:
```bash
# macOS
chmod 644 ~/Library/LaunchAgents/com.backup-suite.*.plist

# Linux
chmod 644 ~/.config/systemd/user/backup-suite-*
```

### ログセキュリティ
- 機密情報がログに含まれないよう、エラーメッセージは適切にフィルタ
- パスワードやトークンはログに出力しない

## テスト

### ビルド確認
```bash
cargo check
# 成功: warning 1件のみ（restore.rs の unused function）
```

### 推奨テスト手順

#### macOS
```bash
# 1. 設定
backup-suite schedule setup --high daily

# 2. セットアップ（plist生成）
backup-suite schedule enable --priority high

# 3. 状態確認
backup-suite schedule status
launchctl list | grep backup-suite

# 4. ログ確認
tail -f /tmp/backup-suite-high.log

# 5. 無効化
backup-suite schedule disable --priority high
```

#### Linux
```bash
# 1. 設定
backup-suite schedule setup --high daily

# 2. セットアップ（service/timer生成）
backup-suite schedule enable --priority high

# 3. 状態確認
backup-suite schedule status
systemctl --user list-timers backup-suite-*

# 4. ログ確認
journalctl --user -u backup-suite-high.service -f

# 5. 無効化
backup-suite schedule disable --priority high
```

## 使用例

### パターン1: 毎日のバックアップ（高優先度のみ）
```bash
# 設定
backup-suite schedule setup --high daily
backup-suite schedule enable --priority high
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
# 1週目: High優先度のみ
backup-suite schedule enable --priority high

# 2週目: Medium優先度追加
backup-suite schedule enable --priority medium

# 3週目: Low優先度追加
backup-suite schedule enable --priority low
```

## 今後の拡張案

### 優先度1（近い将来）
- [ ] macOS Keychain統合（暗号化パスワードの安全な保存）
- [ ] Linux keyring統合
- [ ] スケジュール実行の通知機能

### 優先度2（中期）
- [ ] カスタムスケジュール（複数時刻の設定）
- [ ] 実行時刻のGUI設定
- [ ] スケジュール実行の詳細ログ

### 優先度3（長期）
- [ ] Windows Task Scheduler統合
- [ ] クラウド同期前のバックアップ
- [ ] 条件付きバックアップ（ディスク容量チェック等）

## トラブルシューティング

### Q: スケジュールが実行されない

#### macOS
```bash
# launchd の状態確認
launchctl list | grep backup-suite

# plistファイルの確認
ls -la ~/Library/LaunchAgents/com.backup-suite.*.plist

# ログ確認
tail -f /tmp/backup-suite-high.log
tail -f /tmp/backup-suite-high.error.log

# 手動でロード
launchctl load ~/Library/LaunchAgents/com.backup-suite.high.plist
```

#### Linux
```bash
# systemd の状態確認
systemctl --user status backup-suite-high.timer
systemctl --user list-timers

# ログ確認
journalctl --user -u backup-suite-high.service -n 100

# 手動で有効化
systemctl --user enable backup-suite-high.timer
systemctl --user start backup-suite-high.timer
```

### Q: プラットフォームがサポートされていない

```bash
# エラーメッセージ:
# "このプラットフォームではスケジューリング機能はサポートされていません"

# 対策: cronを使用した代替方法
# crontabに以下を追加:
0 2 * * * /usr/local/bin/backup-suite run --priority high
```

## パフォーマンスへの影響

- バックアップはバックグラウンドで実行
- システムリソースを適切に使用（rayon並列処理）
- 大量のファイルをバックアップする場合、深夜などの低負荷時間帯を推奨
- スケジューラ自体のオーバーヘッドは無視できるレベル

## API使用例（Rust）

```rust
use backup_suite::{Config, Scheduler};

fn main() -> anyhow::Result<()> {
    // 設定を読み込み
    let mut config = Config::load()?;

    // スケジュール設定
    config.schedule.enabled = true;
    config.schedule.high_frequency = "daily".to_string();
    config.save()?;

    // Schedulerインスタンス作成
    let scheduler = Scheduler::new(config)?;

    // セットアップ＆有効化
    scheduler.setup_all()?;
    scheduler.enable_all()?;

    // 状態確認
    let status = scheduler.check_status()?;
    println!("High: {}", status.high_enabled);
    println!("Medium: {}", status.medium_enabled);
    println!("Low: {}", status.low_enabled);

    Ok(())
}
```

## 関連ファイル

- `/Users/sanae.abe/projects/backup-suite/src/core/scheduler.rs` - コアロジック
- `/Users/sanae.abe/projects/backup-suite/src/main.rs` - CLIコマンド統合
- `/Users/sanae.abe/projects/backup-suite/docs/SCHEDULER.md` - 総合ガイド
- `/Users/sanae.abe/projects/backup-suite/docs/schedule-setup-macos.md` - macOSガイド
- `/Users/sanae.abe/projects/backup-suite/docs/schedule-setup-linux.md` - Linuxガイド
- `/Users/sanae.abe/projects/backup-suite/docs/example-config.toml` - 設定例

## 実装完了

すべての実装が完了し、ビルドも成功しました。
ドキュメントも充実しており、ユーザーはすぐに使用を開始できます。

---

**実装日**: 2025-11-07
**担当**: SRE Engineer (Claude Code)
**レビュー状況**: 実装完了、ドキュメント完備
