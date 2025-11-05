//! # 統合パイプライン
//!
//! 暗号化・圧縮・バックアップを統合した高性能処理パイプライン

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use crate::error::{BackupError, Result};
use crate::crypto::{EncryptionEngine, EncryptionConfig, EncryptedData, KeyManager, MasterKey};
use crate::compression::{CompressionEngine, CompressionType, CompressionConfig, CompressedData};

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
    pub fn with_encryption(mut self, config: EncryptionConfig) -> Self {
        self.encryption = Some(config);
        self
    }

    /// 圧縮を設定する
    pub fn with_compression(mut self, compression_type: CompressionType, config: CompressionConfig) -> Self {
        self.compression_type = compression_type;
        self.compression = config;
        self
    }

    /// 高速設定に変更
    pub fn fast(mut self) -> Self {
        self.compression = CompressionConfig::fast(self.compression_type);
        self.performance = PerformanceConfig::fast();
        self
    }

    /// 高圧縮率設定に変更
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
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            parallel_threads: num_cpus::get().min(8), // 最大8スレッド
            buffer_size: 1024 * 1024,                 // 1MB
            memory_limit: 512 * 1024 * 1024,          // 512MB
        }
    }
}

impl PerformanceConfig {
    /// 高速設定
    pub fn fast() -> Self {
        Self {
            parallel_threads: num_cpus::get(),
            buffer_size: 2 * 1024 * 1024,     // 2MB
            memory_limit: 1024 * 1024 * 1024, // 1GB
        }
    }

    /// 品質重視設定
    pub fn quality() -> Self {
        Self {
            parallel_threads: (num_cpus::get() / 2).max(1),
            buffer_size: 512 * 1024,          // 512KB
            memory_limit: 256 * 1024 * 1024,  // 256MB
        }
    }
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
    encryption_engine: Option<EncryptionEngine>,
    compression_engine: CompressionEngine,
    key_manager: Option<KeyManager>,
}

impl ProcessingPipeline {
    /// 新しいパイプラインを作成
    pub fn new(config: PipelineConfig) -> Self {
        let encryption_engine = config.encryption.as_ref().map(|cfg| {
            EncryptionEngine::new(cfg.clone())
        });

        let compression_engine = CompressionEngine::new(
            config.compression_type,
            config.compression.clone(),
        );

        let key_manager = encryption_engine.as_ref().map(|_| {
            KeyManager::default()
        });

        Self {
            config,
            encryption_engine,
            compression_engine,
            key_manager,
        }
    }

    /// デフォルト設定でパイプラインを作成
    pub fn default() -> Self {
        Self::new(PipelineConfig::default())
    }

    /// 暗号化有効でパイプラインを作成
    pub fn with_encryption(password: &str) -> Result<(Self, [u8; 16])> {
        let config = PipelineConfig::default()
            .with_encryption(EncryptionConfig::default());
        let mut pipeline = Self::new(config);

        let key_manager = KeyManager::default();
        let (_master_key, salt) = key_manager.create_master_key(password)?;

        pipeline.key_manager = Some(key_manager);
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
        let (compressed_data, compression_info) = if self.config.compression_type != CompressionType::None {
            let compressed = self.compression_engine.compress(&original_data)?;
            let _compression_ratio = compressed.compression_percentage();
            (compressed.data.clone(), Some(compressed))
        } else {
            (original_data, None)
        };

        let compressed_size = compressed_data.len() as u64;

        // Step 2: 暗号化
        let (final_data, encryption_info) = if let (Some(engine), Some(key), Some(s)) = (&self.encryption_engine, master_key, salt) {
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
                (original_size.saturating_sub(compressed_size) as f64 / original_size as f64) * 100.0
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
                    "復号化にはマスターキーが必要です".to_string()
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
    pub fn process_stream<R: Read, W: Write>(
        &self,
        mut reader: R,
        mut writer: W,
        master_key: Option<&MasterKey>,
        salt: Option<[u8; 16]>,
    ) -> Result<ProcessingMetadata> {
        let start_time = std::time::Instant::now();

        // 暫定的な実装：メモリ内処理
        // 実際の実装では、真のストリーミング処理を行う
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;

        let original_size = buffer.len() as u64;

        // 圧縮
        let compressed = if self.config.compression_type != CompressionType::None {
            self.compression_engine.compress(&buffer)?
        } else {
            CompressedData {
                compression_type: CompressionType::None,
                compression_level: 0,
                original_size,
                compressed_size: original_size,
                data: buffer,
            }
        };

        let compressed_size = compressed.compressed_size;

        // 暗号化
        let final_data = if let (Some(engine), Some(key), Some(s)) = (&self.encryption_engine, master_key, salt) {
            let encrypted = engine.encrypt(&compressed.data, key, s)?;
            encrypted.to_bytes()
        } else {
            compressed.data
        };

        let final_size = final_data.len() as u64;
        writer.write_all(&final_data)?;

        let processing_time = start_time.elapsed().as_millis() as u64;

        Ok(ProcessingMetadata {
            original_size,
            compressed_size,
            final_size,
            processing_time_ms: processing_time,
            compression_ratio: if original_size > 0 {
                (original_size.saturating_sub(compressed_size) as f64 / original_size as f64) * 100.0
            } else {
                0.0
            },
            memory_usage: original_size + compressed_size + final_size,
        })
    }

    /// 設定を取得
    pub fn config(&self) -> &PipelineConfig {
        &self.config
    }

    /// パフォーマンス統計を取得
    pub fn get_performance_stats(&self) -> PerformanceStats {
        PerformanceStats {
            available_threads: self.config.performance.parallel_threads,
            buffer_size: self.config.performance.buffer_size,
            memory_limit: self.config.performance.memory_limit,
            encryption_enabled: self.encryption_engine.is_some(),
            compression_type: self.config.compression_type,
        }
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

// num_cpuクレートを使用するためのプレースホルダー
// 実際の実装では num_cpus クレートを依存関係に追加する必要があります
mod num_cpus {
    pub fn get() -> usize {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
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

        let processed = pipeline.process_file(&temp_file, Some(&master_key), Some(salt)).unwrap();

        // 暗号化と圧縮が適用されていることを確認
        assert!(processed.compression_info.is_some());
        assert!(processed.encryption_info.is_some());
        assert!(processed.metadata.final_size > 0);

        // 復元テスト
        let restored = pipeline.restore_data(&processed, Some(&master_key)).unwrap();
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

        let metadata = pipeline.process_stream(reader, &mut output, None, None).unwrap();

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
}