//! # AES-256-GCM 暗号化エンジン
//!
//! 認証付き暗号化を提供する高セキュリティ暗号化システム

use super::key_management::MasterKey;
use crate::error::{BackupError, Result};
use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce};
use rand::RngCore;
use std::io::{Read, Write};

// デバッグビルド専用: Nonce衝突検出トラッカー
// リリースビルドではコンパイル時に完全削除される（オーバーヘッドゼロ）
#[cfg(debug_assertions)]
use std::collections::HashSet;
#[cfg(debug_assertions)]
use std::sync::Mutex;

#[cfg(debug_assertions)]
static NONCE_TRACKER: Mutex<Option<HashSet<[u8; 12]>>> = Mutex::new(None);

/// 暗号化設定
#[derive(Debug, Clone)]
pub struct EncryptionConfig {
    /// チャンクサイズ（バイト） - 大容量ファイル処理用
    pub chunk_size: usize,
    /// バッファサイズ（バイト）
    pub buffer_size: usize,
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            chunk_size: 1024 * 1024, // 1MB チャンク
            buffer_size: 64 * 1024,  // 64KB バッファ
        }
    }
}

/// 暗号化されたデータ
#[derive(Debug, Clone)]
pub struct EncryptedData {
    /// ナンス（12バイト）
    pub nonce: [u8; 12],
    /// ソルト（16バイト）
    pub salt: [u8; 16],
    /// 暗号化されたデータ + 認証タグ
    pub ciphertext: Vec<u8>,
    /// ファイル長（元のデータサイズ）
    pub original_size: u64,
}

impl EncryptedData {
    /// バイナリ形式にシリアライズ
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(44 + self.ciphertext.len());
        result.extend_from_slice(&self.nonce); // 12バイト
        result.extend_from_slice(&self.salt); // 16バイト
        result.extend_from_slice(&self.original_size.to_le_bytes()); // 8バイト
        result.extend_from_slice(&(self.ciphertext.len() as u64).to_le_bytes()); // 8バイト
        result.extend_from_slice(&self.ciphertext); // 可変長
        result
    }

    /// バイナリ形式からデシリアライズ
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < 44 {
            return Err(BackupError::EncryptionError(
                "暗号化データが短すぎます".to_string(),
            ));
        }

        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(&data[0..12]);

        let mut salt = [0u8; 16];
        salt.copy_from_slice(&data[12..28]);

        let original_size = u64::from_le_bytes([
            data[28], data[29], data[30], data[31], data[32], data[33], data[34], data[35],
        ]);

        let ciphertext_len = u64::from_le_bytes([
            data[36], data[37], data[38], data[39], data[40], data[41], data[42], data[43],
        ]) as usize;

        // オーバーフロー対策: checked_add を使用
        let expected_len = match 44usize.checked_add(ciphertext_len) {
            Some(len) => len,
            None => {
                return Err(BackupError::EncryptionError(
                    "暗号化データの長さが不正です（オーバーフロー）".to_string(),
                ));
            }
        };

        if data.len() != expected_len {
            return Err(BackupError::EncryptionError(
                "暗号化データの長さが一致しません".to_string(),
            ));
        }

        let ciphertext = data[44..].to_vec();

        Ok(Self {
            nonce,
            salt,
            ciphertext,
            original_size,
        })
    }
}

/// AES-256-GCM 暗号化エンジン
pub struct EncryptionEngine {
    config: EncryptionConfig,
}

impl EncryptionEngine {
    /// 新しい暗号化エンジンを作成
    #[must_use]
    pub fn new(config: EncryptionConfig) -> Self {
        Self { config }
    }

    /// ランダムなナンスを生成（内部用）
    ///
    /// デバッグビルドでは、生成された全Nonceを追跡し、衝突を検出します。
    /// リリースビルドでは、追跡コードはコンパイル時に完全削除されます（オーバーヘッドゼロ）。
    fn generate_nonce() -> [u8; 12] {
        let mut nonce = [0u8; 12];
        rand::rng().fill_bytes(&mut nonce);

        // デバッグビルド専用: Nonce衝突検出
        // リリースビルドではこのブロック全体がコンパイル時に削除される
        #[cfg(debug_assertions)]
        {
            let mut tracker = NONCE_TRACKER.lock().unwrap();
            let set = tracker.get_or_insert_with(HashSet::new);

            if !set.insert(nonce) {
                panic!(
                    "\n\
                    ╔══════════════════════════════════════════════════════════════╗\n\
                    ║  🚨 CRITICAL SECURITY VIOLATION: Nonce Collision Detected! ║\n\
                    ╚══════════════════════════════════════════════════════════════╝\n\
                    \n\
                    Nonce (hex): {:02x?}\n\
                    \n\
                    ⚠️  SECURITY IMPACT:\n\
                    This is a CRITICAL security vulnerability in AES-256-GCM encryption.\n\
                    Nonce reuse completely breaks the confidentiality and authenticity guarantees.\n\
                    \n\
                    An attacker can:\n\
                    - Decrypt encrypted data without the key\n\
                    - Forge authenticated messages\n\
                    - Recover the encryption key\n\
                    \n\
                    📊 STATISTICS:\n\
                    - Total unique nonces generated so far: {}\n\
                    - Collision detected on nonce #{}\n\
                    \n\
                    ℹ️  DEBUG BUILD ONLY:\n\
                    This panic only occurs in debug builds to help detect bugs during development.\n\
                    Release builds have zero overhead (this code is removed at compile time).\n\
                    \n\
                    🔧 NEXT STEPS:\n\
                    1. Check if this is a test scenario (intentional collision test)\n\
                    2. If not, investigate random number generation (rand crate)\n\
                    3. Review recent changes to generate_nonce() function\n\
                    4. Run mutation testing to verify detection works correctly\n\
                    \n",
                    nonce,
                    set.len(),
                    set.len() + 1
                );
            }
        }

        nonce
    }

    /// ランダムなナンスを生成（公開API、pipeline.rsから使用）
    #[must_use]
    pub fn generate_nonce_internal() -> [u8; 12] {
        Self::generate_nonce()
    }

    /// チャンクサイズを取得（公開API、pipeline.rsから使用）
    #[must_use]
    pub fn get_chunk_size(&self) -> usize {
        self.config.chunk_size
    }

    /// データを暗号化
    ///
    /// # Errors
    ///
    /// 以下の場合にエラーを返します:
    /// - AES-256-GCM暗号化処理が失敗した場合 (`BackupError::EncryptionError`)
    #[allow(deprecated)]
    pub fn encrypt(
        &self,
        data: &[u8],
        master_key: &MasterKey,
        salt: [u8; 16],
    ) -> Result<EncryptedData> {
        let nonce_bytes = Self::generate_nonce();

        // AES-256-GCM 暗号化
        let key = Key::<Aes256Gcm>::from_slice(master_key.as_bytes());
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, data)
            .map_err(|e| BackupError::EncryptionError(format!("暗号化エラー: {e}")))?;

        Ok(EncryptedData {
            nonce: nonce_bytes,
            salt, // 渡されたsaltを使用（新しく生成しない）
            ciphertext,
            original_size: data.len() as u64,
        })
    }

    /// データを復号化
    ///
    /// # Errors
    ///
    /// 以下の場合にエラーを返します:
    /// - AES-256-GCM復号化処理が失敗した場合 (`BackupError::EncryptionError`)
    ///   - 認証タグの検証に失敗した場合（データが改ざんされている可能性）
    ///   - 不正なマスターキーが使用された場合
    #[allow(deprecated)]
    pub fn decrypt(
        &self,
        encrypted_data: &EncryptedData,
        master_key: &MasterKey,
    ) -> Result<Vec<u8>> {
        let key = Key::<Aes256Gcm>::from_slice(master_key.as_bytes());
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(&encrypted_data.nonce);

        let plaintext = cipher
            .decrypt(nonce, encrypted_data.ciphertext.as_ref())
            .map_err(|e| BackupError::EncryptionError(format!("復号化エラー: {e}")))?;

        Ok(plaintext)
    }

    /// ストリーミング暗号化（大容量ファイル用）
    ///
    /// # Errors
    ///
    /// 以下の場合にエラーを返します:
    /// - ファイル読み込みエラー (`BackupError::IoError`)
    /// - ファイル書き込みエラー (`BackupError::IoError`)
    /// - チャンク毎のAES-256-GCM暗号化処理が失敗した場合 (`BackupError::EncryptionError`)
    #[allow(deprecated)]
    pub fn encrypt_stream<R: Read, W: Write>(
        &self,
        mut reader: R,
        mut writer: W,
        master_key: &MasterKey,
    ) -> Result<EncryptedData> {
        let nonce_bytes = Self::generate_nonce();
        let salt = crate::crypto::key_management::KeyDerivation::generate_salt();

        // ヘッダー情報を書き込み
        writer.write_all(&nonce_bytes)?;
        writer.write_all(&salt)?;

        let key = Key::<Aes256Gcm>::from_slice(master_key.as_bytes());
        let cipher = Aes256Gcm::new(key);

        let mut total_size = 0u64;
        let mut encrypted_chunks = Vec::new();
        let mut buffer = vec![0u8; self.config.chunk_size];

        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            total_size += bytes_read as u64;

            // チャンク毎に異なるナンスを使用（u64カウンター）
            let mut chunk_nonce = nonce_bytes;
            let chunk_index = encrypted_chunks.len() as u64;
            chunk_nonce[4..12].copy_from_slice(&chunk_index.to_le_bytes());

            #[allow(deprecated)]
            let nonce = Nonce::from_slice(&chunk_nonce);
            let chunk_ciphertext = cipher
                .encrypt(nonce, &buffer[..bytes_read])
                .map_err(|e| BackupError::EncryptionError(format!("チャンク暗号化エラー: {e}")))?;

            // チャンクサイズと暗号化データを書き込み
            writer.write_all(&(chunk_ciphertext.len() as u32).to_le_bytes())?;
            writer.write_all(&chunk_ciphertext)?;

            encrypted_chunks.push(chunk_ciphertext);
        }

        Ok(EncryptedData {
            nonce: nonce_bytes,
            salt,
            ciphertext: encrypted_chunks.into_iter().flatten().collect(),
            original_size: total_size,
        })
    }

    /// ストリーミング復号化（大容量ファイル用）
    ///
    /// # Errors
    ///
    /// 以下の場合にエラーを返します:
    /// - ヘッダー情報（ナンス・ソルト）の読み取りエラー (`BackupError::IoError`)
    /// - チャンクサイズまたはチャンクデータの読み取りエラー (`BackupError::IoError`)
    /// - ファイル書き込みエラー (`BackupError::IoError`)
    /// - チャンク毎のAES-256-GCM復号化処理が失敗した場合 (`BackupError::EncryptionError`)
    ///   - 認証タグの検証に失敗した場合（データが改ざんされている可能性）
    ///   - 不正なマスターキーが使用された場合
    #[allow(deprecated)]
    pub fn decrypt_stream<R: Read, W: Write>(
        &self,
        mut reader: R,
        mut writer: W,
        master_key: &MasterKey,
    ) -> Result<u64> {
        // ヘッダー情報を読み取り
        let mut nonce_bytes = [0u8; 12];
        let mut salt = [0u8; 16];
        reader.read_exact(&mut nonce_bytes)?;
        reader.read_exact(&mut salt)?;

        let key = Key::<Aes256Gcm>::from_slice(master_key.as_bytes());
        let cipher = Aes256Gcm::new(key);

        let mut total_decrypted = 0u64;
        let mut chunk_index = 0u64;

        loop {
            // チャンクサイズを読み取り
            let mut chunk_size_bytes = [0u8; 4];
            match reader.read_exact(&mut chunk_size_bytes) {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(BackupError::IoError(e)),
            }

            let chunk_size = u32::from_le_bytes(chunk_size_bytes) as usize;
            let mut chunk_data = vec![0u8; chunk_size];
            reader.read_exact(&mut chunk_data)?;

            // チャンク毎のナンスを再生成（u64カウンター）
            let mut chunk_nonce = nonce_bytes;
            chunk_nonce[4..12].copy_from_slice(&chunk_index.to_le_bytes());

            #[allow(deprecated)]
            let nonce = Nonce::from_slice(&chunk_nonce);
            let plaintext = cipher
                .decrypt(nonce, chunk_data.as_ref())
                .map_err(|e| BackupError::EncryptionError(format!("チャンク復号化エラー: {e}")))?;

            writer.write_all(&plaintext)?;
            total_decrypted += plaintext.len() as u64;
            chunk_index += 1;
        }

        Ok(total_decrypted)
    }
}

impl Default for EncryptionEngine {
    fn default() -> Self {
        Self::new(EncryptionConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() {
        let engine = EncryptionEngine::default();
        let master_key = MasterKey::generate();
        let original_data = b"Hello, World! This is a test message.";
        let salt = [0u8; 16]; // テスト用のsalt

        let encrypted = engine.encrypt(original_data, &master_key, salt).unwrap();
        let decrypted = engine.decrypt(&encrypted, &master_key).unwrap();

        assert_eq!(original_data, decrypted.as_slice());
        assert_eq!(encrypted.original_size, original_data.len() as u64);
    }

    #[test]
    fn test_encrypted_data_serialization() {
        let engine = EncryptionEngine::default();
        let master_key = MasterKey::generate();
        let original_data = b"Test data for serialization";
        let salt = [1u8; 16]; // テスト用のsalt

        let encrypted = engine.encrypt(original_data, &master_key, salt).unwrap();
        let serialized = encrypted.to_bytes();
        let deserialized = EncryptedData::from_bytes(&serialized).unwrap();

        let decrypted = engine.decrypt(&deserialized, &master_key).unwrap();
        assert_eq!(original_data, decrypted.as_slice());
    }

    #[test]
    fn test_different_keys_fail() {
        let engine = EncryptionEngine::default();
        let master_key1 = MasterKey::generate();
        let master_key2 = MasterKey::generate();
        let original_data = b"Secret message";
        let salt = [2u8; 16]; // テスト用のsalt

        let encrypted = engine.encrypt(original_data, &master_key1, salt).unwrap();

        // 異なるキーでの復号化は失敗する
        assert!(engine.decrypt(&encrypted, &master_key2).is_err());
    }

    #[test]
    fn test_stream_encryption() {
        use std::io::Cursor;

        let engine = EncryptionEngine::default();
        let master_key = MasterKey::generate();
        let original_data = b"This is a longer message for stream testing. ".repeat(100);

        let reader = Cursor::new(&original_data);
        let mut encrypted_buffer = Vec::new();
        let encrypted_meta = engine
            .encrypt_stream(reader, &mut encrypted_buffer, &master_key)
            .unwrap();

        assert_eq!(encrypted_meta.original_size, original_data.len() as u64);

        let encrypted_reader = Cursor::new(&encrypted_buffer);
        let mut decrypted_buffer = Vec::new();
        let decrypted_size = engine
            .decrypt_stream(encrypted_reader, &mut decrypted_buffer, &master_key)
            .unwrap();

        assert_eq!(decrypted_size, original_data.len() as u64);
        assert_eq!(original_data, decrypted_buffer);
    }
}
