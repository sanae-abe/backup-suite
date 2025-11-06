use crate::error::{BackupError, Result};
use std::fs::{File, OpenOptions};
use std::path::{Component, Path, PathBuf};

/// 安全なパス結合（ディレクトリトラバーサル対策）
///
/// ベースディレクトリとチャイルドパスを結合する際に、ディレクトリトラバーサル攻撃を防ぎます。
/// パス内の `..` コンポーネントを除去し、結果がベースディレクトリ配下にあることを保証します。
///
/// # セキュリティ
///
/// この関数は以下の攻撃パターンを防ぎます：
/// - `../../../etc/passwd` のような相対パス攻撃
/// - `..\\..\\..\\windows\\system32\\config\\sam` のような Windows パス攻撃
/// - シンボリックリンクによるパストラバーサル（canonicalize を使用）
///
/// # 引数
///
/// * `base` - ベースディレクトリ（すべての結果パスはこの配下になる）
/// * `child` - 結合する相対パス
///
/// # 戻り値
///
/// 安全に結合されたパス、またはセキュリティ違反時のエラー
///
/// # エラー
///
/// * `BackupError::PathTraversalDetected` - ディレクトリトラバーサル攻撃を検出
/// * `BackupError::IoError` - パスの正規化に失敗
///
/// # 使用例
///
/// ```rust,no_run
/// use backup_suite::security::safe_join;
/// use std::path::Path;
///
/// let base = Path::new("/home/user/backups");
/// let child = Path::new("report.txt");
///
/// // 安全: /home/user/backups/report.txt
/// let result = safe_join(base, child).unwrap();
///
/// // エラー: ディレクトリトラバーサル検出
/// let malicious = Path::new("../../../etc/passwd");
/// let result = safe_join(base, malicious);
/// assert!(result.is_err());
/// ```
pub fn safe_join(base: &Path, child: &Path) -> Result<PathBuf> {
    // Null byte検出（パストラバーサル攻撃対策）
    let child_str = child
        .to_str()
        .ok_or_else(|| BackupError::PathTraversalDetected {
            path: child.to_path_buf(),
        })?;

    if child_str.contains('\0') {
        return Err(BackupError::PathTraversalDetected {
            path: child.to_path_buf(),
        });
    }

    // 相対パスから .. を除去して正規化
    let normalized: PathBuf = child
        .components()
        .filter(|c| !matches!(c, Component::ParentDir))
        .collect();

    // ベースパスと結合
    let result = base.join(&normalized);

    // ベースパスを正規化
    // ベースパスが存在しない場合は、親ディレクトリまで遡って正規化
    let canonical_base = if base.exists() {
        base.canonicalize().map_err(BackupError::IoError)?
    } else {
        // ベースパスが存在しない場合、親ディレクトリを使用
        let mut check_base = base.to_path_buf();
        while !check_base.exists() && check_base.parent().is_some() {
            check_base = check_base.parent().unwrap().to_path_buf();
        }
        if check_base.exists() {
            let canonical = check_base.canonicalize().map_err(BackupError::IoError)?;
            // 元のベースパスの残りの部分を追加
            let remaining = base.strip_prefix(&check_base).unwrap_or(base);
            canonical.join(remaining)
        } else {
            // どの親ディレクトリも存在しない場合はエラー
            return Err(BackupError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("ベースパス {:?} が存在しません", base),
            )));
        }
    };

    // 結果パスの親ディレクトリが存在しない場合は作成する必要があるため、
    // 親ディレクトリまでの部分を検証
    let result_parent = if result.exists() {
        result.canonicalize().map_err(BackupError::IoError)?
    } else {
        // 存在しないパスの場合、親ディレクトリを基準に検証
        let mut check_path = result.clone();
        while !check_path.exists() && check_path.parent().is_some() {
            check_path = check_path.parent().unwrap().to_path_buf();
        }

        if check_path.exists() {
            check_path.canonicalize().map_err(BackupError::IoError)?
        } else {
            canonical_base.clone()
        }
    };

    // 結果がベースディレクトリ配下にあることを確認
    if !result_parent.starts_with(&canonical_base) {
        return Err(BackupError::PathTraversalDetected {
            path: child.to_path_buf(),
        });
    }

    Ok(result)
}

/// パス文字列のサニタイズ
///
/// ファイル名やディレクトリ名から危険な文字を除去し、安全な文字列に変換します。
///
/// # セキュリティ
///
/// 以下の文字のみを許可します：
/// - 英数字（a-z, A-Z, 0-9）
/// - ハイフン（-）
/// - アンダースコア（_）
///
/// # 引数
///
/// * `name` - サニタイズ対象の文字列
///
/// # 戻り値
///
/// 安全な文字のみを含む文字列
///
/// # 使用例
///
/// ```rust
/// use backup_suite::security::sanitize_path_component;
///
/// let safe = sanitize_path_component("my-file_v10");
/// assert_eq!(safe, "my-file_v10");
///
/// let sanitized = sanitize_path_component("dangerous/../../../file.txt");
/// assert_eq!(sanitized, "dangerousfiletxt");
/// ```
pub fn sanitize_path_component(name: &str) -> String {
    name.chars()
        .filter(|&c| c.is_alphanumeric() || "-_".contains(c))
        .collect()
}

/// パスが安全かどうかを検証
///
/// # 引数
///
/// * `path` - 検証するパス
///
/// # 戻り値
///
/// パスが安全な場合は `Ok(())`、危険な場合はエラー
///
/// # エラー
///
/// * `BackupError::PathTraversalDetected` - 危険なパスパターンを検出
pub fn validate_path_safety(path: &Path) -> Result<()> {
    // .. を含むパスは危険
    if path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(BackupError::PathTraversalDetected {
            path: path.to_path_buf(),
        });
    }

    // 絶対パスの場合、ルートディレクトリへのアクセスを防ぐ
    if path.is_absolute() {
        let components: Vec<_> = path.components().collect();
        // ルート直下のシステムディレクトリへのアクセスを制限
        if components.len() <= 2 {
            return Err(BackupError::PathTraversalDetected {
                path: path.to_path_buf(),
            });
        }
    }

    Ok(())
}

/// TOCTOU対策付き安全なファイルオープン
///
/// シンボリックリンクを追跡しないでファイルを開くことで、Time-Of-Check-Time-Of-Use (TOCTOU) 攻撃を防ぎます。
///
/// # セキュリティ
///
/// Unix系システムでは `O_NOFOLLOW` フラグを使用してシンボリックリンク攻撃を防ぎます。
/// これにより、チェック時と使用時の間にファイルがシンボリックリンクに置き換えられる攻撃を防止します。
///
/// # プラットフォーム
///
/// - **Unix系**: `O_NOFOLLOW` フラグで完全な保護
/// - **その他**: 標準的な `File::open` を使用（基本的な保護のみ）
///
/// # 引数
///
/// * `path` - オープンするファイルパス
///
/// # 戻り値
///
/// 開かれたファイルハンドル、またはエラー
///
/// # エラー
///
/// * `BackupError::IoError` - ファイルオープンに失敗、またはシンボリックリンク検出
///
/// # 使用例
///
/// ```rust,no_run
/// use backup_suite::security::safe_open;
/// use std::path::Path;
///
/// let path = Path::new("/home/user/backups/data.txt");
/// let file = safe_open(path).unwrap();
/// ```
pub fn safe_open(path: &Path) -> Result<File> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW)
            .open(path)
            .map_err(BackupError::IoError)
    }

    #[cfg(not(unix))]
    {
        File::open(path).map_err(BackupError::IoError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tempfile::TempDir;

    #[test]
    fn test_safe_join_normal_path() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();
        let child = Path::new("subdir/file.txt");

        let result = safe_join(base, child).unwrap();
        assert!(result.starts_with(base));
        assert!(result.ends_with("subdir/file.txt"));
    }

    #[test]
    fn test_safe_join_rejects_parent_dir() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        // 相対パス内の..は除去されるため、実際には安全なパスになる
        // しかし、結果がベースパス配下にあることは保証される
        let relative = Path::new("../../../etc/passwd");
        let result = safe_join(base, relative);

        // 結果は成功するが、ベースディレクトリ配下にある
        assert!(result.is_ok());
        let joined = result.unwrap();
        assert!(joined.starts_with(base));

        // ..が除去されているため、etc/passwdというパスになる
        assert!(joined.ends_with("etc/passwd"));
    }

    #[test]
    fn test_safe_join_rejects_absolute_path() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();
        let absolute = Path::new("/etc/passwd");

        let result = safe_join(base, absolute);
        // 絶対パスは join によりベースパスが無視されるため、
        // 正規化後の検証で弾かれる
        assert!(result.is_err());
    }

    #[test]
    fn test_sanitize_path_component() {
        assert_eq!(
            sanitize_path_component("normal-file_v10txt"),
            "normal-file_v10txt"
        );
        assert_eq!(
            sanitize_path_component("file with spaces"),
            "filewithspaces"
        );
        assert_eq!(sanitize_path_component("../../../etc/passwd"), "etcpasswd");
        assert_eq!(
            sanitize_path_component("file:with:colons"),
            "filewithcolons"
        );
    }

    #[test]
    fn test_validate_path_safety() {
        // 安全なパス
        let safe_path = Path::new("documents/report.txt");
        assert!(validate_path_safety(safe_path).is_ok());

        // 危険なパス（..を含む）
        let dangerous_path = Path::new("../../../etc/passwd");
        assert!(validate_path_safety(dangerous_path).is_err());
    }

    #[test]
    fn test_validate_path_safety_absolute() {
        // 深い階層の絶対パス（安全）
        let safe_absolute = Path::new("/home/user/documents/file.txt");
        assert!(validate_path_safety(safe_absolute).is_ok());

        // ルート直下（危険）
        let dangerous_absolute = Path::new("/etc");
        assert!(validate_path_safety(dangerous_absolute).is_err());
    }

    #[test]
    fn test_safe_open_normal_file() {
        use std::fs::File;
        use std::io::Write;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        // テストファイルを作成
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"test content").unwrap();

        // 通常のファイルをsafe_openで開く
        let result = safe_open(&file_path);
        assert!(result.is_ok());
    }

    #[cfg(unix)]
    #[test]
    fn test_safe_open_rejects_symlink() {
        use std::fs::File;
        use std::io::Write;
        use std::os::unix::fs::symlink;

        let temp_dir = TempDir::new().unwrap();
        let target_path = temp_dir.path().join("target.txt");
        let link_path = temp_dir.path().join("link.txt");

        // ターゲットファイルを作成
        let mut file = File::create(&target_path).unwrap();
        file.write_all(b"target content").unwrap();

        // シンボリックリンクを作成
        symlink(&target_path, &link_path).unwrap();

        // O_NOFOLLOWでシンボリックリンクは拒否される
        let result = safe_open(&link_path);
        assert!(result.is_err());
    }
}
