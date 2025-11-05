//! テスト共通ユーティリティ
//!
//! 統合テストとプロパティベーステストで共通で使用するヘルパー関数

use anyhow::Result;
use backup_suite::{Config, Priority, Target};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// テスト用のディレクトリ構造を作成するビルダー
pub struct TestEnvironment {
    pub temp_dir: TempDir,
    pub source_dir: PathBuf,
    pub backup_dir: PathBuf,
}

impl TestEnvironment {
    /// 新しいテスト環境を作成
    pub fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let source_dir = temp_dir.path().join("source");
        let backup_dir = temp_dir.path().join("backup");

        fs::create_dir_all(&source_dir)?;

        Ok(Self {
            temp_dir,
            source_dir,
            backup_dir,
        })
    }

    /// ソースディレクトリにファイルを作成
    pub fn create_file(&self, path: impl AsRef<Path>, content: &str) -> Result<PathBuf> {
        let full_path = self.source_dir.join(path.as_ref());

        // 親ディレクトリが存在しない場合は作成
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&full_path, content)?;
        Ok(full_path)
    }

    /// 複数のファイルを一度に作成
    pub fn create_files(&self, files: &[(&str, &str)]) -> Result<Vec<PathBuf>> {
        files
            .iter()
            .map(|(path, content)| self.create_file(path, content))
            .collect()
    }

    /// ディレクトリを作成
    pub fn create_dir(&self, path: impl AsRef<Path>) -> Result<PathBuf> {
        let full_path = self.source_dir.join(path.as_ref());
        fs::create_dir_all(&full_path)?;
        Ok(full_path)
    }

    /// デフォルトの設定を作成
    pub fn create_config(&self) -> Config {
        let mut config = Config::default();
        config.backup.destination = self.backup_dir.clone();
        config
    }

    /// ターゲットを追加した設定を作成
    pub fn create_config_with_target(
        &self,
        path: impl Into<PathBuf>,
        priority: Priority,
        category: &str,
    ) -> Config {
        let mut config = self.create_config();
        config.targets.push(Target::new(
            path.into(),
            priority,
            category.to_string(),
        ));
        config
    }

    /// 除外パターン付きターゲットを追加した設定を作成
    pub fn create_config_with_exclude_patterns(
        &self,
        path: impl Into<PathBuf>,
        priority: Priority,
        category: &str,
        exclude_patterns: Vec<String>,
    ) -> Config {
        let mut config = self.create_config();
        let mut target = Target::new(path.into(), priority, category.to_string());
        target.exclude_patterns = exclude_patterns;
        config.targets.push(target);
        config
    }

    /// バックアップディレクトリ内のファイル数をカウント
    pub fn count_backup_files(&self, backup_name: &str) -> Result<usize> {
        let backup_path = self.backup_dir.join(backup_name);
        if !backup_path.exists() {
            return Ok(0);
        }

        let mut count = 0;
        for entry in walkdir::WalkDir::new(&backup_path) {
            let entry = entry?;
            if entry.file_type().is_file() {
                count += 1;
            }
        }
        Ok(count)
    }

    /// バックアップディレクトリ内に特定のファイルが存在するか確認
    pub fn backup_file_exists(&self, backup_name: &str, relative_path: &str) -> bool {
        let full_path = self
            .backup_dir
            .join(backup_name)
            .join(relative_path);
        full_path.exists()
    }

    /// バックアップファイルの内容を読み取る
    pub fn read_backup_file(&self, backup_name: &str, relative_path: &str) -> Result<String> {
        let full_path = self
            .backup_dir
            .join(backup_name)
            .join(relative_path);
        Ok(fs::read_to_string(full_path)?)
    }

    /// ソースディレクトリのパスを取得
    pub fn source_path(&self) -> &Path {
        &self.source_dir
    }

    /// バックアップディレクトリのパスを取得
    pub fn backup_path(&self) -> &Path {
        &self.backup_dir
    }
}

/// テスト用のサンプルファイル構造を作成
pub fn create_sample_directory_structure(env: &TestEnvironment) -> Result<()> {
    // ディレクトリ構造
    env.create_dir("subdir1")?;
    env.create_dir("subdir2")?;
    env.create_dir("subdir1/nested")?;

    // ファイル作成
    env.create_file("file1.txt", "content1")?;
    env.create_file("file2.md", "# Markdown content")?;
    env.create_file("subdir1/file3.txt", "content3")?;
    env.create_file("subdir1/nested/file4.txt", "content4")?;
    env.create_file("subdir2/file5.txt", "content5")?;

    Ok(())
}

/// 大量のファイルを持つディレクトリを作成（パフォーマンステスト用）
pub fn create_large_directory(env: &TestEnvironment, num_files: usize) -> Result<()> {
    for i in 0..num_files {
        env.create_file(
            format!("file_{:04}.txt", i),
            &format!("content {}", i),
        )?;
    }
    Ok(())
}

/// 大きなファイルを作成（パフォーマンステスト用）
#[allow(dead_code)]
pub fn create_large_file(env: &TestEnvironment, filename: &str, size_mb: usize) -> Result<PathBuf> {
    let content = vec![0u8; size_mb * 1024 * 1024];
    env.create_file(filename, &String::from_utf8_lossy(&content))
}

/// 様々な拡張子のファイルを作成（除外パターンテスト用）
#[allow(dead_code)]
pub fn create_mixed_extension_files(env: &TestEnvironment) -> Result<()> {
    env.create_file("document.txt", "text")?;
    env.create_file("document.md", "markdown")?;
    env.create_file("script.sh", "#!/bin/bash")?;
    env.create_file("code.rs", "fn main() {}")?;
    env.create_file("temp.tmp", "temporary")?;
    env.create_file("backup.bak", "backup")?;
    env.create_file(".hidden", "hidden file")?;
    env.create_file("image.png", "png data")?;
    Ok(())
}

/// node_modules風のディレクトリ構造を作成（除外パターンテスト用）
#[allow(dead_code)]
pub fn create_node_modules_structure(env: &TestEnvironment) -> Result<()> {
    env.create_dir("node_modules")?;
    env.create_dir("node_modules/package1")?;
    env.create_dir("node_modules/package2")?;

    env.create_file("node_modules/package1/index.js", "module.exports = {}")?;
    env.create_file("node_modules/package2/index.js", "module.exports = {}")?;
    env.create_file("src/main.js", "console.log('main')")?;
    env.create_file("package.json", r#"{"name": "test"}"#)?;

    Ok(())
}

/// 隠しファイルとドットファイルを作成
#[allow(dead_code)]
pub fn create_hidden_files(env: &TestEnvironment) -> Result<()> {
    env.create_file(".env", "SECRET=value")?;
    env.create_file(".gitignore", "node_modules/")?;
    env.create_file(".config/app.toml", "key = 'value'")?;
    env.create_file("visible.txt", "visible content")?;
    Ok(())
}

/// ファイルのハッシュを計算（内容の検証用）
#[allow(dead_code)]
pub fn calculate_file_hash(path: &Path) -> Result<String> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let content = fs::read(path)?;
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    Ok(format!("{:x}", hasher.finish()))
}

/// バックアップ前後でファイルの内容が同じか確認
#[allow(dead_code)]
pub fn verify_file_integrity(source: &Path, backup: &Path) -> Result<bool> {
    let source_hash = calculate_file_hash(source)?;
    let backup_hash = calculate_file_hash(backup)?;
    Ok(source_hash == backup_hash)
}

/// ディレクトリ内のファイル数を再帰的にカウント
pub fn count_files_recursive(path: &Path) -> Result<usize> {
    let mut count = 0;
    for entry in walkdir::WalkDir::new(path) {
        let entry = entry?;
        if entry.file_type().is_file() {
            count += 1;
        }
    }
    Ok(count)
}

/// ディレクトリ構造を文字列として取得（デバッグ用）
#[allow(dead_code)]
pub fn debug_directory_structure(path: &Path) -> Result<String> {
    let mut result = String::new();
    for entry in walkdir::WalkDir::new(path).sort_by_file_name() {
        let entry = entry?;
        let depth = entry.depth();
        let indent = "  ".repeat(depth);
        let name = entry.file_name().to_string_lossy();
        let file_type = if entry.file_type().is_dir() {
            "📁"
        } else {
            "📄"
        };
        result.push_str(&format!("{}{} {}\n", indent, file_type, name));
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_creation() -> Result<()> {
        let env = TestEnvironment::new()?;
        assert!(env.source_dir.exists());
        Ok(())
    }

    #[test]
    fn test_file_creation() -> Result<()> {
        let env = TestEnvironment::new()?;
        env.create_file("test.txt", "test content")?;

        let content = fs::read_to_string(env.source_dir.join("test.txt"))?;
        assert_eq!(content, "test content");
        Ok(())
    }

    #[test]
    fn test_sample_directory_structure() -> Result<()> {
        let env = TestEnvironment::new()?;
        create_sample_directory_structure(&env)?;

        assert!(env.source_dir.join("file1.txt").exists());
        assert!(env.source_dir.join("subdir1/nested/file4.txt").exists());
        Ok(())
    }

    #[test]
    fn test_count_files_recursive() -> Result<()> {
        let env = TestEnvironment::new()?;
        create_sample_directory_structure(&env)?;

        let count = count_files_recursive(&env.source_dir)?;
        assert_eq!(count, 5); // file1.txt, file2.md, file3.txt, file4.txt, file5.txt
        Ok(())
    }
}
