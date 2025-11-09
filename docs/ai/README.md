# AI機能ドキュメント

backup-suite のAI駆動インテリジェント管理機能に関するドキュメント集です。

## 📚 ユーザー向けドキュメント

### [features.md](features.md) - AI機能ガイド（739行）
AI機能の包括的ガイド：異常検知、ファイル重要度分析、除外パターン推奨、自動設定

**対象**: すべてのユーザー、特にAI機能を活用したい方

**内容**:
- 🤖 AI機能概要
- 📊 異常検知（統計的分析）
- 🎯 ファイル重要度評価
- 🚫 除外パターン推奨
- ⚙️ 自動設定
- 🔒 セキュリティ・プライバシー
- 📈 パフォーマンス指標

### [manual-test.md](manual-test.md) - AI機能手動テスト手順（595行）
実機でのAI機能テスト手順書

**対象**: QA、テスター、コントリビューター

**内容**:
- ✅ テスト項目チェックリスト（17項目）
- 🔧 事前準備手順
- 📋 詳細テストケース
- 🧹 クリーンアップ手順

---

## 🛠️ 開発者向けドキュメント

開発者向けの詳細な技術ドキュメントは [`development/`](development/) ディレクトリにあります。

### [development/implementation-plan.md](development/implementation-plan.md) - 実装計画（30K）
AI機能のフェーズ別実装計画と技術仕様

**内容**:
- Phase 1: 統計的異常検知・推奨エンジン（完了）
- Phase 2-4: 今後の拡張計画
- アーキテクチャ設計
- セキュリティ考慮事項

### [development/recommendation-engine.md](development/recommendation-engine.md) - 推奨エンジン設計（7.6K）
ファイル重要度評価と除外パターン推奨の詳細設計

**内容**:
- ルールベース評価アルゴリズム
- スコアリングロジック
- 除外パターン検出手法

### [development/test-report.md](development/test-report.md) - テストレポート（12K）
AI機能の包括的テストレポート

**内容**:
- 単体テスト結果（271テスト、100% Pass）
- 統合テスト結果
- カバレッジレポート（85%+）
- パフォーマンステスト結果

### [development/benchmark-report.md](development/benchmark-report.md) - ベンチマークレポート（11K）
AI機能のパフォーマンスベンチマーク詳細

**内容**:
- 異常検知性能（< 8μs/1000エントリ）
- ファイル分析性能（59ms/1000ファイル）
- 統計計算性能（< 3μs/1000エントリ）
- メモリ使用量（~2.2MB/10000エントリ）

---

## 🔗 関連ドキュメント

- [../../README.md](../../README.md) - プロジェクトメインREADME（AI機能セクションあり）
- [../development/](../development/) - 開発ドキュメント全般
- [../user/USAGE.md](../user/USAGE.md) - 使用方法ガイド

---

## 📝 ドキュメント更新履歴

- **2025-11**: AI機能ドキュメント整理（docs/ai/に統合）
- **2025-11**: Phase 1実装完了、ドキュメント作成
