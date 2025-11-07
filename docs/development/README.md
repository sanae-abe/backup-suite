# 開発者ドキュメント

backup-suiteの開発者向けドキュメントインデックス

## 🎯 v1.0.0 主要機能

### 🔒 セキュリティ
- **AES-256-GCM暗号化**（認証付き暗号化）
- **Argon2鍵導出**（NIST SP 800-63B準拠）
- **パスワード自動生成**（強力なパスワード）
- **監査ログ**（HMAC-SHA256改ざん検出）
- **整合性検証**（SHA-256ハッシュ）

### 📦 バックアップ機能
- **増分バックアップ**（SHA-256ハッシュ変更検出、90%時間削減、85%容量削減）
- **優先度別管理**（high/medium/low）
- **自動スケジューリング**（macOS launchctl/Linux systemd）

### 🗜️ 圧縮
- **Zstd圧縮**（高速・高圧縮率、レベル1-22、デフォルト3）
- **Gzip圧縮**（互換性重視、レベル1-9）

### 🌍 ユーザビリティ
- **多言語対応**（日本語・英語、LANG環境変数自動検出）
- **インタラクティブUI**（skim統合ファイル選択）
- **プログレスバー**（indicatif）
- **カラフルテーブル表示**

## 📁 ドキュメント構成

### 🏗️ アーキテクチャ

**[architecture/ARCHITECTURE.md](architecture/ARCHITECTURE.md)** (33KB)
- システム全体設計・モジュール構成
- 暗号化・圧縮・並列処理アーキテクチャ
- パフォーマンス最適化戦略

### 🧪 テスト

**ディレクトリ**: [testing/](testing/)

| ドキュメント | 説明 | サイズ |
|-------------|------|--------|
| [README.md](testing/README.md) | テスト全体概要 | 14KB |
| [TESTING_GUIDE.md](testing/TESTING_GUIDE.md) | テスト実行ガイド | 11KB |
| [TESTING_SUMMARY.md](testing/TESTING_SUMMARY.md) | テスト戦略サマリー | 13KB |

**ガイド**: [testing/guides/](testing/guides/)
- [quick-reference.md](testing/guides/quick-reference.md) - クイックリファレンス (9.4KB)
- [implementation.md](testing/guides/implementation.md) - 実装ガイド (22KB)
- [automation.md](testing/guides/automation.md) - 自動化戦略 (47KB)

**テストタイプ (v1.0.0時点)**:
- 単体テスト (135 passed)
- 統合テスト (16 passed)
- 監査テスト (13 passed)
- 増分バックアップテスト (4 passed)
- 整合性検証テスト (5 passed)
- Nonce検証テスト (5 passed)
- Phase 2統合テスト (9 passed)
- Property tests (14 passed)
- Crypto property tests (10 passed)
- Security property tests (13 passed)
- セキュリティテスト (23 passed)
- Doc tests (96 passed)
- **合計**: 343 tests passed (2 ignored)
- ベンチマーク (criterion)

### 🔒 セキュリティ

**ディレクトリ**: [security/](security/)

| ドキュメント | 説明 | サイズ |
|-------------|------|--------|
| [README.md](security/README.md) | セキュリティナビゲーション | 16KB |
| [audit-report.md](security/audit-report.md) | 監査レポート | 11KB |
| [quick-reference.md](security/quick-reference.md) | クイックリファレンス | 8.5KB |
| [checklist.md](security/checklist.md) | 統合チェックリスト | 19KB |
| [delivery-summary.md](security/delivery-summary.md) | 納品サマリー | 13KB |

**セキュリティ機能 (v1.0.0実装完了)**:
- AES-256-GCM暗号化（認証付き暗号化）
- Argon2鍵導出（NIST SP 800-63B準拠）
- パスワードポリシー（強度評価・自動生成）
- 監査ログ（HMAC-SHA256改ざん検出）
- 整合性検証（SHA-256ハッシュ）
- パストラバーサル対策（safe_join実装）
- メモリ安全性（zeroize機密データ消去）

### 🎨 UI/UX

**ディレクトリ**: [ui-ux/](ui-ux/)

**CLI戦略**:
- [cli-strategy.md](ui-ux/cli-strategy.md) - CLI/UX改善戦略 (62KB)

**ヘルプシステム**: [ui-ux/help/](ui-ux/help/)
- [quick-reference.md](ui-ux/help/quick-reference.md) - クイックリファレンス (7.3KB)
- [implementation.md](ui-ux/help/implementation.md) - 実装サマリー (9.6KB)
- [maintenance.md](ui-ux/help/maintenance.md) - メンテナンスガイド (11KB)

## 🚀 クイックスタート

### 開発環境セットアップ
```bash
# Rust 1.70.0+ (MSRV)
rustc --version

# 依存関係インストール
cargo build

# テスト実行
cargo test --all-features

# Clippy + フォーマット
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all
```

### よく使うコマンド
```bash
# 全テスト実行
cargo test --all-features

# セキュリティテスト
cargo test --test security_tests

# プロパティテスト
cargo test --test proptest_crypto
cargo test --test proptest_security

# ベンチマーク
cargo bench

# カバレッジ (要tarpaulin)
cargo tarpaulin --all-features --out Xml --output-dir coverage/
```

## 📊 プロジェクト統計（v1.0.0）

- **言語**: Rust 1.70+ (MSRV)
- **テスト数**: 343 tests passed (2 ignored)
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
- **セキュリティ監査**: cargo-audit, cargo-deny
- **CI/CD**: GitHub Actions (フォーマット、Lint、テスト、セキュリティ、依存関係監査)
- **ドキュメント**: 日英両対応（README.md/README.en.md、CHANGELOG.md/CHANGELOG.en.md）
- **パフォーマンス**: Bash版比53.6倍高速化（Rayon並列処理）

## 🔗 関連リンク

- [プロジェクトREADME](../../README.md)
- [CHANGELOG](../../CHANGELOG.md)
- [PUBLISHING.md](../../PUBLISHING.md) - リリース手順
- [deny.toml](../../deny.toml) - 依存関係監査設定

## 📝 ドキュメント更新

ドキュメントの追加・更新時は：
1. 適切なカテゴリに配置
2. このREADMEのインデックスを更新
3. ファイルサイズ・更新日を記載
