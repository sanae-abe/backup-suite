//! # 増分バックアップエンジン
//!
//! 変更検出ベースの増分バックアップ機能を提供します。
//!
//! # 機能
//!
//! - **変更検出**: SHA-256ハッシュ比較による変更ファイル検出
//! - **増分管理**: 親バックアップへの参照管理
//! - **自動フォールバック**: 初回または前回バックアップなしの場合、フルバックアップに自動切り替え
//!
//! # 使用例
//!
//! ```no_run
//! use backup_suite::core::incremental::{BackupType, IncrementalBackupEngine};
//! use std::path::PathBuf;
//!
//! // 初回バックアップ（自動的にフルバックアップになる）
//! let engine = IncrementalBackupEngine::new(PathBuf::from("./backups"));
//! let backup_type = engine.determine_backup_type().unwrap();
//! assert!(matches!(backup_type, BackupType::Full));
//!
//! // 2回目以降は増分バックアップが可能
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use super::integrity::BackupMetadata;

/// バックアップタイプ
///
/// フルバックアップまたは増分バックアップを識別します。
///
/// # バリアント
///
/// * `Full` - 全ファイルをバックアップ（初回または前回なし）
/// * `Incremental` - 変更ファイルのみバックアップ（前回バックアップからの差分）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackupType {
    Full,
    Incremental,
}

impl Default for BackupType {
    fn default() -> Self {
        Self::Full
    }
}

/// 増分バックアップエンジン
///
/// 変更検出とバックアップタイプの決定を担当します。
///
/// # フィールド
///
/// * `backup_base` - バックアップディレクトリのベースパス
///
/// # 使用例
///
/// ```no_run
/// use backup_suite::core::incremental::IncrementalBackupEngine;
/// use std::path::PathBuf;
///
/// let engine = IncrementalBackupEngine::new(PathBuf::from("./backups"));
/// let backup_type = engine.determine_backup_type().unwrap();
/// ```
pub struct IncrementalBackupEngine {
    backup_base: PathBuf,
}

impl IncrementalBackupEngine {
    /// 新しい IncrementalBackupEngine を作成
    ///
    /// # 引数
    ///
    /// * `backup_base` - バックアップディレクトリのベースパス
    ///
    /// # 戻り値
    ///
    /// IncrementalBackupEngine インスタンス
    ///
    /// # 使用例
    ///
    /// ```no_run
    /// use backup_suite::core::incremental::IncrementalBackupEngine;
    /// use std::path::PathBuf;
    ///
    /// let engine = IncrementalBackupEngine::new(PathBuf::from("./backups"));
    /// ```
    pub fn new(backup_base: PathBuf) -> Self {
        Self { backup_base }
    }

    /// バックアップタイプを決定
    ///
    /// 前回のバックアップが存在するかチェックし、存在すればIncremental、
    /// 存在しなければFullを返します。
    ///
    /// # 戻り値
    ///
    /// 成功時は BackupType、失敗時はエラー
    ///
    /// # エラー
    ///
    /// * バックアップディレクトリの読み込みに失敗した場合
    ///
    /// # 使用例
    ///
    /// ```no_run
    /// use backup_suite::core::incremental::IncrementalBackupEngine;
    /// use std::path::PathBuf;
    ///
    /// let engine = IncrementalBackupEngine::new(PathBuf::from("./backups"));
    /// let backup_type = engine.determine_backup_type().unwrap();
    /// ```
    pub fn determine_backup_type(&self) -> Result<BackupType> {
        match self.find_latest_backup()? {
            Some(_) => Ok(BackupType::Incremental),
            None => Ok(BackupType::Full),
        }
    }

    /// 最新のバックアップディレクトリを検索
    ///
    /// # 戻り値
    ///
    /// 成功時は最新バックアップのパス（存在しない場合はNone）、失敗時はエラー
    ///
    /// # エラー
    ///
    /// * ディレクトリの読み込みに失敗した場合
    pub fn find_latest_backup(&self) -> Result<Option<PathBuf>> {
        if !self.backup_base.exists() {
            return Ok(None);
        }

        let mut backups: Vec<PathBuf> = std::fs::read_dir(&self.backup_base)
            .context("バックアップディレクトリの読み込み失敗")?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("backup_")
            })
            .map(|entry| entry.path())
            .collect();

        if backups.is_empty() {
            return Ok(None);
        }

        // タイムスタンプでソート（降順）
        backups.sort_by(|a, b| b.cmp(a));
        Ok(Some(backups[0].clone()))
    }

    /// 変更ファイルを検出
    ///
    /// 前回のバックアップメタデータと現在のファイルハッシュを比較し、
    /// 変更されたファイルのリストを返します。
    ///
    /// # 引数
    ///
    /// * `current_files` - 現在のファイルリスト（相対パス、絶対パス）
    /// * `previous_metadata` - 前回のバックアップメタデータ
    ///
    /// # 戻り値
    ///
    /// 変更されたファイルのリスト（相対パス、絶対パス）
    ///
    /// # エラー
    ///
    /// * ファイルハッシュの計算に失敗した場合
    ///
    /// # 使用例
    ///
    /// ```no_run
    /// use backup_suite::core::incremental::IncrementalBackupEngine;
    /// use backup_suite::core::integrity::BackupMetadata;
    /// use std::path::PathBuf;
    ///
    /// let engine = IncrementalBackupEngine::new(PathBuf::from("./backups"));
    /// let files = vec![(PathBuf::from("file.txt"), PathBuf::from("/path/to/file.txt"))];
    /// let metadata = BackupMetadata::new();
    /// let changed = engine.detect_changed_files(&files, &metadata).unwrap();
    /// ```
    pub fn detect_changed_files(
        &self,
        current_files: &[(PathBuf, PathBuf)],
        previous_metadata: &BackupMetadata,
    ) -> Result<Vec<(PathBuf, PathBuf)>> {
        let mut changed_files = Vec::new();

        for (relative_path, absolute_path) in current_files {
            // 前回のハッシュを取得
            let previous_hash = previous_metadata.file_hashes.get(relative_path);

            // 現在のハッシュを計算
            let current_hash = BackupMetadata::compute_file_hash(absolute_path)
                .context(format!("ハッシュ計算失敗: {:?}", absolute_path))?;

            // ハッシュが異なる場合、または新規ファイルの場合は変更とみなす
            if previous_hash.map_or(true, |prev| prev != &current_hash) {
                changed_files.push((relative_path.clone(), absolute_path.clone()));
            }
        }

        Ok(changed_files)
    }

    /// 前回のバックアップメタデータを読み込み
    ///
    /// # 戻り値
    ///
    /// 成功時はメタデータ、失敗時はエラー
    ///
    /// # エラー
    ///
    /// * 前回のバックアップが見つからない場合
    /// * メタデータの読み込みに失敗した場合
    pub fn load_previous_metadata(&self) -> Result<BackupMetadata> {
        let latest_backup = self
            .find_latest_backup()?
            .ok_or_else(|| anyhow::anyhow!("前回のバックアップが見つかりません"))?;

        BackupMetadata::load(&latest_backup)
            .context("前回のバックアップメタデータ読み込み失敗")
    }

    /// 前回のバックアップ名を取得
    ///
    /// # 戻り値
    ///
    /// 成功時はバックアップ名、失敗時はエラー
    pub fn get_previous_backup_name(&self) -> Result<Option<String>> {
        match self.find_latest_backup()? {
            Some(path) => {
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_string())
                    .ok_or_else(|| anyhow::anyhow!("バックアップ名取得失敗"))?;
                Ok(Some(name))
            }
            None => Ok(None),
        }
    }
}

/// 増分バックアップチェーンの解決
///
/// 増分バックアップの親チェーンを遡り、完全な復元に必要な
/// 全バックアップディレクトリのリストを返します。
///
/// # 引数
///
/// * `backup_dir` - 増分バックアップディレクトリ
///
/// # 戻り値
///
/// バックアップディレクトリのリスト（フルバックアップ→増分1→増分2...の順）
///
/// # エラー
///
/// * メタデータの読み込みに失敗した場合
/// * 親バックアップが見つからない場合
///
/// # 使用例
///
/// ```no_run
/// use backup_suite::core::incremental::resolve_backup_chain;
/// use std::path::PathBuf;
///
/// let chain = resolve_backup_chain(&PathBuf::from("./backups/backup_20250107_120000")).unwrap();
/// for backup in &chain {
///     println!("復元順: {:?}", backup);
/// }
/// ```
pub fn resolve_backup_chain(backup_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut chain = Vec::new();
    let mut current_dir = backup_dir.to_path_buf();

    loop {
        // 現在のバックアップメタデータを読み込み（存在しない場合は単一バックアップとして扱う）
        let metadata = match BackupMetadata::load(&current_dir) {
            Ok(m) => m,
            Err(_) => {
                // メタデータが存在しない場合は単一のバックアップとして扱う
                chain.push(current_dir.clone());
                break;
            }
        };

        // チェーンに追加（逆順で追加、後で反転）
        chain.push(current_dir.clone());

        // 親バックアップがある場合、そちらへ移動
        match metadata.parent_backup {
            Some(parent_name) => {
                let parent_dir = current_dir
                    .parent()
                    .ok_or_else(|| anyhow::anyhow!("親ディレクトリ取得失敗"))?
                    .join(&parent_name);

                if !parent_dir.exists() {
                    return Err(anyhow::anyhow!(
                        "親バックアップが見つかりません: {:?}",
                        parent_dir
                    ));
                }

                current_dir = parent_dir;
            }
            None => {
                // フルバックアップに到達（ルート）
                break;
            }
        }
    }

    // 正しい順序に反転（フルバックアップ→増分1→増分2...）
    chain.reverse();
    Ok(chain)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_determine_backup_type_no_previous() {
        let temp = TempDir::new().unwrap();
        let engine = IncrementalBackupEngine::new(temp.path().to_path_buf());

        let backup_type = engine.determine_backup_type().unwrap();
        assert_eq!(backup_type, BackupType::Full);
    }

    #[test]
    fn test_determine_backup_type_with_previous() {
        let temp = TempDir::new().unwrap();
        let backup_dir = temp.path().join("backup_20250107_120000");
        fs::create_dir(&backup_dir).unwrap();

        // ダミーメタデータ作成
        let metadata = BackupMetadata::new();
        metadata.save(&backup_dir).unwrap();

        let engine = IncrementalBackupEngine::new(temp.path().to_path_buf());
        let backup_type = engine.determine_backup_type().unwrap();
        assert_eq!(backup_type, BackupType::Incremental);
    }

    #[test]
    fn test_detect_changed_files() {
        use std::io::Write;

        let temp = TempDir::new().unwrap();

        // ファイル1を作成
        let file1 = temp.path().join("file1.txt");
        let mut f1 = fs::File::create(&file1).unwrap();
        f1.write_all(b"original content").unwrap();
        drop(f1);

        // 前回のメタデータを作成
        let mut previous_metadata = BackupMetadata::new();
        let hash1 = BackupMetadata::compute_file_hash(&file1).unwrap();
        previous_metadata
            .file_hashes
            .insert(PathBuf::from("file1.txt"), hash1);

        // ファイル1を変更
        fs::write(&file1, b"modified content").unwrap();

        // ファイル2を新規追加
        let file2 = temp.path().join("file2.txt");
        fs::write(&file2, b"new file").unwrap();

        let current_files = vec![
            (PathBuf::from("file1.txt"), file1.clone()),
            (PathBuf::from("file2.txt"), file2.clone()),
        ];

        let engine = IncrementalBackupEngine::new(temp.path().to_path_buf());
        let changed = engine
            .detect_changed_files(&current_files, &previous_metadata)
            .unwrap();

        // file1（変更）とfile2（新規）の2ファイルが検出されるはず
        assert_eq!(changed.len(), 2);
    }

    #[test]
    fn test_resolve_backup_chain() {
        let temp = TempDir::new().unwrap();

        // フルバックアップ
        let full_backup = temp.path().join("backup_20250107_100000");
        fs::create_dir(&full_backup).unwrap();
        let mut full_metadata = BackupMetadata::new();
        full_metadata.backup_type = BackupType::Full;
        full_metadata.parent_backup = None;
        full_metadata.save(&full_backup).unwrap();

        // 増分バックアップ1
        let inc1_backup = temp.path().join("backup_20250107_110000");
        fs::create_dir(&inc1_backup).unwrap();
        let mut inc1_metadata = BackupMetadata::new();
        inc1_metadata.backup_type = BackupType::Incremental;
        inc1_metadata.parent_backup = Some("backup_20250107_100000".to_string());
        inc1_metadata.save(&inc1_backup).unwrap();

        // 増分バックアップ2
        let inc2_backup = temp.path().join("backup_20250107_120000");
        fs::create_dir(&inc2_backup).unwrap();
        let mut inc2_metadata = BackupMetadata::new();
        inc2_metadata.backup_type = BackupType::Incremental;
        inc2_metadata.parent_backup = Some("backup_20250107_110000".to_string());
        inc2_metadata.save(&inc2_backup).unwrap();

        // チェーン解決
        let chain = resolve_backup_chain(&inc2_backup).unwrap();

        // 順序確認（フル→増分1→増分2）
        assert_eq!(chain.len(), 3);
        assert!(chain[0].ends_with("backup_20250107_100000"));
        assert!(chain[1].ends_with("backup_20250107_110000"));
        assert!(chain[2].ends_with("backup_20250107_120000"));
    }
}
