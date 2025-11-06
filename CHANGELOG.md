# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2025-11-04

### Added

- 初回リリース
- `add` コマンド: バックアップ対象追加
- `list` / `ls` コマンド: 対象一覧表示
- `remove` コマンド: 対象削除
- `clear` コマンド: 一括削除
- `run` コマンド: バックアップ実行
  - `--dry-run` フラグ対応
  - `--priority` フィルタ対応
- `status` コマンド: ステータス表示
- `open` コマンド: バックアップディレクトリを開く
- `version` コマンド: バージョン情報表示
- TOML設定ファイル管理
- Rayon並列処理（53.6倍高速化達成）
- 優先度システム（high/medium/low）
- カテゴリ分類機能
- ファイル/ディレクトリ自動判定

### Performance

- Bash版比: 53.6倍高速化（並列処理による）
- メモリ効率的なストリーミング処理
- 型安全性による実行時エラー削減

### Documentation

- README.md: 基本的な使用方法
- CHANGELOG.md: 変更履歴
- インラインドキュメント: コマンドヘルプ

## [Unreleased]

### Added
- Phase 5: 品質保証・リリース準備機能
  - CI/CD強化
    - セキュリティ監査ワークフロー (`security.yml`)
      - cargo-audit / cargo-deny 統合
      - 依存関係レビュー
      - 脆弱性スキャン
      - シークレットスキャン (gitleaks)
      - Clippy セキュリティリント
      - ライセンスコンプライアンスチェック
    - パフォーマンスベンチマークワークフロー (`benchmark.yml`)
      - Criterion統合
      - パフォーマンス回帰テスト
      - メモリプロファイリング (valgrind)
      - CPUプロファイリング (flamegraph)
    - テストカバレッジ追跡ワークフロー (`coverage.yml`)
      - cargo-tarpaulin統合
      - Codecov連携
      - 80%カバレッジ閾値チェック
      - ドキュメントカバレッジ
    - リリース自動化ワークフロー (`release.yml`)
      - マルチプラットフォームビルド (Linux, macOS, Windows)
      - 自動CHANGELOG生成
      - GitHub Release自動作成
      - crates.io自動公開
      - Dockerイメージビルド
  - ベンチマーク拡張
    - セキュリティベンチマーク (`security_benchmark.rs`)
      - safe_join パフォーマンステスト
      - パストラバーサル検出ベンチマーク
      - 権限チェックベンチマーク
    - UIベンチマーク (`ui_benchmark.rs`)
      - プログレスバー描画パフォーマンス
      - テーブル表示ベンチマーク
      - カラー出力パフォーマンス
    - 統合ベンチマーク (`integration_benchmark.rs`)
      - 実ワークロードシミュレーション
      - 並列処理vs順次処理比較
      - 増分バックアップベンチマーク
  - 品質チェックツール
    - 包括的品質チェックスクリプト (`check_quality.sh`)
      - 13種類の品質チェック
      - 品質スコア自動算出
      - レポート生成機能
    - Clippy設定ファイル (`.clippy.toml`)
      - セキュリティリント強化
      - パフォーマンスチェック設定
      - コード品質基準定義
    - rustfmt設定ファイル (`rustfmt.toml`)
      - プロジェクト統一コードスタイル
      - Edition 2021対応
      - 最適な可読性設定
  - リリース準備
    - Dockerサポート (`Dockerfile`, `.dockerignore`)
      - マルチステージビルド
      - 最小サイズイメージ
      - 非rootユーザー実行
    - リリース準備スクリプト (`prepare_release.sh`)
      - 自動バージョンアップ
      - CHANGELOG自動更新
      - Gitタグ自動作成

### Changed
- CHANGELOG形式を [Keep a Changelog](https://keepachangelog.com/ja/1.0.0/) に準拠

### Planned (Phase 2-4)

- `history` コマンド: バックアップ履歴表示
- `restore` コマンド: バックアップ復元
- `cleanup` コマンド: 古いバックアップ削除
- `enable` / `disable` コマンド: 自動バックアップ制御
- `dashboard` コマンド: 統計ダッシュボード
- ログファイル管理
- 進捗表示バー（indicatif統合）
- パストラバーサル対策 (Phase 1)
- 権限チェック強化 (Phase 1)
- カスタムエラー型 (Phase 1)
- exclude_patterns実装 (Phase 2)
- 設定バリデーション強化 (Phase 2)
- インタラクティブUI (Phase 3)
- I/O最適化 (Phase 3)
- アクセシビリティ対応 (Phase 3)

### Added (2025-11-05)
- **カテゴリ別フォルダ分け機能**
  - `--category` オプションによるカテゴリ別バックアップ実行
  - `Config::filter_by_category()` メソッド追加
  - 新しいディレクトリ構成: `backup_YYYYMMDD_HHMMSS/category/`
    - カテゴリ指定時: `backup_20251105_140500/system/`
    - カテゴリ未指定時: `backup_20251105_140500/all/`
  - シンプルで階層の浅いディレクトリ構成による管理性向上

- **暗号化・圧縮機能実装**
  - AES-256-GCM認証付き暗号化
  - Argon2パスワードベースキー導出
  - zstd高速圧縮（レベル1-22）
  - gzip互換圧縮（レベル1-9）
  - 統合処理パイプライン
  - CLI オプション: `--encrypt`, `--password`, `--compress`, `--compress-level`
  - メモリセーフ設計（zeroize統合）
  - ストリーミング処理対応

- **ベンチマーク追加**
  - `crypto_benchmark.rs`: 暗号化・復号化性能
  - 暗号化/復号化ベンチマーク（1KB-1000KB）
  - 圧縮/展開ベンチマーク（zstd/gzip）
  - キー導出性能ベンチマーク

### Changed (2025-11-05)
- **CLI の改善**
  - `version` サブコマンドを削除（`--version` フラグに統一）
  - ヘルプ表示を日本語に統一（`backup-suite help` と `backup-suite --help` の両方で日本語カスタムヘルプを表示）
  - より標準的なCLI慣習に準拠
  - **暗号化・圧縮のヘルプドキュメント拡充**
    - `run` コマンドのヘルプに暗号化・圧縮オプション詳細を追加
    - 3つの実用例を追加（暗号化のみ、圧縮のみ、両方）
    - AES-256-GCM、zstd/gzipの説明を明記

- **Config管理機能強化**
  - `config open` コマンド実装 (`src/main.rs:1101-1133`)
    - 設定ファイルをデフォルトエディタで開く機能
    - クロスプラットフォーム対応（macOS/Linux/Windows）
    - 環境変数 $EDITOR, $VISUAL の検出
    - プラットフォーム別デフォルトエディタ（macOS: open, Linux: nano, Windows: notepad）

### Fixed

- ✅ ストリーム暗号化テスト問題を解決（全テスト合格）
- ✅ 全79単体テスト・16統合テスト・68 doctest合格

### Future Considerations

- incremental backup機能
- リモートバックアップ対応
- WebUI実装
- プラグインシステム
- クラウドバックアップ対応
