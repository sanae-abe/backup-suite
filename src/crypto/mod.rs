//! # 暗号化モジュール
//!
//! バックアップファイルの暗号化・復号化機能を提供します。
//! AES-256-GCM を使用した認証付き暗号化を実装します。

pub mod encryption;
pub mod key_management;

// 主要な型と関数を再エクスポート
pub use encryption::{EncryptedData, EncryptionConfig, EncryptionEngine};
pub use key_management::{KeyDerivation, KeyDerivationConfig, KeyManager, MasterKey};
