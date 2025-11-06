# backup-suite - プロジェクト固有設定

> **高速・安全・インテリジェントなローカルバックアップツール**
>
> AES-256-GCM暗号化、Zstd圧縮、優先度別バックアップ管理を実現するCLIツール

## 🎯 プロジェクト概要

- **言語**: Rust 1.70+ (MSRV)
- **プロジェクトタイプ**: CLIバックアップツール、セキュリティ重視システム
- **主要技術**:
  - 暗号化: AES-256-GCM、Argon2（鍵導出）
  - 圧縮: Zstd（高速・高圧縮率）、Gzip（互換性）
  - CLI: clap 4.x、dialoguer（対話UI）
  - 並列処理: rayon
- **セキュリティ重視**:
  - 軍事レベルの暗号化（AES-256-GCM）
  - メモリ安全性（zeroize、Rust型システム）
  - パストラバーサル対策
  - 安全な鍵管理
- **パフォーマンス重視**:
  - 並列バックアップ処理（rayon）
  - 高速圧縮（Zstd）
  - 効率的なファイルコピーエンジン

## 🤖 推奨Subagents（プロジェクト特化）

このプロジェクトでは以下のsubagentsを**積極的に活用**してください：

### 🔴 最優先Agents（常時活用）

#### 1. **security-auditor** - セキュリティ監査（最重要）
```yaml
活用シーン:
  - src/crypto/: AES-256-GCM暗号化の実装監査
  - src/security/: パストラバーサル対策レビュー
  - src/core/backup.rs: バックアップ処理のセキュリティ検証
  - 鍵管理の安全性確認
  - 機密データのメモリ管理（zeroize使用確認）

優先タスク:
  - AES-256-GCM実装のOWASP準拠確認
  - Argon2パラメータの妥当性検証
  - パスワード処理のセキュリティレビュー
  - ファイルパス操作のインジェクション対策確認
```

#### 2. **rust-engineer** - Rust専門家
```yaml
活用シーン:
  - Cargo.toml最適化（LTO、codegen設定）
  - src/core/pipeline.rs: 並列処理の最適化
  - src/crypto/: 暗号化処理のRustベストプラクティス
  - メモリ管理・所有権設計の最適化

優先タスク:
  - rayon並列処理の効率化
  - zeroizeによる機密データ消去の徹底
  - エラーハンドリング（anyhow/thiserror）の改善
  - 型安全性の強化
```

#### 3. **cli-developer** - CLI UX専門家
```yaml
活用シーン:
  - clap設定の最適化
  - dialoguer対話UIの改善
  - エラーメッセージの分かりやすさ向上
  - 国際化対応（日本語・英語）

優先タスク:
  - src/ui/: ユーザーインターフェース改善
  - コマンド体系の一貫性確認
  - プログレスバー・テーブル表示の最適化
  - ヘルプメッセージの分かりやすさ向上
```

### 🟡 高優先Agents（定期活用）

#### 4. **performance-engineer** - パフォーマンス最適化
```yaml
活用シーン:
  - バックアップ速度の最適化
  - Zstd圧縮レベルの調整
  - 並列処理の効率化
  - メモリ使用量の削減

優先タスク:
  - src/core/copy_engine.rs: ファイルコピー最適化
  - rayon並列度の調整
  - criterion ベンチマークの設計・実行
  - 大容量ファイル処理の最適化
```

#### 5. **test-automator** - テスト自動化
```yaml
活用シーン:
  - proptest による property-based testing
  - 暗号化・復号化のテスト強化
  - エッジケーステスト（大容量ファイル、特殊文字）
  - 統合テストのカバレッジ向上

優先タスク:
  - tests/: テストカバレッジ拡大
  - 暗号化処理の網羅的テスト
  - ファイル操作のエラーケーステスト
  - CI/CDパイプラインの最適化
```

#### 6. **penetration-tester** - 脆弱性テスト
```yaml
活用シーン:
  - 暗号化実装の脆弱性診断
  - パストラバーサル攻撃の検証
  - サイドチャネル攻撃対策確認
  - 鍵管理の脆弱性テスト

優先タスク:
  - AES-GCMのnonce再利用防止確認
  - タイミング攻撃対策の検証
  - ファイルパス操作の攻撃パターンテスト
```

### 🟢 中優先Agents（特定領域強化）

#### 7. **compliance-auditor** - コンプライアンス監査
```yaml
活用シーン:
  - ライセンス監査（MIT準拠）
  - 依存クレートのライセンス確認
  - セキュリティ標準準拠（OWASP、NIST）

優先タスク:
  - deny.toml設定の妥当性確認
  - 依存関係のセキュリティ脆弱性スキャン
```

#### 8. **sre-engineer** - サイト信頼性
```yaml
活用シーン:
  - バックアップスケジューリング（launchctl/systemd）
  - エラー監視・ロギング戦略
  - 自動リカバリ機能の設計

優先タスク:
  - スケジューラの信頼性向上
  - ログ管理の最適化
  - 障害時の自動復旧機能
```

#### 9. **database-optimizer** - データ構造最適化
```yaml
活用シーン:
  - 履歴データの効率的な管理
  - 設定ファイル（TOML）の最適化
  - メタデータ管理の改善

優先タスク:
  - src/core/history.rs: 履歴データ構造の最適化
  - バックアップメタデータの効率化
```

### 💡 状況依存Agents（特定タスク時）

#### 10. **embedded-systems** - 組み込み最適化
```yaml
活用シーン: メモリ制約環境での動作
タスク: メモリフットプリント削減、バイナリサイズ最小化
```

#### 11. **devops-engineer** - CI/CD・配布
```yaml
活用シーン: リリース自動化、クロスプラットフォームビルド
タスク: Homebrew Formula管理、GitHub Actions最適化
```

#### 12. **multi-agent-coordinator** - 複数エージェント協調
```yaml
活用シーン: 大規模リファクタリング、総合レビュー
タスク: security-auditor + rust-engineer + performance-engineer 並列実行
```

## 📋 Agent活用戦略

### 🎯 開発フェーズ別の推奨Agent

```yaml
新機能実装:
  1. rust-engineer: 設計レビュー
  2. security-auditor: セキュリティ影響評価
  3. cli-developer: UX評価
  4. test-automator: テスト設計

セキュリティ強化:
  1. security-auditor: 脆弱性スキャン
  2. penetration-tester: 攻撃シミュレーション
  3. rust-engineer: 安全な実装
  4. test-automator: セキュリティテスト追加

パフォーマンス改善:
  1. performance-engineer: ボトルネック特定
  2. rust-engineer: 最適化実装
  3. test-automator: ベンチマーク作成

リリース準備:
  1. security-auditor: 最終セキュリティ監査
  2. compliance-auditor: ライセンス監査
  3. test-automator: 統合テスト
  4. devops-engineer: リリース自動化
```

### 🚀 Agent活用の具体例

```bash
# セキュリティ監査（最優先）
Task(security-auditor, "AES-256-GCM暗号化実装とArgon2鍵導出の包括的セキュリティ監査")
Task(penetration-tester, "パストラバーサル攻撃とタイミング攻撃の脆弱性テスト")

# パフォーマンス最適化
Task(performance-engineer, "Zstd圧縮レベルとrayon並列度の最適なバランス分析")
Task(rust-engineer, "src/core/copy_engine.rs の大容量ファイル処理最適化")

# UX改善
Task(cli-developer, "src/ui/dashboard.rs のユーザー体験改善と国際化対応強化")

# 総合レビュー（複数Agent協調）
Task(multi-agent-coordinator, "security-auditor、rust-engineer、performance-engineerを使ってbackup-suite全体を包括的にレビュー")
```

## 🔧 プロジェクト固有の開発ガイドライン

### Rustコーディング規約

```rust
// 推奨パターン
- セキュリティ優先: zeroize、機密データの即座消去
- 暗号化: AES-GCM（認証付き暗号化）、nonce再利用防止
- 並列処理: rayon活用、適切な並列度設定
- エラーハンドリング: anyhow::Result<T>、詳細なエラーコンテキスト
- 型安全: newtype pattern、強い型付け

// 避けるパターン
- unwrap() の多用（暗号化処理では特に厳禁）
- 機密データのログ出力
- 不適切なnonce/IV再利用
- タイミング攻撃につながる条件分岐
```

### セキュリティ基準（最重要）

```yaml
必須チェック項目:
  暗号化:
    - AES-256-GCM使用（認証付き暗号化）
    - nonce/IV の一意性保証
    - Argon2パラメータの妥当性（メモリ、反復回数）
    - 鍵の安全な管理（zeroize使用）

  ファイル操作:
    - パストラバーサル対策（canonicalize使用）
    - シンボリックリンク攻撃対策
    - ファイルパーミッション適切設定

  メモリ管理:
    - 機密データのzeroize徹底
    - タイミング攻撃対策（定数時間比較）
    - メモリリーク防止

  依存関係:
    - cargo-audit定期実行
    - deny.toml設定維持
    - 最新セキュリティパッチ適用
```

### パフォーマンス目標

```yaml
バックアップ速度: 100MB/s以上（Zstd圧縮時）
圧縮率: 平均50-70%削減（Zstdレベル3）
並列処理: CPU コア数に応じた自動最適化
メモリ使用量: 100MB以下（通常動作時）
起動時間: 100ms以下
```

## 📁 重要ディレクトリ構造

```
src/
├── crypto/                 # 暗号化 → security-auditor, penetration-tester
│   ├── encryption.rs       # AES-256-GCM実装
│   └── key_management.rs   # Argon2鍵導出
├── security/               # セキュリティ → security-auditor
│   ├── path.rs             # パストラバーサル対策
│   └── permissions.rs      # ファイルパーミッション
├── core/                   # コアロジック → rust-engineer
│   ├── backup.rs           # バックアップエンジン
│   ├── pipeline.rs         # 並列処理パイプライン
│   ├── copy_engine.rs      # ファイルコピー最適化
│   └── history.rs          # 履歴管理
├── ui/                     # ユーザーインターフェース → cli-developer
│   ├── dashboard.rs        # ダッシュボード表示
│   ├── interactive.rs      # 対話的UI
│   ├── progress.rs         # プログレスバー
│   └── table.rs            # テーブル表示
└── compression/            # 圧縮 → performance-engineer
    ├── zstd.rs             # Zstd圧縮
    └── gzip.rs             # Gzip圧縮

tests/                      # テスト → test-automator
├── integration/            # 統合テスト
├── security/               # セキュリティテスト
└── benchmarks/             # ベンチマーク

benches/                    # ベンチマーク → performance-engineer
├── crypto_benchmark.rs     # 暗号化パフォーマンス
├── backup_benchmark.rs     # バックアップ速度
└── integration_benchmark.rs # 統合ベンチマーク
```

## 🎯 今後の開発方向性

### Phase 1: セキュリティ強化（現在）
- 暗号化実装の監査強化 → **security-auditor + penetration-tester**
- 脆弱性テストの網羅 → **penetration-tester**
- コンプライアンス確認 → **compliance-auditor**

### Phase 2: パフォーマンス最適化
- 大容量ファイル処理の高速化 → **performance-engineer**
- 並列処理の最適化 → **rust-engineer**
- メモリ使用量削減 → **embedded-systems**

### Phase 3: 機能拡張
- クラウドバックアップ対応 → **cloud-architect**
- 増分バックアップ実装 → **rust-engineer**
- リアルタイムバックアップ → **sre-engineer**

### Phase 4: エコシステム
- Docker化・コンテナ対応 → **devops-engineer**
- Kubernetes統合 → **kubernetes-specialist**
- 企業向け機能（監査ログ等） → **compliance-auditor**

## 🔒 セキュリティベストプラクティス

### 暗号化実装チェックリスト

```yaml
AES-256-GCM:
  - [x] nonce/IV の一意性保証（ランダム生成）
  - [x] 認証タグの検証
  - [ ] nonce再利用防止の自動テスト
  - [ ] サイドチャネル攻撃対策の検証

Argon2:
  - [x] メモリコスト適切設定（最低19MB）
  - [x] 反復回数適切設定（最低2回）
  - [ ] パラメータの定期的な見直し
  - [ ] ブルートフォース攻撃耐性の評価

鍵管理:
  - [x] zeroize使用（機密データ消去）
  - [ ] 鍵ローテーション機能
  - [ ] ハードウェアセキュリティモジュール対応検討
```

### パストラバーサル対策チェックリスト

```yaml
ファイルパス処理:
  - [x] canonicalize使用（パス正規化）
  - [x] シンボリックリンクチェック
  - [ ] chroot jail検討
  - [ ] ファジングテストの実施

バックアップ先検証:
  - [x] 書き込み権限確認
  - [x] ディスク容量チェック
  - [ ] クォータ制限対応
```

## 📊 パフォーマンスベンチマーク目標

```yaml
バックアップ速度（100MBファイル x 1000個）:
  Zstd圧縮: 100MB/s以上
  Gzip圧縮: 80MB/s以上
  圧縮なし: 200MB/s以上

暗号化オーバーヘッド:
  AES-256-GCM: 10%以内
  全体処理: 15%以内

並列処理効率:
  4コアCPU: 3.5倍以上のスループット
  8コアCPU: 7倍以上のスループット
```

## 📖 関連ドキュメント

- [README.md](../README.md): プロジェクト概要
- [README.en.md](../README.en.md): English documentation
- [CHANGELOG.md](../CHANGELOG.md): 変更履歴
- [PUBLISHING.md](../PUBLISHING.md): リリース手順
- [deny.toml](../deny.toml): 依存関係監査設定
- [docs/](../docs/): ドキュメント・スクリーンショット

---

**💡 開発時のヒント**:
- **セキュリティは最優先事項** - 新機能追加時は必ず security-auditor でレビュー
- **暗号化処理は慎重に** - penetration-tester で脆弱性テスト必須
- **パフォーマンス測定を忘れずに** - criterion ベンチマークで定量評価
- **複雑なタスクは multi-agent-coordinator で複数agentを協調**
