use regex::Regex;
use std::path::Path;

use crate::error::{BackupError, Result};

/// ファイルフィルタリング機能
///
/// 正規表現パターンを使用してファイルとディレクトリを除外します。
///
/// # 使用例
///
/// ```no_run
/// use backup_suite::core::filter::FileFilter;
/// use std::path::Path;
///
/// let patterns = vec![
///     r"\.git$".to_string(),
///     r"node_modules".to_string(),
///     r"\.DS_Store$".to_string(),
/// ];
///
/// let filter = FileFilter::new(&patterns).unwrap();
/// let path = Path::new("/project/.git");
/// assert!(filter.should_exclude(path));
/// ```
#[derive(Debug)]
pub struct FileFilter {
    patterns: Vec<Regex>,
}

impl FileFilter {
    /// 新しいFileFilterインスタンスを作成
    ///
    /// # 引数
    ///
    /// * `exclude_patterns` - 除外パターンの正規表現文字列のスライス
    ///
    /// # 戻り値
    ///
    /// 成功した場合は `Ok(FileFilter)`、正規表現のコンパイルに失敗した場合はエラー
    ///
    /// # エラー
    ///
    /// * `BackupError::RegexError` - 不正な正規表現パターン
    ///
    /// # 使用例
    ///
    /// ```no_run
    /// use backup_suite::core::filter::FileFilter;
    ///
    /// let patterns = vec![r"\.git$".to_string()];
    /// let filter = FileFilter::new(&patterns).unwrap();
    /// ```
    pub fn new(exclude_patterns: &[String]) -> Result<Self> {
        let mut patterns = Vec::new();

        for pattern in exclude_patterns {
            let regex = Regex::new(pattern).map_err(|e| BackupError::RegexError {
                pattern: pattern.clone(),
                source: e,
            })?;
            patterns.push(regex);
        }

        Ok(Self { patterns })
    }

    /// 指定されたパスを除外すべきかどうかを判定
    ///
    /// パスの文字列表現がいずれかの除外パターンにマッチする場合、`true` を返します。
    ///
    /// # 引数
    ///
    /// * `path` - チェックするパス
    ///
    /// # 戻り値
    ///
    /// パスを除外すべき場合は `true`、含めるべき場合は `false`
    ///
    /// # 使用例
    ///
    /// ```no_run
    /// use backup_suite::core::filter::FileFilter;
    /// use std::path::Path;
    ///
    /// let filter = FileFilter::new(&vec![r"\.git$".to_string()]).unwrap();
    /// assert!(filter.should_exclude(Path::new("/project/.git")));
    /// assert!(!filter.should_exclude(Path::new("/project/src")));
    /// ```
    pub fn should_exclude(&self, path: &Path) -> bool {
        // パターンが空の場合は何も除外しない
        if self.patterns.is_empty() {
            return false;
        }

        // パスを文字列に変換
        let path_str = match path.to_str() {
            Some(s) => s,
            None => {
                // UTF-8ではないパス名は除外しない（警告のみ）
                eprintln!("警告: UTF-8ではないパス名をスキップ: {:?}", path);
                return false;
            }
        };

        // いずれかのパターンにマッチするかチェック
        for regex in &self.patterns {
            if regex.is_match(path_str) {
                return true;
            }
        }

        false
    }

    /// ファイル名のみを使用して除外判定
    ///
    /// パス全体ではなく、ファイル名のみを正規表現パターンとマッチングします。
    ///
    /// # 引数
    ///
    /// * `path` - チェックするパス
    ///
    /// # 戻り値
    ///
    /// ファイル名を除外すべき場合は `true`、含めるべき場合は `false`
    ///
    /// # 使用例
    ///
    /// ```no_run
    /// use backup_suite::core::filter::FileFilter;
    /// use std::path::Path;
    ///
    /// let filter = FileFilter::new(&vec![r"^\.DS_Store$".to_string()]).unwrap();
    /// assert!(filter.should_exclude_filename(Path::new("/any/path/.DS_Store")));
    /// assert!(!filter.should_exclude_filename(Path::new("/any/path/normal.txt")));
    /// ```
    pub fn should_exclude_filename(&self, path: &Path) -> bool {
        // パターンが空の場合は何も除外しない
        if self.patterns.is_empty() {
            return false;
        }

        // ファイル名を取得
        let filename = match path.file_name() {
            Some(name) => match name.to_str() {
                Some(s) => s,
                None => return false,
            },
            None => return false,
        };

        // いずれかのパターンにマッチするかチェック
        for regex in &self.patterns {
            if regex.is_match(filename) {
                return true;
            }
        }

        false
    }

    /// パターン数を取得
    ///
    /// # 戻り値
    ///
    /// 登録されているパターンの数
    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }
}

/// デフォルトの除外パターンを提供
///
/// 一般的なバックアップ対象外のファイル・ディレクトリパターンを返します。
///
/// # 戻り値
///
/// デフォルト除外パターンのベクター
///
/// # 使用例
///
/// ```no_run
/// use backup_suite::core::filter::default_exclude_patterns;
///
/// let patterns = default_exclude_patterns();
/// println!("デフォルトパターン数: {}", patterns.len());
/// ```
pub fn default_exclude_patterns() -> Vec<String> {
    vec![
        // バージョン管理
        r"\.git$".to_string(),
        r"\.svn$".to_string(),
        r"\.hg$".to_string(),
        // ビルド成果物
        r"node_modules$".to_string(),
        r"target$".to_string(), // Rust
        r"dist$".to_string(),
        r"build$".to_string(),
        // キャッシュ
        r"\.cache$".to_string(),
        r"__pycache__$".to_string(),
        r"\.pytest_cache$".to_string(),
        // OS固有ファイル
        r"\.DS_Store$".to_string(),
        r"Thumbs\.db$".to_string(),
        r"desktop\.ini$".to_string(),
        // 一時ファイル
        r"~$".to_string(),
        r"\.tmp$".to_string(),
        r"\.swp$".to_string(),
        r"\.bak$".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_new_filter_with_valid_patterns() {
        let patterns = vec![r"\.git$".to_string(), r"node_modules".to_string()];
        let filter = FileFilter::new(&patterns);
        assert!(filter.is_ok());
        assert_eq!(filter.unwrap().pattern_count(), 2);
    }

    #[test]
    fn test_new_filter_with_invalid_pattern() {
        let patterns = vec![r"[invalid(".to_string()];
        let filter = FileFilter::new(&patterns);
        assert!(filter.is_err());
    }

    #[test]
    fn test_should_exclude_matching_path() {
        let patterns = vec![r"\.git$".to_string()];
        let filter = FileFilter::new(&patterns).unwrap();
        let path = PathBuf::from("/project/.git");
        assert!(filter.should_exclude(&path));
    }

    #[test]
    fn test_should_not_exclude_non_matching_path() {
        let patterns = vec![r"\.git$".to_string()];
        let filter = FileFilter::new(&patterns).unwrap();
        let path = PathBuf::from("/project/src");
        assert!(!filter.should_exclude(&path));
    }

    #[test]
    fn test_should_exclude_node_modules() {
        let patterns = vec![r"node_modules".to_string()];
        let filter = FileFilter::new(&patterns).unwrap();

        assert!(filter.should_exclude(&PathBuf::from("/project/node_modules")));
        assert!(filter.should_exclude(&PathBuf::from("/project/app/node_modules")));
        assert!(!filter.should_exclude(&PathBuf::from("/project/src")));
    }

    #[test]
    fn test_empty_patterns() {
        let filter = FileFilter::new(&[]).unwrap();
        assert!(!filter.should_exclude(&PathBuf::from("/any/path")));
        assert_eq!(filter.pattern_count(), 0);
    }

    #[test]
    fn test_should_exclude_filename() {
        let patterns = vec![r"^\.DS_Store$".to_string()];
        let filter = FileFilter::new(&patterns).unwrap();

        assert!(filter.should_exclude_filename(&PathBuf::from("/any/path/.DS_Store")));
        assert!(!filter.should_exclude_filename(&PathBuf::from("/any/path/normal.txt")));
    }

    #[test]
    fn test_multiple_patterns() {
        let patterns = vec![
            r"\.git$".to_string(),
            r"node_modules$".to_string(),
            r"\.DS_Store$".to_string(),
        ];
        let filter = FileFilter::new(&patterns).unwrap();

        assert!(filter.should_exclude(&PathBuf::from("/project/.git")));
        assert!(filter.should_exclude(&PathBuf::from("/project/node_modules")));
        assert!(filter.should_exclude(&PathBuf::from("/project/.DS_Store")));
        assert!(!filter.should_exclude(&PathBuf::from("/project/src/main.rs")));
    }

    #[test]
    fn test_default_exclude_patterns() {
        let patterns = default_exclude_patterns();
        assert!(!patterns.is_empty());

        let filter = FileFilter::new(&patterns).unwrap();
        assert!(filter.should_exclude(&PathBuf::from("/project/.git")));
        assert!(filter.should_exclude(&PathBuf::from("/project/node_modules")));
        assert!(filter.should_exclude(&PathBuf::from("/project/.DS_Store")));
        assert!(filter.should_exclude(&PathBuf::from("/project/target")));
    }

    #[test]
    fn test_complex_regex_patterns() {
        let patterns = vec![
            r"\.log$".to_string(),
            r"test_.*\.tmp$".to_string(),
            r"backup_\d{8}\.zip$".to_string(),
        ];
        let filter = FileFilter::new(&patterns).unwrap();

        assert!(filter.should_exclude(&PathBuf::from("/logs/app.log")));
        assert!(filter.should_exclude(&PathBuf::from("/test_file.tmp")));
        assert!(filter.should_exclude(&PathBuf::from("/backup_20250105.zip")));
        assert!(!filter.should_exclude(&PathBuf::from("/data.txt")));
    }
}
