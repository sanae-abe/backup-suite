//! セキュリティモジュール
//!
//! backup-suiteのセキュリティ機能を提供します。
//!
//! # 主要機能
//!
//! - **パストラバーサル対策**: ディレクトリトラバーサル攻撃を防ぐ安全なパス操作
//! - **権限チェック**: ファイル/ディレクトリの読み書き権限を検証
//! - **パスサニタイズ**: 危険な文字を除去した安全なパス生成
//!
//! # セキュリティ原則
//!
//! このモジュールは以下のセキュリティ原則に従っています：
//!
//! 1. **深層防御**: 複数のレイヤーでセキュリティチェックを実施
//! 2. **最小権限**: 必要最小限の権限のみを要求
//! 3. **入力検証**: すべての外部入力を検証
//! 4. **安全なデフォルト**: デフォルトで安全な動作を保証
//!
//! # 使用例
//!
//! ```rust,no_run
//! use backup_suite::security::{safe_join, check_read_permission, sanitize_path_component};
//! use std::path::Path;
//!
//! // 安全なパス結合
//! let base = Path::new("/home/user/backups");
//! let child = Path::new("report.txt");
//! let safe_path = safe_join(base, child).unwrap();
//!
//! // 権限チェック
//! check_read_permission(&safe_path).unwrap();
//!
//! // パスコンポーネントのサニタイズ
//! let safe_name = sanitize_path_component("my-file.txt");
//! ```
//!
//! # OWASP Top 10 対応
//!
//! このモジュールは以下のOWASP Top 10脅威に対応しています：
//!
//! - **A01:2021 – Broken Access Control**: 権限チェック機能で対応
//! - **A03:2021 – Injection**: パストラバーサル対策で対応
//! - **A04:2021 – Insecure Design**: 安全なデフォルト設計で対応
//! - **A05:2021 – Security Misconfiguration**: 設定検証機能で対応

pub mod path;
pub mod permissions;

// 再エクスポート：頻繁に使用される機能を簡単にアクセス可能にする
pub use path::{safe_join, safe_open, sanitize_path_component, validate_path_safety};
pub use permissions::{check_permissions, check_read_permission, check_write_permission};

#[cfg(unix)]
pub use permissions::check_execute_permission;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_security_module_integration() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        // safe_joinとpermissionsの連携テスト
        let child = std::path::Path::new("test/file.txt");
        let joined = safe_join(base, child).unwrap();

        // 親ディレクトリを作成
        if let Some(parent) = joined.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }

        // ファイルを作成
        std::fs::write(&joined, "test content").unwrap();

        // 権限チェック
        assert!(check_read_permission(&joined).is_ok());
    }

    #[test]
    fn test_sanitize_and_join() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        // 危険なファイル名をサニタイズしてから結合
        let dangerous_name = "../../../etc/passwd";
        let safe_name = sanitize_path_component(dangerous_name);

        // サニタイズ後は安全なパスになる（ドットも除去される）
        assert_eq!(safe_name, "etcpasswd");

        let child = std::path::Path::new(&safe_name);
        let joined = safe_join(base, child).unwrap();

        // 結果がベースディレクトリ配下にあることを確認
        assert!(joined.starts_with(base));
        assert!(joined.ends_with("etcpasswd"));
    }
}
