# backup-suite AI連携機能 - 詳細実装計画書

**作成日**: 2025-11-09
**バージョン**: 1.0.0
**ステータス**: 設計フェーズ

---

## 📋 目次

1. [エグゼクティブサマリー](#1-エグゼクティブサマリー)
2. [アーキテクチャ設計](#2-アーキテクチャ設計)
3. [Phase 1: 軽量ML機能](#3-phase-1-軽量ml機能)
4. [Phase 2: Ollama統合](#4-phase-2-ollama統合)
5. [セキュリティ考慮事項](#5-セキュリティ考慮事項)
6. [パフォーマンス目標](#6-パフォーマンス目標)
7. [実装ロードマップ](#7-実装ロードマップ)
8. [テスト戦略](#8-テスト戦略)
9. [依存関係](#9-依存関係)
10. [リスクと対策](#10-リスクと対策)

---

## 1. エグゼクティブサマリー

### 1.1 プロジェクト概要

backup-suite に AI/ML 連携機能を追加し、以下の価値を提供します：

- **インテリジェントバックアップ推奨**: ファイル重要度の自動判定、バックアップ対象の自動提案
- **異常検知・予測アラート**: バックアップサイズ異常検知、ディスク容量枯渇予測
- **自然言語バックアップ設定**: 対話的な設定インターフェース
- **AI駆動レポート・分析**: 統計の自然言語サマリー、改善提案生成

### 1.2 実装アプローチ

**ハイブリッドアプローチ（課金なし）**:

- **Phase 1（軽量ML）**: Rust native実装による統計的異常検知と推奨エンジン（常時利用可能）
- **Phase 2（Ollama統合）**: オプショナルな自然言語処理機能（Graceful degradation）

### 1.3 主要成果物

| 成果物 | 説明 | 担当Agent |
|--------|------|-----------|
| 型定義・エラー処理 | newtype pattern、thiserror統合 | rust-engineer |
| 異常検知エンジン | Z-score、線形回帰による統計分析 | rust-engineer |
| 推奨エンジン | ファイル重要度判定、除外提案 | rust-engineer |
| CLI UX | clap 4.x、国際化、Graceful degradation | cli-developer |
| Ollama統合 | HTTP client、自然言語パーサー | rust-engineer |
| セキュリティ監査 | データプライバシー、機密情報保護 | security-auditor |
| パフォーマンス最適化 | ベンチマーク、並列処理戦略 | performance-engineer |

---

## 2. アーキテクチャ設計

### 2.1 モジュール構造

```
src/
├── ai/                              # 新規AIモジュール
│   ├── mod.rs                       # AIモジュールのエクスポート
│   ├── anomaly/                     # Phase 1: 異常検知エンジン
│   │   ├── mod.rs
│   │   ├── detector.rs              # 統計的異常検知（Z-score）
│   │   ├── predictor.rs             # ディスク容量予測（線形回帰）
│   │   └── pattern.rs               # 失敗パターン分析
│   ├── recommendation/              # Phase 1: インテリジェント推奨
│   │   ├── mod.rs
│   │   ├── importance.rs            # ファイル重要度判定
│   │   ├── suggest.rs               # バックアップ対象の自動提案
│   │   └── exclude.rs               # 除外ファイルの自動検出
│   ├── llm/                         # Phase 2: Ollama統合（feature gated）
│   │   ├── mod.rs
│   │   ├── client.rs                # Ollama HTTPクライアント
│   │   ├── parser.rs                # 自然言語パーサー
│   │   └── report.rs                # AI駆動レポート生成
│   ├── types.rs                     # AI共通型定義（newtype pattern）
│   └── error.rs                     # AI固有のエラー型
```

### 2.2 型設計（Newtype Pattern活用）

```rust
// src/ai/types.rs

/// バックアップサイズ（バイト単位）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackupSize(u64);

/// 予測信頼度（0.0 - 1.0）
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PredictionConfidence(f64);

/// ファイル重要度（0 - 100）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileImportance(u8);

/// ディスク容量（バイト単位）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiskCapacity(u64);

/// 失敗率（0.0 - 1.0）
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FailureRate(f64);

/// 時系列データポイント
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub value: f64,
}
```

### 2.3 エラー型設計

```rust
// src/ai/error.rs

#[derive(Error, Debug)]
pub enum AiError {
    #[error("統計計算エラー: {0}")]
    StatisticsError(String),

    #[error("予測モデルエラー: {0}")]
    PredictionError(String),

    #[error("データ不足: 最低{required}件必要ですが、{actual}件しかありません")]
    InsufficientData { required: usize, actual: usize },

    #[error("無効なパラメータ: {0}")]
    InvalidParameter(String),

    #[cfg(feature = "llm")]
    #[error("LLM通信エラー: {0}")]
    LlmCommunicationError(#[from] reqwest::Error),

    #[cfg(feature = "llm")]
    #[error("LLMタイムアウト: {seconds}秒以内に応答がありませんでした")]
    LlmTimeout { seconds: u64 },

    #[cfg(feature = "llm")]
    #[error("Ollamaが未インストールです")]
    OllamaNotInstalled,

    #[error("I/Oエラー: {0}")]
    IoError(#[from] std::io::Error),

    #[error("AIエラー: {0}")]
    Other(#[from] anyhow::Error),
}

pub type AiResult<T> = std::result::Result<T, AiError>;
```

---

## 3. Phase 1: 軽量ML機能

### 3.1 異常検知エンジン

#### 3.1.1 実装概要

**目的**: バックアップ履歴データの統計的異常検知

**アルゴリズム**:
- Z-score 異常値検出（バックアップサイズ急変）
- 移動平均・標準偏差（トレンド分析）
- 線形回帰（ディスク容量予測）

**実装ファイル**: `src/ai/anomaly/detector.rs`

#### 3.1.2 主要コンポーネント

```rust
/// 異常検知器
pub struct AnomalyDetector {
    threshold: AnomalyThreshold,
}

/// 異常検知閾値
#[derive(Debug, Clone, Copy)]
pub struct AnomalyThreshold {
    pub z_score: f64,        // デフォルト: 3.0
    pub window_size: usize,  // デフォルト: 7日間
}

/// 異常検知結果
#[derive(Debug, Clone)]
pub struct AnomalyResult {
    pub is_anomaly: bool,
    pub z_score: f64,
    pub confidence: PredictionConfidence,
    pub description: String,
    pub recommended_action: Option<String>,
}
```

#### 3.1.3 使用クレート

```toml
statrs = "0.17"  # 統計関数（Z-score、回帰等）
ndarray = "0.16" # 数値計算（オプション）
```

#### 3.1.4 テスト戦略

```rust
// proptest による property-based testing
proptest! {
    #[test]
    fn anomaly_detection_is_deterministic(sizes in vec(1000u64..100000, 5..20)) {
        let detector = AnomalyDetector::default_detector();
        let histories = create_mock_histories(&sizes);
        let current = BackupSize::new(50000);

        let result1 = detector.detect_size_anomaly(&histories, current);
        let result2 = detector.detect_size_anomaly(&histories, current);

        assert_eq!(
            result1.as_ref().map(|r| r.is_anomaly),
            result2.as_ref().map(|r| r.is_anomaly)
        );
    }
}
```

### 3.2 インテリジェント推奨エンジン

#### 3.2.1 実装概要

**目的**: ファイル重要度の自動判定、バックアップ対象・除外候補の提案

**手法**:
- ルールベーススコアリング（拡張子、ディレクトリ名）
- 変更頻度分析（頻繁に変更されるファイルは重要度高）
- ファイルサイズ・アクセス時刻分析

**実装ファイル**: `src/ai/recommendation/importance.rs`

#### 3.2.2 主要コンポーネント

```rust
/// 重要度判定エンジン
pub struct ImportanceEvaluator {
    rules: Vec<ImportanceRule>,
}

/// ファイル重要度評価結果
pub struct FileImportanceResult {
    pub path: PathBuf,
    pub score: FileImportance,
    pub priority: Priority,
    pub category: String,
    pub reason: String,
}

/// 除外提案
pub struct ExcludeRecommendation {
    pub pattern: String,
    pub confidence: PredictionConfidence,
    pub size_reduction_gb: f64,
    pub reason: String,
}
```

#### 3.2.3 ルールベーススコアリング

```rust
// 高重要度ファイル（80-100点）
- ドキュメント: .docx, .pdf, .xlsx, .pptx
- ソースコード: .rs, .py, .js, .ts, .java, .go
- 設定ファイル: .toml, .yaml, .json, .ini

// 中重要度ファイル（40-79点）
- 画像ファイル: .jpg, .png, .gif
- データファイル: .csv, .db, .sqlite

// 低重要度ファイル（0-39点）
- 一時ファイル: .tmp, .temp, .cache
- ログファイル: .log

// ディレクトリベース判定
- Documents/, src/, config/ → 高重要度
- cache/, .cache/, node_modules/ → 除外推奨
```

### 3.3 CLI統合（Phase 1）

#### 3.3.1 新規コマンド

```bash
# 異常検知レポート
backup-suite ai detect [--days N] [--format table|json|detailed]

# ファイル重要度分析
backup-suite ai analyze <PATH> [--suggest-priority] [--detailed]

# 除外ファイル提案
backup-suite ai suggest-exclude <PATH> [--apply] [--confidence 0.8]

# AI駆動の自動設定
backup-suite ai auto-configure <PATH>... [--dry-run] [--interactive]
```

#### 3.3.2 出力フォーマット例

**異常検知レポート（テーブル形式）**:
```
🤖 AI異常検知レポート（過去7日間）

┌────┬──────────────────────────┬────────────┬──────────┬─────────────────────────┐
│ No │ 検出日時                   │ 異常種別     │ 信頼度    │ 説明                      │
├────┼──────────────────────────┼────────────┼──────────┼─────────────────────────┤
│ 1  │ 2025-11-09 03:15         │ サイズ急増   │ 95.3%    │ ファイルサイズが通常の3倍  │
└────┴──────────────────────────┴────────────┴──────────┴─────────────────────────┘

📊 サマリー: 1件の異常を検出
💡 推奨アクション: ~/Downloads の一時ファイルを除外設定に追加
```

**ファイル重要度分析（テーブル形式）**:
```
🤖 AIファイル重要度分析: ~/Documents/project

┌─────────────────────────────────┬──────────────┬──────────┬─────────────────────┐
│ ファイル/ディレクトリ             │ 重要度スコア   │ 提案優先度 │ 理由                  │
├─────────────────────────────────┼──────────────┼──────────┼─────────────────────┤
│ src/                            │ ████████ 95  │ 高        │ ソースコード（頻繁更新）│
│ node_modules/                   │ ██░░░░░░ 15  │ 除外推奨  │ 再生成可能な依存関係    │
└─────────────────────────────────┴──────────────┴──────────┴─────────────────────┘
```

---

## 4. Phase 2: Ollama統合

### 4.1 Ollama クライアント

#### 4.1.1 実装概要

**目的**: Ollama HTTP API呼び出し、Graceful degradation

**実装ファイル**: `src/ai/llm/ollama_client.rs`

#### 4.1.2 主要コンポーネント

```rust
pub struct OllamaClient {
    base_url: String,  // デフォルト: http://localhost:11434
    model: String,     // デフォルト: llama3.2:3b
    timeout: Duration,
}

impl OllamaClient {
    /// Ollama が利用可能か確認
    pub async fn is_available(&self) -> bool;

    /// プロンプトを Ollama に送信
    pub async fn generate(&self, prompt: &str) -> Result<String>;

    /// 自然言語からバックアップ設定を生成
    pub async fn parse_backup_request(&self, user_input: &str) -> Result<BackupConfig>;

    /// バックアップレポートの自然言語サマリー生成
    pub async fn generate_report_summary(&self, stats: &BackupStats) -> Result<String>;
}
```

#### 4.1.3 Graceful Degradation

```rust
pub fn check_ollama_status() -> Result<OllamaStatus> {
    // インストール確認
    let installed = Command::new("which")
        .arg("ollama")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !installed {
        return Ok(OllamaStatus {
            installed: false,
            running: false,
            version: None,
        });
    }

    // 起動確認
    let running = Command::new("curl")
        .args(&["-s", "http://localhost:11434/api/tags"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    Ok(OllamaStatus { installed: true, running, version })
}
```

**Ollama未インストール時の動作**:
- Phase 1の軽量ML機能のみ提供
- インストールガイド表示
- 代替機能の案内

### 4.2 自然言語バックアップ設定

#### 4.2.1 実装概要

**目的**: 自然言語入力からバックアップ設定を自動生成

**実装ファイル**: `src/ai/llm/parser.rs`

#### 4.2.2 プロンプトエンジニアリング

```rust
let prompt = format!(
    r#"以下のユーザーリクエストをバックアップ設定JSONに変換してください。

ユーザーリクエスト: {}

出力形式（JSON）：
{{
    "paths": ["パス1", "パス2"],
    "priority": "high" | "medium" | "low",
    "schedule": "daily" | "weekly" | "monthly",
    "time": "HH:MM",
    "compression": "zstd" | "gzip" | "none",
    "encrypt": true | false,
    "category": "カテゴリ名"
}}

JSON のみを出力してください（説明不要）。
"#,
    user_input
);
```

### 4.3 AI駆動レポート生成

#### 4.3.1 実装概要

**目的**: バックアップ統計の自然言語サマリー生成、改善提案

**実装ファイル**: `src/ai/llm/report.rs`

#### 4.3.2 レポートフォーマット

```
🤖 AI駆動バックアップレポート（過去30日間）

📊 サマリー:
  過去30日間で45回のバックアップを実行し、総容量は152GBです。
  バックアップ成功率は97.8%で、平均処理時間は12分です。

💡 改善提案:
  1. ~/Downloads の一時ファイル（3.2GB）を除外設定に追加すると、
     バックアップ時間を15%短縮できます。

  2. src/ ディレクトリの増分バックアップを有効化すると、
     ストレージ使用量を40%削減できます。

⚠️ リスク評価:
  現在のディスク使用率は82%です。今のペースでは約45日後に
  容量不足になる可能性があります。古いバックアップの削除を検討してください。
```

### 4.4 CLI統合（Phase 2）

```bash
# 自然言語でバックアップ設定
backup-suite ai setup "毎日午前2時に重要なプロジェクトをバックアップ"

# AI駆動レポート生成
backup-suite ai report --days 30 --format markdown

# インタラクティブな設定アシスタント
backup-suite ai assistant
```

---

## 5. セキュリティ考慮事項

### 5.1 データプライバシー

#### 5.1.1 機密情報の保護

**原則**:
- パスワード、暗号鍵、個人情報は **絶対にLLMに送信しない**
- バックアップ統計データは匿名化・集約化して送信

**実装**:
```rust
impl LlmClient {
    fn contains_sensitive_info(&self, input: &str) -> bool {
        let sensitive_patterns = [
            "password", "パスワード", "secret", "秘密鍵",
            "api_key", "token", "credential"
        ];

        sensitive_patterns.iter().any(|p| {
            input.to_lowercase().contains(p)
        })
    }

    pub async fn parse_natural_language(&self, input: &str) -> AiResult<BackupConfig> {
        // 機密情報チェック
        if self.contains_sensitive_info(input) {
            return Err(AiError::InvalidParameter(
                "機密情報を含む入力は受け付けられません".to_string()
            ));
        }

        // Ollamaへの送信処理...
    }
}
```

#### 5.1.2 ファイルパス情報の最小化

**対策**:
- ファイルパスの送信時は、ユーザー名やホームディレクトリを `~` に置換
- 機密性の高いディレクトリ名（`.ssh/`, `.gnupg/` 等）は送信前にマスク

**実装**:
```rust
fn sanitize_path_for_llm(path: &Path) -> String {
    let path_str = path.to_string_lossy();

    // ユーザー名を ~ に置換
    let sanitized = path_str.replace(&dirs::home_dir().unwrap().to_string_lossy(), "~");

    // 機密ディレクトリをマスク
    let sensitive_dirs = [".ssh", ".gnupg", ".aws", ".kube"];
    let mut result = sanitized;
    for dir in sensitive_dirs {
        result = result.replace(dir, "[REDACTED]");
    }

    result
}
```

### 5.2 Ollama通信セキュリティ

#### 5.2.1 通信の安全性

**現状のリスク**:
- localhost HTTP 通信（暗号化なし）
- 盗聴リスクは低い（ローカル通信のみ）

**対策**:
- Ollama は localhost:11434 でのみ通信（外部通信禁止）
- タイムアウト設定（5秒）でDoS防止

#### 5.2.2 プロンプトインジェクション対策

**対策**:
- ユーザー入力のサニタイゼーション
- プロンプトテンプレートの厳格化
- 出力のJSON検証

**実装**:
```rust
pub async fn parse_backup_request(&self, user_input: &str) -> AiResult<BackupConfig> {
    // 入力長制限（1000文字まで）
    if user_input.len() > 1000 {
        return Err(AiError::InvalidParameter("入力が長すぎます".to_string()));
    }

    // 危険な文字列パターンの検出
    let dangerous_patterns = ["<script>", "DROP TABLE", "'; --"];
    if dangerous_patterns.iter().any(|p| user_input.contains(p)) {
        return Err(AiError::InvalidParameter("不正な入力です".to_string()));
    }

    let response = self.generate(&prompt).await?;

    // JSON検証
    let config: BackupConfig = serde_json::from_str(&response)
        .map_err(|e| AiError::LlmParseError(format!("JSON解析失敗: {}", e)))?;

    Ok(config)
}
```

### 5.3 ファイルシステム走査

#### 5.3.1 パストラバーサル対策

**対策**:
- 既存の `src/security/path.rs` の `canonicalize` 使用
- シンボリックリンク攻撃対策

**実装**:
```rust
use crate::security::path::validate_path;

pub fn analyze_importance(&self, path: &Path) -> AiResult<FileImportance> {
    // パストラバーサル対策
    validate_path(path)?;

    // 分析処理...
}
```

### 5.4 セキュリティチェックリスト

- [ ] 機密情報（パスワード、暗号鍵）はLLMに送信しない
- [ ] Ollama API呼び出しには厳密なタイムアウト設定（5秒）
- [ ] エラーメッセージからファイルパスの機密情報を除外
- [ ] 統計計算のオーバーフロー対策（checked arithmetic）
- [ ] AI機能の無効化でも基本機能は正常動作（Feature gate）
- [ ] プロンプトインジェクション対策（入力検証）
- [ ] パストラバーサル対策（既存のセキュリティ機能活用）

---

## 6. パフォーマンス目標

### 6.1 Phase 1（軽量ML）

| 操作 | データ量 | 目標時間 | 備考 |
|------|---------|---------|------|
| 異常検知 | 100件履歴 | < 5ms | 統計計算のみ |
| 傾向分析 | 1000件履歴 | < 50ms | 移動平均計算 |
| 重要度評価 | 1ファイル | < 100μs | ルールマッチング |
| ファイル重要度分析 | 10,000ファイル | < 10秒 | walkdir走査 |

### 6.2 Phase 2（Ollama統合）

| 操作 | 処理内容 | 目標時間 | 備考 |
|------|---------|---------|------|
| Ollama API呼び出し | 1リクエスト | < 5秒 | タイムアウト設定 |
| 自然言語設定生成 | 1設定 | < 5秒 | LLM推論時間 |
| AIレポート生成 | 30日分統計 | < 10秒 | LLM推論時間 |

### 6.3 最適化戦略

#### 6.3.1 並列処理（rayon活用）

```rust
use rayon::prelude::*;

pub fn analyze_directory(&self, base_path: &Path) -> AiResult<Vec<FileImportanceResult>> {
    let entries: Vec<_> = WalkDir::new(base_path)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
        .collect();

    // 並列で重要度評価
    let results: Vec<_> = entries
        .par_iter()
        .map(|entry| self.evaluate_importance(entry.path()))
        .collect();

    Ok(results)
}
```

#### 6.3.2 メモリ効率（ストリーミング処理）

```rust
// 大量の履歴データを効率的に処理
pub fn analyze_trend_streaming(&self, histories: impl Iterator<Item = BackupHistory>)
    -> AiResult<Vec<TimeSeriesPoint>>
{
    let mut moving_averages = Vec::new();
    let mut window = VecDeque::with_capacity(self.threshold.window_size);

    for history in histories {
        window.push_back(history.total_bytes as f64);

        if window.len() == self.threshold.window_size {
            let avg = window.iter().sum::<f64>() / self.threshold.window_size as f64;
            moving_averages.push(TimeSeriesPoint::new(history.timestamp, avg));
            window.pop_front();
        }
    }

    Ok(moving_averages)
}
```

#### 6.3.3 キャッシング戦略

```rust
use std::collections::HashMap;
use std::sync::Mutex;

pub struct ImportanceEvaluator {
    rules: Vec<ImportanceRule>,
    cache: Mutex<HashMap<PathBuf, FileImportance>>,
}

impl ImportanceEvaluator {
    pub fn evaluate_cached(&self, path: &Path) -> AiResult<FileImportance> {
        // キャッシュヒット確認
        {
            let cache = self.cache.lock().unwrap();
            if let Some(&importance) = cache.get(path) {
                return Ok(importance);
            }
        }

        // 評価実行
        let importance = self.evaluate(path)?;

        // キャッシュ更新
        {
            let mut cache = self.cache.lock().unwrap();
            cache.insert(path.to_path_buf(), importance);
        }

        Ok(importance)
    }
}
```

### 6.4 ベンチマーク設計

```rust
// benches/ai_benchmark.rs

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

fn anomaly_detection_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("anomaly_detection");

    for data_size in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(data_size),
            data_size,
            |b, &size| {
                let detector = AnomalyDetector::default_detector();
                let histories = create_mock_histories_for_bench(size);
                let current = BackupSize::new(50000);

                b.iter(|| {
                    detector.detect_size_anomaly(black_box(&histories), black_box(current))
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, anomaly_detection_benchmark);
criterion_main!(benches);
```

---

## 7. 実装ロードマップ

### 7.1 Phase 1（軽量ML）- 2週間

#### Week 1: 基盤整備

**Day 1-2**: 型定義・エラー処理
- [ ] `src/ai/types.rs` 実装
- [ ] `src/ai/error.rs` 実装
- [ ] 単体テスト（proptest活用）

**Day 3-5**: 異常検知エンジン
- [ ] `src/ai/anomaly/detector.rs` 実装
- [ ] `src/ai/anomaly/predictor.rs` 実装
- [ ] 統合テスト

#### Week 2: 推奨エンジン・CLI統合

**Day 6-7**: 推奨エンジン
- [ ] `src/ai/recommendation/importance.rs` 実装
- [ ] `src/ai/recommendation/exclude.rs` 実装

**Day 8-10**: CLI統合
- [ ] `src/main.rs` に `ai` サブコマンド追加
- [ ] `src/i18n.rs` にメッセージキー追加
- [ ] 出力フォーマット実装（テーブル・JSON）

**Day 11-14**: テスト・ドキュメント
- [ ] 統合テスト完了
- [ ] ベンチマーク作成
- [ ] README更新

### 7.2 Phase 2（Ollama統合）- 1週間

#### Week 3: Ollama統合

**Day 15-16**: Ollama クライアント
- [ ] `src/ai/llm/client.rs` 実装
- [ ] Graceful degradation実装

**Day 17-18**: 自然言語処理
- [ ] `src/ai/llm/parser.rs` 実装
- [ ] `src/ai/llm/report.rs` 実装

**Day 19-21**: CLI統合・テスト
- [ ] `ai setup/report/assistant` コマンド実装
- [ ] 統合テスト
- [ ] ドキュメント更新

### 7.3 リリース準備

**Day 22-24**:
- [ ] セキュリティ監査
- [ ] パフォーマンスベンチマーク
- [ ] CI/CD パイプライン更新
- [ ] リリースノート作成

---

## 8. テスト戦略

### 8.1 単体テスト

#### 8.1.1 proptest による Property-based Testing

```rust
// tests/ai_property_tests.rs

proptest! {
    #[test]
    fn backup_size_always_non_negative(bytes in 0u64..=u64::MAX) {
        let size = BackupSize::new(bytes);
        assert!(size.get() >= 0);
    }

    #[test]
    fn confidence_always_in_range(confidence in 0.0f64..=1.0) {
        let conf = PredictionConfidence::new(confidence);
        assert!((0.0..=1.0).contains(&conf.get()));
    }

    #[test]
    fn anomaly_detection_is_deterministic(
        sizes in prop::collection::vec(1000u64..100000, 5..20)
    ) {
        let detector = AnomalyDetector::default_detector();
        let histories = create_mock_histories(&sizes);
        let current = BackupSize::new(50000);

        let result1 = detector.detect_size_anomaly(&histories, current);
        let result2 = detector.detect_size_anomaly(&histories, current);

        assert_eq!(
            result1.as_ref().map(|r| r.is_anomaly),
            result2.as_ref().map(|r| r.is_anomaly)
        );
    }
}
```

### 8.2 統合テスト

```rust
// tests/integration_tests/ai_integration.rs

#[test]
fn test_ai_detect_command_integration() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    // モック履歴データ作成
    create_mock_backup_histories(&temp_dir, 10);

    // コマンド実行
    let output = Command::new("backup-suite")
        .arg("ai")
        .arg("detect")
        .arg("--format")
        .arg("json")
        .env("BACKUP_SUITE_CONFIG", &config_path)
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    // JSON出力検証
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(result["anomalies"].is_array());
}
```

### 8.3 テストカバレッジ目標

- **全体カバレッジ**: 95%以上
- **クリティカルパス**: 100%
- **エラーハンドリング**: 100%

---

## 9. 依存関係

### 9.1 Cargo.toml 更新

```toml
[dependencies]
# 既存の依存関係...
anyhow = "1.0.100"
chrono = { version = "0.4.42", features = ["serde"] }
clap = { version = "4.5", features = ["derive", "cargo"] }
serde = { version = "1.0.228", features = ["derive"] }
thiserror = "1.0"

# Phase 1: 軽量ML（統計的異常検知）
statrs = "0.17"
# ndarray は statrs に含まれるため不要

[features]
default = ["ai"]  # デフォルトでAI機能を有効化

# Phase 1: 軽量ML機能
ai = ["statrs"]

# Phase 2: Ollama LLM統合
llm = ["ai", "reqwest", "tokio"]

[dependencies.reqwest]
version = "0.12"
optional = true
features = ["json"]

[dependencies.tokio]
version = "1.0"
optional = true
features = ["full"]
```

### 9.2 MSRV互換性

- **現在のMSRV**: Rust 1.82.0
- **新規依存関係の確認**:
  - `statrs 0.17`: Rust 1.70+ 対応 ✅
  - `reqwest 0.12`: Rust 1.70+ 対応 ✅
  - `tokio 1.0`: Rust 1.70+ 対応 ✅

---

## 10. リスクと対策

### 10.1 技術的リスク

| リスク | 影響 | 確率 | 対策 |
|--------|------|------|------|
| Ollama APIの変更 | 高 | 中 | Feature gateで分離、バージョン固定 |
| 統計計算の精度不足 | 中 | 低 | 閾値調整可能に設計、ユーザーフィードバック収集 |
| パフォーマンス劣化 | 中 | 低 | ベンチマーク監視、並列処理活用 |
| メモリ使用量増加 | 低 | 低 | ストリーミング処理、キャッシュ制限 |

### 10.2 セキュリティリスク

| リスク | 影響 | 確率 | 対策 |
|--------|------|------|------|
| 機密情報のLLM送信 | 高 | 低 | 入力検証、機密パターン検出 |
| プロンプトインジェクション | 中 | 中 | 入力サニタイゼーション、出力検証 |
| パストラバーサル | 高 | 低 | 既存セキュリティ機能活用 |
| DoS攻撃 | 低 | 低 | タイムアウト設定、レート制限 |

### 10.3 運用リスク

| リスク | 影響 | 確率 | 対策 |
|--------|------|------|------|
| Ollama未インストール | 低 | 高 | Graceful degradation、インストールガイド |
| AI機能の誤動作 | 中 | 低 | ドライランモード、ユーザー確認 |
| ユーザーの混乱 | 低 | 中 | 詳細なドキュメント、対話的UI |

---

## 付録

### A. 参考資料

- [rust-engineer レポート](./RUST_ENGINEER_REPORT.md)
- [cli-developer レポート](./CLI_DEVELOPER_REPORT.md)
- [statrs Documentation](https://docs.rs/statrs/latest/statrs/)
- [Ollama API Docs](https://github.com/ollama/ollama/blob/main/docs/api.md)

### B. 実装チェックリスト

#### Phase 1

- [ ] 型定義（newtype pattern）
- [ ] エラー型（thiserror）
- [ ] 異常検知エンジン
- [ ] 推奨エンジン
- [ ] CLI統合
- [ ] テスト（proptest）
- [ ] ベンチマーク
- [ ] ドキュメント

#### Phase 2

- [ ] Ollama クライアント
- [ ] Graceful degradation
- [ ] 自然言語パーサー
- [ ] AIレポート生成
- [ ] CLI統合
- [ ] テスト
- [ ] ドキュメント

#### リリース準備

- [ ] セキュリティ監査
- [ ] パフォーマンスベンチマーク
- [ ] CI/CD更新
- [ ] README更新
- [ ] CHANGELOG更新
- [ ] リリースノート作成

---

**最終更新**: 2025-11-09
**次回レビュー**: 2025-11-16
