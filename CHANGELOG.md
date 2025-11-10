# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Security - 🔒 セキュリティ強化（バリデーション）

#### パス検証の徹底
- **add コマンド**: `safe_join()` + `validate_path_safety()` による二段階検証
- **remove コマンド**: パストラバーサル対策と確認プロンプト追加
- **config set-destination**: 保存先パスの安全性検証とディレクトリ作成時の権限チェック
- **Smart機能**: analyze, suggest-exclude, auto-configure の全パス入力を検証

#### 確認プロンプトの追加
- **remove コマンド**: 削除前の明示的な確認（dialoguer使用、ファイル名表示）
- **clear --all コマンド**: 全削除の危険性を警告する確認プロンプト
- **cleanup コマンド**: 古いバックアップ削除前の日数確認（実行前の早期確認で効率化）

#### 型安全性の強化（clap 4.x enum対応）
- **Priority**: String → clap::ValueEnum（型安全な優先度指定）
- **CompressionType**: String → clap::ValueEnum（型安全な圧縮タイプ指定）
- 不正な値の即座検出（コンパイル時・実行時両方で保証）

#### バリデーション範囲の厳格化
- **cleanup --days**: 1-3650 の範囲検証（0および3650超を拒否）
- **config set-keep-days**: 1-3650 の範囲検証
- **run --compress-level**: 圧縮タイプ別の範囲検証
  - zstd: 1-22（0および23以上を拒否）
  - gzip: 1-9（0および10以上を拒否）
- **smart detect --days**: 1-365 の範囲検証
- **smart suggest-exclude --confidence**: 0.0-1.0 の範囲検証
- **smart auto-configure --max-depth**: 1-255 の範囲検証（0は「サブディレクトリなし」として安全に処理）

### Tests - ✅ テスト強化

#### セキュリティ統合テスト（24件追加）
- **パストラバーサル攻撃テスト**: add, remove, config, AI各コマンドで検証
- **絶対パステスト**: shallow absolute path（/etc/passwd等）の拒否確認
- **シンボリックリンク攻撃テスト**: restore コマンドの `safe_open()` 検証
- **enum型安全性テスト**: Priority, CompressionType の不正値拒否
- **バリデーション範囲テスト**: 全パラメータの境界値検証（min/max）
- **正常パステスト**: 正当なパスが正しく受け入れられることを確認

#### テスト実績
- 24テスト合格、2テスト無視（terminal依存）
- カバレッジ: パストラバーサル対策、確認プロンプト、型安全性、バリデーション範囲

### Added - 🤖 Smart機能（Phase 1: 軽量ML機能）

#### 異常検知エンジン
- **統計的異常検知**: Z-scoreによるバックアップサイズ異常検知（< 1ms/100件履歴）
- **ディスク容量予測**: 線形回帰によるディスク枯渇予測（< 5ms）
- **失敗パターン分析**: カテゴリ別・時刻別の頻発エラー検出

#### インテリジェント推奨エンジン
- **ファイル重要度評価**: ルールベーススコアリング（~50μs/ファイル）
- **バックアップ対象提案**: AI駆動の自動推奨システム
- **除外パターン検出**: 一時ファイル・キャッシュの自動検出・提案

#### CLI統合
- `backup-suite smart detect`: 異常検知レポート生成（--days, --detailed オプション）
- `backup-suite smart analyze`: ファイル重要度分析（--filter, --detailed オプション）
- `backup-suite smart suggest-exclude`: 除外パターン推奨（--apply, --min-size オプション）
- `backup-suite smart auto-configure`: AI駆動の自動設定（--interactive, --dry-run オプション）

#### セキュリティとプライバシー
- **完全オフライン動作**: 外部APIコール・クラウドサービス不要
- **機密情報保護**: パスワード・暗号鍵はSmart機能で処理しない
- **パストラバーサル対策**: すべてのファイルアクセスで安全性検証

#### パフォーマンス実績
- 異常検知: 目標5ms → 実測< 1ms（500%達成）
- ファイル分析: 目標10秒 → 実測~8秒（125%達成）
- 除外パターン検出: 目標10秒 → 実測~5秒（200%達成）

#### テスト・品質
- **63件のテスト**: 単体テスト（Smart機能）全成功
- **Property-based Testing**: proptest活用で境界値・エッジケース網羅
- **統合テスト**: End-to-Endワークフロー検証済み
- **AIモジュールカバレッジ**: 76.39%（目標95%に向けて改善中）

#### ドキュメント
- `docs/smart/features.md`: 包括的なSmart機能ガイド（使用例・アーキテクチャ）
- `docs/smart/development/implementation-plan.md`: 詳細な実装計画書（Phase 1/2ロードマップ）
- `docs/smart/development/recommendation-engine.md`: 推奨エンジン実装報告
- `docs/smart/development/test-report.md`: テストカバレッジ報告
- `docs/smart/development/benchmark-report.md`: ベンチマーク結果
- `docs/smart/manual-test.md`: Smart機能手動テスト手順
- `docs/smart/README.md`: Smart機能ドキュメントインデックス
- README.md/README.en.md: Smart機能セクション追加（インストール方法・使用例）

#### 技術スタック
- **statrs 0.17**: 統計計算（Z-score、線形回帰、標準偏差）
- **rayon 1.11**: 並列ファイル評価（高速化）
- **walkdir 2.5**: ディレクトリ走査
- **Feature Gate**: `--features smart` でオプション機能として実装

### Performance
- AI異常検知: < 1ms（100件履歴）
- AI重要度評価: ~50μs/ファイル
- AIディレクトリ分析: ~8秒（10,000ファイル）
- AI除外パターン検出: ~5秒（10,000ファイル）

### Planned - Phase 2
- Ollama統合（自然言語バックアップ設定）
- AI駆動レポート生成（自然言語サマリー）
- インタラクティブアシスタント
- Graceful degradation（Ollama未インストール時の対応）

---

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
