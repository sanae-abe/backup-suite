// compression_tests.rs - CompressionEngine の統合テスト

use backup_suite::compression::{
    CompressedData, CompressionConfig, CompressionEngine, CompressionType,
};
use std::io::Cursor;

// =============================================================================
// Test 1: Zstd fast設定での圧縮・展開
// =============================================================================

#[test]
fn test_zstd_fast_compression() {
    let config = CompressionConfig::fast(CompressionType::Zstd);
    let engine = CompressionEngine::new(CompressionType::Zstd, config);
    let original_data = b"Fast compression test data".repeat(100);

    let compressed = engine.compress(&original_data).unwrap();
    let decompressed = engine.decompress(&compressed).unwrap();

    assert_eq!(original_data, decompressed);
    assert_eq!(compressed.compression_level, 1);
    assert!(compressed.compressed_size < compressed.original_size);
}

// =============================================================================
// Test 2: Zstd best設定での圧縮・展開
// =============================================================================

#[test]
fn test_zstd_best_compression() {
    let config = CompressionConfig::best(CompressionType::Zstd);
    let engine = CompressionEngine::new(CompressionType::Zstd, config);
    let original_data = b"Best compression test data for maximum compression ratio".repeat(50);

    let compressed = engine.compress(&original_data).unwrap();
    let decompressed = engine.decompress(&compressed).unwrap();

    assert_eq!(original_data, decompressed);
    assert_eq!(compressed.compression_level, 19);
    assert!(compressed.compression_ratio() < 0.5); // 高圧縮率期待
}

// =============================================================================
// Test 3: Zstd adaptive設定での圧縮・展開
// =============================================================================

#[test]
fn test_zstd_adaptive_compression() {
    let config = CompressionConfig::zstd_adaptive();
    let engine = CompressionEngine::new(CompressionType::Zstd, config);
    let original_data = b"Adaptive compression test".repeat(100);

    let compressed = engine.compress(&original_data).unwrap();
    let decompressed = engine.decompress(&compressed).unwrap();

    assert_eq!(original_data, decompressed);
    // 適応的レベルはCPU数に応じて3/5/7のいずれか
    assert!([3, 5, 7].contains(&compressed.compression_level));
}

// =============================================================================
// Test 4: Gzip fast/best設定
// =============================================================================

#[test]
fn test_gzip_fast_best_compression() {
    // Fast
    let fast_config = CompressionConfig::fast(CompressionType::Gzip);
    let fast_engine = CompressionEngine::new(CompressionType::Gzip, fast_config);
    let original_data = b"Gzip configuration test data".repeat(50);

    let fast_compressed = fast_engine.compress(&original_data).unwrap();
    assert_eq!(fast_compressed.compression_level, 1);

    // Best
    let best_config = CompressionConfig::best(CompressionType::Gzip);
    let best_engine = CompressionEngine::new(CompressionType::Gzip, best_config);
    let best_compressed = best_engine.compress(&original_data).unwrap();
    assert_eq!(best_compressed.compression_level, 9);

    // Best圧縮の方が高圧縮率（サイズ小）
    assert!(best_compressed.compressed_size <= fast_compressed.compressed_size);
}

// =============================================================================
// Test 5: 空データの圧縮・展開
// =============================================================================

#[test]
fn test_empty_data_compression() {
    let engine = CompressionEngine::zstd(None);
    let empty_data: &[u8] = &[];

    let compressed = engine.compress(empty_data).unwrap();
    assert_eq!(compressed.original_size, 0);
    assert_eq!(compressed.compression_ratio(), 0.0);
    // 空データの場合、compression_percentage()は1.0 - 0.0 = 1.0 → 100.0%を返す
    assert_eq!(compressed.compression_percentage(), 100.0);

    let decompressed = engine.decompress(&compressed).unwrap();
    assert_eq!(decompressed, empty_data);
}

// =============================================================================
// Test 6: 圧縮率計算の検証
// =============================================================================

#[test]
fn test_compression_ratio_calculation() {
    let engine = CompressionEngine::zstd(None);
    let original_data = b"Compression ratio test".repeat(100);

    let compressed = engine.compress(&original_data).unwrap();

    // 圧縮率 = 圧縮後サイズ / 元のサイズ
    let expected_ratio = (compressed.compressed_size as f64) / (compressed.original_size as f64);
    assert!((compressed.compression_ratio() - expected_ratio).abs() < 1e-10);

    // 圧縮率パーセンテージ = (1 - 圧縮率) * 100
    let expected_percentage = (1.0 - expected_ratio) * 100.0;
    assert!((compressed.compression_percentage() - expected_percentage).abs() < 1e-10);
}

// =============================================================================
// Test 7: CompressedData::from_bytes エラーケース - 短すぎるデータ
// =============================================================================

#[test]
fn test_compressed_data_from_bytes_too_short() {
    let short_data = vec![1, 2, 3, 4, 5]; // 25バイト未満
    let result = CompressedData::from_bytes(&short_data);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("短すぎます"));
}

// =============================================================================
// Test 8: CompressedData::from_bytes エラーケース - 不明な圧縮タイプ
// =============================================================================

#[test]
fn test_compressed_data_from_bytes_invalid_type() {
    let mut invalid_data = vec![0u8; 25];
    invalid_data[0] = 99; // 不明なタイプ（0,1,2以外）
    invalid_data[1..5].copy_from_slice(&5u32.to_le_bytes());
    invalid_data[5..13].copy_from_slice(&100u64.to_le_bytes());
    invalid_data[13..21].copy_from_slice(&50u64.to_le_bytes());

    let result = CompressedData::from_bytes(&invalid_data);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("不明な圧縮タイプ"));
}

// =============================================================================
// Test 9: CompressedData::from_bytes エラーケース - 長さ不一致
// =============================================================================

#[test]
fn test_compressed_data_from_bytes_length_mismatch() {
    let mut invalid_data = vec![0u8; 25];
    invalid_data[0] = 1; // Zstd
    invalid_data[1..5].copy_from_slice(&5u32.to_le_bytes());
    invalid_data[5..13].copy_from_slice(&100u64.to_le_bytes());
    invalid_data[13..21].copy_from_slice(&50u64.to_le_bytes()); // compressed_size=50

    // 実際のデータは4バイトのみ（期待は50バイト）
    invalid_data.extend_from_slice(&[1, 2, 3, 4]);

    let result = CompressedData::from_bytes(&invalid_data);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("長さが一致しません"));
}

// =============================================================================
// Test 10: Gzipストリーム展開
// =============================================================================

#[test]
fn test_gzip_stream_decompression() {
    let engine = CompressionEngine::gzip(None);
    let original_data = b"Gzip stream decompression test".repeat(100);

    // ストリーム圧縮
    let reader = Cursor::new(&original_data);
    let mut compressed_buffer = Vec::new();
    let compressed_meta = engine
        .compress_stream(reader, &mut compressed_buffer)
        .unwrap();

    assert_eq!(compressed_meta.original_size, original_data.len() as u64);
    assert_eq!(compressed_meta.compression_type, CompressionType::Gzip);

    // ストリーム展開
    let compressed_reader = Cursor::new(&compressed_buffer);
    let mut decompressed_buffer = Vec::new();
    let decompressed_size = engine
        .decompress_stream(
            compressed_reader,
            &mut decompressed_buffer,
            CompressionType::Gzip,
        )
        .unwrap();

    assert_eq!(decompressed_size, original_data.len() as u64);
    assert_eq!(original_data, decompressed_buffer.as_slice());
}

// =============================================================================
// Test 11: 圧縮なしストリーム処理
// =============================================================================

#[test]
fn test_none_stream_compression() {
    let engine = CompressionEngine::none();
    let original_data = b"No compression stream test";

    let reader = Cursor::new(&original_data);
    let mut compressed_buffer = Vec::new();
    let compressed_meta = engine
        .compress_stream(reader, &mut compressed_buffer)
        .unwrap();

    assert_eq!(compressed_meta.original_size, original_data.len() as u64);
    assert_eq!(compressed_meta.compressed_size, original_data.len() as u64);
    assert_eq!(compressed_meta.compression_type, CompressionType::None);
    assert_eq!(compressed_buffer, original_data);

    // ストリーム展開
    let compressed_reader = Cursor::new(&compressed_buffer);
    let mut decompressed_buffer = Vec::new();
    let decompressed_size = engine
        .decompress_stream(
            compressed_reader,
            &mut decompressed_buffer,
            CompressionType::None,
        )
        .unwrap();

    assert_eq!(decompressed_size, original_data.len() as u64);
    assert_eq!(original_data, decompressed_buffer.as_slice());
}

// =============================================================================
// Test 12: 大容量データの圧縮・展開
// =============================================================================

#[test]
fn test_large_data_compression() {
    let engine = CompressionEngine::zstd(None);
    // 10MBのテストデータ
    let original_data = vec![0x42u8; 10 * 1024 * 1024];

    let compressed = engine.compress(&original_data).unwrap();
    let decompressed = engine.decompress(&compressed).unwrap();

    assert_eq!(original_data, decompressed);
    // 同じバイトの繰り返しなので高圧縮率期待
    assert!(compressed.compressed_size < original_data.len() as u64 / 100);
    assert!(compressed.compression_percentage() > 99.0);
}
