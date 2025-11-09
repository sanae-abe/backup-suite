//! ファイル重要度判定エンジン
//!
//! ルールベーススコアリングによるファイル重要度の自動判定を提供します。

use crate::ai::error::{AiError, AiResult};
use crate::ai::types::FileImportance;
use crate::core::Priority;
use crate::security::path::validate_path_safety;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// ファイル重要度評価結果
///
/// # 使用例
///
/// ```rust
/// use backup_suite::ai::recommendation::FileImportanceResult;
/// use backup_suite::ai::FileImportance;
/// use backup_suite::Priority;
/// use std::path::PathBuf;
///
/// let result = FileImportanceResult::new(
///     PathBuf::from("/home/user/documents/report.pdf"),
///     FileImportance::new(90).unwrap(),
///     Priority::High,
///     "ドキュメント".to_string(),
///     "PDFファイル（頻繁に更新）".to_string()
/// );
/// assert_eq!(result.priority(), &Priority::High);
/// ```
#[derive(Debug, Clone)]
pub struct FileImportanceResult {
    path: PathBuf,
    score: FileImportance,
    priority: Priority,
    category: String,
    reason: String,
}

impl FileImportanceResult {
    /// 新しい評価結果を作成
    #[must_use]
    pub const fn new(
        path: PathBuf,
        score: FileImportance,
        priority: Priority,
        category: String,
        reason: String,
    ) -> Self {
        Self {
            path,
            score,
            priority,
            category,
            reason,
        }
    }

    /// ファイルパスを取得
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// スコアを取得
    #[must_use]
    pub const fn score(&self) -> FileImportance {
        self.score
    }

    /// 優先度を取得
    #[must_use]
    pub const fn priority(&self) -> &Priority {
        &self.priority
    }

    /// カテゴリを取得
    #[must_use]
    pub fn category(&self) -> &str {
        &self.category
    }

    /// 理由を取得
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

/// 重要度判定ルール
#[derive(Debug, Clone)]
struct ImportanceRule {
    extensions: Vec<String>,
    dir_patterns: Vec<String>,
    base_score: u8,
    category: String,
}

impl ImportanceRule {
    const fn new(
        extensions: Vec<String>,
        dir_patterns: Vec<String>,
        base_score: u8,
        category: String,
    ) -> Self {
        Self {
            extensions,
            dir_patterns,
            base_score,
            category,
        }
    }
}

/// 重要度評価エンジン
///
/// ルールベースでファイルの重要度を判定します。
///
/// # 使用例
///
/// ```rust,no_run
/// use backup_suite::ai::recommendation::ImportanceEvaluator;
/// use std::path::Path;
///
/// let evaluator = ImportanceEvaluator::new();
/// let path = Path::new("/home/user/documents/report.pdf");
///
/// match evaluator.evaluate(path) {
///     Ok(result) => {
///         println!("重要度: {} ({})", result.score().get(), result.category());
///         println!("優先度: {:?}", result.priority());
///         println!("理由: {}", result.reason());
///     }
///     Err(e) => eprintln!("エラー: {}", e),
/// }
/// ```
#[derive(Debug)]
pub struct ImportanceEvaluator {
    rules: Vec<ImportanceRule>,
    cache: Mutex<HashMap<PathBuf, FileImportance>>,
}

impl ImportanceEvaluator {
    /// 新しい評価エンジンを作成
    #[must_use]
    pub fn new() -> Self {
        let rules = vec![
            // 高重要度ファイル（80-100点）
            ImportanceRule::new(
                vec![
                    "docx".to_string(),
                    "pdf".to_string(),
                    "xlsx".to_string(),
                    "pptx".to_string(),
                    "odt".to_string(),
                    "ods".to_string(),
                ],
                vec!["documents".to_string(), "docs".to_string()],
                90,
                "ドキュメント".to_string(),
            ),
            ImportanceRule::new(
                vec![
                    "rs".to_string(),
                    "py".to_string(),
                    "js".to_string(),
                    "ts".to_string(),
                    "tsx".to_string(),
                    "jsx".to_string(),
                    "java".to_string(),
                    "go".to_string(),
                    "cpp".to_string(),
                    "c".to_string(),
                    "h".to_string(),
                ],
                vec!["src".to_string(), "source".to_string()],
                95,
                "ソースコード".to_string(),
            ),
            ImportanceRule::new(
                vec![
                    "toml".to_string(),
                    "yaml".to_string(),
                    "yml".to_string(),
                    "json".to_string(),
                    "ini".to_string(),
                    "conf".to_string(),
                ],
                vec!["config".to_string(), ".config".to_string()],
                85,
                "設定ファイル".to_string(),
            ),
            // 中重要度ファイル（40-79点）
            ImportanceRule::new(
                vec![
                    "jpg".to_string(),
                    "jpeg".to_string(),
                    "png".to_string(),
                    "gif".to_string(),
                    "bmp".to_string(),
                    "svg".to_string(),
                ],
                vec!["photos".to_string(), "images".to_string()],
                60,
                "画像".to_string(),
            ),
            ImportanceRule::new(
                vec![
                    "csv".to_string(),
                    "db".to_string(),
                    "sqlite".to_string(),
                    "sql".to_string(),
                ],
                vec!["data".to_string(), "database".to_string()],
                70,
                "データ".to_string(),
            ),
            ImportanceRule::new(
                vec![
                    "mp4".to_string(),
                    "avi".to_string(),
                    "mkv".to_string(),
                    "mov".to_string(),
                ],
                vec!["videos".to_string()],
                50,
                "動画".to_string(),
            ),
            // 低重要度ファイル（0-39点）
            ImportanceRule::new(
                vec![
                    "tmp".to_string(),
                    "temp".to_string(),
                    "cache".to_string(),
                    "bak".to_string(),
                ],
                vec![".cache".to_string(), "cache".to_string()],
                10,
                "一時ファイル".to_string(),
            ),
            ImportanceRule::new(
                vec!["log".to_string()],
                vec!["logs".to_string(), ".log".to_string()],
                20,
                "ログ".to_string(),
            ),
        ];

        Self {
            rules,
            cache: Mutex::new(HashMap::new()),
        }
    }

    /// ファイルの重要度を評価
    ///
    /// # Errors
    ///
    /// パスが不正な場合やファイルアクセスに失敗した場合はエラーを返します。
    pub fn evaluate(&self, path: &Path) -> AiResult<FileImportanceResult> {
        // パストラバーサル対策
        validate_path_safety(path)?;

        // キャッシュチェック
        {
            let cache = self.cache.lock().unwrap();
            if let Some(&importance) = cache.get(path) {
                return self.create_result(path, importance);
            }
        }

        // 拡張子を取得
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase());

        // パス文字列を取得
        let path_str = path.to_string_lossy().to_lowercase();

        // ルールマッチング
        let mut best_score = 30u8; // デフォルトスコア

        for rule in &self.rules {
            let mut matched = false;

            // 拡張子マッチ
            if let Some(ref ext) = extension {
                if rule.extensions.iter().any(|e| e == ext) {
                    matched = true;
                }
            }

            // ディレクトリパターンマッチ
            if rule.dir_patterns.iter().any(|p| path_str.contains(p)) {
                matched = true;
            }

            if matched && rule.base_score > best_score {
                best_score = rule.base_score;
            }
        }

        // ボーナススコア計算
        let bonus = self.calculate_bonus_score(path);
        let final_score = (best_score.saturating_add(bonus)).min(100);

        let importance = FileImportance::new(final_score).map_err(AiError::InvalidParameter)?;

        // キャッシュ更新
        {
            let mut cache = self.cache.lock().unwrap();
            cache.insert(path.to_path_buf(), importance);
        }

        self.create_result(path, importance)
    }

    /// キャッシュを使った評価（高速）
    ///
    /// # Errors
    ///
    /// パスが不正な場合やファイルアクセスに失敗した場合はエラーを返します。
    pub fn evaluate_cached(&self, path: &Path) -> AiResult<FileImportance> {
        // パストラバーサル対策
        validate_path_safety(path)?;

        // キャッシュヒット確認
        {
            let cache = self.cache.lock().unwrap();
            if let Some(&importance) = cache.get(path) {
                return Ok(importance);
            }
        }

        // 評価実行
        let result = self.evaluate(path)?;
        Ok(result.score)
    }

    /// ボーナススコアを計算
    fn calculate_bonus_score(&self, path: &Path) -> u8 {
        let mut bonus = 0u8;

        // ファイル名にバージョン番号が含まれている場合（重要な可能性）
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if file_name.contains("_v") || file_name.contains("_version") {
            bonus = bonus.saturating_add(5);
        }

        // "final", "important"などのキーワードが含まれている場合
        if file_name.to_lowercase().contains("final")
            || file_name.to_lowercase().contains("important")
        {
            bonus = bonus.saturating_add(10);
        }

        bonus
    }

    /// 評価結果を作成
    fn create_result(
        &self,
        path: &Path,
        importance: FileImportance,
    ) -> AiResult<FileImportanceResult> {
        let priority = if importance.is_high() {
            Priority::High
        } else if importance.is_medium() {
            Priority::Medium
        } else {
            Priority::Low
        };

        // カテゴリと理由を再計算
        let (category, reason) = self.determine_category_and_reason(path, importance);

        Ok(FileImportanceResult::new(
            path.to_path_buf(),
            importance,
            priority,
            category,
            reason,
        ))
    }

    /// カテゴリと理由を決定
    fn determine_category_and_reason(
        &self,
        path: &Path,
        importance: FileImportance,
    ) -> (String, String) {
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase());

        for rule in &self.rules {
            if let Some(ref ext) = extension {
                if rule.extensions.iter().any(|e| e == ext) {
                    let reason = format!(
                        "{} (拡張子: .{}, スコア: {})",
                        rule.category,
                        ext,
                        importance.get()
                    );
                    return (rule.category.clone(), reason);
                }
            }
        }

        (
            "その他".to_string(),
            format!("スコア: {}", importance.get()),
        )
    }

    /// キャッシュをクリア
    pub fn clear_cache(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
    }

    /// キャッシュサイズを取得
    #[must_use]
    pub fn cache_size(&self) -> usize {
        let cache = self.cache.lock().unwrap();
        cache.len()
    }
}

impl Default for ImportanceEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_document() {
        let evaluator = ImportanceEvaluator::new();
        let path = Path::new("/home/user/documents/report.pdf");

        let result = evaluator.evaluate(path).unwrap();
        assert!(result.score().is_high());
        assert_eq!(result.priority(), &Priority::High);
        assert_eq!(result.category(), "ドキュメント");
    }

    #[test]
    fn test_evaluate_source_code() {
        let evaluator = ImportanceEvaluator::new();
        let path = Path::new("/home/user/projects/src/main.rs");

        let result = evaluator.evaluate(path).unwrap();
        assert!(result.score().is_high());
        assert_eq!(result.category(), "ソースコード");
    }

    #[test]
    fn test_evaluate_temp_file() {
        let evaluator = ImportanceEvaluator::new();
        let path = Path::new("/home/user/.cache/temp.tmp");

        let result = evaluator.evaluate(path).unwrap();
        assert!(result.score().is_low());
        assert_eq!(result.priority(), &Priority::Low);
    }

    #[test]
    fn test_evaluate_image() {
        let evaluator = ImportanceEvaluator::new();
        let path = Path::new("/home/user/photos/vacation.jpg");

        let result = evaluator.evaluate(path).unwrap();
        assert!(result.score().is_medium());
        assert_eq!(result.category(), "画像");
    }

    #[test]
    fn test_cache() {
        let evaluator = ImportanceEvaluator::new();
        let path = Path::new("/home/user/documents/report.pdf");

        // 初回評価
        let result1 = evaluator.evaluate(path).unwrap();

        // キャッシュから取得
        let cached = evaluator.evaluate_cached(path).unwrap();
        assert_eq!(result1.score(), cached);

        // キャッシュサイズ確認
        assert_eq!(evaluator.cache_size(), 1);

        // キャッシュクリア
        evaluator.clear_cache();
        assert_eq!(evaluator.cache_size(), 0);
    }

    #[test]
    fn test_bonus_score() {
        let evaluator = ImportanceEvaluator::new();

        // バージョン番号付きファイル
        let path1 = Path::new("/home/user/document_v2.pdf");
        let result1 = evaluator.evaluate(path1).unwrap();

        // 通常のファイル
        let path2 = Path::new("/home/user/document.pdf");
        let result2 = evaluator.evaluate(path2).unwrap();

        // バージョン番号付きの方がスコアが高い
        assert!(result1.score().get() >= result2.score().get());
    }
}
