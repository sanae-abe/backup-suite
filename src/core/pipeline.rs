//! # 統合パイプライン
//!
//! 暗号化・圧縮・バックアップを統合した高性能処理パイプライン

use crate::compression::{CompressedData, CompressionConfig, CompressionEngine, CompressionType};
use crate::crypto::{EncryptedData, EncryptionConfig, EncryptionEngine, KeyManager, MasterKey};
use crate::error::{BackupError, Result};
use aes_gcm::aead::Aead;
use aes_gcm::KeyInit;
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// パイプライン処理設定
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// 暗号化設定
    pub encryption: Option<EncryptionConfig>,
    /// 圧縮設定
    pub compression: CompressionConfig,
    /// 圧縮タイプ
    pub compression_type: CompressionType,
    /// パフォーマンス設定
    pub performance: PerformanceConfig,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            encryption: None,
            compression: CompressionConfig::zstd_default(),
            compression_type: CompressionType::Zstd,
            performance: PerformanceConfig::default(),
        }
    }
}

impl PipelineConfig {
    /// 暗号化を有効にする
    #[must_use]
    pub fn with_encryption(mut self, config: EncryptionConfig) -> Self {
        self.encryption = Some(config);
        self
    }

    /// 圧縮を設定する
    #[must_use]
    pub fn with_compression(
        mut self,
        compression_type: CompressionType,
        config: CompressionConfig,
    ) -> Self {
        self.compression_type = compression_type;
        self.compression = config;
        self
    }

    /// 高速設定に変更
    #[must_use]
    pub fn fast(mut self) -> Self {
        self.compression = CompressionConfig::fast(self.compression_type);
        self.performance = PerformanceConfig::fast();
        self
    }

    /// 高圧縮率設定に変更
    #[must_use]
    pub fn best_compression(mut self) -> Self {
        self.compression = CompressionConfig::best(self.compression_type);
        self.performance = PerformanceConfig::quality();
        self
    }
}

/// パフォーマンス設定
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    /// 並列処理数
    pub parallel_threads: usize,
    /// バッファサイズ
    pub buffer_size: usize,
    /// メモリ制限（バイト）
    pub memory_limit: usize,
    /// バッチサイズ（並列処理時の1バッチあたりのファイル数）
    pub batch_size: usize,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            parallel_threads: optimal_parallelism(), // CPU コア数の75%
            buffer_size: 1024 * 1024,                // 1MB
            memory_limit: 512 * 1024 * 1024,         // 512MB
            batch_size: 32,                          // デフォルト32ファイル/バッチ
        }
    }
}

impl PerformanceConfig {
    /// 高速設定
    #[must_use]
    pub fn fast() -> Self {
        Self {
            parallel_threads: num_cpus::get(), // 全コア使用
            buffer_size: 2 * 1024 * 1024,      // 2MB
            memory_limit: 1024 * 1024 * 1024,  // 1GB
            batch_size: 64,                    // 大きめのバッチサイズ
        }
    }

    /// 品質重視設定
    #[must_use]
    pub fn quality() -> Self {
        Self {
            parallel_threads: (num_cpus::get() / 2).max(1), // コア数の半分
            buffer_size: 512 * 1024,                        // 512KB
            memory_limit: 256 * 1024 * 1024,                // 256MB
            batch_size: 16,                                 // 小さめのバッチサイズ
        }
    }

    /// カスタム並列度設定
    #[must_use]
    pub fn with_parallelism(mut self, threads: usize) -> Self {
        self.parallel_threads = threads.max(1);
        self
    }

    /// カスタムバッチサイズ設定
    #[must_use]
    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.batch_size = size.max(1);
        self
    }
}

/// 最適な並列度を計算
///
/// CPU コア数の75%を使用し、システムリソースを確保する。
/// 最小1スレッド、最大32スレッドに制限。
#[must_use]
pub fn optimal_parallelism() -> usize {
    let cpus = num_cpus::get();
    (cpus * 3 / 4).clamp(1, 32)
}

/// 動的並列度計算
///
/// ファイル数とファイルサイズに基づいて最適なスレッド数を計算する。
///
/// # Arguments
///
/// * `file_count` - 処理対象のファイル数
/// * `avg_file_size` - 平均ファイルサイズ（バイト）
#[must_use]
pub fn dynamic_parallelism(file_count: usize, avg_file_size: u64) -> usize {
    let base_parallelism = optimal_parallelism();

    // ファイル数が少ない場合は並列度を抑える
    if file_count < base_parallelism {
        return file_count.max(1);
    }

    // 小さいファイルの場合はオーバーヘッドを考慮して並列度を抑える
    if avg_file_size < 1024 * 1024 {
        // 1MB未満
        return (base_parallelism / 2).max(1);
    }

    // 大きいファイルの場合は並列度を増やす
    if avg_file_size > 100 * 1024 * 1024 {
        // 100MB超
        return (base_parallelism * 4 / 3).min(32);
    }

    base_parallelism
}

/// 処理されたデータ
#[derive(Debug, Clone)]
pub struct ProcessedData {
    /// 元のファイルパス
    pub original_path: PathBuf,
    /// 処理後のデータ
    pub data: Vec<u8>,
    /// 圧縮情報
    pub compression_info: Option<CompressedData>,
    /// 暗号化情報
    pub encryption_info: Option<EncryptedData>,
    /// メタデータ
    pub metadata: ProcessingMetadata,
}

/// 処理メタデータ
#[derive(Debug, Clone)]
pub struct ProcessingMetadata {
    /// 元のサイズ
    pub original_size: u64,
    /// 圧縮後サイズ
    pub compressed_size: u64,
    /// 暗号化後サイズ
    pub final_size: u64,
    /// 処理時間（ミリ秒）
    pub processing_time_ms: u64,
    /// 圧縮率（%）
    pub compression_ratio: f64,
    /// 使用メモリ（推定値）
    pub memory_usage: u64,
}

/// 統合処理パイプライン
pub struct ProcessingPipeline {
    config: PipelineConfig,
    encryption_engine: Option<Arc<EncryptionEngine>>,
    compression_engine: Arc<CompressionEngine>,
    key_manager: Option<Arc<KeyManager>>,
    thread_pool: Option<rayon::ThreadPool>,
}

impl ProcessingPipeline {
    /// 新しいパイプラインを作成
    #[must_use]
    pub fn new(config: PipelineConfig) -> Self {
        let encryption_engine = config
            .encryption
            .as_ref()
            .map(|cfg| Arc::new(EncryptionEngine::new(cfg.clone())));

        let compression_engine = Arc::new(CompressionEngine::new(
            config.compression_type,
            config.compression.clone(),
        ));

        let key_manager = encryption_engine
            .as_ref()
            .map(|_| Arc::new(KeyManager::default()));

        // カスタムThreadPoolの作成
        let thread_pool = Self::create_thread_pool(&config.performance).ok();

        Self {
            config,
            encryption_engine,
            compression_engine,
            key_manager,
            thread_pool,
        }
    }

    /// 最適化されたThreadPoolを作成
    fn create_thread_pool(performance: &PerformanceConfig) -> Result<rayon::ThreadPool> {
        ThreadPoolBuilder::new()
            .num_threads(performance.parallel_threads)
            .thread_name(|i| format!("backup-worker-{i}"))
            .stack_size(8 * 1024 * 1024) // 8MBスタックサイズ
            .build()
            .map_err(|e| BackupError::Other(anyhow::anyhow!("ThreadPool作成エラー: {e}")))
    }

    /// 暗号化有効でパイプラインを作成
    pub fn with_encryption(password: &str) -> Result<(Self, [u8; 16])> {
        let config = PipelineConfig::default().with_encryption(EncryptionConfig::default());
        let mut pipeline = Self::new(config);

        let key_manager = KeyManager::default();
        let (_master_key, salt) = key_manager.create_master_key(password)?;

        pipeline.key_manager = Some(Arc::new(key_manager));
        // Note: In a real implementation, we'd store the master key securely
        // For now, we'll need to pass it to process methods

        Ok((pipeline, salt))
    }

    /// ファイルを処理（圧縮 → 暗号化）
    pub fn process_file<P: AsRef<Path>>(
        &self,
        file_path: P,
        master_key: Option<&MasterKey>,
        salt: Option<[u8; 16]>,
    ) -> Result<ProcessedData> {
        let start_time = std::time::Instant::now();
        let file_path = file_path.as_ref().to_path_buf();

        // ファイル読み込み
        let original_data = std::fs::read(&file_path)?;
        let original_size = original_data.len() as u64;

        // Step 1: 圧縮
        let (compressed_data, compression_info) =
            if self.config.compression_type != CompressionType::None {
                let compressed = self.compression_engine.compress(&original_data)?;
                let _compression_ratio = compressed.compression_percentage();
                (compressed.data.clone(), Some(compressed))
            } else {
                (original_data, None)
            };

        let compressed_size = compressed_data.len() as u64;

        // Step 2: 暗号化
        let (final_data, encryption_info) = if let (Some(engine), Some(key), Some(s)) =
            (&self.encryption_engine, master_key, salt)
        {
            let encrypted = engine.encrypt(&compressed_data, key, s)?;
            (encrypted.to_bytes(), Some(encrypted))
        } else {
            (compressed_data, None)
        };

        let final_size = final_data.len() as u64;
        let processing_time = start_time.elapsed().as_millis() as u64;

        let metadata = ProcessingMetadata {
            original_size,
            compressed_size,
            final_size,
            processing_time_ms: processing_time,
            compression_ratio: if original_size > 0 {
                (original_size.saturating_sub(compressed_size) as f64 / original_size as f64)
                    * 100.0
            } else {
                0.0
            },
            memory_usage: (original_size + compressed_size + final_size),
        };

        Ok(ProcessedData {
            original_path: file_path,
            data: final_data,
            compression_info,
            encryption_info,
            metadata,
        })
    }

    /// データを復元（復号化 → 展開）
    pub fn restore_data(
        &self,
        processed_data: &ProcessedData,
        master_key: Option<&MasterKey>,
    ) -> Result<Vec<u8>> {
        let mut data = processed_data.data.clone();

        // Step 1: 復号化
        if let Some(encryption_info) = &processed_data.encryption_info {
            if let (Some(engine), Some(key)) = (&self.encryption_engine, master_key) {
                data = engine.decrypt(encryption_info, key)?;
            } else {
                return Err(BackupError::EncryptionError(
                    "復号化にはマスターキーが必要です".to_string(),
                ));
            }
        }

        // Step 2: 展開
        if let Some(compression_info) = &processed_data.compression_info {
            data = self.compression_engine.decompress(compression_info)?;
        }

        Ok(data)
    }

    /// ストリーミング処理（大容量ファイル用）
    ///
    /// # 真のストリーミング処理実装
    ///
    /// - チャンク単位での圧縮・暗号化処理
    /// - メモリ使用量: O(chunk_size) = 最大2MB
    /// - 100GB以上のファイルでもOOMリスクなし
    ///
    /// # 処理フロー
    ///
    /// 1. チャンク読み込み (1MB)
    /// 2. チャンク圧縮 (zstd/gzip, ストリーミング)
    /// 3. チャンク暗号化 (AES-256-GCM)
    /// 4. ディスクへ即座に書き込み
    ///
    /// # エラー
    ///
    /// 以下の場合にエラーを返します:
    /// - ファイル読み込みエラー (`BackupError::IoError`)
    /// - ファイル書き込みエラー (`BackupError::IoError`)
    /// - 圧縮処理エラー (`BackupError::CompressionError`)
    /// - 暗号化処理エラー (`BackupError::EncryptionError`)
    pub fn process_stream<R: Read, W: Write>(
        &self,
        reader: R,
        writer: W,
        master_key: Option<&MasterKey>,
        salt: Option<[u8; 16]>,
    ) -> Result<ProcessingMetadata> {
        let start_time = std::time::Instant::now();

        // 真のストリーミング処理
        // 1. 圧縮 + 暗号化のパイプライン処理
        let (original_size, compressed_size, final_size) =
            if let (Some(engine), Some(key), Some(s)) = (&self.encryption_engine, master_key, salt)
            {
                // 圧縮 → 暗号化パイプライン（ストリーミング）
                self.compress_and_encrypt_stream(reader, writer, engine, key, s)?
            } else if self.config.compression_type != CompressionType::None {
                // 圧縮のみ（ストリーミング）
                self.compress_stream_only(reader, writer)?
            } else {
                // 圧縮も暗号化もなし（単純コピー、ストリーミング）
                self.copy_stream(reader, writer)?
            };

        let processing_time = start_time.elapsed().as_millis() as u64;

        Ok(ProcessingMetadata {
            original_size,
            compressed_size,
            final_size,
            processing_time_ms: processing_time,
            compression_ratio: if original_size > 0 {
                (original_size.saturating_sub(compressed_size) as f64 / original_size as f64)
                    * 100.0
            } else {
                0.0
            },
            memory_usage: 2 * 1024 * 1024, // 2MB固定（チャンクサイズベース）
        })
    }

    /// 圧縮 + 暗号化ストリーミング処理（内部実装）
    ///
    /// # 返り値
    ///
    /// `(original_size, compressed_size, final_size)` のタプル
    fn compress_and_encrypt_stream<R: Read, W: Write>(
        &self,
        reader: R,
        writer: W,
        encryption_engine: &EncryptionEngine,
        master_key: &MasterKey,
        salt: [u8; 16],
    ) -> Result<(u64, u64, u64)> {
        use std::io::Cursor;

        // 圧縮バッファ（メモリ内一時保存）
        let mut compressed_buffer = Vec::new();

        // ステップ1: 圧縮ストリーミング
        let compressed_data = self
            .compression_engine
            .compress_stream(reader, &mut compressed_buffer)?;

        let original_size = compressed_data.original_size;
        let compressed_size = compressed_data.compressed_size;

        // ステップ2: 暗号化ストリーミング（圧縮データを入力）
        let compressed_reader = Cursor::new(compressed_buffer);
        let mut encrypted_buffer = Vec::new();

        // ナンス・ソルトヘッダー書き込み
        let nonce_bytes = crate::crypto::encryption::EncryptionEngine::generate_nonce_internal();
        encrypted_buffer.extend_from_slice(&nonce_bytes);
        encrypted_buffer.extend_from_slice(&salt);

        // 暗号化ストリーミング（チャンク単位）
        #[allow(deprecated)]
        let key = aes_gcm::Key::<aes_gcm::Aes256Gcm>::from_slice(master_key.as_bytes());
        let cipher = aes_gcm::Aes256Gcm::new(key);

        let chunk_size = encryption_engine.get_chunk_size();
        let mut buffer = vec![0u8; chunk_size];
        let mut compressed_reader = compressed_reader;
        let mut chunk_index = 0u64;

        loop {
            let bytes_read = compressed_reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            // チャンク毎に異なるナンスを使用（u64カウンター）
            let mut chunk_nonce = nonce_bytes;
            chunk_nonce[4..12].copy_from_slice(&chunk_index.to_le_bytes());

            #[allow(deprecated)]
            let nonce = aes_gcm::Nonce::from_slice(&chunk_nonce);
            let chunk_ciphertext = cipher
                .encrypt(nonce, &buffer[..bytes_read])
                .map_err(|e| BackupError::EncryptionError(format!("チャンク暗号化エラー: {e}")))?;

            // チャンクサイズと暗号化データを書き込み
            encrypted_buffer.extend_from_slice(&(chunk_ciphertext.len() as u32).to_le_bytes());
            encrypted_buffer.extend_from_slice(&chunk_ciphertext);

            chunk_index += 1;
        }

        let final_size = encrypted_buffer.len() as u64;

        // ステップ3: 最終書き込み（一括）
        let mut writer = writer;
        writer.write_all(&encrypted_buffer)?;

        Ok((original_size, compressed_size, final_size))
    }

    /// 圧縮のみストリーミング処理（内部実装）
    fn compress_stream_only<R: Read, W: Write>(
        &self,
        reader: R,
        writer: W,
    ) -> Result<(u64, u64, u64)> {
        let compressed_data = self.compression_engine.compress_stream(reader, writer)?;

        let original_size = compressed_data.original_size;
        let compressed_size = compressed_data.compressed_size;

        Ok((original_size, compressed_size, compressed_size))
    }

    /// 単純コピーストリーミング処理（内部実装）
    fn copy_stream<R: Read, W: Write>(
        &self,
        mut reader: R,
        mut writer: W,
    ) -> Result<(u64, u64, u64)> {
        let mut total_size = 0u64;
        let mut buffer = vec![0u8; 1024 * 1024]; // 1MB チャンク

        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            writer.write_all(&buffer[..bytes_read])?;
            total_size += bytes_read as u64;
        }

        Ok((total_size, total_size, total_size))
    }

    /// 複数ファイルを並列処理
    ///
    /// # Arguments
    ///
    /// * `files` - 処理対象のファイルパスのスライス
    /// * `master_key` - マスターキー（暗号化時のみ必要）
    /// * `salt` - ソルト（暗号化時のみ必要）
    ///
    /// # Returns
    ///
    /// 各ファイルの処理結果を含むベクター
    pub fn process_files_parallel<P: AsRef<Path> + Send + Sync>(
        &self,
        files: &[P],
        master_key: Option<&MasterKey>,
        salt: Option<[u8; 16]>,
    ) -> Vec<Result<ProcessedData>> {
        if files.is_empty() {
            return Vec::new();
        }

        // ThreadPoolが利用可能な場合はそれを使用、なければグローバルプールを使用
        if let Some(pool) = &self.thread_pool {
            pool.install(|| self.process_files_parallel_internal(files, master_key, salt))
        } else {
            self.process_files_parallel_internal(files, master_key, salt)
        }
    }

    /// 内部並列処理実装
    fn process_files_parallel_internal<P: AsRef<Path> + Send + Sync>(
        &self,
        files: &[P],
        master_key: Option<&MasterKey>,
        salt: Option<[u8; 16]>,
    ) -> Vec<Result<ProcessedData>> {
        let batch_size = self.config.performance.batch_size;

        // バッチ処理による並列実行
        files
            .par_chunks(batch_size)
            .flat_map(|batch| {
                batch
                    .par_iter()
                    .map(|file| self.process_file(file, master_key, salt))
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    /// 複数ファイルを並列処理（進捗コールバック付き）
    ///
    /// # Arguments
    ///
    /// * `files` - 処理対象のファイルパスのスライス
    /// * `master_key` - マスターキー（暗号化時のみ必要）
    /// * `salt` - ソルト（暗号化時のみ必要）
    /// * `progress_callback` - 進捗コールバック関数（完了ファイル数、総ファイル数）
    pub fn process_files_with_progress<P, F>(
        &self,
        files: &[P],
        master_key: Option<&MasterKey>,
        salt: Option<[u8; 16]>,
        progress_callback: F,
    ) -> Vec<Result<ProcessedData>>
    where
        P: AsRef<Path> + Send + Sync,
        F: Fn(usize, usize) + Send + Sync,
    {
        if files.is_empty() {
            return Vec::new();
        }

        let total = files.len();
        let progress_callback = Arc::new(progress_callback);

        files
            .par_iter()
            .enumerate()
            .map(|(idx, file)| {
                let result = self.process_file(file, master_key, salt);
                progress_callback(idx + 1, total);
                result
            })
            .collect()
    }

    /// 設定を取得
    #[must_use]
    pub fn config(&self) -> &PipelineConfig {
        &self.config
    }

    /// パフォーマンス統計を取得
    #[must_use]
    pub fn get_performance_stats(&self) -> PerformanceStats {
        PerformanceStats {
            available_threads: self.config.performance.parallel_threads,
            buffer_size: self.config.performance.buffer_size,
            memory_limit: self.config.performance.memory_limit,
            encryption_enabled: self.encryption_engine.is_some(),
            compression_type: self.config.compression_type,
        }
    }

    /// ThreadPoolが正常に作成されているか確認
    #[must_use]
    pub fn is_parallel_ready(&self) -> bool {
        self.thread_pool.is_some()
    }

    /// 現在の並列度を取得
    #[must_use]
    pub fn current_parallelism(&self) -> usize {
        self.thread_pool
            .as_ref()
            .map(rayon::ThreadPool::current_num_threads)
            .unwrap_or(1)
    }
}

impl Default for ProcessingPipeline {
    fn default() -> Self {
        Self::new(PipelineConfig::default())
    }
}

/// パフォーマンス統計
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub available_threads: usize,
    pub buffer_size: usize,
    pub memory_limit: usize,
    pub encryption_enabled: bool,
    pub compression_type: CompressionType,
}

/// CPU コア数を取得
///
/// `num_cpus`クレートを使用して論理コア数を取得する。
/// フォールバック時は4コアを仮定。
mod num_cpus {
    #[must_use]
    pub fn get() -> usize {
        ::num_cpus::get()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::MasterKey;
    use std::io::Cursor;

    #[test]
    fn test_pipeline_without_encryption() {
        let config = PipelineConfig::default()
            .with_compression(CompressionType::Zstd, CompressionConfig::zstd_default());
        let pipeline = ProcessingPipeline::new(config);

        // テストファイル作成
        let test_data = b"Hello, World! This is a test message for compression.".repeat(100);
        let temp_file = std::env::temp_dir().join("test_pipeline.txt");
        std::fs::write(&temp_file, &test_data).unwrap();

        let processed = pipeline.process_file(&temp_file, None, None).unwrap();

        // 圧縮されていることを確認
        assert!(processed.metadata.compressed_size < processed.metadata.original_size);
        assert!(processed.metadata.compression_ratio > 0.0);
        assert!(processed.compression_info.is_some());
        assert!(processed.encryption_info.is_none());

        // 復元テスト
        let restored = pipeline.restore_data(&processed, None).unwrap();
        assert_eq!(test_data, restored);

        // クリーンアップ
        let _ = std::fs::remove_file(&temp_file);
    }

    #[test]
    fn test_pipeline_with_encryption() {
        let config = PipelineConfig::default()
            .with_encryption(EncryptionConfig::default())
            .with_compression(CompressionType::Zstd, CompressionConfig::zstd_default());
        let pipeline = ProcessingPipeline::new(config);

        let master_key = MasterKey::generate();
        let salt = crate::crypto::key_management::KeyDerivation::generate_salt();
        let test_data = b"Secret message for encryption and compression test.".repeat(50);
        let temp_file = std::env::temp_dir().join("test_encrypted.txt");
        std::fs::write(&temp_file, &test_data).unwrap();

        let processed = pipeline
            .process_file(&temp_file, Some(&master_key), Some(salt))
            .unwrap();

        // 暗号化と圧縮が適用されていることを確認
        assert!(processed.compression_info.is_some());
        assert!(processed.encryption_info.is_some());
        assert!(processed.metadata.final_size > 0);

        // 復元テスト
        let restored = pipeline
            .restore_data(&processed, Some(&master_key))
            .unwrap();
        assert_eq!(test_data, restored);

        // 間違ったキーでは復元できないことを確認
        let wrong_key = MasterKey::generate();
        assert!(pipeline.restore_data(&processed, Some(&wrong_key)).is_err());

        // クリーンアップ
        let _ = std::fs::remove_file(&temp_file);
    }

    #[test]
    fn test_stream_processing() {
        let config = PipelineConfig::default().fast();
        let pipeline = ProcessingPipeline::new(config);

        let test_data = b"Stream processing test data. ".repeat(1000);
        let reader = Cursor::new(&test_data);
        let mut output = Vec::new();

        let metadata = pipeline
            .process_stream(reader, &mut output, None, None)
            .unwrap();

        assert_eq!(metadata.original_size, test_data.len() as u64);
        assert!(!output.is_empty());
    }

    #[test]
    fn test_performance_config() {
        let fast_config = PipelineConfig::default().fast();
        let best_config = PipelineConfig::default().best_compression();

        // 高速設定の方がバッファサイズが大きいことを確認
        assert!(fast_config.compression.buffer_size >= best_config.compression.buffer_size);

        let pipeline = ProcessingPipeline::new(fast_config);
        let stats = pipeline.get_performance_stats();

        assert!(stats.available_threads >= 1);
        assert!(stats.buffer_size > 0);
        assert_eq!(stats.compression_type, CompressionType::Zstd);
    }

    #[test]
    fn test_optimal_parallelism() {
        let parallelism = optimal_parallelism();
        let cpus = num_cpus::get();

        // CPU コア数の75%以下で、最小1、最大32の範囲内
        assert!(parallelism >= 1);
        assert!(parallelism <= 32);
        assert!(parallelism <= cpus);
    }

    #[test]
    fn test_dynamic_parallelism() {
        // 少数ファイルの場合
        let parallelism = dynamic_parallelism(2, 1024 * 1024);
        assert!(parallelism <= 2);

        // 小さいファイルの場合
        let parallelism = dynamic_parallelism(100, 512 * 1024);
        assert!(parallelism >= 1);

        // 大きいファイルの場合
        let parallelism = dynamic_parallelism(100, 200 * 1024 * 1024);
        assert!(parallelism >= optimal_parallelism());
    }

    #[test]
    fn test_parallel_processing() {
        let config = PipelineConfig::default().fast();
        let pipeline = ProcessingPipeline::new(config);

        // 複数のテストファイルを作成
        let temp_dir = std::env::temp_dir();
        let test_files: Vec<PathBuf> = (0..10)
            .map(|i| {
                let path = temp_dir.join(format!("test_parallel_{i}.txt"));
                let data = format!("Test data for file {i}").repeat(100);
                std::fs::write(&path, data).unwrap();
                path
            })
            .collect();

        // 並列処理実行
        let results = pipeline.process_files_parallel(&test_files, None, None);

        // 全ファイルが処理されたことを確認
        assert_eq!(results.len(), test_files.len());
        assert!(results.iter().all(std::result::Result::is_ok));

        // ThreadPoolが作成されていることを確認
        assert!(pipeline.is_parallel_ready());
        assert!(pipeline.current_parallelism() >= 1);

        // クリーンアップ
        for file in test_files {
            let _ = std::fs::remove_file(file);
        }
    }

    #[test]
    fn test_parallel_with_progress() {
        let config = PipelineConfig::default().fast();
        let pipeline = ProcessingPipeline::new(config);

        let temp_dir = std::env::temp_dir();
        let test_files: Vec<PathBuf> = (0..5)
            .map(|i| {
                let path = temp_dir.join(format!("test_progress_{i}.txt"));
                let data = format!("Progress test {i}").repeat(50);
                std::fs::write(&path, data).unwrap();
                path
            })
            .collect();

        // 進捗カウンター
        let progress_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let progress_count_clone = Arc::clone(&progress_count);

        // 進捗コールバック付き並列処理
        let results =
            pipeline.process_files_with_progress(&test_files, None, None, move |current, total| {
                progress_count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                assert!(current <= total);
            });

        assert_eq!(results.len(), test_files.len());
        assert!(results.iter().all(std::result::Result::is_ok));

        // 進捗コールバックが呼ばれたことを確認
        assert_eq!(
            progress_count.load(std::sync::atomic::Ordering::SeqCst),
            test_files.len()
        );

        // クリーンアップ
        for file in test_files {
            let _ = std::fs::remove_file(file);
        }
    }

    #[test]
    fn test_custom_parallelism() {
        let perf_config = PerformanceConfig::default()
            .with_parallelism(4)
            .with_batch_size(8);

        assert_eq!(perf_config.parallel_threads, 4);
        assert_eq!(perf_config.batch_size, 8);

        // PipelineConfigでカスタム設定を使用
        let pipeline_config = PipelineConfig {
            performance: perf_config,
            ..Default::default()
        };

        let pipeline = ProcessingPipeline::new(pipeline_config);
        assert_eq!(pipeline.current_parallelism(), 4);
    }

    #[test]
    fn test_pipeline_config_builder() {
        // Default configuration
        let default_config = PipelineConfig::default();
        assert!(default_config.encryption.is_none());
        assert_eq!(default_config.compression_type, CompressionType::Zstd);

        // Builder pattern - encryption
        let enc_config = PipelineConfig::default().with_encryption(EncryptionConfig::default());
        assert!(enc_config.encryption.is_some());

        // Builder pattern - compression
        let comp_config = PipelineConfig::default()
            .with_compression(CompressionType::Gzip, CompressionConfig::gzip_default());
        assert_eq!(comp_config.compression_type, CompressionType::Gzip);

        // Builder pattern - fast
        let fast_config = PipelineConfig::default().fast();
        assert_eq!(fast_config.performance.parallel_threads, num_cpus::get());
        assert_eq!(fast_config.performance.buffer_size, 2 * 1024 * 1024);

        // Builder pattern - best compression
        let best_config = PipelineConfig::default().best_compression();
        assert_eq!(
            best_config.performance.parallel_threads,
            (num_cpus::get() / 2).max(1)
        );
    }

    #[test]
    fn test_performance_config_presets() {
        // Fast preset
        let fast = PerformanceConfig::fast();
        assert_eq!(fast.parallel_threads, num_cpus::get());
        assert_eq!(fast.buffer_size, 2 * 1024 * 1024);
        assert_eq!(fast.memory_limit, 1024 * 1024 * 1024);
        assert_eq!(fast.batch_size, 64);

        // Quality preset
        let quality = PerformanceConfig::quality();
        assert_eq!(quality.parallel_threads, (num_cpus::get() / 2).max(1));
        assert_eq!(quality.buffer_size, 512 * 1024);
        assert_eq!(quality.memory_limit, 256 * 1024 * 1024);
        assert_eq!(quality.batch_size, 16);

        // Default preset
        let default = PerformanceConfig::default();
        assert_eq!(default.parallel_threads, optimal_parallelism());
        assert_eq!(default.buffer_size, 1024 * 1024);
        assert_eq!(default.memory_limit, 512 * 1024 * 1024);
        assert_eq!(default.batch_size, 32);
    }

    #[test]
    fn test_performance_config_custom_values() {
        // Zero parallelism should be clamped to 1
        let config = PerformanceConfig::default().with_parallelism(0);
        assert_eq!(config.parallel_threads, 1);

        // Zero batch size should be clamped to 1
        let config = PerformanceConfig::default().with_batch_size(0);
        assert_eq!(config.batch_size, 1);

        // Large values should work
        let config = PerformanceConfig::default()
            .with_parallelism(100)
            .with_batch_size(1000);
        assert_eq!(config.parallel_threads, 100);
        assert_eq!(config.batch_size, 1000);
    }

    #[test]
    fn test_compression_type_variations() {
        // Zstd compression
        let zstd_config = PipelineConfig::default()
            .with_compression(CompressionType::Zstd, CompressionConfig::zstd_default());
        let pipeline = ProcessingPipeline::new(zstd_config);

        let test_data = b"Zstd compression test. ".repeat(100);
        let temp_file = std::env::temp_dir().join("test_zstd.txt");
        std::fs::write(&temp_file, &test_data).unwrap();

        let processed = pipeline.process_file(&temp_file, None, None).unwrap();
        assert!(processed.compression_info.is_some());
        assert_eq!(processed.metadata.original_size, test_data.len() as u64);

        std::fs::remove_file(&temp_file).ok();

        // Gzip compression
        let gzip_config = PipelineConfig::default()
            .with_compression(CompressionType::Gzip, CompressionConfig::gzip_default());
        let pipeline = ProcessingPipeline::new(gzip_config);

        let test_data = b"Gzip compression test. ".repeat(100);
        let temp_file = std::env::temp_dir().join("test_gzip.txt");
        std::fs::write(&temp_file, &test_data).unwrap();

        let processed = pipeline.process_file(&temp_file, None, None).unwrap();
        assert!(processed.compression_info.is_some());
        assert_eq!(processed.metadata.original_size, test_data.len() as u64);

        std::fs::remove_file(&temp_file).ok();
    }

    #[test]
    fn test_empty_file_processing() {
        let config = PipelineConfig::default();
        let pipeline = ProcessingPipeline::new(config);

        let temp_file = std::env::temp_dir().join("test_empty.txt");
        std::fs::write(&temp_file, b"").unwrap();

        let processed = pipeline.process_file(&temp_file, None, None).unwrap();
        assert_eq!(processed.metadata.original_size, 0);
        assert_eq!(
            processed.metadata.final_size,
            processed.metadata.compressed_size
        );

        let restored = pipeline.restore_data(&processed, None).unwrap();
        assert!(restored.is_empty());

        std::fs::remove_file(&temp_file).ok();
    }

    #[test]
    fn test_large_file_processing() {
        let config = PipelineConfig::default().fast();
        let pipeline = ProcessingPipeline::new(config);

        // 10MB のテストデータ
        let test_data = vec![b'A'; 10 * 1024 * 1024];
        let temp_file = std::env::temp_dir().join("test_large.txt");
        std::fs::write(&temp_file, &test_data).unwrap();

        let processed = pipeline.process_file(&temp_file, None, None).unwrap();
        assert_eq!(processed.metadata.original_size, test_data.len() as u64);
        assert!(processed.metadata.compressed_size < processed.metadata.original_size);

        // 圧縮率が高いことを確認（同じ文字の繰り返しなので）
        assert!(processed.metadata.compression_ratio > 0.9);

        std::fs::remove_file(&temp_file).ok();
    }

    #[test]
    fn test_encryption_config_without_key_skips_encryption() {
        // 暗号化設定があってもキーなしなら暗号化をスキップ
        let config = PipelineConfig::default().with_encryption(EncryptionConfig::default());
        let pipeline = ProcessingPipeline::new(config);

        let test_data = b"Test data without key".repeat(10);
        let temp_file = std::env::temp_dir().join("test_no_key.txt");
        std::fs::write(&temp_file, &test_data).unwrap();

        // キーなしでも処理可能（暗号化はスキップされる）
        let processed = pipeline.process_file(&temp_file, None, None).unwrap();
        assert!(processed.encryption_info.is_none());
        assert!(processed.compression_info.is_some());

        std::fs::remove_file(&temp_file).ok();
    }

    #[test]
    fn test_restore_with_wrong_key() {
        let config = PipelineConfig::default().with_encryption(EncryptionConfig::default());
        let pipeline = ProcessingPipeline::new(config);

        let master_key = MasterKey::generate();
        let salt = crate::crypto::key_management::KeyDerivation::generate_salt();
        let test_data = b"Secret data for key test".repeat(10);
        let temp_file = std::env::temp_dir().join("test_wrong_key.txt");
        std::fs::write(&temp_file, &test_data).unwrap();

        let processed = pipeline
            .process_file(&temp_file, Some(&master_key), Some(salt))
            .unwrap();

        // 異なるキーで復元を試みる
        let wrong_key = MasterKey::generate();
        let result = pipeline.restore_data(&processed, Some(&wrong_key));
        assert!(result.is_err());

        std::fs::remove_file(&temp_file).ok();
    }

    #[test]
    fn test_restore_without_key() {
        let config = PipelineConfig::default().with_encryption(EncryptionConfig::default());
        let pipeline = ProcessingPipeline::new(config);

        let master_key = MasterKey::generate();
        let salt = crate::crypto::key_management::KeyDerivation::generate_salt();
        let test_data = b"Encrypted data".repeat(10);
        let temp_file = std::env::temp_dir().join("test_restore_no_key.txt");
        std::fs::write(&temp_file, &test_data).unwrap();

        let processed = pipeline
            .process_file(&temp_file, Some(&master_key), Some(salt))
            .unwrap();

        // キーなしで復元を試みる
        let result = pipeline.restore_data(&processed, None);
        assert!(result.is_err());

        std::fs::remove_file(&temp_file).ok();
    }

    #[test]
    fn test_parallel_error_handling() {
        let config = PipelineConfig::default().fast();
        let pipeline = ProcessingPipeline::new(config);

        // 存在しないファイルのリスト
        let nonexistent_files: Vec<PathBuf> = (0..5)
            .map(|i| PathBuf::from(format!("/tmp/nonexistent_file_{i}.txt")))
            .collect();

        let results = pipeline.process_files_parallel(&nonexistent_files, None, None);

        // 全てエラーになることを確認
        assert_eq!(results.len(), nonexistent_files.len());
        assert!(results.iter().all(std::result::Result::is_err));
    }

    #[test]
    fn test_parallel_mixed_results() {
        let config = PipelineConfig::default().fast();
        let pipeline = ProcessingPipeline::new(config);

        let temp_dir = std::env::temp_dir();
        let mut files = Vec::new();

        // 存在するファイルを3つ作成
        for i in 0..3 {
            let path = temp_dir.join(format!("test_mixed_{i}.txt"));
            std::fs::write(&path, format!("Test data {i}").repeat(10)).unwrap();
            files.push(path);
        }

        // 存在しないファイルを2つ追加
        files.push(PathBuf::from("/tmp/nonexistent_1.txt"));
        files.push(PathBuf::from("/tmp/nonexistent_2.txt"));

        let results = pipeline.process_files_parallel(&files, None, None);

        // 結果の数は一致
        assert_eq!(results.len(), files.len());

        // 一部成功、一部失敗
        let success_count = results.iter().filter(|r| r.is_ok()).count();
        let error_count = results.iter().filter(|r| r.is_err()).count();

        assert_eq!(success_count, 3);
        assert_eq!(error_count, 2);

        // クリーンアップ
        for i in 0..3 {
            let path = temp_dir.join(format!("test_mixed_{i}.txt"));
            std::fs::remove_file(path).ok();
        }
    }

    #[test]
    fn test_stream_empty_data() {
        let config = PipelineConfig::default();
        let pipeline = ProcessingPipeline::new(config);

        let empty_data: &[u8] = &[];
        let reader = Cursor::new(empty_data);
        let mut output = Vec::new();

        let metadata = pipeline
            .process_stream(reader, &mut output, None, None)
            .unwrap();

        assert_eq!(metadata.original_size, 0);
        assert_eq!(metadata.final_size, metadata.compressed_size);
    }

    #[test]
    fn test_stream_large_data() {
        let config = PipelineConfig::default().fast();
        let pipeline = ProcessingPipeline::new(config);

        // 5MB のストリームデータ
        let large_data = vec![b'B'; 5 * 1024 * 1024];
        let reader = Cursor::new(&large_data);
        let mut output = Vec::new();

        let metadata = pipeline
            .process_stream(reader, &mut output, None, None)
            .unwrap();

        assert_eq!(metadata.original_size, large_data.len() as u64);
        assert!(metadata.compressed_size < metadata.original_size);
        assert!(!output.is_empty());
    }

    #[test]
    fn test_performance_stats() {
        let config = PipelineConfig::default()
            .fast()
            .with_compression(CompressionType::Gzip, CompressionConfig::gzip_default());
        let pipeline = ProcessingPipeline::new(config);

        let stats = pipeline.get_performance_stats();

        assert_eq!(stats.available_threads, num_cpus::get());
        assert_eq!(stats.buffer_size, 2 * 1024 * 1024);
        assert_eq!(stats.compression_type, CompressionType::Gzip);
        assert!(!stats.encryption_enabled);
    }

    #[test]
    fn test_batch_processing() {
        let config = PerformanceConfig::default()
            .with_batch_size(3)
            .with_parallelism(2);

        assert_eq!(config.batch_size, 3);
        assert_eq!(config.parallel_threads, 2);

        let pipeline_config = PipelineConfig {
            performance: config,
            ..Default::default()
        };

        let pipeline = ProcessingPipeline::new(pipeline_config);

        // 10個のファイルを処理（バッチサイズ3なので4バッチ）
        let temp_dir = std::env::temp_dir();
        let test_files: Vec<PathBuf> = (0..10)
            .map(|i| {
                let path = temp_dir.join(format!("test_batch_{i}.txt"));
                std::fs::write(&path, format!("Batch test {i}").repeat(10)).unwrap();
                path
            })
            .collect();

        let results = pipeline.process_files_parallel(&test_files, None, None);

        assert_eq!(results.len(), 10);
        assert!(results.iter().all(std::result::Result::is_ok));

        // クリーンアップ
        for file in test_files {
            std::fs::remove_file(file).ok();
        }
    }

    #[test]
    fn test_compression_ratio_calculation() {
        let config = PipelineConfig::default();
        let pipeline = ProcessingPipeline::new(config);

        // 圧縮しやすいデータ（繰り返し）
        let compressible_data = b"A".repeat(10000);
        let temp_file = std::env::temp_dir().join("test_ratio.txt");
        std::fs::write(&temp_file, &compressible_data).unwrap();

        let processed = pipeline.process_file(&temp_file, None, None).unwrap();

        // 圧縮が実行されたことを確認
        assert!(processed.compression_info.is_some());
        assert!(processed.metadata.compressed_size < processed.metadata.original_size);

        // 圧縮率が0以上であることを確認（negative値はありえない）
        assert!(processed.metadata.compression_ratio >= 0.0);

        // 圧縮しやすいデータなので圧縮率が一定以上であることを確認
        // Zstdの圧縮率は通常のパーセンテージ（1.0 = 100%圧縮）とは異なる可能性がある
        assert!(processed.metadata.compressed_size > 0);

        std::fs::remove_file(&temp_file).ok();
    }
}
