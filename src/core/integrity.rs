//! # ファイル整合性検証モジュール
//!
//! SHA-256ハッシュベースのファイル整合性検証機能を提供します。
//!
//! # 機能
//!
//! - **ハッシュ計算**: ファイルのSHA-256ハッシュ計算
//! - **メタデータ管理**: `.integrity` ファイルによるハッシュ保存
//! - **検証**: 復元時のファイル整合性検証
//!
//! # 使用例
//!
//! ```no_run
//! use backup_suite::core::integrity::{IntegrityChecker, BackupMetadata};
//! use std::path::PathBuf;
//!
//! // バックアップ時：ハッシュ計算と保存
//! let mut checker = IntegrityChecker::new();
//! let file_path = PathBuf::from("test.txt");
//! let hash = checker.compute_hash(&file_path).unwrap();
//! checker.add_file_hash(file_path.clone(), hash);
//!
//! // メタデータ保存
//! let backup_dir = PathBuf::from("/backup/backup_20250107_120000");
//! checker.save_metadata(&backup_dir).unwrap();
//!
//! // 復元時：メタデータ読み込みと検証
//! let metadata = BackupMetadata::load(&backup_dir).unwrap();
//! let is_valid = metadata.verify_file(&file_path, &file_path).unwrap();
//! assert!(is_valid);
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use super::incremental::BackupType;

/// バックアップメタデータ
///
/// バックアップディレクトリ内のファイルハッシュ情報を管理します。
///
/// # フィールド
///
/// * `version` - メタデータ形式のバージョン
/// * `file_hashes` - ファイルパスとSHA-256ハッシュのマップ
/// * `timestamp` - バックアップ作成日時
///
/// # 使用例
///
/// ```no_run
/// use backup_suite::core::integrity::BackupMetadata;
/// use std::path::PathBuf;
///
/// // メタデータ読み込み
/// let backup_dir = PathBuf::from("/backup/backup_20250107_120000");
/// let metadata = BackupMetadata::load(&backup_dir).unwrap();
///
/// // ファイル検証
/// let file_path = PathBuf::from("test.txt");
/// let is_valid = metadata.verify_file(&file_path, &file_path).unwrap();
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    /// メタデータ形式のバージョン
    pub version: String,
    /// ファイルパス（相対パス）とSHA-256ハッシュのマップ
    pub file_hashes: HashMap<PathBuf, String>,
    /// バックアップ作成日時（ISO 8601形式）
    pub timestamp: String,
    /// バックアップタイプ（Full/Incremental）
    #[serde(default)]
    pub backup_type: BackupType,
    /// 親バックアップ名（増分バックアップの場合のみ）
    #[serde(default)]
    pub parent_backup: Option<String>,
    /// 変更ファイルリスト（増分バックアップ時の変更ファイル）
    #[serde(default)]
    pub changed_files: Vec<PathBuf>,
}

impl BackupMetadata {
    /// 新しいBackupMetadataを作成
    ///
    /// # 戻り値
    ///
    /// 空のファイルハッシュマップを持つ BackupMetadata インスタンス
    ///
    /// # 使用例
    ///
    /// ```
    /// use backup_suite::core::integrity::BackupMetadata;
    ///
    /// let metadata = BackupMetadata::new();
    /// assert_eq!(metadata.version, "1.0");
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            version: "1.0".to_string(),
            file_hashes: HashMap::new(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            backup_type: BackupType::Full,
            parent_backup: None,
            changed_files: Vec::new(),
        }
    }

    /// バックアップディレクトリからメタデータを読み込み
    ///
    /// `.integrity` ファイルから JSON 形式のメタデータを読み込みます。
    ///
    /// # 引数
    ///
    /// * `backup_dir` - バックアップディレクトリのパス
    ///
    /// # 戻り値
    ///
    /// 成功時は読み込まれた BackupMetadata、失敗時はエラー
    ///
    /// # Errors
    ///
    /// 以下の場合にエラーを返します:
    /// * `.integrity` ファイルが存在しない場合
    /// * ファイルの読み込みに失敗した場合
    /// * JSON 解析に失敗した場合
    ///
    /// # 使用例
    ///
    /// ```no_run
    /// use backup_suite::core::integrity::BackupMetadata;
    /// use std::path::PathBuf;
    ///
    /// let backup_dir = PathBuf::from("/backup/backup_20250107_120000");
    /// let metadata = BackupMetadata::load(&backup_dir).unwrap();
    /// ```
        pub fn load(backup_dir: &Path) -> Result<Self> {
        let metadata_path = backup_dir.join(".integrity");
        if !metadata_path.exists() {
            return Err(anyhow::anyhow!(
                "整合性メタデータが見つかりません: metadata_path.display()".to_string()
            ));
        }

        let content = fs::read_to_string(&metadata_path)
            .context("メタデータ読み込み失敗: metadata_path.display()".to_string())?;
        let metadata: BackupMetadata =
            serde_json::from_str(&content).context("メタデータJSON解析失敗")?;

        Ok(metadata)
    }

    /// バックアップディレクトリにメタデータを保存
    ///
    /// `.integrity` ファイルに JSON 形式でメタデータを保存します。
    ///
    /// # 引数
    ///
    /// * `backup_dir` - バックアップディレクトリのパス
    ///
    /// # 戻り値
    ///
    /// 成功時は `Ok(())`、失敗時はエラー
    ///
    /// # Errors
    ///
    /// 以下の場合にエラーを返します:
    /// * JSON 生成に失敗した場合
    /// * ファイル書き込みに失敗した場合
    ///
    /// # 使用例
    ///
    /// ```no_run
    /// use backup_suite::core::integrity::BackupMetadata;
    /// use std::path::PathBuf;
    ///
    /// let mut metadata = BackupMetadata::new();
    /// metadata.file_hashes.insert(PathBuf::from("test.txt"), "abc123...".to_string());
    ///
    /// let backup_dir = PathBuf::from("/backup/backup_20250107_120000");
    /// metadata.save(&backup_dir).unwrap();
    /// ```
        pub fn save(&self, backup_dir: &Path) -> Result<()> {
        let metadata_path = backup_dir.join(".integrity");
        let content = serde_json::to_string_pretty(self).context("メタデータJSON生成失敗")?;
        fs::write(&metadata_path, content)
            .context("メタデータ保存失敗: metadata_path.display()".to_string())?;
        Ok(())
    }

    /// ファイルの整合性を検証
    ///
    /// ファイルの現在のSHA-256ハッシュを計算し、保存されたハッシュと比較します。
    ///
    /// # 引数
    ///
    /// * `relative_path` - バックアップ内の相対パス
    /// * `actual_file_path` - 検証対象の実際のファイルパス
    ///
    /// # 戻り値
    ///
    /// ハッシュが一致する場合 `true`、不一致の場合 `false`
    ///
    /// # Errors
    ///
    /// 以下の場合にエラーを返します:
    /// * ファイルに対応するハッシュ情報が見つからない場合
    /// * ファイルの読み込みに失敗した場合
    /// * ハッシュ計算に失敗した場合
    ///
    /// # 使用例
    ///
    /// ```no_run
    /// use backup_suite::core::integrity::BackupMetadata;
    /// use std::path::PathBuf;
    ///
    /// let backup_dir = PathBuf::from("/backup/backup_20250107_120000");
    /// let metadata = BackupMetadata::load(&backup_dir).unwrap();
    ///
    /// let relative = PathBuf::from("test.txt");
    /// let actual = PathBuf::from("/restore/test.txt");
    /// let is_valid = metadata.verify_file(&relative, &actual).unwrap();
    ///
    /// if is_valid {
    ///     println!("✓ ファイル整合性確認済み");
    /// } else {
    ///     eprintln!("⚠ ファイルが改ざんされています");
    /// }
    /// ```
        pub fn verify_file(&self, relative_path: &Path, actual_file_path: &Path) -> Result<bool> {
        let expected_hash = match self.file_hashes.get(relative_path) {
            Some(h) => h,
            None => {
                return Err(anyhow::anyhow!(
                    "ファイルのハッシュ情報が見つかりません: relative_path.display()".to_string()
                ));
            }
        };

        let actual_hash = Self::compute_file_hash(actual_file_path)?;
        Ok(&actual_hash == expected_hash)
    }

    /// ファイルのSHA-256ハッシュを計算（公開静的メソッド）
    ///
    /// # 引数
    ///
    /// * `file_path` - ハッシュ計算対象のファイルパス
    ///
    /// # 戻り値
    ///
    /// 成功時は16進数文字列形式のSHA-256ハッシュ、失敗時はエラー
    ///
    /// # Errors
    ///
    /// 以下の場合にエラーを返します:
    /// * ファイルのオープンに失敗した場合
    /// * ファイルの読み込みに失敗した場合
        pub fn compute_file_hash(file_path: &Path) -> Result<String> {
        let mut file =
            fs::File::open(file_path).context("ファイル読み込み失敗: file_path.display()".to_string())?;

        let mut hasher = Sha256::new();
        let mut buffer = vec![0u8; 8192]; // 8KB バッファ

        loop {
            let bytes_read = file.read(&mut buffer).context("ファイル読み込みエラー")?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        let result = hasher.finalize();
        Ok(format!("{result:x}"))
    }
}

impl Default for BackupMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// 整合性検証エンジン
///
/// バックアップ時のハッシュ計算と保存を担当します。
///
/// # 使用例
///
/// ```no_run
/// use backup_suite::core::integrity::IntegrityChecker;
/// use std::path::PathBuf;
///
/// let mut checker = IntegrityChecker::new();
///
/// // ファイルハッシュ計算と追加
/// let file = PathBuf::from("test.txt");
/// let hash = checker.compute_hash(&file).unwrap();
/// checker.add_file_hash(file, hash);
///
/// // メタデータ保存
/// let backup_dir = PathBuf::from("/backup/backup_20250107_120000");
/// checker.save_metadata(&backup_dir).unwrap();
/// ```
pub struct IntegrityChecker {
    pub metadata: BackupMetadata,
}

impl IntegrityChecker {
    /// 新しいIntegrityCheckerを作成
    ///
    /// # 戻り値
    ///
    /// 空のメタデータを持つ IntegrityChecker インスタンス
    ///
    /// # 使用例
    ///
    /// ```
    /// use backup_suite::core::integrity::IntegrityChecker;
    ///
    /// let checker = IntegrityChecker::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            metadata: BackupMetadata::new(),
        }
    }

    /// ファイルのSHA-256ハッシュを計算
    ///
    /// # 引数
    ///
    /// * `file_path` - ハッシュ計算対象のファイルパス
    ///
    /// # 戻り値
    ///
    /// 成功時は16進数文字列形式のSHA-256ハッシュ、失敗時はエラー
    ///
    /// # Errors
    ///
    /// 以下の場合にエラーを返します:
    /// * ファイルのオープンに失敗した場合
    /// * ファイルの読み込みに失敗した場合
    ///
    /// # 使用例
    ///
    /// ```no_run
    /// use backup_suite::core::integrity::IntegrityChecker;
    /// use std::path::PathBuf;
    ///
    /// let checker = IntegrityChecker::new();
    /// let hash = checker.compute_hash(&PathBuf::from("test.txt")).unwrap();
    /// println!("SHA-256: {}", hash);
    /// ```
        pub fn compute_hash(&self, file_path: &Path) -> Result<String> {
        BackupMetadata::compute_file_hash(file_path)
    }

    /// ファイルハッシュをメタデータに追加
    ///
    /// # 引数
    ///
    /// * `relative_path` - バックアップ内の相対パス
    /// * `hash` - SHA-256ハッシュ（16進数文字列）
    ///
    /// # 使用例
    ///
    /// ```
    /// use backup_suite::core::integrity::IntegrityChecker;
    /// use std::path::PathBuf;
    ///
    /// let mut checker = IntegrityChecker::new();
    /// checker.add_file_hash(PathBuf::from("test.txt"), "abc123...".to_string());
    /// ```
    pub fn add_file_hash(&mut self, relative_path: PathBuf, hash: String) {
        self.metadata.file_hashes.insert(relative_path, hash);
    }

    /// メタデータをバックアップディレクトリに保存
    ///
    /// # 引数
    ///
    /// * `backup_dir` - バックアップディレクトリのパス
    ///
    /// # 戻り値
    ///
    /// 成功時は `Ok(())`、失敗時はエラー
    ///
    /// # Errors
    ///
    /// 以下の場合にエラーを返します:
    /// * メタデータの保存に失敗した場合
    ///
    /// # 使用例
    ///
    /// ```no_run
    /// use backup_suite::core::integrity::IntegrityChecker;
    /// use std::path::PathBuf;
    ///
    /// let mut checker = IntegrityChecker::new();
    /// checker.add_file_hash(PathBuf::from("test.txt"), "abc123...".to_string());
    /// checker.save_metadata(&PathBuf::from("/backup/backup_20250107_120000")).unwrap();
    /// ```
        pub fn save_metadata(&self, backup_dir: &Path) -> Result<()> {
        self.metadata.save(backup_dir)
    }

    /// 保存予定のファイル数を取得
    ///
    /// # 戻り値
    ///
    /// 登録されているファイルハッシュの数
    ///
    /// # 使用例
    ///
    /// ```
    /// use backup_suite::core::integrity::IntegrityChecker;
    /// use std::path::PathBuf;
    ///
    /// let mut checker = IntegrityChecker::new();
    /// checker.add_file_hash(PathBuf::from("test.txt"), "abc123...".to_string());
    /// assert_eq!(checker.file_count(), 1);
    /// ```
    #[must_use]
    pub fn file_count(&self) -> usize {
        self.metadata.file_hashes.len()
    }
}

impl Default for IntegrityChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_compute_hash() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("test.txt");
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(b"test content").unwrap();
        drop(file);

        let checker = IntegrityChecker::new();
        let hash = checker.compute_hash(&file_path).unwrap();

        // SHA-256("test content") の期待値
        let expected = "6ae8a75555209fd6c44157c0aed8016e763ff435a19cf186f76863140143ff72";
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_add_file_hash() {
        let mut checker = IntegrityChecker::new();
        let path = PathBuf::from("test.txt");
        let hash = "abc123".to_string();

        checker.add_file_hash(path.clone(), hash.clone());
        assert_eq!(checker.file_count(), 1);
        assert_eq!(checker.metadata.file_hashes.get(&path), Some(&hash));
    }

    #[test]
    fn test_save_and_load_metadata() {
        let temp = TempDir::new().unwrap();
        let backup_dir = temp.path().join("backup");
        fs::create_dir(&backup_dir).unwrap();

        // メタデータ保存
        let mut checker = IntegrityChecker::new();
        checker.add_file_hash(PathBuf::from("test.txt"), "hash1".to_string());
        checker.add_file_hash(PathBuf::from("data/file.dat"), "hash2".to_string());
        checker.save_metadata(&backup_dir).unwrap();

        // メタデータ読み込み
        let loaded = BackupMetadata::load(&backup_dir).unwrap();
        assert_eq!(loaded.file_hashes.len(), 2);
        assert_eq!(
            loaded.file_hashes.get(&PathBuf::from("test.txt")),
            Some(&"hash1".to_string())
        );
        assert_eq!(
            loaded.file_hashes.get(&PathBuf::from("data/file.dat")),
            Some(&"hash2".to_string())
        );
    }

    #[test]
    fn test_verify_file() {
        let temp = TempDir::new().unwrap();
        let backup_dir = temp.path().join("backup");
        fs::create_dir(&backup_dir).unwrap();

        // テストファイル作成
        let test_file = temp.path().join("test.txt");
        let mut file = fs::File::create(&test_file).unwrap();
        file.write_all(b"test content").unwrap();
        drop(file);

        // ハッシュ計算と保存
        let mut checker = IntegrityChecker::new();
        let hash = checker.compute_hash(&test_file).unwrap();
        checker.add_file_hash(PathBuf::from("test.txt"), hash);
        checker.save_metadata(&backup_dir).unwrap();

        // 検証
        let metadata = BackupMetadata::load(&backup_dir).unwrap();
        let is_valid = metadata
            .verify_file(&PathBuf::from("test.txt"), &test_file)
            .unwrap();
        assert!(is_valid);
    }

    #[test]
    fn test_verify_file_tampered() {
        let temp = TempDir::new().unwrap();
        let backup_dir = temp.path().join("backup");
        fs::create_dir(&backup_dir).unwrap();

        // テストファイル作成
        let test_file = temp.path().join("test.txt");
        let mut file = fs::File::create(&test_file).unwrap();
        file.write_all(b"test content").unwrap();
        drop(file);

        // ハッシュ計算と保存
        let mut checker = IntegrityChecker::new();
        let hash = checker.compute_hash(&test_file).unwrap();
        checker.add_file_hash(PathBuf::from("test.txt"), hash);
        checker.save_metadata(&backup_dir).unwrap();

        // ファイルを改ざん
        fs::write(&test_file, b"tampered content").unwrap();

        // 検証（失敗するはず）
        let metadata = BackupMetadata::load(&backup_dir).unwrap();
        let is_valid = metadata
            .verify_file(&PathBuf::from("test.txt"), &test_file)
            .unwrap();
        assert!(!is_valid);
    }

    #[test]
    fn test_metadata_json_format() {
        let temp = TempDir::new().unwrap();
        let backup_dir = temp.path().join("backup");
        fs::create_dir(&backup_dir).unwrap();

        let mut checker = IntegrityChecker::new();
        checker.add_file_hash(PathBuf::from("test.txt"), "hash123".to_string());
        checker.save_metadata(&backup_dir).unwrap();

        // JSON ファイルの内容を確認
        let content = fs::read_to_string(backup_dir.join(".integrity")).unwrap();
        assert!(content.contains("\"version\""));
        assert!(content.contains("\"file_hashes\""));
        assert!(content.contains("\"timestamp\""));
        assert!(content.contains("test.txt"));
        assert!(content.contains("hash123"));
    }
}
