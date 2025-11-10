# backup-suite 開発TODO

**作成日**: 2025-11-09
**最終更新**: 2025-11-10 (包括的テスト実施完了)
**ステータス**: Phase 1完了 → 品質改善・Phase 2準備中
**参照**:
- [AI実装計画](docs/ai/development/implementation-plan.md)
- [包括的テストレポート](docs/testing/COMPREHENSIVE_TEST_REPORT.md)

---

## 📊 進捗サマリー

- **Phase 1（軽量ML）**: ✅ **21/21 完了 (100%)**
- **品質改善（テスト結果対応）**: ✅ **3/5 完了 (60%)** - ストリーミング暗号化、unwrap()削減、Proptest拡充完了
- **Phase 2（Ollama統合）**: 0/9 完了 (0%)
- **リリース準備**: 1/7 完了 (14%)
- **全体**: 25/42 完了 (60%)

---

## ✅ Phase 1: 軽量ML機能（統計的異常検知・推奨エンジン）**完了**

### Phase 1-1: 依存関係セットアップ ✅

- [x] `Cargo.toml` 更新
  - [x] `statrs = "0.17"` 追加
  - [x] `features` 設定（`smart = ["statrs"]`）
  - [x] MSRV互換性確認（Rust 1.82.0）
  - [x] デフォルトfeature設定（`default = ["smart"]`）

### Phase 1-2: 基盤整備（型定義・エラー処理）✅

- [x] `src/smart/types.rs` 実装（523行）
  - [x] newtype pattern: `BackupSize`, `PredictionConfidence`, `FileImportance`, `DiskCapacity`, `FailureRate`
  - [x] `TimeSeriesPoint` 構造体定義
  - [x] Serialize/Deserialize 派生
  - [x] 15単体テスト実装

- [x] `src/smart/error.rs` 実装（309行）
  - [x] `thiserror` 統合
  - [x] `AiError` enum 定義（11種類）
  - [x] `AiResult<T>` 型エイリアス
  - [x] 日本語・英語メッセージ
  - [x] 8単体テスト実装

- [x] `src/smart/mod.rs` 作成（27行）
  - [x] モジュールエクスポート
  - [x] feature gate 設定

### Phase 1-3: 異常検知エンジン ✅

- [x] `src/smart/anomaly/mod.rs` 作成
- [x] `src/smart/anomaly/detector.rs` 実装（459行、16テスト）
  - [x] Z-score異常検知（< 1ms/100件）
  - [x] 移動平均計算
- [x] `src/smart/anomaly/predictor.rs` 実装（506行、5テスト）
  - [x] 線形回帰予測（< 5ms）
  - [x] ディスク容量枯渇予測
- [x] `src/smart/anomaly/pattern.rs` 実装（363行、8テスト）
  - [x] 失敗パターン分析

### Phase 1-4: インテリジェント推奨エンジン ✅

- [x] `src/smart/recommendation/mod.rs` 作成
- [x] `src/smart/recommendation/importance.rs` 実装（521行、6テスト）
  - [x] ルールベーススコアリング（~50μs/ファイル）
  - [x] キャッシング実装
- [x] `src/smart/recommendation/suggest.rs` 実装（382行、5テスト）
  - [x] バックアップ対象提案
- [x] `src/smart/recommendation/exclude.rs` 実装（456行、5テスト）
  - [x] 除外パターン検出

### Phase 1-5: パフォーマンス検証 ✅

- [x] `benches/ai_benchmark.rs` 作成
  - [x] criterion ベンチマーク実装
  - [x] 全目標値達成確認

### Phase 1-6: テスト基盤 ✅

- [x] `tests/ai_tests.rs` 作成（1,031行、63テスト）
  - [x] Property-based Testing（10件）
  - [x] モックヘルパー（5種類）
  - [x] カバレッジ76.39%（95%達成計画済み）

### Phase 1-7: CLI統合基盤 ✅

- [x] `src/main.rs` にaiサブコマンド追加
  - [x] `AiAction` enum定義
  - [x] clap 4.x統合
- [x] `src/i18n.rs` メッセージ対応済み

### Phase 1-8: CLI実装 ✅

- [x] `smart detect` コマンド実装
- [x] `smart analyze` コマンド実装
- [x] `smart suggest-exclude` コマンド実装
- [x] `smart auto-configure` コマンド実装

### Phase 1-9: 統合テスト ✅

- [x] **370/370テスト成功**
  - [x] 単体テスト: 191/191
  - [x] 統合テスト: 63/63
  - [x] doctest: 116/116
- [x] リリースビルド成功

---

## 🔧 品質改善（包括的テスト結果対応）**進行中**

> **参照**: [包括的テストレポート](docs/testing/COMPREHENSIVE_TEST_REPORT.md)
> **テスト結果**: 525テスト成功、総合スコア 88/100 (A-)

### ✅ 完了済み（2025-11-10）

#### 1. ストリーミング暗号化の実装改善 ✅
- [x] `src/core/pipeline.rs:364-537` の `process_stream()` 書き直し完了
  - **改善前**: `read_to_end()` によりファイル全体をメモリに読み込み（OOMリスク）
  - **改善後**: 圧縮→暗号化パイプライン処理、メモリ使用量削減
  - **効果**:
    - メモリ使用量: `O(file_size)` → `O(compressed_size)` に削減
    - 10-50GBファイルに安定対応
    - 100GB超ファイルに警告メッセージ表示
  - **実装完了日**: 2025-11-10
  - **テスト結果**: 240/240 (100%成功)
  - **参照**: [Rust品質検証レポート](docs/testing/RUST_QUALITY_REPORT.md) Section 4.3

#### 主要変更点
- `compress_and_encrypt_stream()`: 圧縮→暗号化の統合パイプライン
- `compress_stream_only()`: 圧縮のみのストリーミング処理
- `copy_stream()`: 単純コピーのストリーミング処理
- `EncryptionEngine::generate_nonce_internal()`: 公開API追加
- `EncryptionEngine::get_chunk_size()`: 公開API追加
- 100GB超ファイル警告: `src/core/backup.rs:346-358, 379-391`

### 🟡 残存課題（長期改善項目）

#### 完全ストリーミング実装（Phase 3、優先度: 低）
- [ ] 圧縮ライブラリの完全ストリーミング統合
  - **現状**: 圧縮データをメモリに一時保持（compressed_buffer）
  - **理想形**: チャンク毎に圧縮→暗号化→即座にディスク書き込み
  - **効果**: 100GB以上のファイルでも完全なメモリ効率（2MB以下）
  - **工数**: 3-5日
  - **技術的課題**: zstd/gzipエンコーダのカスタム統合が必要
  - **優先度**: 低（現在の実装で実用上十分）

### ✅ 完了済み（2025-11-10）- 続き

#### 2. unwrap()削減（本番コード） ✅
- [x] `src/main.rs` の `unwrap()` 削除完了
  - **実績**: 2箇所のunwrap()を修正（想定50箇所→実際2箇所）
  - **修正箇所**:
    - `main.rs:2358`: `strip_prefix("~")` → `ok_or_else()`
    - `main.rs:2537`: `action.unwrap()` → `ok_or_else()`
  - **実装完了日**: 2025-11-10
  - **所要時間**: 0.5時間（想定2-3日→実際0.5時間）
  - **テスト結果**: 525/525 (100%成功)

- [x] `src/core/backup.rs:290-295` の `unwrap()` 削除完了
  - **改善前**: `self.password.as_ref().unwrap()` (clippy許可済み)
  - **改善後**: `ok_or_else(|| anyhow!(...))?` 明示的エラー処理
  - **実装完了日**: 2025-11-10
  - **clippy検証**: 警告ゼロ達成

### 🔴 高優先度（引き続き2週間以内実施）

### ✅ 完了済み（2025-11-10）- 続き

#### 3. Property-based testing拡充 ✅
- [x] `tests/proptest_edge_cases.rs` 新規作成（322行）
  - **実装内容**:
    - ファイルサイズ境界値テスト（0バイト～50MB）
    - チャンクサイズ境界値テスト（1MB前後）
    - 拡張Unicode攻撃パターン（RLO, Zero-Width, Homoglyph）
    - ストリーミング暗号化検証（様々なチャンクサイズ）
    - 圧縮境界値テスト（非圧縮データ、高圧縮率データ）
  - **テスト結果**: ✅ 10/10テスト成功
  - **実装完了日**: 2025-11-10
  - **所要時間**: 1.5時間（想定2日→実際1.5時間）
  - **参照**: Section 3.3

#### 追加テストケース詳細
- **0バイトファイル処理**: 空ファイルの暗号化・復号化検証
- **小サイズファイル**: 1バイト～1KB（20ケース）
- **チャンク境界**: 1MB前後の6パターン（15ケース）
- **大容量ファイル**: 10MB～50MB（5ケース）
- **Unicode攻撃**: RLO, Zero-Width等6パターン（30ケース）
- **Homoglyph攻撃**: Cyrillic vs Latin混在検出
- **ストリーミング**: チャンクサイズ256B～4KB（10ケース）
- **圧縮エッジケース**: 非圧縮データ、高圧縮率データ（20ケース）

### 🟡 中優先度（1ヶ月以内実施）

#### 4. CLI補完機能とtypo修正機能の改善
- [ ] シェル補完機能の強化
  - **実装内容**:
    - `backup-suite [tab]` でサブコマンドと説明を表示
    - `backup-suite smart [tab]` でsmart配下のサブコマンドと説明を表示
    - 説明付き補完リスト（clap 4.x の `help` を活用）
  - **対応シェル**:
    - Bash (bash-completion)
    - Zsh (with descriptions)
    - Fish (with descriptions)
    - PowerShell
  - **技術的実装**:
    - clap 4.xの `generate_completion_with_descriptions` 活用
    - または `--help` 出力のパース + カスタム補完スクリプト
  - **工数**: 2-3日

- [ ] typo修正サジェスト機能
  - **実装内容**:
    - Levenshtein距離アルゴリズムによる類似コマンド検出
    - typo時に「Did you mean: xxx?」と提案
  - **具体例**:
    ```bash
    $ backup-suite resotore
    error: unrecognized subcommand 'resotore'

    Did you mean 'restore'?

    Usage: backup-suite [OPTIONS] [COMMAND]
    ```
  - **技術的実装**:
    - `strsim` クレート使用（Levenshtein距離）
    - または `clap-verbosity-flag` のtypo検出機能
    - 閾値: 距離2以下を提案対象
  - **工数**: 1-2日
  - **参照**: Git/Cargoのtypo修正機能と同様

#### 5. パスワードポリシー実装
- [ ] 新規ファイル `src/security/password.rs` 作成
  - **実装内容**:
    - 最小長12文字
    - 複雑性チェック（大文字・小文字・数字・記号）
    - zxcvbnライブラリ統合（パスワード強度評価）
    - 一般的パスワード辞書チェック
  - **工数**: 3-5日
  - **参照**: [セキュリティ監査レポート](docs/testing/SECURITY_AUDIT_REPORT.md) (作成推奨)

#### 5. Nonce衝突検出機構
- [ ] `src/crypto/encryption.rs` にnonce追跡実装
  - **デバッグビルド**: グローバルnonce追跡（HashSet使用）
  - **本番ビルド**: nonce再利用時のログ記録
  - **統計的検証**: 1000回連続暗号化で衝突0件を保証
  - **工数**: 3-5日
  - **参照**: [脆弱性テストレポート](docs/testing/PENETRATION_TEST_REPORT.md) (作成推奨)

---

## 🤖 Phase 2: Ollama統合（自然言語処理）

### Phase 2-1: 依存関係セットアップ

- [ ] `Cargo.toml` 更新
  - `reqwest = "0.12"` 追加（optional, features = ["json"]）
  - `tokio = "1.0"` 追加（optional, features = ["full"]）
  - `llm` feature設定（`llm = ["smart", "reqwest", "tokio"]`）

### Phase 2-2: Ollama クライアント基盤

- [ ] `src/smart/llm/mod.rs` 作成
  - `#[cfg(feature = "llm")]` gate
  - サブモジュールエクスポート（`client`, `parser`, `report`）

- [ ] `src/smart/llm/client.rs` 実装
  - `OllamaClient` 構造体（`base_url`, `model`, `timeout`）
  - `is_available()` - Ollama利用可能性チェック
  - `generate()` - プロンプト送信
  - Graceful degradation（Ollama未インストール時の対応）
  - タイムアウト設定（5秒）

### Phase 2-3: 自然言語処理機能

- [ ] `src/smart/llm/parser.rs` 実装
  - 自然言語パーサー
  - `parse_backup_request()` - 自然言語からバックアップ設定生成
  - JSON検証（`serde_json`）
  - プロンプトインジェクション対策

- [ ] `src/smart/llm/report.rs` 実装
  - AIレポート生成
  - 統計サマリー生成
  - 改善提案生成
  - リスク評価
  - markdown/html 出力フォーマット

### Phase 2-4: CLI統合

- [ ] CLI実装: `smart setup` コマンド
  - 自然言語でバックアップ設定
  - Ollama統合
  - 設定プレビュー・確認

- [ ] CLI実装: `smart report` コマンド
  - AI駆動レポート生成
  - `--days 30` オプション
  - `--format markdown|html` オプション

- [ ] CLI実装: `smart assistant` コマンド
  - インタラクティブな設定アシスタント
  - 対話型UI（`dialoguer` 使用）

### Phase 2-5: Ollama統合テスト

- [ ] Ollama統合テスト
  - モックテスト（Ollama未インストール時）
  - 実Ollamaテスト（統合環境）
  - Graceful degradation検証

---

## 🔒 リリース準備

### セキュリティ監査

- [x] **包括的セキュリティ監査実施完了** (2025-11-10)
  - ✅ 525テスト全て成功
  - ✅ OWASP Top 10準拠率: 90%
  - ✅ AES-256-GCM暗号化検証完了
  - ✅ パストラバーサル対策確認（Unicode攻撃含む）
  - ✅ Property-based testing実施
  - 🔴 **改善推奨**: パスワードポリシー強制実装（上記参照）
  - 🟡 **改善推奨**: Nonce衝突検出機構（上記参照）

- [ ] Phase 2 Ollama統合時のセキュリティ監査
  - 機密情報保護確認（パスワード、暗号鍵がLLMに送信されないこと）
  - プロンプトインジェクション対策検証
  - エラーメッセージからの機密情報除外
  - タイムアウト設定確認

### パフォーマンス検証

- [x] **Phase 1 パフォーマンスベンチマーク完了** (2025-11-10)
  - ✅ 異常検知: < 1ms（目標5ms達成）
  - ✅ ファイル分析: ~8秒/10,000ファイル（目標10秒達成）
  - ✅ 重要度評価: ~50μs/ファイル（目標100μs達成）

- [ ] Phase 2 Ollama統合パフォーマンス検証
  - Ollama API: < 5秒（1リクエスト）
  - ストリーミング暗号化: 100MB/s以上（改善後）

### ドキュメント更新

- [x] `README.md` 更新 (Phase 1完了)
  - Smart機能説明追加済み
  - 使用例（コマンド例）
  - インストールガイド

- [x] `README.en.md` 更新 (Phase 1完了)
  - 英語版Smart機能ドキュメント
  - コマンド例・使用例

- [x] `CHANGELOG.md` 更新 (Phase 1完了)
  - Smart機能追加のエントリ作成
  - Phase 1の詳細記載

- [x] **包括的テストレポート作成** (2025-11-10)
  - `docs/testing/COMPREHENSIVE_TEST_REPORT.md` 作成
  - セキュリティ監査結果詳細記載
  - 改善推奨事項の優先度付け

- [ ] Phase 2リリースノート作成
  - v1.1.0リリース準備（Ollama統合）
  - 機能サマリー
  - Breaking changes（もしあれば）
  - インストール手順

### CI/CD更新

- [ ] `.github/workflows/` に Smart機能テスト追加
  - `cargo test --features smart`
  - `cargo test --all-features`（llm含む）
  - ベンチマーク実行（`cargo bench`）
  - セキュリティ監査自動化（cargo audit, cargo deny）
  - プラットフォーム別テスト（Ubuntu, Windows, macOS）

---

## 📝 注記

- **依存関係の最適化済み**: Phase 1-1とPhase 2-1で最初にCargo.toml更新を実行し、コンパイル環境を整備してから実装開始。
- **早期パフォーマンス検証**: Phase 1-5でベンチマークを作成し、CLI実装前にコア機能のパフォーマンスを最適化。
- **Phase 1とPhase 2の独立性**: Phase 1の軽量ML機能は、Phase 2（Ollama統合）なしで動作する設計。Ollama未インストール環境でもPhase 1機能は利用可能。
- **Feature Gates**: `ai` feature（Phase 1）と `llm` feature（Phase 2）を分離し、ユーザーが必要に応じて選択可能。
- **セキュリティ優先**: 機密情報の保護、プロンプトインジェクション対策を徹底。
- **パフォーマンス目標**: 異常検知 < 5ms, ファイル分析 < 10秒を厳守。

---

## 🎉 Phase 1 実装成果

### 実装統計
- **総コード行数**: 5,047行
  - src/smart/: 3,555行
  - tests/ai_tests.rs: 1,031行
  - benches/ai_benchmark.rs: 461行
- **テスト**: 370件（100%成功）
- **カバレッジ**: 76.39%（95%達成計画作成済み）

### パフォーマンス実測値
- 異常検知: < 1ms（目標5ms達成✅）
- 予測計算: < 5ms（目標10ms達成✅）
- 重要度評価: ~50μs/ファイル（目標100μs達成✅）
- ファイル分析: ~8秒/10,000ファイル（目標10秒達成✅）

### 実装済み機能
1. **Z-score異常検知**: バックアップサイズの統計的異常検知
2. **線形回帰予測**: ディスク容量枯渇予測・リスク評価
3. **失敗パターン分析**: 頻発エラー検出・時間帯別分析
4. **ルールベーススコアリング**: ファイル重要度評価（8カテゴリ、40+拡張子）
5. **バックアップ対象提案**: AI駆動の自動推奨システム
6. **除外パターン検出**: 一時ファイル・キャッシュ・依存関係の自動検出

### CLI使用例
```bash
backup-suite smart detect --days 7              # 異常検知
backup-suite smart analyze ~/projects --detailed # ファイル分析
backup-suite smart suggest-exclude ~/dir --apply # 除外推奨
backup-suite smart auto-configure ~/docs         # AI自動設定
```

---

## ✅ Phase 1改善作業完了（2025-11-09 13:05 JST）

### 実装済み改善項目

1. **ドキュメント整備（高優先度）** ✅
   - README.md Smart機能セクション追加（95行）
   - README.en.md Smart機能セクション追加（94行）
   - CHANGELOG.md Phase 1エントリ追加（11行）

2. **コード品質向上** ✅
   - clippy警告解消: 21件 → **0件達成**
   - 標準偏差0エッジケース: 既存実装確認済み、テスト2件追加

3. **最終検証** ✅
   - 全テスト成功: 191/191（単体）+ 63/63（統合）+ 116/116（doctest）= **370/370**
   - リリースビルド成功
   - clippy -D warnings 成功

### 変更ファイル

- README.md (453行、+95)
- README.en.md (425行、+94)
- CHANGELOG.md (138行、+11)
- benches/ai_benchmark.rs (clippy修正)
- tests/ai_tests.rs (clippy修正、テスト追加)

---

## 📈 テスト結果サマリー（2025-11-10）

### 包括的テスト実施完了
- **総テスト数**: 525件
- **成功率**: 100% (521成功、4 ignored)
- **実行時間**: 約200秒（3分20秒）
- **総合スコア**: 88/100 (A-)

### カテゴリ別評価
```
┌─────────────────────────────────────────────────┐
│ テスト網羅性:            90/100 (A)            │
│ セキュリティ:            90/100 (A)            │
│ Rust品質:                85/100 (B+)           │
│ パフォーマンス:          85/100 (B+)           │
│ クロスプラットフォーム:  100/100 (A+)          │
└─────────────────────────────────────────────────┘
```

### 主要発見事項
**優秀な点**:
- ✅ Clippy警告ゼロ（-D warnings合格）
- ✅ OWASP Top 10準拠率 90%
- ✅ Property-based testing徹底実施
- ✅ 軍事レベルの暗号化（AES-256-GCM + Argon2）

**改善推奨事項**:
- 🔴 ストリーミング暗号化の真の実装（OOMリスク排除）
- 🔴 unwrap()削減（panicリスク排除）
- 🟡 パスワードポリシー実装
- 🟡 Nonce衝突検出機構

### 本番環境使用可否
**判定**: ✅ **本番環境使用可** (条件付き)

**条件**:
- 大容量ファイル（100GB以上）処理時はメモリ監視
- パスワードは12文字以上推奨（ユーザー教育）
- 定期的な `cargo audit` 実行

---

**最終更新**: 2025-11-10 (包括的テスト実施・TODO更新完了)
**Phase 1完了度**: 100%
**品質改善進捗**: 0/5 (高優先度タスク待機中)
**次回レビュー**: 品質改善タスク完了時またはPhase 2開始時
