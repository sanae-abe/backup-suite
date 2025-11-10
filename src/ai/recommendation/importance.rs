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
    file_names: Vec<String>,
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
            file_names: Vec::new(),
            base_score,
            category,
        }
    }

    const fn with_file_names(
        extensions: Vec<String>,
        dir_patterns: Vec<String>,
        file_names: Vec<String>,
        base_score: u8,
        category: String,
    ) -> Self {
        Self {
            extensions,
            dir_patterns,
            file_names,
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
                vec!["md".to_string()],
                vec![
                    "docs".to_string(),
                    ".claude".to_string(),
                    "documentation".to_string(),
                ],
                85,
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
                    "cfg".to_string(),
                    "xml".to_string(),
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
                vec![".cache".to_string(), "cache".to_string(), "temp".to_string()],
                10,
                "一時ファイル".to_string(),
            ),
            ImportanceRule::new(
                vec!["log".to_string()],
                vec!["logs".to_string(), ".log".to_string()],
                20,
                "ログ".to_string(),
            ),
            // OS固有ファイル（極低重要度）
            ImportanceRule::with_file_names(
                vec![],
                vec![],
                vec![
                    ".DS_Store".to_string(),   // macOS
                    "Thumbs.db".to_string(),   // Windows
                    "desktop.ini".to_string(), // Windows
                ],
                5,
                "OS固有ファイル".to_string(),
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

        // ファイル/ディレクトリの存在確認
        if !path.exists() {
            return Err(AiError::InvalidParameter(format!(
                "File or directory does not exist: {}",
                path.display()
            )));
        }

        // キャッシュチェック
        {
            let cache = self.cache.lock().unwrap();
            if let Some(&importance) = cache.get(path) {
                return self.create_result(path, importance);
            }
        }

        // ディレクトリの場合は専用ロジックを使用
        if path.is_dir() {
            return self.evaluate_directory(path);
        }

        // 拡張子を取得
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase());

        // パス文字列を取得
        let path_str = path.to_string_lossy().to_lowercase();

        // dotfiles（設定ファイル）の検出
        let is_dotfile = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.starts_with('.') && s != "." && s != "..")
            .unwrap_or(false);

        // ファイル名を取得
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_default();

        // ファイル名マッチング（最優先）
        let mut best_score = 30u8; // デフォルトスコア
        let mut file_name_matched = false;

        for rule in &self.rules {
            if !file_name.is_empty()
                && rule
                    .file_names
                    .iter()
                    .any(|n| n.to_lowercase() == file_name)
            {
                best_score = rule.base_score;
                file_name_matched = true;
                break; // ファイル名マッチは最優先なので即座に終了
            }
        }

        // ファイル名マッチングで見つからなかった場合のみ、拡張子とディレクトリパターンを確認
        if !file_name_matched {
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
        }

        // dotfiles は設定ファイルとして扱う（デフォルトスコアの場合のみ）
        // OS固有ファイル等の極低重要度ファイル（スコア < 30）は除外
        if is_dotfile && (30..80).contains(&best_score) {
            best_score = 85;
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

    /// ディレクトリの重要度を評価
    ///
    /// # Errors
    ///
    /// パスが不正な場合やファイルアクセスに失敗した場合はエラーを返します。
    fn evaluate_directory(&self, path: &Path) -> AiResult<FileImportanceResult> {
        use walkdir::WalkDir;

        let mut score = 30u8; // デフォルトスコア
        let mut category = "ディレクトリ".to_string();

        // プロジェクトマーカーファイルを検出
        let has_package_json = path.join("package.json").exists();
        let has_cargo_toml = path.join("Cargo.toml").exists();
        let has_requirements_txt = path.join("requirements.txt").exists();
        let has_git = path.join(".git").exists();
        let has_src = path.join("src").exists();
        let _has_tests = path.join("tests").exists() || path.join("test").exists();

        // プロジェクトタイプ判定
        if has_cargo_toml {
            score = 95;
            category = "Rustプロジェクト".to_string();
        } else if has_package_json {
            score = 95;
            category = "Node.jsプロジェクト".to_string();
        } else if has_requirements_txt {
            score = 90;
            category = "Pythonプロジェクト".to_string();
        } else if has_src {
            score = 85;
            category = "ソースコードプロジェクト".to_string();
        } else if has_git {
            score = 75;
            category = "Git管理ディレクトリ".to_string();
        }

        // ディレクトリ内のファイルをサンプリングして評価
        let mut file_count = 0;
        let mut total_score = 0u32;
        let mut high_importance_count = 0;

        for entry in WalkDir::new(path)
            .max_depth(3) // 深すぎる探索を避ける
            .follow_links(false)
            .into_iter()
            .filter_map(Result::ok)
            .take(100)
        // 最大100ファイルをサンプリング
        {
            let entry_path = entry.path();
            if entry_path.is_file() {
                if let Ok(result) = self.evaluate(entry_path) {
                    file_count += 1;
                    total_score += result.score().get() as u32;
                    if result.score().is_high() {
                        high_importance_count += 1;
                    }
                }
            }
        }

        // サンプリング結果に基づいてスコアを調整
        if file_count > 0 {
            let avg_score = (total_score / file_count) as u8;
            let high_ratio = high_importance_count as f64 / file_count as f64;

            // 高重要度ファイルが50%以上なら高スコア
            if high_ratio >= 0.5 {
                score = score.max(90);
            } else if high_ratio >= 0.3 {
                score = score.max(75);
            } else {
                // 平均スコアとマーカー検出スコアの高い方を採用
                score = score.max(avg_score);
            }
        }

        let importance = FileImportance::new(score).map_err(AiError::InvalidParameter)?;

        // キャッシュ更新
        {
            let mut cache = self.cache.lock().unwrap();
            cache.insert(path.to_path_buf(), importance);
        }

        let priority = if importance.is_high() {
            Priority::High
        } else if importance.is_medium() {
            Priority::Medium
        } else {
            Priority::Low
        };

        let reason = if file_count > 0 {
            format!(
                "{} (サンプリング: {}ファイル, 高重要度: {}件, スコア: {})",
                category, file_count, high_importance_count, score
            )
        } else {
            format!("{} (スコア: {})", category, score)
        };

        Ok(FileImportanceResult::new(
            path.to_path_buf(),
            importance,
            priority,
            category,
            reason,
        ))
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
        // ファイル名を取得
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_default();

        // dotfiles チェック
        let is_dotfile = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.starts_with('.') && s != "." && s != "..")
            .unwrap_or(false);

        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase());

        // ファイル名マッチング（最優先）
        if !file_name.is_empty() {
            for rule in &self.rules {
                if rule
                    .file_names
                    .iter()
                    .any(|n| n.to_lowercase() == file_name)
                {
                    let filename = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown");
                    let reason = format!(
                        "{} (ファイル名: {}, スコア: {})",
                        rule.category,
                        filename,
                        importance.get()
                    );
                    return (rule.category.clone(), reason);
                }
            }
        }

        // 拡張子マッチング
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

        // dotfile の場合
        if is_dotfile {
            let filename = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            let reason = format!(
                "設定ファイル (dotfile: {}, スコア: {})",
                filename,
                importance.get()
            );
            return ("設定ファイル".to_string(), reason);
        }

        // デフォルト
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
        use std::fs;
        use std::io::Write;

        let evaluator = ImportanceEvaluator::new();

        // 一時ファイルを作成
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_eval_doc_report.pdf");
        let mut file = fs::File::create(&path).unwrap();
        file.write_all(b"test pdf content").unwrap();

        let result = evaluator.evaluate(&path).unwrap();
        assert!(result.score().is_high());
        assert_eq!(result.priority(), &Priority::High);
        assert_eq!(result.category(), "ドキュメント");

        // クリーンアップ
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_evaluate_source_code() {
        use std::fs;
        use std::io::Write;

        let evaluator = ImportanceEvaluator::new();

        // 一時ファイルを作成
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_eval_src_main.rs");
        let mut file = fs::File::create(&path).unwrap();
        file.write_all(b"fn main() {}").unwrap();

        let result = evaluator.evaluate(&path).unwrap();
        assert!(result.score().is_high());
        assert_eq!(result.category(), "ソースコード");

        // クリーンアップ
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_evaluate_temp_file() {
        use std::fs;
        use std::io::Write;

        let evaluator = ImportanceEvaluator::new();

        // 一時ファイルを作成
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_eval_temp_temp.tmp");
        let mut file = fs::File::create(&path).unwrap();
        file.write_all(b"temporary data").unwrap();

        let result = evaluator.evaluate(&path).unwrap();
        assert!(result.score().is_low());
        assert_eq!(result.priority(), &Priority::Low);

        // クリーンアップ
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_evaluate_image() {
        use std::fs;
        use std::io::Write;

        let evaluator = ImportanceEvaluator::new();

        // 一時ファイルを作成
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_eval_img_vacation.jpg");
        let mut file = fs::File::create(&path).unwrap();
        file.write_all(b"fake jpg data").unwrap();

        let result = evaluator.evaluate(&path).unwrap();
        assert!(result.score().is_medium());
        assert_eq!(result.category(), "画像");

        // クリーンアップ
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_cache() {
        use std::fs;
        use std::io::Write;

        let evaluator = ImportanceEvaluator::new();

        // 一時ファイルを作成
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("cache_test_report.pdf");
        let mut file = fs::File::create(&path).unwrap();
        file.write_all(b"test pdf content").unwrap();

        // 初回評価
        let result1 = evaluator.evaluate(&path).unwrap();

        // キャッシュから取得
        let cached = evaluator.evaluate_cached(&path).unwrap();
        assert_eq!(result1.score(), cached);

        // キャッシュサイズ確認
        assert_eq!(evaluator.cache_size(), 1);

        // キャッシュクリア
        evaluator.clear_cache();
        assert_eq!(evaluator.cache_size(), 0);

        // クリーンアップ
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_bonus_score() {
        use std::fs;
        use std::io::Write;

        let evaluator = ImportanceEvaluator::new();

        // 一時ファイルを作成
        let temp_dir = std::env::temp_dir();

        // バージョン番号付きファイル
        let path1 = temp_dir.join("test_bonus_document_v2.pdf");
        let mut file1 = fs::File::create(&path1).unwrap();
        file1.write_all(b"test pdf content v2").unwrap();
        let result1 = evaluator.evaluate(&path1).unwrap();

        // 通常のファイル
        let path2 = temp_dir.join("test_bonus_document.pdf");
        let mut file2 = fs::File::create(&path2).unwrap();
        file2.write_all(b"test pdf content").unwrap();
        let result2 = evaluator.evaluate(&path2).unwrap();

        // バージョン番号付きの方がスコアが高い
        assert!(result1.score().get() >= result2.score().get());

        // クリーンアップ
        let _ = fs::remove_file(&path1);
        let _ = fs::remove_file(&path2);
    }

    #[test]
    fn test_dotfiles() {
        use std::fs;
        use std::io::Write;

        let evaluator = ImportanceEvaluator::new();

        // テスト用の一時ファイルを作成
        let temp_dir = std::env::temp_dir();
        let test_files = vec![
            temp_dir.join(".zshrc"),
            temp_dir.join(".bashrc"),
            temp_dir.join(".gitconfig"),
            temp_dir.join(".vimrc"),
        ];

        for path in &test_files {
            let mut file = fs::File::create(path).unwrap();
            file.write_all(b"test content").unwrap();
        }

        // .zshrc
        let result1 = evaluator.evaluate(&test_files[0]).unwrap();
        assert!(result1.score().is_high());
        assert_eq!(result1.category(), "設定ファイル");
        assert!(result1.reason().contains("dotfile"));

        // .bashrc
        let result2 = evaluator.evaluate(&test_files[1]).unwrap();
        assert!(result2.score().is_high());
        assert_eq!(result2.category(), "設定ファイル");

        // .gitconfig
        let result3 = evaluator.evaluate(&test_files[2]).unwrap();
        assert!(result3.score().is_high());
        assert_eq!(result3.category(), "設定ファイル");

        // .vimrc
        let result4 = evaluator.evaluate(&test_files[3]).unwrap();
        assert!(result4.score().is_high());
        assert_eq!(result4.category(), "設定ファイル");

        // クリーンアップ
        for path in &test_files {
            let _ = fs::remove_file(path);
        }
    }

    #[test]
    fn test_nonexistent_file() {
        let evaluator = ImportanceEvaluator::new();
        let path = Path::new("/nonexistent/file.txt");

        let result = evaluator.evaluate(path);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("File or directory does not exist"));
    }
}
