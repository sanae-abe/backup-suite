//! 監査ログモジュール
//!
//! HMAC-SHA256による改ざん防止機能を備えた監査ログシステムを提供します。
//!
//! # 主要機能
//!
//! - **イベント記録**: バックアップ、復元、削除などの重要操作をログ記録
//! - **改ざん防止**: HMAC-SHA256による整合性検証
//! - **JSON形式**: 構造化ログデータの保存と検索
//! - **自動ログローテーション**: サイズベースのログファイル管理
//!
//! # セキュリティ設計
//!
//! このモジュールは以下のセキュリティ原則に従っています：
//!
//! 1. **改ざん検知**: すべてのログエントリにHMACを付与
//! 2. **暗号学的安全性**: SHA256ハッシュ関数の使用
//! 3. **監査証跡**: 削除不可能なログ記録（append-only）
//! 4. **時系列保証**: タイムスタンプによる順序保証
//!
//! # 使用例
//!
//! ```rust,no_run
//! use backup_suite::security::audit::{AuditLog, AuditEvent, EventType};
//!
//! // 監査ログの初期化
//! let mut audit_log = AuditLog::new()?;
//!
//! // バックアップイベントの記録
//! let event = AuditEvent::backup_started("/path/to/backup", "admin");
//! audit_log.log(event)?;
//!
//! // ログの検証
//! let is_valid = audit_log.verify_all()?;
//! assert!(is_valid, "ログが改ざんされています");
//! # Ok::<(), anyhow::Error>(())
//! ```

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use zeroize::Zeroize;

/// 監査イベントの種類
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EventType {
    /// バックアップ開始
    BackupStarted,
    /// バックアップ完了
    BackupCompleted,
    /// バックアップ失敗
    BackupFailed,
    /// 復元開始
    RestoreStarted,
    /// 復元完了
    RestoreCompleted,
    /// 復元失敗
    RestoreFailed,
    /// クリーンアップ開始
    CleanupStarted,
    /// クリーンアップ完了
    CleanupCompleted,
    /// クリーンアップ失敗
    CleanupFailed,
    /// 設定変更
    ConfigurationChanged,
    /// セキュリティ警告
    SecurityWarning,
    /// 権限エラー
    PermissionDenied,
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::BackupStarted => write!(f, "BACKUP_STARTED"),
            EventType::BackupCompleted => write!(f, "BACKUP_COMPLETED"),
            EventType::BackupFailed => write!(f, "BACKUP_FAILED"),
            EventType::RestoreStarted => write!(f, "RESTORE_STARTED"),
            EventType::RestoreCompleted => write!(f, "RESTORE_COMPLETED"),
            EventType::RestoreFailed => write!(f, "RESTORE_FAILED"),
            EventType::CleanupStarted => write!(f, "CLEANUP_STARTED"),
            EventType::CleanupCompleted => write!(f, "CLEANUP_COMPLETED"),
            EventType::CleanupFailed => write!(f, "CLEANUP_FAILED"),
            EventType::ConfigurationChanged => write!(f, "CONFIGURATION_CHANGED"),
            EventType::SecurityWarning => write!(f, "SECURITY_WARNING"),
            EventType::PermissionDenied => write!(f, "PERMISSION_DENIED"),
        }
    }
}

/// 監査イベント
///
/// 監査ログの個別エントリを表します。
/// 各エントリにはHMAC-SHA256による署名が含まれ、改ざん検知が可能です。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// イベントのタイムスタンプ（UTC）
    pub timestamp: DateTime<Utc>,
    /// イベント種別
    pub event_type: EventType,
    /// ユーザー/プロセス識別子
    pub user: String,
    /// 対象パス（該当する場合）
    pub target: Option<String>,
    /// 追加のメタデータ
    pub metadata: Option<serde_json::Value>,
    /// HMAC-SHA256署名（hex文字列）
    pub hmac: String,
}

impl AuditEvent {
    /// バックアップ開始イベントを作成
    #[must_use]
    pub fn backup_started(target: impl Into<String>, user: impl Into<String>) -> Self {
        Self::new(
            EventType::BackupStarted,
            user.into(),
            Some(target.into()),
            None,
        )
    }

    /// バックアップ完了イベントを作成
    pub fn backup_completed(
        target: impl Into<String>,
        user: impl Into<String>,
        metadata: serde_json::Value,
    ) -> Self {
        Self::new(
            EventType::BackupCompleted,
            user.into(),
            Some(target.into()),
            Some(metadata),
        )
    }

    /// バックアップ失敗イベントを作成
    pub fn backup_failed(
        target: impl Into<String>,
        user: impl Into<String>,
        error: impl Into<String>,
    ) -> Self {
        let metadata = serde_json::json!({ "error": error.into() });
        Self::new(
            EventType::BackupFailed,
            user.into(),
            Some(target.into()),
            Some(metadata),
        )
    }

    /// 復元開始イベントを作成
    #[must_use]
    pub fn restore_started(target: impl Into<String>, user: impl Into<String>) -> Self {
        Self::new(
            EventType::RestoreStarted,
            user.into(),
            Some(target.into()),
            None,
        )
    }

    /// 復元完了イベントを作成
    pub fn restore_completed(
        target: impl Into<String>,
        user: impl Into<String>,
        metadata: serde_json::Value,
    ) -> Self {
        Self::new(
            EventType::RestoreCompleted,
            user.into(),
            Some(target.into()),
            Some(metadata),
        )
    }

    /// 復元失敗イベントを作成
    pub fn restore_failed(
        target: impl Into<String>,
        user: impl Into<String>,
        error: impl Into<String>,
    ) -> Self {
        let metadata = serde_json::json!({ "error": error.into() });
        Self::new(
            EventType::RestoreFailed,
            user.into(),
            Some(target.into()),
            Some(metadata),
        )
    }

    /// クリーンアップ開始イベントを作成
    #[must_use]
    pub fn cleanup_started(user: impl Into<String>, days: u32) -> Self {
        let metadata = serde_json::json!({ "days": days });
        Self::new(EventType::CleanupStarted, user.into(), None, Some(metadata))
    }

    /// クリーンアップ完了イベントを作成
    #[must_use]
    pub fn cleanup_completed(user: impl Into<String>, metadata: serde_json::Value) -> Self {
        Self::new(
            EventType::CleanupCompleted,
            user.into(),
            None,
            Some(metadata),
        )
    }

    /// クリーンアップ失敗イベントを作成
    #[must_use]
    pub fn cleanup_failed(user: impl Into<String>, error: impl Into<String>) -> Self {
        let metadata = serde_json::json!({ "error": error.into() });
        Self::new(EventType::CleanupFailed, user.into(), None, Some(metadata))
    }

    /// セキュリティ警告イベントを作成
    #[must_use]
    pub fn security_warning(message: impl Into<String>, user: impl Into<String>) -> Self {
        let metadata = serde_json::json!({ "warning": message.into() });
        Self::new(
            EventType::SecurityWarning,
            user.into(),
            None,
            Some(metadata),
        )
    }

    /// 権限拒否イベントを作成
    #[must_use]
    pub fn permission_denied(path: impl Into<String>, user: impl Into<String>) -> Self {
        Self::new(
            EventType::PermissionDenied,
            user.into(),
            Some(path.into()),
            None,
        )
    }

    /// 新しい監査イベントを作成（内部使用）
    fn new(
        event_type: EventType,
        user: String,
        target: Option<String>,
        metadata: Option<serde_json::Value>,
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            event_type,
            user,
            target,
            metadata,
            hmac: String::new(), // HMACは後でcompute_hmacで計算
        }
    }

    /// イベントのペイロード（HMAC計算用）を生成
    fn payload(&self) -> String {
        format!(
            "{}|{}|{}|{}|{}",
            self.timestamp.to_rfc3339(),
            self.event_type,
            self.user,
            self.target.as_deref().unwrap_or(""),
            self.metadata
                .as_ref()
                .map(std::string::ToString::to_string)
                .unwrap_or_default()
        )
    }

    /// HMAC-SHA256を計算
    #[must_use]
    pub fn compute_hmac(&self, secret: &[u8]) -> String {
        let payload = self.payload();
        let mut mac = hmac_sha256(secret, payload.as_bytes());
        let result = hex::encode(&mac);
        mac.zeroize(); // 機密データを消去
        result
    }

    /// HMACを検証
    #[must_use]
    pub fn verify_hmac(&self, secret: &[u8]) -> bool {
        let computed = self.compute_hmac(secret);
        // タイミング攻撃対策：定数時間比較
        constant_time_eq(self.hmac.as_bytes(), computed.as_bytes())
    }
}

/// 監査ログ管理
///
/// 監査イベントの記録、検証、検索機能を提供します。
///
/// # セキュリティ特性
///
/// - **HMAC-SHA256**: すべてのログエントリに署名
/// - **append-only**: ログは追記のみ（上書き・削除不可）
/// - **自動ローテーション**: 10MBを超えるとログファイルをローテーション
pub struct AuditLog {
    /// ログファイルのパス
    log_path: PathBuf,
    /// HMAC計算用の秘密鍵
    pub(crate) secret: Vec<u8>,
    /// 最大ログファイルサイズ（バイト）
    pub max_log_size: u64,
}

impl AuditLog {
    /// デフォルトのログパスを使用して監査ログを初期化
    ///
    /// # Errors
    ///
    /// * 設定ディレクトリが取得できない場合（`dirs::config_dir()`が`None`を返す）
    /// * 設定ディレクトリ（`~/.config/backup-suite`等）の作成に失敗した場合
    /// * 秘密鍵の読み込みまたは生成に失敗した場合
        pub fn new() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .context("設定ディレクトリが取得できません")?
            .join("backup-suite");
        std::fs::create_dir_all(&config_dir).context("設定ディレクトリの作成に失敗しました")?;

        let log_path = config_dir.join("audit.log");
        Self::with_path(log_path)
    }

    /// カスタムパスで監査ログを初期化
    ///
    /// # Errors
    ///
    /// * 秘密鍵ファイルの読み込みに失敗した場合
    /// * 秘密鍵の新規生成・保存に失敗した場合
    /// * 秘密鍵ファイルのパーミッション設定に失敗した場合（Unix系）
        pub fn with_path(log_path: PathBuf) -> Result<Self> {
        // 秘密鍵の生成または読み込み
        let secret = Self::load_or_generate_secret(&log_path)?;

        Ok(Self {
            log_path,
            secret,
            max_log_size: 10 * 1024 * 1024, // 10MB
        })
    }

    /// 秘密鍵を読み込みまたは生成
    fn load_or_generate_secret(log_path: &Path) -> Result<Vec<u8>> {
        let secret_path = log_path.with_extension("key");

        if secret_path.exists() {
            // 既存の秘密鍵を読み込み
            std::fs::read(&secret_path).context("秘密鍵の読み込みに失敗しました")
        } else {
            // 新しい秘密鍵を生成
            let secret = generate_random_bytes(32); // 256ビット
            std::fs::write(&secret_path, &secret).context("秘密鍵の保存に失敗しました")?;

            // セキュリティ強化：Unix系でファイル権限を600に設定
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = std::fs::metadata(&secret_path)?.permissions();
                perms.set_mode(0o600);
                std::fs::set_permissions(&secret_path, perms)?;
            }

            Ok(secret)
        }
    }

    /// 監査イベントをログに記録
    ///
    /// # Errors
    ///
    /// * ログファイルのメタデータ取得に失敗した場合（ローテーションチェック時）
    /// * ログローテーション（ファイル名変更）に失敗した場合
    /// * ログファイルのオープンに失敗した場合
    /// * イベントのJSON形式へのシリアライズに失敗した場合
    /// * ログファイルへの書き込みに失敗した場合
        pub fn log(&mut self, mut event: AuditEvent) -> Result<()> {
        // HMACを計算
        event.hmac = event.compute_hmac(&self.secret);

        // ログローテーションチェック
        if self.should_rotate()? {
            self.rotate_log()?;
        }

        // JSON形式でログファイルに追記
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
            .context("ログファイルのオープンに失敗しました")?;

        let json_line = serde_json::to_string(&event)?;
        writeln!(file, "{json_line}").context("ログの書き込みに失敗しました")?;

        Ok(())
    }

    /// ログローテーションが必要かチェック
    fn should_rotate(&self) -> Result<bool> {
        if !self.log_path.exists() {
            return Ok(false);
        }

        let metadata = std::fs::metadata(&self.log_path)?;
        Ok(metadata.len() > self.max_log_size)
    }

    /// ログファイルをローテーション
    fn rotate_log(&self) -> Result<()> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let rotated_path = self
            .log_path
            .with_file_name(format!("audit_{timestamp}.log"));

        std::fs::rename(&self.log_path, &rotated_path)
            .context("ログローテーションに失敗しました")?;

        Ok(())
    }

    /// すべてのログエントリを読み込み
    ///
    /// # Errors
    ///
    /// * ログファイルのオープンに失敗した場合
    /// * ログファイルの行読み込みに失敗した場合
    /// * ログエントリのJSON形式パースに失敗した場合（ログファイル破損時）
        pub fn read_all(&self) -> Result<Vec<AuditEvent>> {
        if !self.log_path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&self.log_path).context("ログファイルのオープンに失敗しました")?;
        let reader = BufReader::new(file);

        let mut events = Vec::new();
        for (line_num, line) in reader.lines().enumerate() {
            let line = line.context(format!("{}行目の読み込みに失敗しました", line_num + 1))?;
            if line.trim().is_empty() {
                continue;
            }

            let event: AuditEvent = serde_json::from_str(&line).context(format!(
                "{}行目のJSONパースに失敗しました: {}",
                line_num + 1,
                line
            ))?;
            events.push(event);
        }

        Ok(events)
    }

    /// すべてのログエントリのHMACを検証
    ///
    /// # Errors
    ///
    /// * ログファイルの読み込みに失敗した場合（`read_all()`のエラーを参照）
    /// * ログエントリのパースに失敗した場合
    ///
    /// # 戻り値
    ///
    /// * `Ok(true)` - すべてのログエントリのHMAC検証が成功
    /// * `Ok(false)` - 1つ以上のログエントリのHMAC検証が失敗（ログ改ざん検出）
        pub fn verify_all(&self) -> Result<bool> {
        let events = self.read_all()?;

        for (i, event) in events.iter().enumerate() {
            if !event.verify_hmac(&self.secret) {
                eprintln!("警告: {}行目のログエントリのHMAC検証に失敗しました", i + 1);
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// 特定の期間のログエントリを取得
    ///
    /// # Errors
    ///
    /// * ログファイルの読み込みに失敗した場合（`read_all()`のエラーを参照）
        pub fn get_events_since(&self, since: DateTime<Utc>) -> Result<Vec<AuditEvent>> {
        let all_events = self.read_all()?;
        Ok(all_events
            .into_iter()
            .filter(|e| e.timestamp >= since)
            .collect())
    }

    /// 特定の種類のイベントを取得
    ///
    /// # Errors
    ///
    /// * ログファイルの読み込みに失敗した場合（`read_all()`のエラーを参照）
        pub fn get_events_by_type(&self, event_type: &EventType) -> Result<Vec<AuditEvent>> {
        let all_events = self.read_all()?;
        Ok(all_events
            .into_iter()
            .filter(|e| &e.event_type == event_type)
            .collect())
    }

    /// 現在のユーザー名を取得（システム環境変数から）
    #[must_use]
    pub fn current_user() -> String {
        std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "unknown".to_string())
    }
}

impl Drop for AuditLog {
    fn drop(&mut self) {
        // 機密データを消去
        self.secret.zeroize();
    }
}

/// HMAC-SHA256の計算
fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    // RFC 2104に基づくHMAC実装
    const BLOCK_SIZE: usize = 64; // SHA256のブロックサイズ
    let mut key_padded = vec![0u8; BLOCK_SIZE];

    if key.len() > BLOCK_SIZE {
        // 鍵が長すぎる場合はハッシュ化
        let hash = Sha256::digest(key);
        key_padded[..hash.len()].copy_from_slice(&hash);
    } else {
        key_padded[..key.len()].copy_from_slice(key);
    }

    // ipad (0x36の繰り返し)
    let mut ipad = key_padded.clone();
    for byte in &mut ipad {
        *byte ^= 0x36;
    }

    // opad (0x5cの繰り返し)
    let mut opad = key_padded;
    for byte in &mut opad {
        *byte ^= 0x5c;
    }

    // HMAC = H(opad || H(ipad || message))
    let mut hasher = Sha256::new();
    hasher.update(&ipad);
    hasher.update(data);
    let inner_hash = hasher.finalize();

    let mut hasher = Sha256::new();
    hasher.update(&opad);
    hasher.update(inner_hash);
    hasher.finalize().to_vec()
}

/// 定数時間での文字列比較（タイミング攻撃対策）
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    result == 0
}

/// 暗号学的に安全な乱数バイト列を生成
fn generate_random_bytes(len: usize) -> Vec<u8> {
    use rand::RngCore;
    let mut bytes = vec![0u8; len];
    rand::rng().fill_bytes(&mut bytes);
    bytes
}

// hexエンコード用の簡易実装
mod hex {
    #[must_use]
    pub fn encode(data: &[u8]) -> String {
        data.iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_audit_event_creation() {
        let event = AuditEvent::backup_started("/path/to/backup", "testuser");
        assert_eq!(event.event_type, EventType::BackupStarted);
        assert_eq!(event.user, "testuser");
        assert_eq!(event.target, Some("/path/to/backup".to_string()));
    }

    #[test]
    fn test_hmac_computation_and_verification() {
        let secret = b"test_secret_key";
        let mut event = AuditEvent::backup_started("/path/to/backup", "testuser");
        event.hmac = event.compute_hmac(secret);

        assert!(event.verify_hmac(secret));
        assert!(!event.verify_hmac(b"wrong_secret"));
    }

    #[test]
    fn test_hmac_tampering_detection() {
        let secret = b"test_secret_key";
        let mut event = AuditEvent::backup_started("/path/to/backup", "testuser");
        event.hmac = event.compute_hmac(secret);

        // イベントを改ざん
        event.user = "attacker".to_string();

        // 改ざん検知
        assert!(!event.verify_hmac(secret));
    }

    #[test]
    fn test_audit_log_write_and_read() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let log_path = temp_dir.path().join("test_audit.log");

        let mut audit_log = AuditLog::with_path(log_path)?;

        // イベント記録
        audit_log.log(AuditEvent::backup_started("/path/1", "user1"))?;
        audit_log.log(AuditEvent::backup_completed(
            "/path/1",
            "user1",
            serde_json::json!({"files": 10}),
        ))?;

        // イベント読み込み
        let events = audit_log.read_all()?;
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event_type, EventType::BackupStarted);
        assert_eq!(events[1].event_type, EventType::BackupCompleted);

        Ok(())
    }

    #[test]
    fn test_audit_log_verification() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let log_path = temp_dir.path().join("test_audit.log");

        let mut audit_log = AuditLog::with_path(log_path.clone())?;

        // イベント記録
        audit_log.log(AuditEvent::backup_started("/path/1", "user1"))?;
        audit_log.log(AuditEvent::restore_completed(
            "/path/1",
            "user1",
            serde_json::json!({"files": 5}),
        ))?;

        // 検証成功
        assert!(audit_log.verify_all()?);

        // ログファイルを直接改ざん
        let mut content = std::fs::read_to_string(&log_path)?;
        content = content.replace("user1", "attacker");
        std::fs::write(&log_path, content)?;

        // 検証失敗（改ざん検知）
        let audit_log = AuditLog::with_path(log_path)?;
        assert!(!audit_log.verify_all()?);

        Ok(())
    }

    #[test]
    fn test_constant_time_eq() {
        assert!(constant_time_eq(b"hello", b"hello"));
        assert!(!constant_time_eq(b"hello", b"world"));
        assert!(!constant_time_eq(b"short", b"longer"));
    }

    #[test]
    fn test_hmac_sha256() {
        let key = b"secret";
        let data = b"message";
        let hmac1 = hmac_sha256(key, data);
        let hmac2 = hmac_sha256(key, data);

        // 同じ入力は同じHMACを生成
        assert_eq!(hmac1, hmac2);

        // 異なる鍵は異なるHMACを生成
        let hmac3 = hmac_sha256(b"different_secret", data);
        assert_ne!(hmac1, hmac3);
    }

    #[test]
    fn test_event_type_display() {
        assert_eq!(EventType::BackupStarted.to_string(), "BACKUP_STARTED");
        assert_eq!(EventType::RestoreCompleted.to_string(), "RESTORE_COMPLETED");
        assert_eq!(EventType::SecurityWarning.to_string(), "SECURITY_WARNING");
    }

    #[test]
    fn test_get_events_by_type() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let log_path = temp_dir.path().join("test_audit.log");

        let mut audit_log = AuditLog::with_path(log_path)?;

        // 異なる種類のイベントを記録
        audit_log.log(AuditEvent::backup_started("/path/1", "user1"))?;
        audit_log.log(AuditEvent::restore_started("/path/2", "user1"))?;
        audit_log.log(AuditEvent::backup_completed(
            "/path/1",
            "user1",
            serde_json::json!({}),
        ))?;

        // 特定種類のイベントのみ取得
        let backup_events = audit_log.get_events_by_type(&EventType::BackupStarted)?;
        assert_eq!(backup_events.len(), 1);
        assert_eq!(backup_events[0].target, Some("/path/1".to_string()));

        Ok(())
    }

    #[test]
    fn test_log_rotation() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let log_path = temp_dir.path().join("test_audit.log");

        let mut audit_log = AuditLog::with_path(log_path.clone())?;
        audit_log.max_log_size = 100; // テスト用に小さい値に設定

        // 多数のイベントを記録（ローテーションをトリガー）
        for i in 0..50 {
            audit_log.log(AuditEvent::backup_started(format!("/path/{i}"), "user1"))?;
        }

        // ローテーションされたファイルが存在することを確認
        let entries: Vec<_> = std::fs::read_dir(temp_dir.path())?
            .filter_map(std::result::Result::ok)
            .filter(|e| {
                e.file_name().to_string_lossy().starts_with("audit_")
                    && e.file_name().to_string_lossy().ends_with(".log")
            })
            .collect();

        assert!(
            !entries.is_empty(),
            "ログローテーションが実行されませんでした"
        );

        Ok(())
    }
}
