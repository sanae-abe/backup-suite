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
