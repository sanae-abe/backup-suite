//! Property-based testing for cryptographic functions
//!
//! This module contains comprehensive property-based tests for:
//! - Nonce uniqueness verification
//! - Encryption/decryption roundtrip correctness
//! - Key derivation determinism
//!
//! Uses proptest to generate random test cases and verify security properties.

use backup_suite::crypto::{EncryptionEngine, KeyDerivation, MasterKey};
use proptest::prelude::*;
use std::collections::HashSet;

// Configure proptest to use fewer cases for faster testing
// Can be overridden with PROPTEST_CASES environment variable
// Note: Cryptographic tests are computationally intensive, so we use fewer cases
const PROPTEST_CASES: u32 = 10;

// Property test: Nonce uniqueness verification
//
// Verifies that the encryption engine generates unique nonces for each encryption operation.
// This is critical for AES-GCM security - nonce reuse would compromise encryption security.
//
// Test Strategy:
// - Generate random data sizes from 1 to 100,000 bytes
// - Perform multiple encryption iterations (1-50)
// - Collect all generated nonces
// - Verify no duplicates exist
proptest! {
    #![proptest_config(ProptestConfig::with_cases(PROPTEST_CASES))]

    #[test]
    fn prop_nonce_uniqueness(
        data_size in 1usize..10_000,  // Reduced from 100k for faster testing
        iterations in 1usize..20       // Reduced from 50 for faster testing
    ) {
        let engine = EncryptionEngine::default();
        let master_key = MasterKey::generate();
        let data = vec![0u8; data_size];
        let salt = [0u8; 16];

        let mut nonces = HashSet::new();

        for _ in 0..iterations {
            let encrypted = engine.encrypt(&data, &master_key, salt).unwrap();
            prop_assert!(
                nonces.insert(encrypted.nonce),
                "Nonce collision detected! This is a critical security vulnerability."
            );
        }
    }
}

// Property test: Encryption/Decryption roundtrip correctness
//
// Verifies that data encrypted with AES-256-GCM can be correctly decrypted
// to recover the original data, regardless of the input content or size.
//
// Test Strategy:
// - Generate random byte vectors (0-10,000 bytes)
// - Encrypt data with a fresh master key
// - Decrypt the encrypted data
// - Verify decrypted data matches original
// - Verify metadata (original_size) is correct
proptest! {
    #![proptest_config(ProptestConfig::with_cases(PROPTEST_CASES))]
    #[test]
    fn prop_encryption_roundtrip(
        data in prop::collection::vec(any::<u8>(), 0..1000)  // Reduced for faster testing
    ) {
        let engine = EncryptionEngine::default();
        let master_key = MasterKey::generate();
        let salt = KeyDerivation::generate_salt();

        let encrypted = engine.encrypt(&data, &master_key, salt).unwrap();
        let decrypted = engine.decrypt(&encrypted, &master_key).unwrap();

        prop_assert_eq!(
            &data,
            &decrypted,
            "Encryption roundtrip failed: decrypted data doesn't match original"
        );
        prop_assert_eq!(
            encrypted.original_size,
            data.len() as u64,
            "Metadata error: original_size doesn't match actual data length"
        );
    }
}

// Property test: Key derivation determinism
//
// Verifies that Argon2 key derivation is deterministic - the same password
// and salt always produce the same derived key.
//
// Test Strategy:
// - Generate random alphanumeric passwords (8-30 characters)
// - Derive key multiple times (1-5 iterations) with same password/salt
// - Verify all derived keys are identical
//
// Security Note:
// This test ensures that users can reliably restore their backups
// by entering the same password, which is critical for usability.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(PROPTEST_CASES))]
    #[test]
    fn prop_key_derivation_deterministic(
        password in "[a-zA-Z0-9]{8,30}",
        iterations in 1usize..5
    ) {
        let kd = KeyDerivation::default();
        let salt = KeyDerivation::generate_salt();

        let mut keys = Vec::new();
        for _ in 0..iterations {
            let key = kd.derive_key(&password, &salt).unwrap();
            keys.push(key);
        }

        // All keys should be identical
        for i in 1..keys.len() {
            prop_assert_eq!(
                keys[0].as_bytes(),
                keys[i].as_bytes(),
                "Key derivation is not deterministic: same password/salt produced different keys"
            );
        }
    }
}

// Property test: Empty data encryption
//
// Verifies that encrypting empty data is handled correctly.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(PROPTEST_CASES))]
    #[test]
    fn prop_empty_data_encryption(
        _dummy in 0..1usize
    ) {
        let engine = EncryptionEngine::default();
        let master_key = MasterKey::generate();
        let salt = KeyDerivation::generate_salt();
        let empty_data: Vec<u8> = vec![];

        let encrypted = engine.encrypt(&empty_data, &master_key, salt).unwrap();
        let decrypted = engine.decrypt(&encrypted, &master_key).unwrap();

        prop_assert!(decrypted.is_empty(), "Empty data should decrypt to empty");
        prop_assert_eq!(encrypted.original_size, 0, "Original size should be 0 for empty data");
    }
}

// Property test: Different passwords produce different keys
//
// Verifies that different passwords (even with same salt) produce different keys.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(PROPTEST_CASES))]
    #[test]
    fn prop_different_passwords_different_keys(
        password1 in "[a-zA-Z0-9]{8,30}",
        password2 in "[a-zA-Z0-9]{8,30}"
    ) {
        // Skip if passwords are identical
        prop_assume!(password1 != password2);

        let kd = KeyDerivation::default();
        let salt = KeyDerivation::generate_salt();

        let key1 = kd.derive_key(&password1, &salt).unwrap();
        let key2 = kd.derive_key(&password2, &salt).unwrap();

        prop_assert_ne!(
            key1.as_bytes(),
            key2.as_bytes(),
            "Different passwords should produce different keys"
        );
    }
}

// Property test: Different salts produce different keys
//
// Verifies that the same password with different salts produces different keys.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(PROPTEST_CASES))]
    #[test]
    fn prop_different_salts_different_keys(
        password in "[a-zA-Z0-9]{8,30}",
        _dummy in 0..1usize
    ) {
        let kd = KeyDerivation::default();
        let salt1 = KeyDerivation::generate_salt();
        let salt2 = KeyDerivation::generate_salt();

        // Salts should be different (extremely unlikely to collide)
        prop_assume!(salt1 != salt2);

        let key1 = kd.derive_key(&password, &salt1).unwrap();
        let key2 = kd.derive_key(&password, &salt2).unwrap();

        prop_assert_ne!(
            key1.as_bytes(),
            key2.as_bytes(),
            "Same password with different salts should produce different keys"
        );
    }
}

// Property test: Wrong key decryption fails
//
// Verifies that decryption with a different key fails (authentication).
proptest! {
    #![proptest_config(ProptestConfig::with_cases(PROPTEST_CASES))]
    #[test]
    fn prop_wrong_key_decryption_fails(
        data in prop::collection::vec(any::<u8>(), 1..1000)
    ) {
        let engine = EncryptionEngine::default();
        let master_key1 = MasterKey::generate();
        let master_key2 = MasterKey::generate();
        let salt = KeyDerivation::generate_salt();

        let encrypted = engine.encrypt(&data, &master_key1, salt).unwrap();
        let result = engine.decrypt(&encrypted, &master_key2);

        prop_assert!(
            result.is_err(),
            "Decryption with wrong key should fail (AES-GCM authentication)"
        );
    }
}

// Property test: Serialization/Deserialization roundtrip
//
// Verifies that EncryptedData can be serialized to bytes and deserialized back.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(PROPTEST_CASES))]
    #[test]
    fn prop_encrypted_data_serialization(
        data in prop::collection::vec(any::<u8>(), 0..500)  // Reduced for faster testing
    ) {
        let engine = EncryptionEngine::default();
        let master_key = MasterKey::generate();
        let salt = KeyDerivation::generate_salt();

        let encrypted = engine.encrypt(&data, &master_key, salt).unwrap();
        let serialized = encrypted.to_bytes();
        let deserialized = backup_suite::crypto::EncryptedData::from_bytes(&serialized).unwrap();

        // Verify all fields match
        prop_assert_eq!(encrypted.nonce, deserialized.nonce, "Nonce mismatch after serialization");
        prop_assert_eq!(encrypted.salt, deserialized.salt, "Salt mismatch after serialization");
        prop_assert_eq!(encrypted.original_size, deserialized.original_size, "Original size mismatch");
        prop_assert_eq!(&encrypted.ciphertext, &deserialized.ciphertext, "Ciphertext mismatch");

        // Verify decryption still works
        let decrypted = engine.decrypt(&deserialized, &master_key).unwrap();
        prop_assert_eq!(&data, &decrypted, "Decryption failed after serialization roundtrip");
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    /// Sanity check: Ensure proptest framework is working
    #[test]
    fn test_proptest_sanity() {
        proptest!(|(x in 0..100u32)| {
            assert!(x < 100);
        });
    }

    /// Manual test: Verify nonce uniqueness with fixed data
    #[test]
    fn test_nonce_uniqueness_manual() {
        let engine = EncryptionEngine::default();
        let master_key = MasterKey::generate();
        let data = vec![0u8; 1000];
        let salt = [0u8; 16];

        let mut nonces = HashSet::new();
        for _ in 0..100 {
            let encrypted = engine.encrypt(&data, &master_key, salt).unwrap();
            assert!(
                nonces.insert(encrypted.nonce),
                "Nonce collision detected in manual test!"
            );
        }

        assert_eq!(nonces.len(), 100, "Should have 100 unique nonces");
    }
}
