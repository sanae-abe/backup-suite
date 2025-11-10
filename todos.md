# backup-suite タスク管理

## 🔴 高優先度（緊急・重要）

- [ ] CI環境でのWindowsテスト確認 | Priority: critical | Context: test | Due: 2025-11-11
- [ ] 統合テスト失敗の原因調査と修正 | Priority: high | Context: test | Due: 2025-11-11

## 🟡 中優先度（1ヶ月以内実施）

- [ ] CLI補完機能の強化 - Zsh/Fish/Bash対応 | Priority: medium | Context: ui | Due: 2025-12-10
- [ ] typo修正サジェスト機能 - Levenshtein距離検出 | Priority: medium | Context: ui | Due: 2025-12-10
- [ ] パスワードポリシー実装 - 最小長12文字・複雑性チェック | Priority: medium | Context: security | Due: 2025-12-15
- [ ] Nonce衝突検出機構 - デバッグビルド追跡 | Priority: medium | Context: security | Due: 2025-12-15

## 🟢 低優先度（Phase 3以降）

- [ ] ストリーミング暗号化の完全実装 - チャンク毎の圧縮暗号化 | Priority: low | Context: build | Due: 2026-01-31
- [ ] Phase 2 Ollama統合 - 依存関係セットアップ | Priority: low | Context: api | Due: 2025-12-31
- [ ] Phase 2 Ollama統合 - クライアント基盤実装 | Priority: low | Context: api | Due: 2025-12-31
- [ ] Phase 2 Ollama統合 - 自然言語処理機能 | Priority: low | Context: api | Due: 2026-01-15
- [ ] Phase 2 Ollama統合 - CLI統合 | Priority: low | Context: ui | Due: 2026-01-15
- [ ] Phase 2 Ollama統合テスト | Priority: low | Context: test | Due: 2026-01-31

## 📚 ドキュメント・リリース準備

- [ ] Phase 2リリースノート作成 | Priority: low | Context: docs | Due: 2026-01-31
- [ ] CI/CD Smart機能テスト追加 | Priority: medium | Context: build | Due: 2025-11-20
- [ ] セキュリティ監査レポート作成推奨 | Priority: medium | Context: security | Due: 2025-12-01
- [ ] 脆弱性テストレポート作成推奨 | Priority: medium | Context: security | Due: 2025-12-01

## ✅ 完了済み（最近完了）

- [x] Windows環境テスト修正 - test_evaluate_temp_file | Priority: high | Context: test | Due: 2025-11-10
- [x] ai → smart モジュールリネーム対応 | Priority: high | Context: build | Due: 2025-11-10
- [x] 型エラー修正 - 浮動小数点型明示化 | Priority: high | Context: build | Due: 2025-11-10
- [x] ドキュメントコメント警告修正 | Priority: medium | Context: build | Due: 2025-11-10
- [x] TODO.md更新 - CLI補完・typo修正機能追加 | Priority: low | Context: docs | Due: 2025-11-10
- [x] Phase 1実装完了 - 370/370テスト成功 | Priority: critical | Context: build | Due: 2025-11-09
- [x] ストリーミング暗号化改善 - process_stream書き直し | Priority: high | Context: build | Due: 2025-11-10
- [x] unwrap削減 - main.rsとcore/backup.rs | Priority: high | Context: build | Due: 2025-11-10
- [x] Property-based testing拡充 - proptest_edge_cases.rs | Priority: high | Context: test | Due: 2025-11-10
- [x] 包括的セキュリティ監査実施 - 525テスト成功 | Priority: critical | Context: security | Due: 2025-11-10
- [x] パフォーマンスベンチマーク完了 - 全目標達成 | Priority: high | Context: test | Due: 2025-11-10
