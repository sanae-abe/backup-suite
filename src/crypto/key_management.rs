//! # キー管理システム
//!
//! パスワードからの安全な鍵導出とマスターキー管理を提供します。

use crate::error::{BackupError, Result};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand::RngCore;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// マスターキー（32バイト）
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct MasterKey {
    key: [u8; 32],
}

impl MasterKey {
    /// 新しいマスターキーを生成
    pub fn generate() -> Self {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        Self { key }
    }

    /// バイト配列からマスターキーを作成
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self { key: bytes }
    }

    /// キーのバイト配列を取得
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.key
    }
}

/// キー導出設定
#[derive(Debug, Clone)]
pub struct KeyDerivationConfig {
    /// メモリ使用量（KB）
    pub memory_cost: u32,
    /// 反復回数
    pub time_cost: u32,
    /// 並列度
    pub parallelism: u32,
}

impl Default for KeyDerivationConfig {
    fn default() -> Self {
        Self {
            memory_cost: 131072, // 128MB（OWASP推奨）
            time_cost: 4,        // 4回反復（OWASP推奨）
            parallelism: 2,      // 並列度2（セキュリティと性能のバランス）
        }
    }
}

/// キー導出エンジン
pub struct KeyDerivation {
    config: KeyDerivationConfig,
}

impl KeyDerivation {
    /// 新しいキー導出エンジンを作成
    pub fn new(config: KeyDerivationConfig) -> Self {
        Self { config }
    }

    /// パスワードからマスターキーを導出
    ///
    /// # Errors
    ///
    /// 以下の場合にエラーを返します:
    /// - Argon2パラメータが無効な場合 (`BackupError::EncryptionError`)
    /// - ソルトのBase64エンコードに失敗した場合 (`BackupError::EncryptionError`)
    /// - パスワードハッシュ生成に失敗した場合 (`BackupError::EncryptionError`)
    /// - ハッシュの生成に失敗した場合 (`BackupError::EncryptionError`)
    /// - 生成されたキーの長さが32バイトでない場合 (`BackupError::EncryptionError`)
    pub fn derive_key(&self, password: &str, salt: &[u8]) -> Result<MasterKey> {
        let argon2 = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            argon2::Params::new(
                self.config.memory_cost,
                self.config.time_cost,
                self.config.parallelism,
                Some(32),
            )
            .map_err(|e| BackupError::EncryptionError(format!("Argon2パラメータエラー: {e}")))?,
        );

        let salt_string = SaltString::encode_b64(salt)
            .map_err(|e| BackupError::EncryptionError(format!("Salt エンコードエラー: {e}")))?;

        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt_string)
            .map_err(|e| {
                BackupError::EncryptionError(format!("パスワードハッシュエラー: {e}"))
            })?;

        let hash = password_hash
            .hash
            .ok_or_else(|| BackupError::EncryptionError("ハッシュ生成に失敗".to_string()))?;
        let hash_bytes = hash.as_bytes();

        if hash_bytes.len() != 32 {
            return Err(BackupError::EncryptionError("無効なキー長".to_string()));
        }

        let mut key = [0u8; 32];
        key.copy_from_slice(hash_bytes);
        Ok(MasterKey::from_bytes(key))
    }

    /// ランダムなソルトを生成
    pub fn generate_salt() -> [u8; 16] {
        let mut salt = [0u8; 16];
        OsRng.fill_bytes(&mut salt);
        salt
    }

    /// パスワードを検証
    ///
    /// # Errors
    ///
    /// 以下の場合にエラーを返します:
    /// - ハッシュ文字列のパースに失敗した場合 (`BackupError::EncryptionError`)
    ///   - 無効なPHC文字列形式の場合
    ///   - 破損したハッシュデータの場合
    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| BackupError::EncryptionError(format!("ハッシュ解析エラー: {e}")))?;

        let argon2 = Argon2::default();
        Ok(argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
}

impl Default for KeyDerivation {
    fn default() -> Self {
        Self::new(KeyDerivationConfig::default())
    }
}

/// キーマネージャー
#[derive(Default)]
pub struct KeyManager {
    derivation: KeyDerivation,
}

impl KeyManager {
    /// 新しいキーマネージャーを作成
    pub fn new(config: KeyDerivationConfig) -> Self {
        Self {
            derivation: KeyDerivation::new(config),
        }
    }

    /// パスワードからマスターキーを生成（新しいソルト付き）
    ///
    /// # Errors
    ///
    /// 以下の場合にエラーを返します:
    /// - `derive_key` 関数内で発生するエラー（詳細は `KeyDerivation::derive_key` を参照）
    ///   - Argon2パラメータエラー
    ///   - パスワードハッシュ生成エラー
    ///   - 無効なキー長エラー
    pub fn create_master_key(&self, password: &str) -> Result<(MasterKey, [u8; 16])> {
        let salt = KeyDerivation::generate_salt();
        let key = self.derivation.derive_key(password, &salt)?;
        Ok((key, salt))
    }

    /// 既存のソルトでマスターキーを復元
    ///
    /// # Errors
    ///
    /// 以下の場合にエラーを返します:
    /// - `derive_key` 関数内で発生するエラー（詳細は `KeyDerivation::derive_key` を参照）
    ///   - Argon2パラメータエラー
    ///   - パスワードハッシュ生成エラー
    ///   - 無効なキー長エラー
    pub fn restore_master_key(&self, password: &str, salt: &[u8]) -> Result<MasterKey> {
        self.derivation.derive_key(password, salt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_master_key_generation() {
        let key1 = MasterKey::generate();
        let key2 = MasterKey::generate();

        // キーは異なる値になる
        assert_ne!(key1.as_bytes(), key2.as_bytes());

        // キー長は32バイト
        assert_eq!(key1.as_bytes().len(), 32);
    }

    #[test]
    fn test_key_derivation() {
        let kd = KeyDerivation::default();
        let password = "test_password_123";
        let salt = KeyDerivation::generate_salt();

        let key1 = kd.derive_key(password, &salt).unwrap();
        let key2 = kd.derive_key(password, &salt).unwrap();

        // 同じパスワード・ソルトからは同じキーが生成される
        assert_eq!(key1.as_bytes(), key2.as_bytes());

        // 異なるソルトからは異なるキーが生成される
        let salt2 = KeyDerivation::generate_salt();
        let key3 = kd.derive_key(password, &salt2).unwrap();
        assert_ne!(key1.as_bytes(), key3.as_bytes());
    }

    #[test]
    fn test_key_manager() {
        let km = KeyManager::default();
        let password = "secure_password_456";

        let (key1, salt) = km.create_master_key(password).unwrap();
        let key2 = km.restore_master_key(password, &salt).unwrap();

        // 作成したキーと復元したキーは同じ
        assert_eq!(key1.as_bytes(), key2.as_bytes());
    }
}
