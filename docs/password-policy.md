# パスワードポリシー実装ドキュメント

## 📋 概要

backup-suiteは、NIST SP 800-63Bに準拠した包括的なパスワードポリシーを実装しています。
ユーザーの利便性を維持しつつ、セキュリティを向上させる「警告のみ（non-enforcing）」アプローチを採用しています。

## 🎯 設計方針

### 利便性重視の非強制アプローチ

- **警告のみ**: 弱いパスワードでも使用可能（ユーザーの自由を尊重）
- **教育的**: 具体的な改善提案を提示
- **選択肢の提供**: 自動生成機能で強力なパスワードを簡単に作成可能

### セキュリティ基準

- **NIST SP 800-63B準拠**: 最小8文字（業界標準）
- **Shannon entropy計算**: 数学的に複雑さを評価
- **パターン検出**: 一般的な弱いパスワードを検出

## 🔧 実装機能

### 1. パスワード強度評価

3段階の強度レベルで評価：

```rust
pub enum PasswordStrength {
    Weak,    // 脆弱
    Medium,  // 中程度
    Strong,  // 強力
}
```

#### 評価アルゴリズム

**スコアリングシステム**:
```
total_score = length_score + entropy_score + pattern_penalty

length_score:
  - < 8文字: 0点
  - 8-9文字: 1点
  - 10-13文字: 2点
  - 14文字以上: 3点

entropy_score:
  - < 25: 0点
  - 25-39: 1点
  - 40-59: 2点
  - 60以上: 3点

pattern_penalty:
  - 一般的なパスワード/繰り返し/連続パターン: -2点
```

**強度判定**:
- 0-2点: Weak
- 3-4点: Medium
- 5点以上: Strong

### 2. Shannon Entropy計算

文字の多様性を数学的に評価：

```rust
entropy = -Σ(p_i * log2(p_i)) * length
```

- `p_i`: 各文字の出現頻度
- より多様な文字 = より高いentropy

### 3. パターン検出

#### 一般的なパスワード検出

Top 30の弱いパスワードを検出：
- "password", "12345678", "qwerty", "abc123", etc.

#### 繰り返し文字検出

ユニーク文字が30%未満の場合に警告：
```
例: "aaaaaaaa" → 検出
```

#### 連続パターン検出

キーボード配列・数字列を検出：
```
例: "12345678", "qwerty", "asdfgh" → 検出
```

### 4. 強力なパスワード自動生成

```rust
// 20文字のランダムパスワード生成
let password = policy.generate_password(20);
```

文字セット:
- 大文字: A-Z
- 小文字: a-z
- 数字: 0-9
- 記号: !@#$%^&*()-_=+[]{}|;:,.<>?

## 📖 使用方法

### CLI統合

#### 1. パスワード自動生成

```bash
backup-suite run --encrypt --generate-password
```

出力例:
```
🔐 暗号化パスワード: MyS3cur3!P@ss#2024
⚠️ このパスワードを安全に保管してください
```

#### 2. パスワード指定（強度チェック付き）

```bash
backup-suite run --encrypt --password "MyBackup2024"
```

出力例（Mediumの場合）:
```
Password Strength: Medium
  This password provides moderate security. Adding special characters or length would improve it.
```

#### 3. 対話的入力

```bash
backup-suite run --encrypt
```

プロンプト:
```
🔐 暗号化パスワード: [入力]
```

入力後、自動的に強度評価が表示されます。

### プログラムからの使用

```rust
use backup_suite::crypto::{PasswordPolicy, PasswordStrength};

let policy = PasswordPolicy::default();

// パスワード評価
let strength = policy.evaluate("MyPassword123");

// レポート表示
println!("{}", policy.display_report("MyPassword123"));

// パスワード生成
let strong_password = policy.generate_password(20);
```

## 📊 動作例

### 実際の評価結果

#### Weak - "weak"
```
Password Strength: Weak
  This password may be vulnerable to attacks. Consider using a longer password with varied characters.

Tip: Use --generate-password to create a strong random password.
```

#### Weak - "12345678"
```
Password Strength: Weak
  This password may be vulnerable to attacks. Consider using a longer password with varied characters.

Warnings:
  - Contains sequential pattern
  - This is a commonly used password

Tip: Use --generate-password to create a strong random password.
```

#### Medium - "MyBackup2024"
```
Password Strength: Medium
  This password provides moderate security. Adding special characters or length would improve it.
```

#### Strong - "MyS3cur3!B@ckup#2024"
```
Password Strength: Strong
  This password provides strong security.
```

## 🔒 セキュリティ考慮事項

### メモリ安全性

- **zeroize使用**: パスワードをメモリから確実に削除
```rust
use zeroize::Zeroizing;
let password = Zeroizing::new(password_string);
// スコープ外で自動的にゼロクリア
```

### 標準準拠

- **NIST SP 800-63B**: 最小8文字、エントロピーベース評価
- **OWASP**: パスワード保存ガイドライン準拠
- **ISO 27001**: セキュリティ管理基準準拠

## 🧪 テストカバレッジ

### ユニットテスト

10個のテストで全機能をカバー：

```bash
cargo test --lib password_policy

running 10 tests
test crypto::password_policy::tests::test_password_strength_weak ... ok
test crypto::password_policy::tests::test_password_strength_medium ... ok
test crypto::password_policy::tests::test_password_strength_strong ... ok
test crypto::password_policy::tests::test_entropy_calculation ... ok
test crypto::password_policy::tests::test_repeated_chars_detection ... ok
test crypto::password_policy::tests::test_sequential_detection ... ok
test crypto::password_policy::tests::test_common_password_detection ... ok
test crypto::password_policy::tests::test_pattern_warnings ... ok
test crypto::password_policy::tests::test_password_generation ... ok
test crypto::password_policy::tests::test_display_report ... ok

test result: ok. 10 passed; 0 failed; 0 ignored
```

### テスト項目

- ✅ 強度評価（Weak/Medium/Strong）
- ✅ Shannon entropy計算
- ✅ 繰り返し文字検出
- ✅ 連続パターン検出
- ✅ 一般的なパスワード検出
- ✅ 警告メッセージ生成
- ✅ パスワード自動生成
- ✅ レポート表示

## 🔄 将来の拡張

現在の実装は完全ですが、以下の拡張が可能：

### Option A: 最小長の調整

```rust
// 現在: 8文字（NIST標準）
min_length: 8

// より厳格: 12文字
min_length: 12
```

### Option B: 環境変数での制御

```bash
# 厳格モード（企業利用）
BACKUP_PASSWORD_POLICY=strict backup-suite run --encrypt

# ポリシーなし（開発環境）
BACKUP_PASSWORD_POLICY=none backup-suite run --encrypt
```

### Option C: 複雑性チェックの追加

```rust
pub struct PasswordPolicy {
    pub min_length: usize,
    pub check_entropy: bool,
    pub require_uppercase: bool,    // 新規
    pub require_lowercase: bool,    // 新規
    pub require_digit: bool,        // 新規
    pub require_special: bool,      // 新規
}
```

## 📚 参考資料

- [NIST SP 800-63B](https://pages.nist.gov/800-63-3/sp800-63b.html) - Digital Identity Guidelines
- [OWASP Password Storage Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html)
- [Shannon Entropy](https://en.wikipedia.org/wiki/Entropy_(information_theory)) - Information Theory

## 📝 まとめ

backup-suiteのパスワードポリシーは：

✅ **完全実装済み** - Shannon entropy、パターン検出、自動生成
✅ **標準準拠** - NIST SP 800-63B、OWASP、ISO 27001
✅ **利便性重視** - 警告のみで強制しない
✅ **教育的** - 具体的な改善提案を提示
✅ **安全** - zeroizeによるメモリ保護
✅ **テスト済み** - 10個のユニットテストで全機能カバー

**推奨事項**: 現在の実装（最小8文字、非強制）は業界標準に準拠しており、
ユーザビリティとセキュリティのバランスが取れています。
特別な要件がない限り、変更の必要はありません。
