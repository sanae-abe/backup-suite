# Linux スケジュール設定ガイド

## 概要

backup-suiteはLinuxの`systemd`を使用して自動バックアップをスケジューリングします。
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

# systemdでの状態確認
systemctl --user status backup-suite-high.timer
systemctl --user list-timers backup-suite-*
```

## スケジュール頻度オプション

- **daily**: 毎日午前2時に実行（`*-*-* 02:00:00`）
- **weekly**: 毎週日曜午前2時に実行（`Sun *-*-* 02:00:00`）
- **monthly**: 毎月1日午前2時に実行（`*-*-01 02:00:00`）
- **hourly**: 毎時0分に実行（`*-*-* *:00:00`）

## 生成されるファイル

スケジュールを有効化すると、以下のsystemdユニットファイルが生成されます：

```
~/.config/systemd/user/
├── backup-suite-high.service
├── backup-suite-high.timer
├── backup-suite-medium.service
├── backup-suite-medium.timer
├── backup-suite-low.service
└── backup-suite-low.timer
```

## systemdユニットファイルの例

### サービスファイル (`backup-suite-high.service`)

```ini
[Unit]
Description=Backup Suite - high Priority Backup
After=network.target

[Service]
Type=oneshot
ExecStart=/usr/local/bin/backup-suite run --priority high
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=default.target
```

### タイマーファイル (`backup-suite-high.timer`)

```ini
[Unit]
Description=Backup Suite - high Priority Backup Timer

[Timer]
OnCalendar=*-*-* 02:00:00
Persistent=true

[Install]
WantedBy=timers.target
```

## ログ確認

systemdジャーナルでログを確認できます：

```bash
# 最新のバックアップログを表示
journalctl --user -u backup-suite-high.service -n 100

# リアルタイムでログを監視
journalctl --user -u backup-suite-high.service -f

# 過去24時間のログを表示
journalctl --user -u backup-suite-high.service --since "24 hours ago"

# エラーのみ表示
journalctl --user -u backup-suite-high.service -p err
```

## タイマー管理

### タイマーの状態確認

```bash
# 全タイマーの状態確認
systemctl --user list-timers backup-suite-*

# 特定タイマーの詳細状態
systemctl --user status backup-suite-high.timer
```

### 次回実行時刻の確認

```bash
systemctl --user list-timers
```

出力例：
```
NEXT                         LEFT          LAST                         PASSED       UNIT                         ACTIVATES
Tue 2024-01-02 02:00:00 JST  8h left       Mon 2024-01-01 02:00:00 JST  16h ago      backup-suite-high.timer      backup-suite-high.service
```

## トラブルシューティング

### タイマーが実行されない

```bash
# タイマーの状態確認
systemctl --user status backup-suite-high.timer

# タイマーの再起動
systemctl --user restart backup-suite-high.timer

# ユニットファイルのリロード
systemctl --user daemon-reload
```

### サービスが失敗する

```bash
# エラーログを確認
journalctl --user -u backup-suite-high.service -p err

# 手動でサービスを実行（デバッグ用）
systemctl --user start backup-suite-high.service

# サービスの状態確認
systemctl --user status backup-suite-high.service
```

### スケジュールの削除

```bash
# 全優先度を無効化
backup-suite schedule disable

# または特定優先度のみ無効化
backup-suite schedule disable --priority high
```

### 手動での無効化（必要な場合のみ）

```bash
# タイマーを停止
systemctl --user stop backup-suite-high.timer
systemctl --user disable backup-suite-high.timer

# ユニットファイルを削除
rm ~/.config/systemd/user/backup-suite-high.service
rm ~/.config/systemd/user/backup-suite-high.timer

# systemdをリロード
systemctl --user daemon-reload
```

## 実行時刻のカスタマイズ

タイマーファイルを直接編集して実行時刻を変更できます：

```bash
# タイマーファイルを編集
nano ~/.config/systemd/user/backup-suite-high.timer
```

```ini
[Timer]
# 午前3時30分に変更
OnCalendar=*-*-* 03:30:00
Persistent=true
```

変更後、systemdをリロード：

```bash
systemctl --user daemon-reload
systemctl --user restart backup-suite-high.timer
```

## OnCalendar形式の例

```ini
# 毎日午前3時
OnCalendar=*-*-* 03:00:00

# 毎週月曜午前2時
OnCalendar=Mon *-*-* 02:00:00

# 毎月15日午前2時
OnCalendar=*-*-15 02:00:00

# 毎週月・水・金曜午前2時
OnCalendar=Mon,Wed,Fri *-*-* 02:00:00

# 2時間ごと
OnCalendar=*-*-* 0/2:00:00
```

## セキュリティ上の注意

### パーミッション

ユニットファイルは適切なパーミッションで保護してください：

```bash
chmod 644 ~/.config/systemd/user/backup-suite-*.service
chmod 644 ~/.config/systemd/user/backup-suite-*.timer
```

### パスワード管理

暗号化バックアップを使用する場合、パスワードをユニットファイルに含めることは**推奨しません**。
代わりに以下の方法を使用してください：

1. **systemd credentials**（systemd 250+）
2. **環境変数ファイル**（`EnvironmentFile=`）
3. **Linux keyring統合**（将来実装予定）

例（環境変数ファイル使用）：

```bash
# 環境変数ファイルを作成
echo "BACKUP_PASSWORD=your_password" > ~/.config/backup-suite/env
chmod 600 ~/.config/backup-suite/env

# サービスファイルに追加
[Service]
EnvironmentFile=%h/.config/backup-suite/env
ExecStart=/usr/local/bin/backup-suite run --priority high --encrypt
```

## ベストプラクティス

1. **ログローテーション**: journaldのログサイズ設定
   ```bash
   # /etc/systemd/journald.conf
   SystemMaxUse=500M
   SystemKeepFree=1G
   ```

2. **リソース制限**: メモリ・CPU使用制限
   ```ini
   [Service]
   MemoryMax=1G
   CPUQuota=50%
   ```

3. **失敗時の通知**（systemd 240+）:
   ```ini
   [Service]
   OnFailure=backup-notify@%n.service
   ```

4. **テスト実行**: dry-runで動作確認
   ```bash
   backup-suite run --priority high --dry-run
   ```

5. **段階的導入**: Highから順に有効化
   ```bash
   backup-suite schedule enable --priority high
   # 1週間後、動作確認できたら
   backup-suite schedule enable --priority medium
   ```

## 設定ファイル例

`~/.config/backup-suite/config.toml`:

```toml
version = "1.0.0"

[backup]
destination = "/backup/storage"
auto_cleanup = true
keep_days = 30

[schedule]
enabled = true
high_frequency = "daily"
medium_frequency = "weekly"
low_frequency = "monthly"

[[targets]]
path = "/home/username/documents"
priority = "High"
category = "重要ドキュメント"
exclude_patterns = ["*.tmp", "*.log"]

[[targets]]
path = "/home/username/projects"
priority = "Medium"
category = "プロジェクト"
exclude_patterns = ["\\.git/.*", "node_modules/.*", "target/.*"]
```

## 参考リンク

- [systemd.timer man page](https://www.freedesktop.org/software/systemd/man/systemd.timer.html)
- [systemd.service man page](https://www.freedesktop.org/software/systemd/man/systemd.service.html)
- [systemd.time man page](https://www.freedesktop.org/software/systemd/man/systemd.time.html)
