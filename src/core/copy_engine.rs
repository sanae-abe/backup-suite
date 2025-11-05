use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

use crate::error::Result;

/// ファイルコピー最適化エンジン
///
/// ファイルサイズに応じて最適なコピー手法を選択します。
///
/// # 機能
///
/// - 小ファイル: 標準のfs::copyを使用（高速）
/// - 大ファイル: バッファリングコピー（メモリ効率）
/// - 並列処理対応: 複数ファイルの同時コピー
///
/// # 使用例
///
/// ```no_run
/// use backup_suite::core::copy_engine::CopyEngine;
/// use std::path::Path;
///
/// let engine = CopyEngine::new();
/// let bytes = engine.copy_file(
///     Path::new("/source/file.txt"),
///     Path::new("/dest/file.txt")
/// ).unwrap();
/// println!("{}バイトをコピーしました", bytes);
/// ```
#[derive(Debug, Clone)]
pub struct CopyEngine {
    /// バッファサイズ（バイト）
    buffer_size: usize,
    /// 並列処理を開始するファイルサイズの閾値（バイト）
    parallel_threshold: u64,
}

impl CopyEngine {
    /// デフォルト設定でCopyEngineを作成
    ///
    /// - バッファサイズ: 64KB
    /// - 並列処理閾値: 10MB
    ///
    /// # 戻り値
    ///
    /// CopyEngineインスタンス
    ///
    /// # 使用例
    ///
    /// ```no_run
    /// use backup_suite::core::copy_engine::CopyEngine;
    ///
    /// let engine = CopyEngine::new();
    /// ```
    pub fn new() -> Self {
        Self {
            buffer_size: 64 * 1024,        // 64KB
            parallel_threshold: 10 * 1024 * 1024, // 10MB
        }
    }

    /// カスタム設定でCopyEngineを作成
    ///
    /// # 引数
    ///
    /// * `buffer_size` - バッファサイズ（バイト）
    /// * `parallel_threshold` - 並列処理閾値（バイト）
    ///
    /// # 戻り値
    ///
    /// カスタム設定のCopyEngineインスタンス
    ///
    /// # 使用例
    ///
    /// ```no_run
    /// use backup_suite::core::copy_engine::CopyEngine;
    ///
    /// let engine = CopyEngine::with_config(128 * 1024, 20 * 1024 * 1024);
    /// ```
    pub fn with_config(buffer_size: usize, parallel_threshold: u64) -> Self {
        Self {
            buffer_size,
            parallel_threshold,
        }
    }

    /// ファイルをコピー
    ///
    /// ファイルサイズに応じて最適なコピー手法を自動選択します。
    ///
    /// # 引数
    ///
    /// * `source` - コピー元ファイルパス
    /// * `dest` - コピー先ファイルパス
    ///
    /// # 戻り値
    ///
    /// コピーしたバイト数
    ///
    /// # エラー
    ///
    /// * ファイルの読み取り失敗
    /// * ファイルの書き込み失敗
    /// * I/Oエラー
    ///
    /// # 使用例
    ///
    /// ```no_run
    /// use backup_suite::core::copy_engine::CopyEngine;
    /// use std::path::Path;
    ///
    /// let engine = CopyEngine::new();
    /// let bytes = engine.copy_file(
    ///     Path::new("/source/file.txt"),
    ///     Path::new("/dest/file.txt")
    /// ).unwrap();
    /// ```
    pub fn copy_file(&self, source: &Path, dest: &Path) -> Result<u64> {
        // ファイルサイズを取得
        let metadata = std::fs::metadata(source)?;
        let size = metadata.len();

        // 小さいファイルは標準のコピーを使用（最速）
        if size < self.parallel_threshold {
            return std::fs::copy(source, dest).map_err(Into::into);
        }

        // 大きいファイルはバッファリングコピー（メモリ効率的）
        self.buffered_copy(source, dest)
    }

    /// バッファリングコピーを実行
    ///
    /// 大きいファイルをメモリ効率的にコピーします。
    ///
    /// # 引数
    ///
    /// * `source` - コピー元ファイルパス
    /// * `dest` - コピー先ファイルパス
    ///
    /// # 戻り値
    ///
    /// コピーしたバイト数
    fn buffered_copy(&self, source: &Path, dest: &Path) -> Result<u64> {
        let mut reader = BufReader::with_capacity(self.buffer_size, File::open(source)?);
        let mut writer = BufWriter::with_capacity(self.buffer_size, File::create(dest)?);

        let mut buffer = vec![0u8; self.buffer_size];
        let mut total_bytes = 0u64;

        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            writer.write_all(&buffer[..bytes_read])?;
            total_bytes += bytes_read as u64;
        }

        writer.flush()?;

        // メタデータのコピー（パーミッション等）
        #[cfg(unix)]
        {
            let perms = std::fs::metadata(source)?.permissions();
            std::fs::set_permissions(dest, perms)?;
        }

        Ok(total_bytes)
    }

    /// バッファサイズを取得
    ///
    /// # 戻り値
    ///
    /// 現在のバッファサイズ（バイト）
    pub fn buffer_size(&self) -> usize {
        self.buffer_size
    }

    /// 並列処理閾値を取得
    ///
    /// # 戻り値
    ///
    /// 並列処理を開始するファイルサイズ（バイト）
    pub fn parallel_threshold(&self) -> u64 {
        self.parallel_threshold
    }
}

impl Default for CopyEngine {
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
    fn test_new_engine() {
        let engine = CopyEngine::new();
        assert_eq!(engine.buffer_size(), 64 * 1024);
        assert_eq!(engine.parallel_threshold(), 10 * 1024 * 1024);
    }

    #[test]
    fn test_with_config() {
        let engine = CopyEngine::with_config(128 * 1024, 20 * 1024 * 1024);
        assert_eq!(engine.buffer_size(), 128 * 1024);
        assert_eq!(engine.parallel_threshold(), 20 * 1024 * 1024);
    }

    #[test]
    fn test_copy_small_file() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source.txt");
        let dest = temp_dir.path().join("dest.txt");

        // 小さいファイルを作成
        let mut file = File::create(&source).unwrap();
        file.write_all(b"test content").unwrap();

        let engine = CopyEngine::new();
        let bytes = engine.copy_file(&source, &dest).unwrap();

        assert_eq!(bytes, 12);
        assert!(dest.exists());

        let content = std::fs::read_to_string(&dest).unwrap();
        assert_eq!(content, "test content");
    }

    #[test]
    fn test_copy_large_file() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("large_source.bin");
        let dest = temp_dir.path().join("large_dest.bin");

        // 大きいファイルを作成（15MB）
        let large_content = vec![0u8; 15 * 1024 * 1024];
        std::fs::write(&source, &large_content).unwrap();

        let engine = CopyEngine::new();
        let bytes = engine.copy_file(&source, &dest).unwrap();

        assert_eq!(bytes, 15 * 1024 * 1024);
        assert!(dest.exists());

        let metadata = std::fs::metadata(&dest).unwrap();
        assert_eq!(metadata.len(), 15 * 1024 * 1024);
    }

    #[test]
    fn test_buffered_copy() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("buffered_source.txt");
        let dest = temp_dir.path().join("buffered_dest.txt");

        // テストファイル作成
        let content = "buffered copy test content";
        std::fs::write(&source, content).unwrap();

        let engine = CopyEngine::new();
        let bytes = engine.buffered_copy(&source, &dest).unwrap();

        assert_eq!(bytes, content.len() as u64);
        assert!(dest.exists());

        let copied_content = std::fs::read_to_string(&dest).unwrap();
        assert_eq!(copied_content, content);
    }

    #[test]
    fn test_default_engine() {
        let engine = CopyEngine::default();
        assert_eq!(engine.buffer_size(), 64 * 1024);
    }

    #[cfg(unix)]
    #[test]
    fn test_preserve_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("perm_source.txt");
        let dest = temp_dir.path().join("perm_dest.txt");

        // ファイル作成とパーミッション設定
        let mut file = File::create(&source).unwrap();
        file.write_all(b"permission test").unwrap();
        drop(file);

        let mut perms = std::fs::metadata(&source).unwrap().permissions();
        perms.set_mode(0o644);
        std::fs::set_permissions(&source, perms).unwrap();

        // コピー実行
        let engine = CopyEngine::new();
        engine.copy_file(&source, &dest).unwrap();

        // パーミッション確認
        let dest_perms = std::fs::metadata(&dest).unwrap().permissions();
        assert_eq!(dest_perms.mode() & 0o777, 0o644);
    }
}
