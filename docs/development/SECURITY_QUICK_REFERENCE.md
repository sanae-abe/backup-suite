# Security Quick Reference

**目的**: SECURITY_AUDIT_AND_IMPLEMENTATION_PLAN.md の即座実行ガイド
**最終更新**: 2025-11-05

---

## 🚨 緊急対応（即座実施）

### 重大脆弱性トップ3

#### 1. パストラバーサル脆弱性（CVSS 8.6）
**場所**: `src/core/backup.rs:81-82`

```bash
# 即座実行
cd /Users/sanae.abe/projects/backup-suite
mkdir -p src/security
```

**修正コード**:
```rust
// src/security/path_utils.rs（新規作成）
use std::path::{Path, PathBuf, Component};
use anyhow::Result;

pub fn safe_join(base: &Path, child: &Path) -> Result<PathBuf> {
    let normalized: PathBuf = child
        .components()
        .filter(|c| !matches!(c, Component::ParentDir | Component::RootDir))
        .collect();

    let result = base.join(&normalized);
    let canonical_result = result.canonicalize()?;
    let canonical_base = base.canonicalize()?;

    if !canonical_result.starts_with(&canonical_base) {
        return Err(anyhow::anyhow!(
            "パストラバーサル攻撃を検出: {:?} は {:?} の外部",
            child, base
        ));
    }

    Ok(result)
}
```

**適用箇所**: `src/core/backup.rs:81-82`
```rust
// 修正前
let relative = source.strip_prefix(&target.path).unwrap();
let dest = backup_dir.join(relative);

// 修正後
use crate::security::safe_join;

let relative = source.strip_prefix(&target.path)
    .context("パスのstrip_prefixに失敗")?;
let dest = safe_join(&backup_dir, &relative)?;
```

#### 2. シンボリックリンク攻撃（CVSS 7.8）
**修正コード**:
```rust
// src/security/file_ops.rs（新規作成）
use std::path::Path;
use anyhow::{Result, Context};

pub fn safe_copy(source: &Path, dest: &Path) -> Result<u64> {
    // シンボリックリンクチェック
    let metadata = std::fs::symlink_metadata(source)
        .context("ソースファイルのメタデータ取得失敗")?;

    if metadata.is_symlink() {
        return Err(anyhow::anyhow!(
            "セキュリティ: シンボリックリンクのコピーは禁止されています: {:?}",
            source
        ));
    }

    std::fs::copy(source, dest).map_err(Into::into)
}
```

**適用箇所**: `src/core/backup.rs:122`
```rust
// 修正前
match std::fs::copy(source, dest) {

// 修正後
use crate::security::safe_copy;

match safe_copy(source, dest) {
```

#### 3. 権限チェック不在（CVSS 7.2）
**修正コード**:
```rust
// src/security/permissions.rs（新規作成）
use std::path::Path;
use anyhow::{Result, Context};

pub fn check_read_permission(path: &Path) -> Result<()> {
    let metadata = std::fs::metadata(path)
        .with_context(|| format!("メタデータ取得失敗: {:?}", path))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = metadata.permissions().mode();
        if mode & 0o400 == 0 {
            return Err(anyhow::anyhow!("読み取り権限がありません: {:?}", path));
        }
    }

    Ok(())
}
```

---

## ⚡ 即座実行コマンド

### セキュリティツールセットアップ
```bash
cd /Users/sanae.abe/projects/backup-suite

# セキュリティツールインストール
cargo install cargo-audit
cargo install cargo-deny
cargo install cargo-geiger

# 初回セキュリティスキャン
cargo audit
cargo clippy -- \
    -W clippy::unwrap_used \
    -W clippy::expect_used \
    -W clippy::panic
```

### セキュリティ設定ファイル作成
```bash
# .cargo/config.toml作成
mkdir -p .cargo
cat > .cargo/config.toml << 'EOF'
[target.'cfg(all())']
rustflags = [
    "-D", "warnings",
    "-D", "clippy::unwrap_used",
    "-D", "clippy::expect_used",
]
EOF

# deny.toml作成
cat > deny.toml << 'EOF'
[advisories]
vulnerability = "deny"
unmaintained = "warn"
yanked = "deny"

[licenses]
unlicensed = "deny"
allow = ["MIT", "Apache-2.0", "BSD-3-Clause"]

[bans]
multiple-versions = "warn"
wildcards = "deny"
EOF
```

---

## 📋 Phase 1実装チェックリスト（1週間）

### Day 1-2: パストラバーサル対策
- [ ] `src/security/path_utils.rs` 作成
- [ ] `safe_join()` 関数実装
- [ ] `src/core/backup.rs:81-82` 修正
- [ ] テスト `tests/security_path_traversal.rs` 作成
- [ ] テスト実行・通過確認

### Day 3-4: シンボリックリンク対策
- [ ] `src/security/file_ops.rs` 作成
- [ ] `safe_copy()` 関数実装
- [ ] `src/core/backup.rs:122` 修正
- [ ] テスト `tests/security_symlink.rs` 作成
- [ ] テスト実行・通過確認

### Day 5-6: 権限チェック強化
- [ ] `src/security/permissions.rs` 作成
- [ ] `check_read_permission()` 実装
- [ ] `check_write_permission()` 実装
- [ ] 全モジュールへの統合
- [ ] テスト作成・実行

### Day 7: カスタムエラー型・統合テスト
- [ ] `src/error.rs` 強化
- [ ] `BackupError` enum 定義
- [ ] 全モジュールのエラー型統一
- [ ] 統合セキュリティテスト実行
- [ ] Clippy warnings 0件確認

---

## 🧪 セキュリティテスト実行方法

### 基本テスト
```bash
# セキュリティテストのみ実行
cargo test security_ --release -- --nocapture

# パストラバーサルテスト
cargo test test_path_traversal -- --nocapture

# シンボリックリンクテスト
cargo test test_symlink_attack -- --nocapture
```

### 包括的セキュリティチェック
```bash
# 依存関係脆弱性スキャン
cargo audit

# セキュリティリント
cargo clippy -- \
    -D clippy::unwrap_used \
    -D clippy::expect_used \
    -D clippy::panic \
    -D clippy::security

# unsafeコード検出
cargo geiger
```

---

## 📊 セキュリティKPI追跡

### 現状（2025-11-05）
```
セキュリティスコア: 🔴 5/10
├─ 入力検証: 🔴 3/10
├─ パス処理: 🔴 4/10
├─ 権限管理: 🟡 5/10
├─ エラーハンドリング: 🟡 6/10
├─ 暗号化: 🔴 0/10
├─ 監査ログ: 🔴 2/10
└─ 依存関係: 🟢 7/10

重大脆弱性: 3件（即座対応必要）
```

### 目標（Phase 1完了後）
```
セキュリティスコア: 🟡 7/10
├─ 入力検証: 🟢 8/10
├─ パス処理: 🟢 9/10
├─ 権限管理: 🟢 8/10
├─ エラーハンドリング: 🟢 8/10
├─ 暗号化: 🔴 0/10（Phase 3で実装）
├─ 監査ログ: 🟡 5/10（Phase 2で実装）
└─ 依存関係: 🟢 9/10

重大脆弱性: 0件
```

---

## 🔧 トラブルシューティング

### Q1: cargo auditで脆弱性検出された場合
```bash
# 詳細確認
cargo audit --json | jq

# 依存関係更新
cargo update

# 再スキャン
cargo audit
```

### Q2: Clippyで多数の警告が出る場合
```bash
# 段階的修正
cargo clippy --fix --allow-dirty

# 手動確認が必要な項目のみ表示
cargo clippy -- -D warnings
```

### Q3: テストが失敗する場合
```bash
# 詳細ログ付き実行
RUST_BACKTRACE=1 cargo test security_ -- --nocapture

# 単一テスト実行
cargo test test_path_traversal -- --exact --nocapture
```

---

## 📚 セキュリティ学習リソース

### 必読ドキュメント
1. **OWASP Top 10**: https://owasp.org/www-project-top-ten/
2. **Rust Security Guidelines**: https://anssi-fr.github.io/rust-guide/
3. **Secure Rust Guidelines**: https://doc.rust-lang.org/nomicon/

### Rustセキュリティベストプラクティス
1. **unwrap()の回避**: `?` オペレータ使用
2. **パニックの回避**: `Result` 型での適切なエラー伝播
3. **整数オーバーフロー**: `checked_add()` 等の使用
4. **unsafeの最小化**: 原則使用しない
5. **依存関係の定期更新**: 月次 `cargo update`

---

## 🚀 次のステップ

### 即座実施（今日）
1. セキュリティツールインストール
2. 初回脆弱性スキャン実行
3. `src/security/` ディレクトリ作成

### Week 1（Phase 1）
1. パストラバーサル対策実装
2. シンボリックリンク対策実装
3. 権限チェック強化
4. カスタムエラー型導入

### Week 2-6（Phase 2-5）
- 詳細は `SECURITY_AUDIT_AND_IMPLEMENTATION_PLAN.md` 参照

---

## 📞 サポート

### 質問・相談
- セキュリティ懸念: security@backup-suite.dev
- 技術的質問: support@backup-suite.dev

### 緊急セキュリティインシデント
- 重大脆弱性発見時は即座に報告
- PGP公開鍵: https://backup-suite.dev/security.asc

---

**重要**: このドキュメントは `SECURITY_AUDIT_AND_IMPLEMENTATION_PLAN.md` の簡易版です。詳細な実装ガイドは本編を参照してください。
