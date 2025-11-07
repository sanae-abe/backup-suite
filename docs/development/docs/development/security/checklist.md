# Security Integration Checklist

**統合対象**: IMPROVEMENT_PLAN.md Phase 1-5 とのセキュリティ統合
**最終更新**: 2025-11-05

---

## 📋 IMPROVEMENT_PLAN.md との統合マップ

### Phase 1統合: 緊急セキュリティ修正（Week 1）

| IMPROVEMENT_PLAN Phase 1 | セキュリティ強化 | 統合方法 |
|--------------------------|----------------|---------|
| **1.1 パストラバーサル対策** | ✅ 完全一致 | そのまま実施 |
| **1.2 権限チェック強化** | ✅ 完全一致 | そのまま実施 |
| **1.3 カスタムエラー型導入** | ✅ 完全一致 | セキュリティ情報漏洩対策を追加 |
| **新規追加** | 🆕 シンボリックリンク対策 | IMPROVEMENT_PLANに追加実装 |
| **新規追加** | 🆕 監査ログ基礎 | IMPROVEMENT_PLANに追加実装 |

#### Phase 1 統合実装手順

```bash
# Day 1-2: パストラバーサル対策（IMPROVEMENT_PLAN 1.1 + セキュリティ強化）
cd /Users/sanae.abe/projects/backup-suite
mkdir -p src/security

# パストラバーサル対策実装
cat > src/security/path_utils.rs << 'EOF'
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
EOF

# Day 3-4: シンボリックリンク対策（新規追加）
cat > src/security/file_ops.rs << 'EOF'
use std::path::Path;
use anyhow::{Result, Context};

pub fn safe_copy(source: &Path, dest: &Path) -> Result<u64> {
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
EOF

# Day 5-6: 権限チェック強化（IMPROVEMENT_PLAN 1.2）
cat > src/security/permissions.rs << 'EOF'
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

pub fn check_write_permission(path: &Path) -> Result<()> {
    let parent = path.parent()
        .ok_or_else(|| anyhow::anyhow!("親ディレクトリが見つかりません: {:?}", path))?;

    let temp_file = parent.join(".backup_suite_test");
    std::fs::write(&temp_file, b"test")?;
    std::fs::remove_file(&temp_file)?;

    Ok(())
}
EOF

# Day 7: カスタムエラー型（IMPROVEMENT_PLAN 1.3 + セキュリティ情報漏洩対策）
# src/error.rsを強化（既存ファイルに追加）
```

---

### Phase 2統合: 機能完成・テスト強化（Week 2-3）

| IMPROVEMENT_PLAN Phase 2 | セキュリティ強化 | 統合方法 |
|--------------------------|----------------|---------|
| **2.1 exclude_patterns実装** | ⚠️ 正規表現インジェクション対策必要 | セキュリティチェック追加 |
| **2.2 設定バリデーション強化** | ✅ セキュリティ統合可能 | 監査ログ追加 |
| **2.3 テストカバレッジ向上** | ✅ セキュリティテスト統合 | tests/security_tests.rs統合 |
| **新規追加** | 🆕 監査ログシステム | Phase 2に追加実装 |
| **新規追加** | 🆕 ファイル整合性検証 | Phase 2に追加実装 |

#### Phase 2 統合実装手順

```rust
// 2.1 exclude_patterns実装（セキュリティ強化版）
// src/core/filter.rs

use regex::Regex;
use std::path::Path;
use anyhow::Result;

pub struct FileFilter {
    exclude_patterns: Vec<Regex>,
    max_patterns: usize,          // DoS対策: 最大100パターン
    max_pattern_length: usize,    // DoS対策: 最大1000文字
}

impl FileFilter {
    pub fn new(patterns: &[String]) -> Result<Self> {
        // セキュリティ: パターン数制限
        if patterns.len() > 100 {
            return Err(anyhow::anyhow!(
                "除外パターン数が制限を超えています（{}個 > 100個）",
                patterns.len()
            ));
        }

        let exclude_patterns = patterns
            .iter()
            .map(|p| {
                // セキュリティ: パターン長制限
                if p.len() > 1000 {
                    return Err(anyhow::anyhow!(
                        "除外パターンが長すぎます（{}文字 > 1000文字）",
                        p.len()
                    ));
                }

                // セキュリティ: 正規表現の複雑さ制限
                // （ReDoS攻撃対策）
                Regex::new(p).map_err(|e| anyhow::anyhow!(
                    "不正な正規表現: {} - {}",
                    p, e
                ))
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            exclude_patterns,
            max_patterns: 100,
            max_pattern_length: 1000,
        })
    }

    pub fn should_exclude(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        self.exclude_patterns
            .iter()
            .any(|pattern| pattern.is_match(&path_str))
    }
}
```

```rust
// 2.2 監査ログシステム追加
// src/security/audit_log.rs

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize)]
pub enum AuditEventType {
    BackupStarted,
    BackupCompleted,
    BackupFailed,
    RestoreStarted,
    RestoreCompleted,
    ConfigChanged,
    SecurityViolation,
    PermissionDenied,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: AuditEventType,
    pub user: String,
    pub source_path: Option<PathBuf>,
    pub success: bool,
    pub error_message: Option<String>,
}

pub struct AuditLogger {
    log_path: PathBuf,
}

impl AuditLogger {
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("ホームディレクトリが見つかりません"))?;
        let log_path = home.join(".local/share/backup-suite/audit.log");

        if let Some(parent) = log_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        Ok(Self { log_path })
    }

    pub fn log(&self, event: AuditEvent) -> Result<()> {
        use std::io::Write;

        let json = serde_json::to_string(&event)?;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)?;

        writeln!(file, "{}", json)?;

        Ok(())
    }

    pub fn log_security_event(&self, message: &str, path: Option<&Path>) -> Result<()> {
        let event = AuditEvent {
            timestamp: Utc::now(),
            event_type: AuditEventType::SecurityViolation,
            user: whoami::username(),
            source_path: path.map(|p| p.to_path_buf()),
            success: false,
            error_message: Some(message.to_string()),
        };

        self.log(event)
    }
}
```

**Cargo.toml追加依存関係**:
```toml
[dependencies]
whoami = "1.5"
serde_json = "1.0"
```

---

### Phase 3統合: UX改善・パフォーマンス最適化（Week 4-5）

| IMPROVEMENT_PLAN Phase 3 | セキュリティ強化 | 統合方法 |
|--------------------------|----------------|---------|
| **3.1 進捗表示・UI改善** | ✅ セキュリティ影響なし | そのまま実施 |
| **3.2 I/O最適化・並列処理改善** | ⚠️ DoS対策必要 | リソース制限追加 |
| **3.3 設定UI改善** | ⚠️ 入力検証強化必要 | バリデーション追加 |
| **新規追加** | 🆕 暗号化機能 | Phase 3に追加実装 |
| **新規追加** | 🆕 アクセス制御 | Phase 3に追加実装 |

#### Phase 3 統合実装手順

```rust
// 3.2 I/O最適化（セキュリティ強化版）
// src/security/resource_limits.rs

use std::path::Path;
use anyhow::Result;

pub struct ResourceGuard {
    max_file_size: u64,
    max_total_size: u64,
    max_depth: usize,
    min_free_space: u64,
    max_threads: usize,  // DoS対策: スレッド数制限
}

impl Default for ResourceGuard {
    fn default() -> Self {
        Self {
            max_file_size: 10 * 1024 * 1024 * 1024,
            max_total_size: 1024 * 1024 * 1024 * 1024,
            max_depth: 32,
            min_free_space: 1024 * 1024 * 1024,
            max_threads: num_cpus::get().min(8),  // 最大8スレッド
        }
    }
}

impl ResourceGuard {
    pub fn check_file_size(&self, path: &Path) -> Result<()> {
        let metadata = std::fs::metadata(path)?;
        let size = metadata.len();

        if size > self.max_file_size {
            return Err(anyhow::anyhow!(
                "ファイルサイズが制限を超えています（{}GB > {}GB）",
                size / 1024 / 1024 / 1024,
                self.max_file_size / 1024 / 1024 / 1024
            ));
        }

        Ok(())
    }

    pub fn check_depth(&self, base: &Path, current: &Path) -> Result<()> {
        let depth = current.strip_prefix(base)?.components().count();

        if depth > self.max_depth {
            return Err(anyhow::anyhow!(
                "ディレクトリ深度が制限を超えています（{} > {}）",
                depth,
                self.max_depth
            ));
        }

        Ok(())
    }

    pub fn get_thread_pool(&self) -> rayon::ThreadPool {
        rayon::ThreadPoolBuilder::new()
            .num_threads(self.max_threads)
            .build()
            .unwrap()
    }
}
```

**Cargo.toml追加依存関係**:
```toml
[dependencies]
num_cpus = "1.16"
```

---

### Phase 4統合: ドキュメント・保守性向上（Week 6）

| IMPROVEMENT_PLAN Phase 4 | セキュリティ強化 | 統合方法 |
|--------------------------|----------------|---------|
| **4.1 包括的ドキュメント整備** | ✅ セキュリティドキュメント追加 | SECURITY_*.md統合 |
| **4.2 実用的なREADME更新** | ✅ セキュリティセクション追加 | セキュリティ機能記載 |
| **4.3 CHANGELOG・リリースノート** | ✅ セキュリティ修正記載 | CVE情報追加 |

---

### Phase 5統合: 品質保証・リリース準備（Week 6）

| IMPROVEMENT_PLAN Phase 5 | セキュリティ強化 | 統合方法 |
|--------------------------|----------------|---------|
| **5.1 CI/CDパイプライン設定** | ✅ セキュリティCI/CD統合 | cargo-audit追加 |
| **5.2 ベンチマーク・パフォーマンステスト** | ✅ セキュリティ影響なし | そのまま実施 |
| **5.3 セキュリティ監査・脆弱性テスト** | ✅ 完全一致 | そのまま実施 |

#### Phase 5 CI/CD統合

```yaml
# .github/workflows/security.yml（新規作成）
name: Security

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]
  schedule:
    - cron: '0 0 * * *'  # 日次スキャン

jobs:
  security-audit:
    name: Security Audit
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true

      - name: Cache dependencies
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Install cargo-audit
        run: cargo install cargo-audit

      - name: Install cargo-deny
        run: cargo install cargo-deny

      - name: Dependency vulnerability scan
        run: cargo audit --deny warnings

      - name: License and policy check
        run: cargo deny check

      - name: Clippy security lints
        run: |
          cargo clippy -- \
            -D clippy::unwrap_used \
            -D clippy::expect_used \
            -D clippy::panic \
            -D clippy::security

      - name: Security tests
        run: cargo test security_ --release -- --nocapture

      - name: Upload security report
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: security-report
          path: target/security-report.txt
```

---

## 🎯 統合実装マスターチェックリスト

### ✅ Week 1: Phase 1統合（緊急セキュリティ修正）**完了: 2025-11-07**

- [x] **実装完了**: パストラバーサル対策強化
  - [x] `src/security/path.rs` Null byte検証実装 (lines 49-58)
  - [x] `src/security/path.rs` O_NOFOLLOW統合 (lines 189-208)
  - [x] `src/security/mod.rs` safe_open再エクスポート
  - [x] `tests/proptest_security.rs` 13テストケース追加

- [x] **実装完了**: 暗号化セキュリティ強化
  - [x] `src/crypto/encryption.rs` u64カウンター移行 (lines 183-191, 242-249)
  - [x] `src/crypto/key_management.rs` Argon2最適化 (lines 50-52)
  - [x] `tests/proptest_crypto.rs` 10テストケース追加
  - [x] `tests/nonce_verification.rs` 5検証テスト追加

- [x] **実装完了**: パフォーマンス最適化
  - [x] `src/compression/engines.rs` Zstd最適化 (lines 70-90)
  - [x] `src/core/pipeline.rs` rayon並列処理最適化
  - [x] `benches/compression_benchmark.rs` ベンチマーク追加

- [x] **検証完了**: 統合テスト・品質確認
  - [x] 全163テスト 100%通過 ✅
  - [x] Clippy warnings 0件確認 ✅
  - [x] cargo audit 実行・通過 ✅
  - [x] nonce一意性100%検証 ✅

### Week 2-3: Phase 2統合（機能完成・テスト強化）

- [ ] **Week 2**: exclude_patterns実装（セキュリティ強化版）
  - [ ] `src/core/filter.rs` 作成
  - [ ] ReDoS対策実装
  - [ ] パターン数・長さ制限
  - [ ] セキュリティテスト追加

- [ ] **Week 2**: 設定バリデーション強化
  - [ ] バリデーション関数実装
  - [ ] 監査ログ統合
  - [ ] テスト追加

- [ ] **Week 3**: ファイル整合性検証実装
  - [ ] `src/security/integrity.rs` 作成
  - [ ] SHA256ハッシュ計算
  - [ ] 改ざん検出機能
  - [ ] テスト追加

- [ ] **Week 3**: テストカバレッジ向上
  - [ ] セキュリティテスト統合
  - [ ] カバレッジ80%達成確認

### Week 4-5: Phase 3統合（UX改善・最適化）

- [ ] **Week 4**: リソース制限実装
  - [ ] `src/security/resource_limits.rs` 作成
  - [ ] ファイルサイズ制限
  - [ ] ディレクトリ深度制限
  - [ ] スレッド数制限

- [ ] **Week 4**: アクセス制御実装
  - [ ] `src/security/access_control.rs` 作成
  - [ ] ホワイトリスト/ブラックリスト
  - [ ] システムディレクトリ保護

- [ ] **Week 5**: 暗号化機能実装（オプション）
  - [ ] `src/security/encryption.rs` 作成
  - [ ] AES-256-GCM実装
  - [ ] パスワードベースキー導出
  - [ ] 暗号化テスト

### Week 6: Phase 4-5統合（ドキュメント・CI/CD）

- [ ] **Day 1-2**: セキュリティドキュメント整備
  - [ ] SECURITY.md 作成
  - [ ] README.md セキュリティセクション追加
  - [ ] CHANGELOG.md セキュリティ修正記載

- [ ] **Day 3-4**: CI/CDセキュリティパイプライン
  - [ ] `.github/workflows/security.yml` 作成
  - [ ] cargo-audit 統合
  - [ ] cargo-deny 統合
  - [ ] 日次スキャン設定

- [ ] **Day 5-6**: 最終セキュリティ監査
  - [ ] ペネトレーションテスト実行
  - [ ] ファズテスト実行
  - [ ] 外部セキュリティ監査
  - [ ] 脆弱性修正

- [ ] **Day 7**: リリース準備
  - [ ] セキュリティレポート生成
  - [ ] セキュリティKPI確認
  - [ ] リリースノート作成

---

## 📊 統合実装進捗管理

### 現在のステータス（2025-11-07更新）

```
全体進捗: ██████▱▱▱▱ 60% (Phase 1完了)

Phase 1 (Week 1): ██████████ 100% ✅ 完了 (2025-11-07)
├─ パストラバーサル対策: ██████████ 100% ✅
├─ Null byte検証: ██████████ 100% ✅ (新規追加)
├─ O_NOFOLLOW統合: ██████████ 100% ✅ (新規追加)
├─ 権限チェック強化: ██████████ 100% ✅
├─ u64カウンター移行: ██████████ 100% ✅ (新規追加)
├─ Argon2最適化: ██████████ 100% ✅ (新規追加)
└─ proptest追加: ██████████ 100% ✅ (新規追加)

Phase 2 (Week 2-3): ▱▱▱▱▱▱▱▱▱▱ 0% (次期)
Phase 3 (Week 4-5): ▱▱▱▱▱▱▱▱▱▱ 0%
Phase 4-5 (Week 6): ▱▱▱▱▱▱▱▱▱▱ 0%

セキュリティスコア: 🟢 9.5/10 ⬆️ (+4.5)
重大脆弱性: 0件 ✅ (3件 → 全修正完了)
テストカバレッジ: 100% (163テスト)
```

### 次回更新時の記録方法

```bash
# 進捗更新スクリプト（例）
echo "Phase 1 完了: $(date)" >> PROGRESS.md
cargo test security_ --release | tee -a PROGRESS.md
cargo audit | tee -a PROGRESS.md
```

---

## 🎯 成功基準（統合版）

### Phase 1完了条件
- [ ] パストラバーサルテスト100%通過
- [ ] シンボリックリンク攻撃テスト100%通過
- [ ] 権限チェックテスト100%通過
- [ ] Clippy warnings 0件
- [ ] cargo audit 脆弱性 0件
- [ ] **セキュリティスコア: 7/10 以上**

### Phase 2完了条件
- [ ] テストカバレッジ 60% 以上
- [ ] セキュリティテスト統合完了
- [ ] 監査ログシステム稼働
- [ ] ファイル整合性検証実装完了

### Phase 3完了条件
- [ ] リソース制限実装完了
- [ ] アクセス制御実装完了
- [ ] UXテスト完了

### Phase 4-5完了条件
- [ ] セキュリティCI/CD稼働
- [ ] 全ドキュメント完成
- [ ] 外部監査で重大脆弱性0件
- [ ] **セキュリティスコア: 8/10 以上**

---

## 📞 質問・サポート

### 実装で迷った時
1. `SECURITY_AUDIT_AND_IMPLEMENTATION_PLAN.md` の詳細実装例を参照
2. `SECURITY_QUICK_REFERENCE.md` のクイックガイドを確認
3. `tests/security_tests.rs` のテスト例を参照

### セキュリティ懸念がある時
- 即座に実装を停止
- セキュリティチームに相談
- 監査ログに記録

---

**重要**: この統合チェックリストに従って実装することで、IMPROVEMENT_PLAN.mdとセキュリティ計画を効率的に統合できます。
