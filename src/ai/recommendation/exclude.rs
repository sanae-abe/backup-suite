//! 除外ファイル自動検出エンジン
//!
//! 一時ファイル、キャッシュ、再生成可能なファイルを検出して除外を提案します。

use crate::ai::error::{AiError, AiResult};
use crate::ai::types::PredictionConfidence;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// 除外推奨
///
/// # 使用例
///
/// ```rust
/// use backup_suite::ai::recommendation::ExcludeRecommendation;
/// use backup_suite::ai::PredictionConfidence;
///
/// let recommendation = ExcludeRecommendation::new(
///     "node_modules/".to_string(),
///     PredictionConfidence::new(0.95).unwrap(),
///     5.2,
///     "再生成可能な依存関係ファイル".to_string()
/// );
/// assert_eq!(recommendation.pattern(), "node_modules/");
/// assert_eq!(recommendation.size_reduction_gb(), 5.2);
/// ```
#[derive(Debug, Clone)]
pub struct ExcludeRecommendation {
    pattern: String,
    confidence: PredictionConfidence,
    size_reduction_gb: f64,
    reason: String,
}

impl ExcludeRecommendation {
    /// 新しい除外推奨を作成
    #[must_use]
    pub const fn new(
        pattern: String,
        confidence: PredictionConfidence,
        size_reduction_gb: f64,
        reason: String,
    ) -> Self {
        Self {
            pattern,
            confidence,
            size_reduction_gb,
            reason,
        }
    }

    /// 除外パターンを取得
    #[must_use]
    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    /// 信頼度を取得
    #[must_use]
    pub const fn confidence(&self) -> PredictionConfidence {
        self.confidence
    }

    /// サイズ削減量（GB）を取得
    #[must_use]
    pub const fn size_reduction_gb(&self) -> f64 {
        self.size_reduction_gb
    }

    /// 理由を取得
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

/// 除外パターン定義
#[derive(Debug, Clone)]
struct ExcludePattern {
    pattern: String,
    reason: String,
    confidence: f64,
    is_directory: bool,
}

/// 除外推奨エンジン
///
/// ディレクトリを走査して除外すべきファイル/ディレクトリを検出します。
///
/// # 使用例
///
/// ```rust,no_run
/// use backup_suite::ai::recommendation::ExcludeRecommendationEngine;
/// use std::path::Path;
///
/// let engine = ExcludeRecommendationEngine::new();
/// let project_dir = Path::new("/home/user/projects");
///
/// match engine.suggest_exclude_patterns(project_dir) {
///     Ok(recommendations) => {
///         for rec in recommendations {
///             println!("除外推奨: {} (削減: {:.2}GB)", rec.pattern(), rec.size_reduction_gb());
///             println!("理由: {}", rec.reason());
///             println!("信頼度: {:.0}%", rec.confidence().as_percentage());
///         }
///     }
///     Err(e) => eprintln!("エラー: {}", e),
/// }
/// ```
#[derive(Debug)]
pub struct ExcludeRecommendationEngine {
    patterns: Vec<ExcludePattern>,
}

impl ExcludeRecommendationEngine {
    /// 新しい除外推奨エンジンを作成
    #[must_use]
    pub fn new() -> Self {
        let patterns = vec![
            // 開発環境の依存関係（高信頼度）
            ExcludePattern {
                pattern: "node_modules".to_string(),
                reason: "npm/yarn依存関係（package.jsonから再生成可能）".to_string(),
                confidence: 0.99,
                is_directory: true,
            },
            ExcludePattern {
                pattern: "target".to_string(),
                reason: "Rustビルド成果物（Cargo.tomlから再生成可能）".to_string(),
                confidence: 0.99,
                is_directory: true,
            },
            ExcludePattern {
                pattern: "vendor".to_string(),
                reason: "依存関係ベンダリング（再生成可能）".to_string(),
                confidence: 0.95,
                is_directory: true,
            },
            ExcludePattern {
                pattern: "__pycache__".to_string(),
                reason: "Pythonキャッシュ（自動生成）".to_string(),
                confidence: 0.99,
                is_directory: true,
            },
            ExcludePattern {
                pattern: ".pytest_cache".to_string(),
                reason: "pytestキャッシュ（自動生成）".to_string(),
                confidence: 0.99,
                is_directory: true,
            },
            ExcludePattern {
                pattern: "dist".to_string(),
                reason: "ビルド成果物ディレクトリ（再ビルド可能）".to_string(),
                confidence: 0.90,
                is_directory: true,
            },
            ExcludePattern {
                pattern: "build".to_string(),
                reason: "ビルド成果物ディレクトリ（再ビルド可能）".to_string(),
                confidence: 0.90,
                is_directory: true,
            },
            // キャッシュディレクトリ（高信頼度）
            ExcludePattern {
                pattern: ".cache".to_string(),
                reason: "キャッシュディレクトリ（一時データ）".to_string(),
                confidence: 0.95,
                is_directory: true,
            },
            ExcludePattern {
                pattern: "cache".to_string(),
                reason: "キャッシュディレクトリ（一時データ）".to_string(),
                confidence: 0.85,
                is_directory: true,
            },
            // バージョン管理システム（中信頼度）
            ExcludePattern {
                pattern: r"\.git".to_string(),
                reason: "Gitリポジトリメタデータ（リモートから復元可能）".to_string(),
                confidence: 0.70,
                is_directory: true,
            },
            ExcludePattern {
                pattern: r"\.svn".to_string(),
                reason: "SVNリポジトリメタデータ（リモートから復元可能）".to_string(),
                confidence: 0.70,
                is_directory: true,
            },
            // 一時ファイル（高信頼度）
            ExcludePattern {
                pattern: r".*\.tmp$".to_string(),
                reason: "一時ファイル".to_string(),
                confidence: 0.99,
                is_directory: false,
            },
            ExcludePattern {
                pattern: r".*\.temp$".to_string(),
                reason: "一時ファイル".to_string(),
                confidence: 0.99,
                is_directory: false,
            },
            ExcludePattern {
                pattern: r".*\.bak$".to_string(),
                reason: "バックアップファイル（元ファイルがあれば不要）".to_string(),
                confidence: 0.85,
                is_directory: false,
            },
            ExcludePattern {
                pattern: r".*~$".to_string(),
                reason: "エディタ一時ファイル".to_string(),
                confidence: 0.95,
                is_directory: false,
            },
            // ログファイル（中信頼度）
            ExcludePattern {
                pattern: r".*\.log$".to_string(),
                reason: "ログファイル（古いログは通常不要）".to_string(),
                confidence: 0.70,
                is_directory: false,
            },
        ];

        Self { patterns }
    }

    /// 除外パターンを提案
    ///
    /// # Errors
    ///
    /// ファイルシステムアクセスに失敗した場合はエラーを返します。
    pub fn suggest_exclude_patterns(
        &self,
        base_path: &Path,
    ) -> AiResult<Vec<ExcludeRecommendation>> {
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

        let mut recommendations = Vec::new();

        // パターンごとにマッチするディレクトリ/ファイルを検索
        for pattern_def in &self.patterns {
            let matches = self.find_matching_paths(base_path, pattern_def)?;

            if !matches.is_empty() {
                // 合計サイズを計算
                let total_size = self.calculate_total_size(&matches)?;
                let size_gb = total_size as f64 / 1_073_741_824.0;

                // 推奨が意味のあるサイズの場合のみ追加（10MB以上）
                if size_gb >= 0.01 {
                    let confidence = PredictionConfidence::new(pattern_def.confidence)
                        .map_err(AiError::InvalidParameter)?;

                    recommendations.push(ExcludeRecommendation::new(
                        pattern_def.pattern.clone(),
                        confidence,
                        size_gb,
                        pattern_def.reason.clone(),
                    ));
                }
            }
        }

        // サイズ削減量の大きい順にソート
        recommendations.sort_by(|a, b| {
            b.size_reduction_gb
                .partial_cmp(&a.size_reduction_gb)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(recommendations)
    }

    /// パターンにマッチするパスを検索
    fn find_matching_paths(
        &self,
        base_path: &Path,
        pattern: &ExcludePattern,
    ) -> AiResult<Vec<PathBuf>> {
        let mut matches = Vec::new();

        for entry in WalkDir::new(base_path)
            .follow_links(false)
            .into_iter()
            .filter_map(Result::ok)
        {
            let path = entry.path();

            // ディレクトリパターンのマッチング
            if pattern.is_directory && path.is_dir() {
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if file_name == pattern.pattern || file_name.contains(&pattern.pattern) {
                        matches.push(path.to_path_buf());
                    }
                }
            }
            // ファイルパターンのマッチング（正規表現）
            else if !pattern.is_directory && path.is_file() {
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if self.matches_pattern(file_name, &pattern.pattern) {
                        matches.push(path.to_path_buf());
                    }
                }
            }
        }

        Ok(matches)
    }

    /// 簡易的なパターンマッチング
    fn matches_pattern(&self, file_name: &str, pattern: &str) -> bool {
        // 正規表現パターン（簡易版）
        if pattern.starts_with(".*") && pattern.ends_with('$') {
            // ".*\.tmp$" のようなパターン → ".tmp" に変換
            let extension = pattern
                .trim_start_matches(".*")
                .trim_end_matches('$')
                .replace(r"\.", "."); // エスケープされたドットを実際のドットに変換
            return file_name.ends_with(&extension);
        }

        // 単純な文字列マッチ
        file_name.contains(pattern)
    }

    /// 合計サイズを計算
    fn calculate_total_size(&self, paths: &[PathBuf]) -> AiResult<u64> {
        let mut total_size = 0u64;

        for path in paths {
            if path.is_file() {
                if let Ok(metadata) = std::fs::metadata(path) {
                    total_size = total_size.saturating_add(metadata.len());
                }
            } else if path.is_dir() {
                // ディレクトリの場合、再帰的にサイズを計算
                for entry in WalkDir::new(path)
                    .follow_links(false)
                    .into_iter()
                    .filter_map(Result::ok)
                {
                    if entry.file_type().is_file() {
                        if let Ok(metadata) = std::fs::metadata(entry.path()) {
                            total_size = total_size.saturating_add(metadata.len());
                        }
                    }
                }
            }
        }

        Ok(total_size)
    }
}

impl Default for ExcludeRecommendationEngine {
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
    fn test_exclude_recommendation_creation() {
        let confidence = PredictionConfidence::new(0.95).unwrap();
        let rec = ExcludeRecommendation::new(
            "node_modules".to_string(),
            confidence,
            5.2,
            "npm依存関係".to_string(),
        );

        assert_eq!(rec.pattern(), "node_modules");
        assert_eq!(rec.confidence().get(), 0.95);
        assert_eq!(rec.size_reduction_gb(), 5.2);
    }

    #[test]
    fn test_exclude_engine_creation() {
        let engine = ExcludeRecommendationEngine::new();
        assert!(!engine.patterns.is_empty());
    }

    #[test]
    fn test_matches_pattern() {
        let engine = ExcludeRecommendationEngine::new();

        assert!(engine.matches_pattern("test.tmp", r".*\.tmp$"));
        assert!(engine.matches_pattern("file.log", r".*\.log$"));
        assert!(!engine.matches_pattern("test.txt", r".*\.tmp$"));
    }

    #[test]
    fn test_suggest_exclude_patterns() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        // テスト用のディレクトリ構造を作成（10MB以上のサイズにする）
        let node_modules = base_path.join("node_modules");
        fs::create_dir(&node_modules).unwrap();
        // 10MB以上のファイルを作成（除外推奨の閾値をクリアするため）
        let large_content = vec![b'x'; 11_000_000]; // 11MB
        fs::write(node_modules.join("package.json"), &large_content).unwrap();

        let cache_dir = base_path.join(".cache");
        fs::create_dir(&cache_dir).unwrap();
        fs::write(cache_dir.join("data.cache"), &large_content).unwrap();

        // 一時ファイル（小さくてもパターンマッチで検出される）
        fs::write(base_path.join("temp.tmp"), &large_content).unwrap();

        let engine = ExcludeRecommendationEngine::new();
        let recommendations = engine.suggest_exclude_patterns(base_path).unwrap();

        // node_modules が検出されるはず
        let has_node_modules = recommendations
            .iter()
            .any(|r| r.pattern() == "node_modules");
        assert!(
            has_node_modules,
            "node_modules should be detected. Recommendations: {:?}",
            recommendations
                .iter()
                .map(|r| r.pattern())
                .collect::<Vec<_>>()
        );

        // .cache が検出されるはず
        let has_cache = recommendations.iter().any(|r| r.pattern() == ".cache");
        assert!(
            has_cache,
            ".cache should be detected. Recommendations: {:?}",
            recommendations
                .iter()
                .map(|r| r.pattern())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_calculate_total_size() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        // ファイルを作成
        let file1 = base_path.join("file1.txt");
        fs::write(&file1, b"1234567890").unwrap(); // 10 bytes

        let file2 = base_path.join("file2.txt");
        fs::write(&file2, b"abcdefghij").unwrap(); // 10 bytes

        let engine = ExcludeRecommendationEngine::new();
        let paths = vec![file1, file2];
        let total_size = engine.calculate_total_size(&paths).unwrap();

        assert_eq!(total_size, 20);
    }
}
