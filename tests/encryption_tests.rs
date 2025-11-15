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
// Test 11: Nonce一意性の強化テスト - encrypt()経由で10,000個
// MISSED変異への対策: generate_nonce() → [0; 12], [1; 12]
// =============================================================================

#[test]
fn test_nonce_uniqueness_10000_generations() {
    use std::collections::HashSet;

    let engine = EncryptionEngine::default();
    let master_key = MasterKey::generate();
    let mut nonces = HashSet::new();
    let salt = [1u8; 16];

    // 10,000回暗号化して、生成されたnonceの一意性を確認
    for i in 0..10_000 {
        let data = format!("test data {}", i);
        let encrypted = engine.encrypt(data.as_bytes(), &master_key, salt).unwrap();

        assert!(
            nonces.insert(encrypted.nonce),
            "Nonce collision detected at {} generations! CRITICAL SECURITY ISSUE.",
            nonces.len()
        );
    }

    assert_eq!(nonces.len(), 10_000, "Expected 10,000 unique nonces");
}

// =============================================================================
// Test 12: Nonce がゼロや固定値でないことを確認 - encrypt()経由
// MISSED変異への対策: generate_nonce() → [0; 12], [1; 12]
// =============================================================================

#[test]
fn test_nonce_not_zero_or_fixed() {
    let engine = EncryptionEngine::default();
    let master_key = MasterKey::generate();
    let salt = [2u8; 16];

    // 100回暗号化して、生成されたnonceがゼロや固定値でないことを確認
    for i in 0..100 {
        let data = format!("test {}", i);
        let encrypted = engine.encrypt(data.as_bytes(), &master_key, salt).unwrap();

        assert_ne!(
            encrypted.nonce, [0u8; 12],
            "Zero nonce detected! CRITICAL: AES-GCM completely broken."
        );
        assert_ne!(
            encrypted.nonce, [1u8; 12],
            "Fixed nonce [1; 12] detected! CRITICAL: Nonce reuse attack possible."
        );
        assert_ne!(
            encrypted.nonce, [255u8; 12],
            "Fixed nonce [255; 12] detected! CRITICAL: Nonce reuse attack possible."
        );
    }
}

// =============================================================================
// Test 13: Nonce衝突検出機構（デバッグビルド専用）
// 目的: generate_nonce()の衝突検出が正常に動作することを確認
// セキュリティ影響: Nonce再利用はAES-GCMの致命的脆弱性
// =============================================================================

#[test]
fn test_nonce_collision_detection_debug_only() {
    // デバッグビルド専用の衝突検出機構をテスト
    // この機構は src/crypto/encryption.rs の generate_nonce() 内で
    // #[cfg(debug_assertions)] により実装されている
    //
    // 【セキュリティ上の重要性】
    // AES-256-GCMでは、同じ鍵で同じNonceを2回使用すると：
    // - 暗号化データの機密性が完全に失われる
    // - 認証タグが偽造可能になる
    // - 攻撃者が暗号鍵を復元できる可能性がある
    //
    // 【デバッグビルド専用の理由】
    // - リリースビルドではパフォーマンスへの影響ゼロ
    // - 開発中のバグ検出に特化
    // - コンパイル時にコードが完全削除される（#[cfg(debug_assertions)]）

    // 大量のNonceを生成して全てユニークであることを確認
    let iterations = 10_000;
    let mut nonces = std::collections::HashSet::new();

    for _ in 0..iterations {
        let nonce = EncryptionEngine::generate_nonce_internal();

        // 衝突が発生した場合、デバッグビルドでは即座にpanicする
        // このテストは衝突が発生しないことを確認するもの
        assert!(
            nonces.insert(nonce),
            "CRITICAL: Nonce collision detected! This should never happen.\n\
             Debug build should have panicked in generate_nonce() with detailed error message."
        );
    }

    // 全てのNonceがユニークであることを確認
    assert_eq!(
        nonces.len(),
        iterations,
        "All generated nonces must be unique"
    );

    // 【注意】
    // 意図的な衝突テスト（同じNonceを2回生成させるテスト）は、
    // 現在の実装では不可能です。なぜなら：
    // - generate_nonce()は内部的にMutex<HashSet>で追跡
    // - 衝突が発生した時点でpanicするため、テストとして捕捉できない
    //
    // 衝突検出の動作を確認したい場合は、以下の方法があります：
    // 1. 手動でgenerate_nonce()を改変し、固定値を返すようにする
    // 2. モックライブラリを使用してrand::rng()をモック化する
    // 3. デバッグビルドで実際にpanicが発生することを手動確認する
}

// =============================================================================
// Test 14: EncryptedData::from_bytes 境界値テスト（厳密版）
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
    // MISSED mutation: get_chunk_size() → 0, 1 を検出するテスト
    // 複数の異なるchunk_sizeで get_chunk_size() が異なる値を返すことを確認

    let config_small = EncryptionConfig {
        chunk_size: 1024, // 1KB
        buffer_size: 512,
    };
    let engine_small = EncryptionEngine::new(config_small);

    let config_medium = EncryptionConfig {
        chunk_size: 64 * 1024, // 64KB
        buffer_size: 8192,
    };
    let engine_medium = EncryptionEngine::new(config_medium);

    let config_large = EncryptionConfig {
        chunk_size: 1024 * 1024, // 1MB
        buffer_size: 16384,
    };
    let engine_large = EncryptionEngine::new(config_large);

    // MISSED mutation: get_chunk_size() → 0 would make all these equal to 0
    assert_ne!(
        engine_small.get_chunk_size(),
        0,
        "CRITICAL: get_chunk_size() returned 0! MISSED mutation detected."
    );
    assert_ne!(
        engine_medium.get_chunk_size(),
        0,
        "CRITICAL: get_chunk_size() returned 0! MISSED mutation detected."
    );
    assert_ne!(
        engine_large.get_chunk_size(),
        0,
        "CRITICAL: get_chunk_size() returned 0! MISSED mutation detected."
    );

    // MISSED mutation: get_chunk_size() → 1 would make all these equal to 1
    assert_ne!(
        engine_small.get_chunk_size(),
        1,
        "CRITICAL: get_chunk_size() returned 1! MISSED mutation detected."
    );
    assert_ne!(
        engine_medium.get_chunk_size(),
        1,
        "CRITICAL: get_chunk_size() returned 1! MISSED mutation detected."
    );
    assert_ne!(
        engine_large.get_chunk_size(),
        1,
        "CRITICAL: get_chunk_size() returned 1! MISSED mutation detected."
    );

    // 各サイズが正しく設定されていることを確認
    assert_eq!(engine_small.get_chunk_size(), 1024);
    assert_eq!(engine_medium.get_chunk_size(), 64 * 1024);
    assert_eq!(engine_large.get_chunk_size(), 1024 * 1024);

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
    let original_data =
        b"Test data for truncation check - this is a longer message for comprehensive testing";

    // 正常な暗号化
    let reader = Cursor::new(&original_data[..]);
    let mut encrypted_buffer = Vec::new();
    engine
        .encrypt_stream(reader, &mut encrypted_buffer, &master_key)
        .unwrap();

    // MISSED mutation: += with *= in decrypt_stream (line 311)
    // 正常な復号化サイズを確認（算術演算子変異の検出）
    let encrypted_reader = Cursor::new(&encrypted_buffer);
    let mut decrypted_buffer = Vec::new();
    let decrypted_size = engine
        .decrypt_stream(encrypted_reader, &mut decrypted_buffer, &master_key)
        .unwrap();

    assert_eq!(
        decrypted_size,
        original_data.len() as u64,
        "Decrypted size must match original size. MISSED mutation: += with *= would break this."
    );
    assert_eq!(decrypted_buffer, original_data);

    // 暗号化データを切り詰める（末尾10バイト削除）
    let truncated_len = encrypted_buffer.len().saturating_sub(10);
    encrypted_buffer.truncate(truncated_len);

    // 切り詰められたデータの復号化は失敗すべき
    let encrypted_reader_truncated = Cursor::new(encrypted_buffer);
    let mut decrypted_buffer_truncated = Vec::new();
    let result = engine.decrypt_stream(
        encrypted_reader_truncated,
        &mut decrypted_buffer_truncated,
        &master_key,
    );

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
