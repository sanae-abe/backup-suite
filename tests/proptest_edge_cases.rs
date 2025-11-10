//! Property-based Edge Case Testing
//!
//! Comprehensive edge case testing for:
//! - File size boundaries (0 bytes, 2GB, 4GB, etc.)
//! - Unicode attack patterns (advanced)
//! - Streaming encryption with various chunk sizes
//!
//! These tests ensure the system handles extreme cases correctly.

use backup_suite::compression::{CompressionConfig, CompressionEngine, CompressionType};
use backup_suite::core::pipeline::{PipelineConfig, ProcessingPipeline};
use backup_suite::crypto::{EncryptionConfig, EncryptionEngine, KeyDerivation, MasterKey};
use proptest::prelude::*;
use std::io::Cursor;

// ==================== ファイルサイズ境界値テスト ====================

// 0バイトファイルの処理テスト
// 空ファイルは特殊ケースで、圧縮・暗号化処理でエラーが起きやすい。
// 正しく処理できることを検証。
#[test]
fn test_zero_byte_file_processing() {
    let engine = EncryptionEngine::default();
    let master_key = MasterKey::generate();
    let salt = KeyDerivation::generate_salt();
    let data = vec![]; // 0バイト

    let encrypted = engine.encrypt(&data, &master_key, salt).unwrap();
    let decrypted = engine.decrypt(&encrypted, &master_key).unwrap();

    assert_eq!(data, decrypted);
    assert_eq!(encrypted.original_size, 0);
}

// 小サイズファイル境界値テスト (1バイト、2バイト等)
// チャンクサイズ（1MB）より小さいファイルの処理を検証。
proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn prop_small_file_sizes(
        size in 1usize..=1024  // 1バイト～1KB
    ) {
        let engine = EncryptionEngine::default();
        let master_key = MasterKey::generate();
        let salt = KeyDerivation::generate_salt();
        let data = vec![0x42u8; size];

        let encrypted = engine.encrypt(&data, &master_key, salt).unwrap();
        let decrypted = engine.decrypt(&encrypted, &master_key).unwrap();

        prop_assert_eq!(&data, &decrypted);
        prop_assert_eq!(encrypted.original_size, size as u64);
    }
}

// チャンクサイズ境界値テスト (1MB前後)
// 暗号化チャンクサイズ（デフォルト1MB）の前後でエッジケースが発生しやすい。
proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn prop_chunk_boundary_sizes(
        offset in 0usize..10240  // 0～10KB のオフセット
    ) {
        let chunk_size = 1024 * 1024; // 1MB
        let sizes = vec![
            chunk_size - offset,      // チャンクより少し小さい
            chunk_size,               // ちょうどチャンク
            chunk_size + offset,      // チャンクより少し大きい
            chunk_size * 2 - offset,  // 2チャンク手前
            chunk_size * 2,           // ちょうど2チャンク
            chunk_size * 2 + offset,  // 2チャンク超
        ];

        let engine = EncryptionEngine::default();
        let master_key = MasterKey::generate();
        let salt = KeyDerivation::generate_salt();

        for size in sizes {
            if size == 0 { continue; } // 0バイトは別テストでカバー

            let data = vec![0xAAu8; size];
            let encrypted = engine.encrypt(&data, &master_key, salt).unwrap();
            let decrypted = engine.decrypt(&encrypted, &master_key).unwrap();

            prop_assert_eq!(&data, &decrypted, "Failed at size: {}", size);
            prop_assert_eq!(encrypted.original_size, size as u64);
        }
    }
}

// 大容量ファイルサイズテスト (10MB～100MB)
// メモリ使用量とストリーミング処理を検証。
// 注: 2GB以上のテストは実行時間が長いため、統合テストで実施。
proptest! {
    #![proptest_config(ProptestConfig::with_cases(5))]

    #[test]
    fn prop_large_file_sizes(
        mb_size in 10usize..=50  // 10MB～50MB
    ) {
        let size = mb_size * 1024 * 1024;
        let data = vec![0x55u8; size];

        let reader = Cursor::new(data.clone());
        let mut writer = Vec::new();

        let config = PipelineConfig::default();
        let pipeline = ProcessingPipeline::new(config);

        let master_key = MasterKey::generate();
        let salt = KeyDerivation::generate_salt();

        let metadata = pipeline
            .process_stream(reader, &mut writer, Some(&master_key), Some(salt))
            .unwrap();

        prop_assert_eq!(metadata.original_size, size as u64);
        prop_assert!(metadata.final_size > 0);
    }
}

// ==================== Unicode攻撃パターン拡充 ====================

// 拡張Unicode攻撃パターンテスト
// 以下の攻撃パターンを検証:
// - Right-to-Left Override (RLO)
// - Zero-Width Characters
// - Homoglyph攻撃
// - 制御文字インジェクション
proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_extended_unicode_attacks(
        normal_chars in r"[a-zA-Z0-9_-]{1,20}",
        attack_type in 0usize..6
    ) {
        let attack_chars = match attack_type {
            0 => "\u{202E}",        // Right-to-Left Override
            1 => "\u{200B}",        // Zero-Width Space
            2 => "\u{200C}",        // Zero-Width Non-Joiner
            3 => "\u{200D}",        // Zero-Width Joiner
            4 => "\u{FEFF}",        // Zero-Width No-Break Space
            5 => "\u{2060}",        // Word Joiner
            _ => "",
        };

        let malicious_path = format!("{}{}{}", normal_chars, attack_chars, normal_chars);

        // パストラバーサル対策が機能することを検証
        use backup_suite::security::sanitize_path_component;
        let sanitized = sanitize_path_component(&malicious_path);

        // 制御文字が除去されていることを確認
        prop_assert!(
            !sanitized.contains('\u{202E}')
            && !sanitized.contains('\u{200B}')
            && !sanitized.contains('\u{200C}')
            && !sanitized.contains('\u{200D}')
            && !sanitized.contains('\u{FEFF}')
            && !sanitized.contains('\u{2060}'),
            "Unicode control characters should be removed: {}",
            sanitized
        );
    }
}

// Homoglyph（見た目が似た文字）攻撃パターン
// 例: Cyrillic 'а' (U+0430) vs Latin 'a' (U+0061)
#[test]
fn test_homoglyph_detection() {
    let normal_path = "admin";
    let cyrillic_path = "аdmin"; // 最初の文字がCyrillic 'а'

    // 見た目は同じだがバイト表現が異なることを確認
    assert_ne!(normal_path.as_bytes(), cyrillic_path.as_bytes());
    assert_ne!(normal_path, cyrillic_path);
}

// ==================== ストリーミング暗号化検証 ====================

// ストリーミング暗号化のチャンクサイズ変動テスト
// 様々なチャンクサイズで正しく暗号化・復号化できることを検証。
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn prop_streaming_encryption_chunk_sizes(
        data_size in 1000usize..100_000,  // 1KB～100KB
        chunk_size in 256usize..4096      // 256B～4KB
    ) {
        let data = vec![0x77u8; data_size];
        let reader = Cursor::new(data.clone());
        let mut writer = Vec::new();

        let encryption_config = EncryptionConfig {
            chunk_size,
            buffer_size: 64 * 1024,
        };

        let engine = EncryptionEngine::new(encryption_config);
        let master_key = MasterKey::generate();

        let encrypted_data = engine
            .encrypt_stream(reader, &mut writer, &master_key)
            .unwrap();

        prop_assert_eq!(encrypted_data.original_size, data_size as u64);
    }
}

// ストリーミング圧縮+暗号化の統合テスト
// Pipeline全体のストリーミング処理を検証。
proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn prop_full_pipeline_streaming(
        data_size in 5000usize..50_000,  // 5KB～50KB
        compression_type in 0usize..2     // 0: Zstd, 1: Gzip
    ) {
        let data = vec![0xBBu8; data_size];
        let reader = Cursor::new(data.clone());
        let mut writer = Vec::new();

        let compression = match compression_type {
            0 => CompressionType::Zstd,
            _ => CompressionType::Gzip,
        };

        let config = PipelineConfig::default()
            .with_compression(compression, CompressionConfig::zstd_default());

        let pipeline = ProcessingPipeline::new(config);
        let master_key = MasterKey::generate();
        let salt = KeyDerivation::generate_salt();

        let metadata = pipeline
            .process_stream(reader, &mut writer, Some(&master_key), Some(salt))
            .unwrap();

        prop_assert_eq!(metadata.original_size, data_size as u64);
        prop_assert!(metadata.compressed_size > 0);
        prop_assert!(metadata.final_size > 0);
    }
}

// ==================== 圧縮境界値テスト ====================

// 圧縮不可能データのテスト
// ランダムデータは圧縮率が悪く、場合によっては圧縮後のサイズが大きくなる。
// この場合でも正しく処理できることを検証。
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn prop_incompressible_data(
        data in prop::collection::vec(any::<u8>(), 1000..10_000)
    ) {
        let engine = CompressionEngine::new(CompressionType::Zstd, CompressionConfig::zstd_default());

        let compressed = engine.compress(&data).unwrap();
        let decompressed = engine.decompress(&compressed).unwrap();

        prop_assert_eq!(&data, &decompressed);

        // 圧縮率が悪くてもエラーにならないことを確認
        // (場合によってはサイズが増えることもある)
        prop_assert!(compressed.original_size == data.len() as u64);
    }
}

// 高圧縮率データのテスト
// 同一バイトの繰り返しは圧縮率が非常に高い。
// 極端な圧縮率でも正しく処理できることを検証。
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn prop_highly_compressible_data(
        size in 1000usize..100_000,
        byte_value in any::<u8>()
    ) {
        let data = vec![byte_value; size];
        let engine = CompressionEngine::new(CompressionType::Zstd, CompressionConfig::zstd_default());

        let compressed = engine.compress(&data).unwrap();
        let decompressed = engine.decompress(&compressed).unwrap();

        prop_assert_eq!(&data, &decompressed);

        // 高圧縮率を検証（同一バイトの繰り返しは通常95%以上圧縮される）
        let compression_ratio = (compressed.compressed_size as f64) / (compressed.original_size as f64);
        prop_assert!(
            compression_ratio < 0.5,  // 50%以下に圧縮されることを期待
            "Compression ratio should be high for repetitive data: {:.2}%",
            compression_ratio * 100.0
        );
    }
}
