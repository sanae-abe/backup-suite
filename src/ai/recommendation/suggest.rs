//! バックアップ対象自動提案エンジン
//!
//! ファイルシステムを走査し、バックアップ対象の自動提案を行います。

use crate::ai::error::{AiError, AiResult};
use crate::ai::recommendation::ImportanceEvaluator;
use crate::ai::types::FileImportance;
use crate::core::Priority;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// バックアップ提案
///
/// # 使用例
///
/// ```rust
/// use backup_suite::ai::recommendation::BackupSuggestion;
/// use backup_suite::ai::FileImportance;
/// use backup_suite::Priority;
/// use std::path::PathBuf;
///
/// let suggestion = BackupSuggestion::new(
///     PathBuf::from("/home/user/documents"),
///     Priority::High,
///     FileImportance::new(90).unwrap(),
///     "重要なドキュメントが含まれています".to_string()
/// );
/// assert_eq!(suggestion.priority(), &Priority::High);
/// ```
#[derive(Debug, Clone)]
pub struct BackupSuggestion {
    path: PathBuf,
    priority: Priority,
    importance: FileImportance,
    reason: String,
}

impl BackupSuggestion {
    /// 新しい提案を作成
    #[must_use]
    pub const fn new(
        path: PathBuf,
        priority: Priority,
        importance: FileImportance,
        reason: String,
    ) -> Self {
        Self {
            path,
            priority,
            importance,
            reason,
        }
    }

    /// パスを取得
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// 優先度を取得
    #[must_use]
    pub const fn priority(&self) -> &Priority {
        &self.priority
    }

    /// 重要度を取得
    #[must_use]
    pub const fn importance(&self) -> FileImportance {
        self.importance
    }

    /// 理由を取得
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

/// 提案エンジン
///
/// ディレクトリを走査してバックアップ対象を自動提案します。
///
/// # 使用例
///
/// ```rust,no_run
/// use backup_suite::ai::recommendation::SuggestEngine;
/// use std::path::Path;
///
/// let engine = SuggestEngine::new();
/// let home_dir = Path::new("/home/user");
///
/// match engine.suggest_backup_targets(home_dir) {
///     Ok(suggestions) => {
///         for suggestion in suggestions {
///             println!("提案: {:?} (優先度: {:?})", suggestion.path(), suggestion.priority());
///             println!("理由: {}", suggestion.reason());
///         }
///     }
///     Err(e) => eprintln!("エラー: {}", e),
/// }
/// ```
#[derive(Debug)]
pub struct SuggestEngine {
    evaluator: ImportanceEvaluator,
    max_depth: usize,
    min_importance_threshold: u8,
}

impl SuggestEngine {
    /// 新しい提案エンジンを作成
    #[must_use]
    pub fn new() -> Self {
        Self {
            evaluator: ImportanceEvaluator::new(),
            max_depth: 3,
            min_importance_threshold: 70, // 70点以上を提案対象とする
        }
    }

    /// バックアップ対象を提案
    ///
    /// # Errors
    ///
    /// ファイルシステムアクセスに失敗した場合はエラーを返します。
    pub fn suggest_backup_targets(&self, base_path: &Path) -> AiResult<Vec<BackupSuggestion>> {
        if !base_path.exists() {
            return Err(AiError::InvalidParameter(format!(
                "パスが存在しません: {:?}",
                base_path
            )));
        }

        if !base_path.is_dir() {
            return Err(AiError::InvalidParameter(format!(
                "ディレクトリではありません: {:?}",
                base_path
            )));
        }

        let mut suggestions = Vec::new();

        // ディレクトリを走査
        for entry in WalkDir::new(base_path)
            .max_depth(self.max_depth)
            .follow_links(false)
        {
            let entry = entry.map_err(|e| AiError::IoError(e.into()))?;

            // ディレクトリのみを対象
            if !entry.file_type().is_dir() {
                continue;
            }

            let dir_path = entry.path();

            // ベースパス自体はスキップ（サブディレクトリのみ評価）
            if dir_path == base_path {
                continue;
            }

            // システムディレクトリをスキップ
            if self.is_system_directory(dir_path) {
                continue;
            }

            // ディレクトリ内のファイルを評価
            let (avg_importance, file_count) = self.evaluate_directory(dir_path)?;

            // 重要度が閾値以上で、かつファイルが存在する場合のみ提案
            if avg_importance.get() >= self.min_importance_threshold && file_count > 0 {
                let priority = if avg_importance.is_high() {
                    Priority::High
                } else if avg_importance.is_medium() {
                    Priority::Medium
                } else {
                    Priority::Low
                };

                let reason = format!(
                    "平均重要度: {}, ファイル数: {}",
                    avg_importance.get(),
                    file_count
                );

                suggestions.push(BackupSuggestion::new(
                    dir_path.to_path_buf(),
                    priority,
                    avg_importance,
                    reason,
                ));
            }
        }

        // 優先度と重要度でソート
        suggestions.sort_by(|a, b| {
            b.importance.get().cmp(&a.importance.get()).then_with(|| {
                let a_priority_score = match a.priority {
                    Priority::High => 3,
                    Priority::Medium => 2,
                    Priority::Low => 1,
                };
                let b_priority_score = match b.priority {
                    Priority::High => 3,
                    Priority::Medium => 2,
                    Priority::Low => 1,
                };
                b_priority_score.cmp(&a_priority_score)
            })
        });

        Ok(suggestions)
    }

    /// ディレクトリ内のファイルを評価
    fn evaluate_directory(&self, dir_path: &Path) -> AiResult<(FileImportance, usize)> {
        let mut total_score = 0u64;
        let mut file_count = 0usize;

        for entry in WalkDir::new(dir_path).max_depth(1).follow_links(false) {
            let entry = entry.map_err(|e| AiError::IoError(e.into()))?;

            if entry.file_type().is_file() {
                match self.evaluator.evaluate(entry.path()) {
                    Ok(result) => {
                        total_score += result.score().get() as u64;
                        file_count += 1;
                    }
                    Err(_) => {
                        // 個別ファイルのエラーは無視して続行
                        continue;
                    }
                }
            }
        }

        if file_count == 0 {
            return Ok((
                FileImportance::new(0).map_err(AiError::InvalidParameter)?,
                0,
            ));
        }

        let avg_score = (total_score / file_count as u64) as u8;
        let importance =
            FileImportance::new(avg_score.min(100)).map_err(AiError::InvalidParameter)?;

        Ok((importance, file_count))
    }

    /// システムディレクトリかどうかを判定
    fn is_system_directory(&self, path: &Path) -> bool {
        // ディレクトリ名のみをチェック（フルパスではなく）
        let dir_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_default();

        let system_patterns = [
            ".cache",
            ".tmp",
            "node_modules",
            "target",
            ".git",
            ".svn",
            "__pycache__",
            "dist",
            "build",
        ];

        system_patterns.iter().any(|pattern| {
            // 完全一致または含むかをチェック
            &dir_name == pattern || dir_name.contains(pattern)
        })
    }

    /// 最小重要度閾値を設定
    pub fn with_min_importance_threshold(mut self, threshold: u8) -> Self {
        self.min_importance_threshold = threshold.min(100);
        self
    }

    /// 最大深度を設定
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }
}

impl Default for SuggestEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_suggest_engine_creation() {
        let engine = SuggestEngine::new();
        assert_eq!(engine.max_depth, 3);
        assert_eq!(engine.min_importance_threshold, 70);
    }

    #[test]
    fn test_suggest_engine_builder() {
        let engine = SuggestEngine::new()
            .with_max_depth(5)
            .with_min_importance_threshold(80);

        assert_eq!(engine.max_depth, 5);
        assert_eq!(engine.min_importance_threshold, 80);
    }

    #[test]
    fn test_is_system_directory() {
        let engine = SuggestEngine::new();

        assert!(engine.is_system_directory(Path::new("/home/user/.cache")));
        assert!(engine.is_system_directory(Path::new("/project/node_modules")));
        assert!(engine.is_system_directory(Path::new("/code/target")));
        assert!(!engine.is_system_directory(Path::new("/home/user/documents")));
    }

    #[test]
    fn test_suggest_backup_targets_invalid_path() {
        let engine = SuggestEngine::new();
        let result = engine.suggest_backup_targets(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }

    #[test]
    fn test_suggest_backup_targets() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        // テスト用のディレクトリ構造を作成
        let docs_dir = base_path.join("documents");
        fs::create_dir(&docs_dir).unwrap();
        // PDFファイルを作成（重要度90点）
        fs::write(docs_dir.join("report.pdf"), b"important document content").unwrap();
        fs::write(docs_dir.join("final.docx"), b"final document").unwrap();

        let cache_dir = base_path.join(".cache");
        fs::create_dir(&cache_dir).unwrap();
        fs::write(cache_dir.join("temp.tmp"), b"test").unwrap();

        // 重要度閾値を下げてテスト（PDFファイルは90点なので確実にマッチ）
        let engine = SuggestEngine::new()
            .with_min_importance_threshold(50)
            .with_max_depth(3);
        let suggestions = engine.suggest_backup_targets(base_path).unwrap();

        // documentsディレクトリが提案されるはず
        let has_documents = suggestions
            .iter()
            .any(|s| s.path().to_string_lossy().contains("documents"));
        assert!(
            has_documents,
            "documents directory should be suggested. Suggestions: {:?}",
            suggestions.iter().map(|s| s.path()).collect::<Vec<_>>()
        );

        // .cacheディレクトリはシステムディレクトリなので提案されない
        let has_cache = suggestions
            .iter()
            .any(|s| s.path().to_string_lossy().contains(".cache"));
        assert!(
            !has_cache,
            ".cache should not be suggested (system directory)"
        );
    }
}
