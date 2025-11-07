# 開発者ドキュメント

backup-suiteの開発者向けドキュメントインデックス

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

**テストタイプ**:
- 単体テスト (86件)
- 統合テスト (16件)
- Property-based Testing (proptest: 38件)
- セキュリティテスト (23件)
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

**セキュリティ機能**:
- AES-256-GCM暗号化
- Argon2鍵導出
- パストラバーサル対策
- メモリ安全性 (zeroize)

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

## 📊 プロジェクト統計

- **言語**: Rust 1.70+
- **テスト数**: 158+ (86 unit + 16 integration + 56 property)
- **セキュリティ監査**: cargo-audit, cargo-deny
- **CI/CD**: GitHub Actions (フォーマット、Lint、テスト、セキュリティ)
- **ドキュメント**: 16ファイル、300KB+

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
