# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2025-11-07

### Added

#### 基本機能
- 初回リリース
- `add` コマンド: バックアップ対象追加（重複登録防止機能付き）
- `list` / `ls` コマンド: 対象一覧表示
- `remove` コマンド: 対象削除
- `clear` コマンド: 一括削除
- `run` コマンド: バックアップ実行
  - `--dry-run` フラグ対応
  - `--priority` フィルタ対応
  - `--category` フィルタ対応
  - `--incremental` 増分バックアップ対応
  - `--encrypt` AES-256-GCM暗号化対応
  - `--password` パスワード指定
  - `--generate-password` 強力なパスワード自動生成
  - `--compress` zstd/gzip圧縮対応
  - `--compress-level` 圧縮レベル調整
- `restore` コマンド: バックアップ復元（増分チェーン自動解決）
- `cleanup` コマンド: 古いバックアップ削除
- `history` コマンド: バックアップ履歴表示
- `status` コマンド: ステータス表示
- `dashboard` コマンド: 統計ダッシュボード
- `schedule` コマンド: 自動バックアップスケジュール管理（macOS launchctl対応）
- `config` コマンド: 設定管理（保存先・保持期間）
- `open` コマンド: バックアップディレクトリを開く
- `completion` コマンド: シェル補完スクリプト生成

#### セキュリティ機能
- **AES-256-GCM暗号化**: 認証付き暗号化による機密データ保護
- **Argon2鍵導出**: パスワードベースの安全な鍵生成
- **パスワードポリシー**: 強度評価・自動生成機能
- **監査ログ**: HMAC-SHA256による改ざん検出機能付きイベントログ
- **整合性検証**: SHA-256ハッシュによるバックアップ検証
- **パストラバーサル対策**: safe_join実装
- **セキュアメモリ管理**: zeroizeによる機密データ消去

#### 増分バックアップ
- SHA-256ハッシュベースの変更検出
- 親バックアップへの参照管理
- 増分チェーンの自動解決（復元時）
- 初回実行時の自動フルバックアップフォールバック

#### 圧縮機能
- zstd高速圧縮（レベル1-22、デフォルト3）
- gzip互換圧縮（レベル1-9）
- 統合処理パイプライン

#### ユーザビリティ
- 多言語対応（日本語・英語、LANG環境変数自動検出）
- インタラクティブファイル選択（skim統合）
- プログレスバー表示（indicatif）
- カラフルなテーブル表示
- 包括的なヘルプドキュメント（18個のオプション詳細説明）

#### 設定管理
- TOML設定ファイル管理
- 優先度システム（high/medium/low）
- カテゴリ分類機能
- カテゴリ別ディレクトリ構造（`backup_YYYYMMDD_HHMMSS/category/`）
- ファイル/ディレクトリ自動判定
- 除外パターン（正規表現・glob対応）

### Fixed

- **増分バックアップの重大バグ修正**: 未変更ファイルが誤ってコピーされる問題を解決
  - メタデータに全ファイルハッシュを保存（次回増分比較用）
  - 変更ファイルのみコピー（パフォーマンス維持）
- 初回増分バックアップ時の情報メッセージ追加
- パスワード警告の国際化対応（日英自動切り替え）

### Performance

- Bash版比: 53.6倍高速化（Rayon並列処理による）
- メモリ効率的なストリーミング処理
- 型安全性による実行時エラー削減
- 増分バックアップによる差分コピー最適化

### Documentation

- README.md: 包括的な使用方法・機能説明
- README.en.md: 英語版ドキュメント
- CHANGELOG.md: 詳細な変更履歴
- PUBLISHING.md: リリース手順
- docs/user/VERIFICATION_CHECKLIST.md: ユーザー向け動作確認チェックリスト（20項目）
- 包括的なインラインドキュメント・Rustdoc

### Tests

- 343テスト合格（2 ignored）
  - Unit tests: 135 passed
  - Integration tests: 16 passed
  - Audit tests: 13 passed
  - Incremental tests: 4 passed
  - Integrity tests: 5 passed
  - Nonce verification: 5 passed
  - Phase 2 integration: 9 passed
  - Property tests: 14 passed
  - Crypto property tests: 10 passed
  - Security property tests: 13 passed
  - Security tests: 23 passed
  - Doc tests: 96 passed

## [Unreleased]

### Planned

- リモートバックアップ対応（SSH/S3/WebDAV）
- WebUI実装（ブラウザベース管理画面）
- プラグインシステム（カスタムハンドラー拡張）
- クラウドバックアップ対応（AWS S3/Google Cloud Storage/Azure Blob）
- スナップショット機能（Btrfs/ZFS統合）
- データ重複排除（deduplication）
- マルチバージョンバックアップ（Git-like履歴管理）
