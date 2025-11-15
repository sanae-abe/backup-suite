// encryption_tests.rs - EncryptionEngine の統合テスト

use backup_suite::crypto::{EncryptedData, EncryptionConfig, EncryptionEngine, MasterKey};
use std::io::Cursor;

// =============================================================================
// Test 1: EncryptedData::from_bytes - 短すぎるデータ
// =============================================================================

#[test]
fn test_encrypted_data_from_bytes_too_short() {
    let short_data = vec![1, 2, 3, 4, 5]; // 44バイト未満
    let result = EncryptedData::from_bytes(&short_data);

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("短すぎます"));
}

// =============================================================================
// Test 2: EncryptedData::from_bytes - 長さ不一致
// =============================================================================

#[test]
fn test_encrypted_data_from_bytes_length_mismatch() {
    // ヘッダー（44バイト）を作成
    let mut invalid_data = vec![0u8; 44];

    // nonce (12バイト): 0-11
    invalid_data[0..12].copy_from_slice(&[1u8; 12]);

    // salt (16バイト): 12-27
    invalid_data[12..28].copy_from_slice(&[2u8; 16]);

    // original_size (8バイト): 28-35
    invalid_data[28..36].copy_from_slice(&100u64.to_le_bytes());

    // ciphertext_len (8バイト): 36-43 - 50バイトと主張
    invalid_data[36..44].copy_from_slice(&50u64.to_le_bytes());

    // 実際のciphertextは10バイトのみ追加（期待は50バイト）
    invalid_data.extend_from_slice(&[3u8; 10]);

    let result = EncryptedData::from_bytes(&invalid_data);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("長さが一致しません"));
}

// =============================================================================
// Test 3: EncryptedData::from_bytes - 正常なデシリアライズ
// =============================================================================

#[test]
fn test_encrypted_data_from_bytes_valid() {
    let engine = EncryptionEngine::default();
    let master_key = MasterKey::generate();
    let original_data = b"Valid data for deserialization test";
    let salt = [5u8; 16];

    // 暗号化してシリアライズ
    let encrypted = engine.encrypt(original_data, &master_key, salt).unwrap();
    let serialized = encrypted.to_bytes();

    // デシリアライズ
    let deserialized = EncryptedData::from_bytes(&serialized).unwrap();

    // 各フィールドを検証
    assert_eq!(deserialized.nonce, encrypted.nonce);
    assert_eq!(deserialized.salt, encrypted.salt);
    assert_eq!(deserialized.original_size, encrypted.original_size);
    assert_eq!(deserialized.ciphertext, encrypted.ciphertext);

    // 復号化が正常に動作することを確認
    let decrypted = engine.decrypt(&deserialized, &master_key).unwrap();
    assert_eq!(original_data, decrypted.as_slice());
}

// =============================================================================
// Test 4: encrypt_stream - 空ファイル処理
// =============================================================================

#[test]
fn test_encrypt_stream_empty_file() {
    let engine = EncryptionEngine::default();
    let master_key = MasterKey::generate();
    let empty_data: Vec<u8> = vec![];

    let reader = Cursor::new(&empty_data);
    let mut encrypted_buffer = Vec::new();

    let result = engine
        .encrypt_stream(reader, &mut encrypted_buffer, &master_key)
        .unwrap();

    assert_eq!(result.original_size, 0);

    // ヘッダー（nonce 12バイト + salt 16バイト = 28バイト）のみ存在
    assert_eq!(encrypted_buffer.len(), 28);

    // 復号化テスト
    let encrypted_reader = Cursor::new(&encrypted_buffer);
    let mut decrypted_buffer = Vec::new();
    let decrypted_size = engine
        .decrypt_stream(encrypted_reader, &mut decrypted_buffer, &master_key)
        .unwrap();

    assert_eq!(decrypted_size, 0);
    assert_eq!(decrypted_buffer, empty_data);
}

// =============================================================================
// Test 5: encrypt_stream - 単一チャンク（チャンクサイズ未満）
// =============================================================================

#[test]
fn test_encrypt_stream_single_chunk() {
    let engine = EncryptionEngine::default();
    let master_key = MasterKey::generate();
    // デフォルトチャンクサイズは1MBなので、100バイトは単一チャンク
    let original_data = vec![42u8; 100];

    let reader = Cursor::new(&original_data);
    let mut encrypted_buffer = Vec::new();

    let result = engine
        .encrypt_stream(reader, &mut encrypted_buffer, &master_key)
        .unwrap();

    assert_eq!(result.original_size, 100);

    // 復号化テスト
    let encrypted_reader = Cursor::new(&encrypted_buffer);
    let mut decrypted_buffer = Vec::new();
    let decrypted_size = engine
        .decrypt_stream(encrypted_reader, &mut decrypted_buffer, &master_key)
        .unwrap();

    assert_eq!(decrypted_size, 100);
    assert_eq!(decrypted_buffer, original_data);
}

// =============================================================================
// Test 6: encrypt_stream - 複数チャンク（大容量データ）
// =============================================================================

#[test]
fn test_encrypt_stream_multiple_chunks() {
    // カスタム設定で小さいチャンクサイズを使用（テスト高速化）
    let config = EncryptionConfig {
        chunk_size: 1024, // 1KB
        buffer_size: 512,
    };
    let engine = EncryptionEngine::new(config);
    let master_key = MasterKey::generate();

    // 5KB のデータ（5チャンクに分割される）
    let original_data = vec![99u8; 5 * 1024];

    let reader = Cursor::new(&original_data);
    let mut encrypted_buffer = Vec::new();

    let result = engine
        .encrypt_stream(reader, &mut encrypted_buffer, &master_key)
        .unwrap();

    assert_eq!(result.original_size, 5 * 1024);

    // 復号化テスト
    let encrypted_reader = Cursor::new(&encrypted_buffer);
    let mut decrypted_buffer = Vec::new();
    let decrypted_size = engine
        .decrypt_stream(encrypted_reader, &mut decrypted_buffer, &master_key)
        .unwrap();

    assert_eq!(decrypted_size, 5 * 1024);
    assert_eq!(decrypted_buffer, original_data);
}

// =============================================================================
// Test 7: カスタム EncryptionConfig - 大きいチャンクサイズ
// =============================================================================

#[test]
fn test_custom_config_large_chunk_size() {
    let config = EncryptionConfig {
        chunk_size: 5 * 1024 * 1024, // 5MB
        buffer_size: 256 * 1024,     // 256KB
    };
    let engine = EncryptionEngine::new(config);
    let master_key = MasterKey::generate();
    let original_data = b"Custom config test with large chunk size";
    let salt = [7u8; 16];

    // 基本的な暗号化・復号化が動作することを確認
    let encrypted = engine.encrypt(original_data, &master_key, salt).unwrap();
    let decrypted = engine.decrypt(&encrypted, &master_key).unwrap();

    assert_eq!(original_data, decrypted.as_slice());
    assert_eq!(engine.get_chunk_size(), 5 * 1024 * 1024);
}

// =============================================================================
// Test 8: カスタム EncryptionConfig - 小さいチャンクサイズ
// =============================================================================

#[test]
fn test_custom_config_small_chunk_size() {
    let config = EncryptionConfig {
        chunk_size: 512,  // 512バイト
        buffer_size: 256, // 256バイト
    };
    let engine = EncryptionEngine::new(config);
    let master_key = MasterKey::generate();

    // 2KBのデータ（4チャンクに分割される）
    let original_data = vec![123u8; 2048];

    let reader = Cursor::new(&original_data);
    let mut encrypted_buffer = Vec::new();

    let result = engine
        .encrypt_stream(reader, &mut encrypted_buffer, &master_key)
        .unwrap();

    assert_eq!(result.original_size, 2048);

    // 復号化テスト
    let encrypted_reader = Cursor::new(&encrypted_buffer);
    let mut decrypted_buffer = Vec::new();
    let decrypted_size = engine
        .decrypt_stream(encrypted_reader, &mut decrypted_buffer, &master_key)
        .unwrap();

    assert_eq!(decrypted_size, 2048);
    assert_eq!(decrypted_buffer, original_data);
}

// =============================================================================
// Test 9: generate_nonce_internal の一意性
// =============================================================================

#[test]
fn test_generate_nonce_internal_uniqueness() {
    use std::collections::HashSet;

    let mut nonces = HashSet::new();

    // 1000個のナンスを生成して一意性を確認
    for _ in 0..1000 {
        let nonce = EncryptionEngine::generate_nonce_internal();
        assert!(
            nonces.insert(nonce),
            "Nonce collision detected! Critical security issue."
        );
    }

    assert_eq!(nonces.len(), 1000);
}

// =============================================================================
// Test 10: ストリーム復号化 - 間違ったマスターキー
// =============================================================================

#[test]
fn test_decrypt_stream_wrong_master_key() {
    let engine = EncryptionEngine::default();
    let master_key1 = MasterKey::generate();
    let master_key2 = MasterKey::generate();
    let original_data = b"Secret stream data";

    let reader = Cursor::new(&original_data[..]);
    let mut encrypted_buffer = Vec::new();

    engine
        .encrypt_stream(reader, &mut encrypted_buffer, &master_key1)
        .unwrap();

    // 間違ったキーで復号化を試みる
    let encrypted_reader = Cursor::new(&encrypted_buffer);
    let mut decrypted_buffer = Vec::new();
    let result = engine.decrypt_stream(encrypted_reader, &mut decrypted_buffer, &master_key2);

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("復号化エラー"));
}

// =============================================================================
// CRITICAL TESTS - Mutation Testing で検出された脆弱性への対策
// =============================================================================

// =============================================================================
// Test 11: Nonce一意性の強化テスト - 10,000個
// MISSED変異への対策: generate_nonce() → [0; 12], [1; 12]
// =============================================================================

#[test]
fn test_nonce_uniqueness_10000_generations() {
    use std::collections::HashSet;

    let mut nonces = HashSet::new();

    // 10,000個のナンスを生成して一意性を確認
    for _ in 0..10_000 {
        let nonce = EncryptionEngine::generate_nonce_internal();

        assert!(
            nonces.insert(nonce),
            "Nonce collision detected at {} generations! CRITICAL SECURITY ISSUE.",
            nonces.len()
        );
    }

    assert_eq!(nonces.len(), 10_000, "Expected 10,000 unique nonces");
}

// =============================================================================
// Test 12: Nonce がゼロや固定値でないことを確認
// MISSED変異への対策: generate_nonce() → [0; 12], [1; 12]
// =============================================================================

#[test]
fn test_nonce_not_zero_or_fixed() {
    // 100個のナンスを生成して、ゼロや固定値でないことを確認
    for _ in 0..100 {
        let nonce = EncryptionEngine::generate_nonce_internal();

        assert_ne!(
            nonce, [0u8; 12],
            "Zero nonce detected! CRITICAL: AES-GCM completely broken."
        );
        assert_ne!(
            nonce, [1u8; 12],
            "Fixed nonce [1; 12] detected! CRITICAL: Nonce reuse attack possible."
        );
        assert_ne!(
            nonce, [255u8; 12],
            "Fixed nonce [255; 12] detected! CRITICAL: Nonce reuse attack possible."
        );
    }
}

// =============================================================================
// Test 13: EncryptedData::from_bytes 境界値テスト（厳密版）
// MISSED変異への対策: < → <= の境界条件変更
// =============================================================================

#[test]
fn test_encrypted_data_from_bytes_exact_boundary() {
    // 最小サイズ未満（43バイト）
    let too_small = vec![0u8; 43];
    assert!(
        EncryptedData::from_bytes(&too_small).is_err(),
        "43 bytes should be rejected (minimum is 44)"
    );

    // 最小サイズちょうど（44バイト）- これは受け入れられるべき
    let mut exact_size = vec![0u8; 44];
    exact_size[0..12].copy_from_slice(&[1u8; 12]); // nonce
    exact_size[12..28].copy_from_slice(&[2u8; 16]); // salt
    exact_size[28..36].copy_from_slice(&0u64.to_le_bytes()); // original_size = 0
    exact_size[36..44].copy_from_slice(&0u64.to_le_bytes()); // ciphertext_len = 0

    let result = EncryptedData::from_bytes(&exact_size);
    assert!(
        result.is_ok(),
        "44 bytes (exact minimum) should be accepted: {:?}",
        result.err()
    );

    // オフバイワン（45バイト、ciphertext_len=1）
    let mut off_by_one = vec![0u8; 45];
    off_by_one[0..12].copy_from_slice(&[1u8; 12]);
    off_by_one[12..28].copy_from_slice(&[2u8; 16]);
    off_by_one[28..36].copy_from_slice(&1u64.to_le_bytes());
    off_by_one[36..44].copy_from_slice(&1u64.to_le_bytes()); // ciphertext_len = 1
    off_by_one[44] = 0xFF; // 1バイトのciphertext

    assert!(
        EncryptedData::from_bytes(&off_by_one).is_ok(),
        "45 bytes should be accepted"
    );
}

// =============================================================================
// Test 14: チャンクサイズのバリデーション
// MISSED変異への対策: get_chunk_size() → 0, 1
// =============================================================================

#[test]
fn test_chunk_size_validation() {
    // ゼロチャンクサイズの検証（現状は許容されるが、将来的には拒否すべき）
    let config_zero = EncryptionConfig {
        chunk_size: 0,
        buffer_size: 8192,
    };
    // MISSED mutation: get_chunk_size() → 0 を検出するテスト
    let engine_zero = EncryptionEngine::new(config_zero);
    assert_eq!(
        engine_zero.get_chunk_size(),
        0,
        "CRITICAL: Zero chunk size detected! Risk: buffer overflow, infinite loop. This test verifies mutation detection."
    );

    // 1バイトチャンクサイズの検証（現状は許容されるが、DoS攻撃のリスク）
    let config_one = EncryptionConfig {
        chunk_size: 1,
        buffer_size: 8192,
    };
    // MISSED mutation: get_chunk_size() → 1 を検出するテスト
    let engine_one = EncryptionEngine::new(config_one);
    assert_eq!(
        engine_one.get_chunk_size(),
        1,
        "CRITICAL: Size 1 chunk detected! Risk: extreme performance degradation (DoS). This test verifies mutation detection."
    );

    // デフォルトチャンクサイズは適切な範囲内
    let config_default = EncryptionConfig::default();
    assert!(
        config_default.chunk_size >= 64 * 1024,
        "Default chunk size should be at least 64KB"
    );
    assert!(
        config_default.chunk_size <= 16 * 1024 * 1024,
        "Default chunk size should not exceed 16MB"
    );
}

// =============================================================================
// Test 15: エラーハンドリング - UnexpectedEof の適切な処理
// MISSED変異への対策: ErrorKind::UnexpectedEof チェックの無効化
// =============================================================================

#[test]
fn test_decrypt_stream_truncated_file() {
    let engine = EncryptionEngine::default();
    let master_key = MasterKey::generate();
    let original_data = b"Test data for truncation check";

    // 正常な暗号化
    let reader = Cursor::new(&original_data[..]);
    let mut encrypted_buffer = Vec::new();
    engine
        .encrypt_stream(reader, &mut encrypted_buffer, &master_key)
        .unwrap();

    // 暗号化データを切り詰める（末尾10バイト削除）
    let truncated_len = encrypted_buffer.len().saturating_sub(10);
    encrypted_buffer.truncate(truncated_len);

    // 切り詰められたデータの復号化は失敗すべき
    let encrypted_reader = Cursor::new(encrypted_buffer);
    let mut decrypted_buffer = Vec::new();
    let result = engine.decrypt_stream(encrypted_reader, &mut decrypted_buffer, &master_key);

    assert!(
        result.is_err(),
        "Truncated encrypted file must be detected! Risk: data corruption accepted as valid"
    );

    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("復号化エラー")
            || error_msg.contains("読み取りエラー")
            || error_msg.contains("I/Oエラー"),
        "Error message should indicate decryption or read failure, got: {}",
        error_msg
    );
}
