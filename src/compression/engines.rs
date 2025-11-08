//! # 圧縮エンジン
//!
//! zstd と gzip アルゴリズムによる高性能データ圧縮システム

use crate::error::{BackupError, Result};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use std::io::{Read, Write};
use std::str::FromStr;
use zstd::{Decoder as ZstdDecoder, Encoder as ZstdEncoder};

/// 圧縮アルゴリズムタイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionType {
    /// zstd 圧縮（高速・高圧縮率）
    Zstd,
    /// gzip 圧縮（互換性重視）
    Gzip,
    /// 圧縮なし
    None,
}

impl FromStr for CompressionType {
    type Err = BackupError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "zstd" => Ok(Self::Zstd),
            "gzip" => Ok(Self::Gzip),
            "none" => Ok(Self::None),
            _ => Err(BackupError::CompressionError(format!(
                "不明な圧縮タイプ: {s}"
            ))),
        }
    }
}

impl CompressionType {
    /// 圧縮タイプを文字列に変換
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Zstd => "zstd",
            Self::Gzip => "gzip",
            Self::None => "none",
        }
    }

    /// ファイル拡張子を取得
    pub fn file_extension(&self) -> &'static str {
        match self {
            Self::Zstd => ".zst",
            Self::Gzip => ".gz",
            Self::None => "",
        }
    }
}

/// 圧縮設定
#[derive(Debug, Clone)]
pub struct CompressionConfig {
    /// 圧縮レベル（1-22 for zstd, 1-9 for gzip）
    pub level: i32,
    /// チャンクサイズ（バイト）
    pub chunk_size: usize,
    /// バッファサイズ（バイト）
    pub buffer_size: usize,
}

impl CompressionConfig {
    /// zstd用のデフォルト設定（最適化版）
    pub fn zstd_default() -> Self {
        Self {
            level: 5,                    // 速度と圧縮率のバランス（3→5に最適化）
            chunk_size: 2 * 1024 * 1024, // 2MB チャンク（キャッシュ効率向上）
            buffer_size: 128 * 1024,     // 128KB バッファ（I/O効率向上）
        }
    }

    /// zstd用の適応的設定（CPU数に基づく動的調整）
    pub fn zstd_adaptive() -> Self {
        let cpu_count = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);

        Self {
            level: if cpu_count >= 8 {
                7
            } else if cpu_count >= 4 {
                5
            } else {
                3
            },
            chunk_size: 2 * 1024 * 1024,
            buffer_size: 128 * 1024,
        }
    }

    /// gzip用のデフォルト設定
    pub fn gzip_default() -> Self {
        Self {
            level: 6,                // デフォルトレベル
            chunk_size: 1024 * 1024, // 1MB チャンク
            buffer_size: 64 * 1024,  // 64KB バッファ
        }
    }

    /// 高速圧縮設定
    pub fn fast(compression_type: CompressionType) -> Self {
        match compression_type {
            CompressionType::Zstd => Self {
                level: 1,
                chunk_size: 2 * 1024 * 1024, // 2MB チャンク（高速化）
                buffer_size: 128 * 1024,     // 128KB バッファ
            },
            CompressionType::Gzip => Self {
                level: 1,
                chunk_size: 2 * 1024 * 1024,
                buffer_size: 128 * 1024,
            },
            CompressionType::None => Self::none(),
        }
    }

    /// 高圧縮率設定
    pub fn best(compression_type: CompressionType) -> Self {
        match compression_type {
            CompressionType::Zstd => Self {
                level: 19,              // 高圧縮率
                chunk_size: 512 * 1024, // 512KB チャンク（圧縮率重視）
                buffer_size: 32 * 1024, // 32KB バッファ
            },
            CompressionType::Gzip => Self {
                level: 9,
                chunk_size: 512 * 1024,
                buffer_size: 32 * 1024,
            },
            CompressionType::None => Self::none(),
        }
    }

    /// 圧縮なし設定
    pub fn none() -> Self {
        Self {
            level: 0,
            chunk_size: 4 * 1024 * 1024, // 4MB チャンク（コピーのみ）
            buffer_size: 256 * 1024,     // 256KB バッファ
        }
    }
}

/// 圧縮されたデータ
#[derive(Debug, Clone)]
pub struct CompressedData {
    /// 圧縮タイプ
    pub compression_type: CompressionType,
    /// 圧縮レベル
    pub compression_level: i32,
    /// 元のデータサイズ
    pub original_size: u64,
    /// 圧縮後のデータサイズ
    pub compressed_size: u64,
    /// 圧縮されたデータ
    pub data: Vec<u8>,
}

impl CompressedData {
    /// 圧縮率を計算
    pub fn compression_ratio(&self) -> f64 {
        if self.original_size == 0 {
            return 0.0;
        }
        (self.compressed_size as f64) / (self.original_size as f64)
    }

    /// 圧縮率をパーセンテージで取得
    pub fn compression_percentage(&self) -> f64 {
        (1.0 - self.compression_ratio()) * 100.0
    }

    /// バイナリ形式にシリアライズ
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(25 + self.data.len());

        // ヘッダー情報
        result.push(match self.compression_type {
            CompressionType::Zstd => 1,
            CompressionType::Gzip => 2,
            CompressionType::None => 0,
        });
        result.extend_from_slice(&(self.compression_level as u32).to_le_bytes());
        result.extend_from_slice(&self.original_size.to_le_bytes());
        result.extend_from_slice(&self.compressed_size.to_le_bytes());

        // データ
        result.extend_from_slice(&self.data);
        result
    }

    /// バイナリ形式からデシリアライズ
    ///
    /// # Errors
    ///
    /// 以下の場合にエラーを返します:
    /// - データが最小長（25バイト）未満の場合
    /// - 不明な圧縮タイプの場合
    /// - データの長さが一致しない場合
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < 25 {
            return Err(BackupError::CompressionError(
                "圧縮データが短すぎます".to_string(),
            ));
        }

        // SAFETY: Length check above ensures data has at least 25 bytes
        let compression_type = match *data
            .first()
            .ok_or_else(|| BackupError::CompressionError("データが空です".to_string()))?
        {
            1 => CompressionType::Zstd,
            2 => CompressionType::Gzip,
            0 => CompressionType::None,
            _ => {
                return Err(BackupError::CompressionError(
                    "不明な圧縮タイプ".to_string(),
                ))
            }
        };

        let compression_level = u32::from_le_bytes(
            data.get(1..5)
                .and_then(|s| s.try_into().ok())
                .ok_or_else(|| {
                    BackupError::CompressionError("圧縮レベルの読み取りに失敗".to_string())
                })?,
        ) as i32;
        let original_size =
            u64::from_le_bytes(data.get(5..13).and_then(|s| s.try_into().ok()).ok_or_else(
                || BackupError::CompressionError("元のサイズの読み取りに失敗".to_string()),
            )?);
        let compressed_size = u64::from_le_bytes(
            data.get(13..21)
                .and_then(|s| s.try_into().ok())
                .ok_or_else(|| {
                    BackupError::CompressionError("圧縮後サイズの読み取りに失敗".to_string())
                })?,
        );

        if data.len() != 21 + compressed_size as usize {
            return Err(BackupError::CompressionError(
                "圧縮データの長さが一致しません".to_string(),
            ));
        }

        Ok(Self {
            compression_type,
            compression_level,
            original_size,
            compressed_size,
            data: data
                .get(21..)
                .ok_or_else(|| BackupError::CompressionError("データの読み取りに失敗".to_string()))?
                .to_vec(),
        })
    }
}

/// 圧縮エンジン
pub struct CompressionEngine {
    config: CompressionConfig,
    compression_type: CompressionType,
}

impl CompressionEngine {
    /// 新しい圧縮エンジンを作成
    pub fn new(compression_type: CompressionType, config: CompressionConfig) -> Self {
        Self {
            config,
            compression_type,
        }
    }

    /// zstd圧縮エンジンを作成
    pub fn zstd(config: Option<CompressionConfig>) -> Self {
        Self::new(
            CompressionType::Zstd,
            config.unwrap_or_else(CompressionConfig::zstd_default),
        )
    }

    /// gzip圧縮エンジンを作成
    pub fn gzip(config: Option<CompressionConfig>) -> Self {
        Self::new(
            CompressionType::Gzip,
            config.unwrap_or_else(CompressionConfig::gzip_default),
        )
    }

    /// 圧縮なしエンジンを作成
    pub fn none() -> Self {
        Self::new(CompressionType::None, CompressionConfig::none())
    }

    /// データを圧縮
    ///
    /// # Errors
    ///
    /// 圧縮エンジンがデータの圧縮に失敗した場合にエラーを返します。
    pub fn compress(&self, data: &[u8]) -> Result<CompressedData> {
        let original_size = data.len() as u64;

        let compressed_data = match self.compression_type {
            CompressionType::Zstd => self.compress_zstd(data)?,
            CompressionType::Gzip => self.compress_gzip(data)?,
            CompressionType::None => data.to_vec(),
        };

        let compressed_size = compressed_data.len() as u64;

        Ok(CompressedData {
            compression_type: self.compression_type,
            compression_level: self.config.level,
            original_size,
            compressed_size,
            data: compressed_data,
        })
    }

    /// データを展開
    ///
    /// # Errors
    ///
    /// 圧縮エンジンがデータの展開に失敗した場合にエラーを返します。
    pub fn decompress(&self, compressed_data: &CompressedData) -> Result<Vec<u8>> {
        match compressed_data.compression_type {
            CompressionType::Zstd => self.decompress_zstd(&compressed_data.data),
            CompressionType::Gzip => self.decompress_gzip(&compressed_data.data),
            CompressionType::None => Ok(compressed_data.data.clone()),
        }
    }

    /// ストリーミング圧縮
    ///
    /// # Errors
    ///
    /// 以下の場合にエラーを返します:
    /// - リーダーからの読み取りに失敗した場合
    /// - 圧縮エンジンの作成・実行に失敗した場合
    /// - ライターへの書き込みに失敗した場合
    #[allow(clippy::indexing_slicing)] // read() guarantees bytes_read <= buffer.len()
    pub fn compress_stream<R: Read, W: Write>(
        &self,
        mut reader: R,
        mut writer: W,
    ) -> Result<CompressedData> {
        let mut original_size = 0u64;
        let mut compressed_buffer = Vec::new();

        match self.compression_type {
            CompressionType::Zstd => {
                let mut encoder = ZstdEncoder::new(&mut compressed_buffer, self.config.level)
                    .map_err(|e| {
                        BackupError::CompressionError(format!("Zstdエンコーダ作成エラー: {e}"))
                    })?;

                let mut buffer = vec![0u8; self.config.buffer_size];
                loop {
                    let bytes_read = reader.read(&mut buffer)?;
                    if bytes_read == 0 {
                        break;
                    }
                    original_size += bytes_read as u64;
                    encoder.write_all(&buffer[..bytes_read]).map_err(|e| {
                        BackupError::CompressionError(format!("Zstd圧縮エラー: {e}"))
                    })?;
                }

                encoder
                    .finish()
                    .map_err(|e| BackupError::CompressionError(format!("Zstd完了エラー: {e}")))?;
            }
            CompressionType::Gzip => {
                let mut encoder = GzEncoder::new(
                    &mut compressed_buffer,
                    Compression::new(self.config.level as u32),
                );

                let mut buffer = vec![0u8; self.config.buffer_size];
                loop {
                    let bytes_read = reader.read(&mut buffer)?;
                    if bytes_read == 0 {
                        break;
                    }
                    original_size += bytes_read as u64;
                    encoder.write_all(&buffer[..bytes_read])?;
                }

                encoder.finish()?;
            }
            CompressionType::None => {
                let mut buffer = vec![0u8; self.config.buffer_size];
                loop {
                    let bytes_read = reader.read(&mut buffer)?;
                    if bytes_read == 0 {
                        break;
                    }
                    original_size += bytes_read as u64;
                    compressed_buffer.extend_from_slice(&buffer[..bytes_read]);
                }
            }
        }

        writer.write_all(&compressed_buffer)?;

        Ok(CompressedData {
            compression_type: self.compression_type,
            compression_level: self.config.level,
            original_size,
            compressed_size: compressed_buffer.len() as u64,
            data: compressed_buffer,
        })
    }

    /// ストリーミング展開
    ///
    /// # Errors
    ///
    /// 以下の場合にエラーを返します:
    /// - デコーダーの作成に失敗した場合
    /// - リーダーからの読み取りに失敗した場合
    /// - ライターへの書き込みに失敗した場合
    #[allow(clippy::indexing_slicing)] // read() guarantees bytes_read <= buffer.len()
    pub fn decompress_stream<R: Read, W: Write>(
        &self,
        reader: R,
        mut writer: W,
        compression_type: CompressionType,
    ) -> Result<u64> {
        let mut decompressed_size = 0u64;

        match compression_type {
            CompressionType::Zstd => {
                let mut decoder = ZstdDecoder::new(reader).map_err(|e| {
                    BackupError::CompressionError(format!("Zstdデコーダ作成エラー: {e}"))
                })?;

                let mut buffer = vec![0u8; self.config.buffer_size];
                loop {
                    let bytes_read = decoder.read(&mut buffer).map_err(|e| {
                        BackupError::CompressionError(format!("Zstd展開エラー: {e}"))
                    })?;
                    if bytes_read == 0 {
                        break;
                    }
                    writer.write_all(&buffer[..bytes_read])?;
                    decompressed_size += bytes_read as u64;
                }
            }
            CompressionType::Gzip => {
                let mut decoder = GzDecoder::new(reader);

                let mut buffer = vec![0u8; self.config.buffer_size];
                loop {
                    let bytes_read = decoder.read(&mut buffer)?;
                    if bytes_read == 0 {
                        break;
                    }
                    writer.write_all(&buffer[..bytes_read])?;
                    decompressed_size += bytes_read as u64;
                }
            }
            CompressionType::None => {
                let mut reader = reader;
                let mut buffer = vec![0u8; self.config.buffer_size];
                loop {
                    let bytes_read = reader.read(&mut buffer)?;
                    if bytes_read == 0 {
                        break;
                    }
                    writer.write_all(&buffer[..bytes_read])?;
                    decompressed_size += bytes_read as u64;
                }
            }
        }

        Ok(decompressed_size)
    }

    // プライベートメソッド

    fn compress_zstd(&self, data: &[u8]) -> Result<Vec<u8>> {
        zstd::encode_all(data, self.config.level)
            .map_err(|e| BackupError::CompressionError(format!("Zstd圧縮エラー: {e}")))
    }

    fn decompress_zstd(&self, data: &[u8]) -> Result<Vec<u8>> {
        zstd::decode_all(data)
            .map_err(|e| BackupError::CompressionError(format!("Zstd展開エラー: {e}")))
    }

    fn compress_gzip(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::new(self.config.level as u32));
        encoder.write_all(data)?;
        encoder
            .finish()
            .map_err(|e| BackupError::CompressionError(format!("Gzip圧縮エラー: {e}")))
    }

    fn decompress_gzip(&self, data: &[u8]) -> Result<Vec<u8>> {
        use std::io::Cursor;
        let mut decoder = GzDecoder::new(Cursor::new(data));
        let mut result = Vec::new();
        decoder.read_to_end(&mut result)?;
        Ok(result)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)] // Tests can use unwrap
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_compression_types() {
        assert_eq!(
            CompressionType::from_str("zstd").unwrap(),
            CompressionType::Zstd
        );
        assert_eq!(
            CompressionType::from_str("gzip").unwrap(),
            CompressionType::Gzip
        );
        assert_eq!(
            CompressionType::from_str("none").unwrap(),
            CompressionType::None
        );

        assert_eq!(CompressionType::Zstd.to_str(), "zstd");
        assert_eq!(CompressionType::Gzip.file_extension(), ".gz");
    }

    #[test]
    fn test_zstd_compression() {
        let engine = CompressionEngine::zstd(None);
        let original_data = b"Hello, World! This is a test message for compression.".repeat(100);

        let compressed = engine.compress(&original_data).unwrap();
        let decompressed = engine.decompress(&compressed).unwrap();

        assert_eq!(original_data, decompressed);
        assert!(compressed.compressed_size < compressed.original_size);
        assert!(compressed.compression_percentage() > 0.0);
    }

    #[test]
    fn test_gzip_compression() {
        let engine = CompressionEngine::gzip(None);
        let original_data = b"Test data for gzip compression algorithm.".repeat(50);

        let compressed = engine.compress(&original_data).unwrap();
        let decompressed = engine.decompress(&compressed).unwrap();

        assert_eq!(original_data, decompressed);
        assert!(compressed.compressed_size < compressed.original_size);
    }

    #[test]
    fn test_no_compression() {
        let engine = CompressionEngine::none();
        let original_data = b"This data should not be compressed";

        let compressed = engine.compress(original_data).unwrap();
        let decompressed = engine.decompress(&compressed).unwrap();

        assert_eq!(original_data, decompressed.as_slice());
        assert_eq!(compressed.compressed_size, compressed.original_size);
        assert_eq!(compressed.compression_percentage(), 0.0);
    }

    #[test]
    fn test_compressed_data_serialization() {
        let engine = CompressionEngine::zstd(None);
        let original_data = b"Serialization test data";

        let compressed = engine.compress(original_data).unwrap();
        let serialized = compressed.to_bytes();
        let deserialized = CompressedData::from_bytes(&serialized).unwrap();

        let decompressed = engine.decompress(&deserialized).unwrap();
        assert_eq!(original_data, decompressed.as_slice());
    }

    #[test]
    fn test_stream_compression() {
        let engine = CompressionEngine::zstd(None);
        let original_data = b"Stream compression test data. ".repeat(1000);

        let reader = Cursor::new(&original_data);
        let mut compressed_buffer = Vec::new();
        let compressed_meta = engine
            .compress_stream(reader, &mut compressed_buffer)
            .unwrap();

        assert_eq!(compressed_meta.original_size, original_data.len() as u64);

        let compressed_reader = Cursor::new(&compressed_buffer);
        let mut decompressed_buffer = Vec::new();
        let decompressed_size = engine
            .decompress_stream(
                compressed_reader,
                &mut decompressed_buffer,
                CompressionType::Zstd,
            )
            .unwrap();

        assert_eq!(decompressed_size, original_data.len() as u64);
        assert_eq!(original_data, decompressed_buffer);
    }
}
