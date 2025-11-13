# backup-suite 手動テストガイド

> **完璧な動作確認のための包括的QAチェックリスト**
>
> このドキュメントは、backup-suiteの全機能を手動で確認するための詳細な手順書です。

---

## 📋 目次

1. [事前準備](#事前準備)
2. [基本機能テスト](#基本機能テスト)
3. [セキュリティ機能テスト](#セキュリティ機能テスト)
4. [パフォーマンステスト](#パフォーマンステスト)
5. [エラーハンドリングテスト](#エラーハンドリングテスト)
6. [ユーザビリティテスト](#ユーザビリティテスト)
7. [統合シナリオテスト](#統合シナリオテスト)

---

## 事前準備

### 環境セットアップ

```bash
# 1. プロジェクトのビルド
cargo build --release

# 2. テスト用ディレクトリの作成
mkdir -p ~/backup-suite-manual-test/{source,backup,restore}
cd ~/backup-suite-manual-test
```

### テストデータの作成

```bash
# source ディレクトリにテスト用ファイルを作成

# 小さいテキストファイル
echo "This is a test document" > source/document.txt

# 設定ファイル（TOML）
cat > source/config.toml <<EOF
[settings]
key = "value"
debug = true
EOF

# 大きなバイナリファイル（10MB）
dd if=/dev/urandom of=source/large_file.bin bs=1024 count=10240

# サブディレクトリ構造
mkdir -p source/subdir/deep
echo "Nested file 1" > source/subdir/file1.txt
echo "Nested file 2" > source/subdir/deep/file2.txt

# 各種ファイルタイプ
touch source/image.png
touch source/script.sh
touch source/data.json
```

---

## 基本機能テスト

### Test 1.1: 基本的なバックアップ

**目的**: 最もシンプルなバックアップが正常に動作することを確認

**手順**:
```bash
# バックアップ実行
./target/release/backup-suite backup source/ backup/

# 期待結果チェック
ls -la backup/
```

**期待結果**:
- ✅ コマンドが正常終了（exit code 0）
- ✅ `backup/` ディレクトリにファイルが作成される
- ✅ 成功メッセージが表示される
- ✅ バックアップファイル数が元のファイル数と一致

**確認項目**:
- [ ] エラーなく完了
- [ ] バックアップファイルが存在
- [ ] ファイル数が正しい

---

### Test 1.2: 基本的な復元

**目的**: バックアップからの復元が正常に動作することを確認

**手順**:
```bash
# 復元実行
./target/release/backup-suite restore backup/ restore/

# 復元されたファイルの確認
ls -laR restore/
```

**期待結果**:
- ✅ 復元が正常完了
- ✅ 全ファイルが復元される
- ✅ ディレクトリ構造が保持される

**確認項目**:
- [ ] 全ファイルが復元されている
- [ ] ディレクトリ階層が正しい
- [ ] ファイル内容が元と一致

**検証コマンド**:
```bash
diff -r source/ restore/
# 出力なし（差分なし）が期待結果
```

---

### Test 1.3: ファイル内容の完全性検証

**目的**: 復元されたファイルの内容が元のファイルと完全に一致することを確認

**手順**:
```bash
# 各ファイルのハッシュ値比較
sha256sum source/document.txt
sha256sum restore/document.txt

sha256sum source/large_file.bin
sha256sum restore/large_file.bin
```

**期待結果**:
- ✅ 全ファイルのSHA256ハッシュが元のファイルと一致

**確認項目**:
- [ ] `document.txt` のハッシュ一致
- [ ] `config.toml` のハッシュ一致
- [ ] `large_file.bin` のハッシュ一致
- [ ] ネストしたファイルのハッシュ一致

---

## セキュリティ機能テスト

### Test 2.1: パスワード暗号化バックアップ

**目的**: AES-256-GCM暗号化が正しく機能することを確認

**手順**:
```bash
# 暗号化バックアップ実行
rm -rf backup/*
./target/release/backup-suite backup source/ backup/ --password "SecurePassword123!"

# バックアップファイルが暗号化されていることを確認
file backup/*
strings backup/* | head -20  # 元のテキストが読めないことを確認
```

**期待結果**:
- ✅ バックアップが成功
- ✅ バックアップファイルから元のテキストが読み取れない（暗号化されている）
- ✅ ファイルタイプが暗号化データであることを示す

**確認項目**:
- [ ] 暗号化バックアップ成功
- [ ] バックアップファイルが暗号化されている
- [ ] 元のテキストが読めない

---

### Test 2.2: 暗号化復元（正しいパスワード）

**目的**: 正しいパスワードで復号化できることを確認

**手順**:
```bash
# 復元ディレクトリクリア
rm -rf restore/*

# 正しいパスワードで復元
./target/release/backup-suite restore backup/ restore/ --password "SecurePassword123!"

# 復元されたファイルの確認
diff -r source/ restore/
```

**期待結果**:
- ✅ 復元が成功
- ✅ ファイル内容が元と完全に一致

**確認項目**:
- [ ] 復元成功
- [ ] 全ファイルが正しく復元
- [ ] `diff` で差分なし

---

### Test 2.3: 暗号化復元（誤ったパスワード）

**目的**: 誤ったパスワードでは復号化できないことを確認（セキュリティ検証）

**手順**:
```bash
# 復元ディレクトリクリア
rm -rf restore/*

# 誤ったパスワードで復元を試みる
./target/release/backup-suite restore backup/ restore/ --password "WrongPassword"
```

**期待結果**:
- ✅ 復元が**失敗**する
- ✅ エラーメッセージが表示される
- ✅ ファイルが復元されない

**確認項目**:
- [ ] 復元が失敗（exit code != 0）
- [ ] 適切なエラーメッセージ
- [ ] restore/ が空のまま

---

### Test 2.4: パスワード未指定での復元失敗

**目的**: 暗号化バックアップにパスワードなしでアクセスできないことを確認

**手順**:
```bash
rm -rf restore/*

# パスワードなしで復元を試みる
./target/release/backup-suite restore backup/ restore/
```

**期待結果**:
- ✅ 復元が失敗
- ✅ 「パスワードが必要」というエラーメッセージ

**確認項目**:
- [ ] 復元失敗
- [ ] パスワード要求エラー表示

---

## パフォーマンステスト

### Test 3.1: Zstd圧縮バックアップ

**目的**: Zstd圧縮が正しく機能し、ファイルサイズが削減されることを確認

**手順**:
```bash
rm -rf backup/*

# 圧縮なしバックアップ
./target/release/backup-suite backup source/ backup/no-compress/

# Zstd圧縮バックアップ
./target/release/backup-suite backup source/ backup/zstd/ --compression zstd

# サイズ比較
du -sh backup/no-compress/
du -sh backup/zstd/
```

**期待結果**:
- ✅ 両方のバックアップが成功
- ✅ Zstd圧縮版のサイズが小さい
- ✅ 圧縮率が表示される

**確認項目**:
- [ ] 圧縮バックアップ成功
- [ ] ファイルサイズが削減されている
- [ ] 圧縮率が妥当（30-70%削減が一般的）

---

### Test 3.2: 大容量ファイルバックアップ

**目的**: 大容量ファイル（100MB以上）のバックアップが正常に動作することを確認

**手順**:
```bash
# 100MBファイルを作成
dd if=/dev/urandom of=source/huge_file.bin bs=1024 count=102400

rm -rf backup/*

# バックアップ実行（時間計測）
time ./target/release/backup-suite backup source/ backup/ --compression zstd

# サイズ確認
ls -lh source/huge_file.bin
ls -lh backup/huge_file.bin*
```

**期待結果**:
- ✅ バックアップが完了
- ✅ 進捗表示が正しく動作
- ✅ メモリ使用量が一定範囲内（100MB以下）

**確認項目**:
- [ ] 大容量ファイルのバックアップ成功
- [ ] 進捗表示が正常
- [ ] メモリ使用量が妥当

---

### Test 3.3: 並列処理の確認

**目的**: 複数ファイルの並列バックアップが機能することを確認

**手順**:
```bash
# 多数の小さいファイルを作成
for i in {1..100}; do
    echo "File $i content" > source/file_$i.txt
done

rm -rf backup/*

# 並列バックアップ実行
time ./target/release/backup-suite backup source/ backup/

# CPU使用率確認（別ターミナルで）
# top コマンドで複数コアが使用されていることを確認
```

**期待結果**:
- ✅ 全ファイルがバックアップされる
- ✅ 複数CPUコアが使用される
- ✅ 処理時間が妥当

**確認項目**:
- [ ] 100ファイル全てバックアップ
- [ ] 並列処理動作（複数コア使用）
- [ ] 処理速度が妥当

---

## エラーハンドリングテスト

### Test 4.1: 存在しないディレクトリ指定

**目的**: 存在しないディレクトリを指定した場合の適切なエラー処理を確認

**手順**:
```bash
# 存在しないソースディレクトリを指定
./target/release/backup-suite backup /nonexistent/ backup/

# 存在しないバックアップディレクトリから復元
./target/release/backup-suite restore /nonexistent/ restore/
```

**期待結果**:
- ✅ 適切なエラーメッセージが表示される
- ✅ プログラムが異常終了しない（クリーンな終了）

**確認項目**:
- [ ] エラーメッセージが分かりやすい
- [ ] exit code が0以外
- [ ] クラッシュしない

---

### Test 4.2: 権限エラーハンドリング

**目的**: 読み取り/書き込み権限がない場合の適切なエラー処理を確認

**手順**:
```bash
# 読み取り不可ファイルを作成
echo "protected" > source/protected.txt
chmod 000 source/protected.txt

# バックアップ実行
./target/release/backup-suite backup source/ backup/

# 権限を戻す
chmod 644 source/protected.txt
```

**期待結果**:
- ✅ 権限エラーが適切に報告される
- ✅ 他のファイルのバックアップは続行される

**確認項目**:
- [ ] 権限エラーメッセージ表示
- [ ] 他のファイルは正常にバックアップ
- [ ] エラーファイルのリストが表示される

---

### Test 4.3: ディスク容量不足のシミュレーション

**目的**: ディスク容量が不足した場合のエラーハンドリングを確認

**注**: 実際のディスク容量不足は危険なため、ログメッセージの確認のみ

**手順**:
```bash
# 小さいディスクイメージを作成してマウント（macOS/Linux）
# または、設定ファイルでmax_sizeを小さく設定

# バックアップ実行
./target/release/backup-suite backup source/ backup/
```

**期待結果**:
- ✅ ディスク容量チェックが実行される
- ✅ 容量不足の場合は適切なエラー

**確認項目**:
- [ ] ディスク容量チェック実行
- [ ] エラーメッセージが適切

---

## ユーザビリティテスト

### Test 5.1: ヘルプメッセージの確認

**目的**: ヘルプメッセージが分かりやすく、完全であることを確認

**手順**:
```bash
# メインヘルプ
./target/release/backup-suite --help

# backupサブコマンドヘルプ
./target/release/backup-suite backup --help

# restoreサブコマンドヘルプ
./target/release/backup-suite restore --help

# listサブコマンドヘルプ
./target/release/backup-suite list --help
```

**期待結果**:
- ✅ 全コマンドの説明が表示される
- ✅ オプションの説明が明確
- ✅ 使用例が含まれる

**確認項目**:
- [ ] ヘルプが読みやすい
- [ ] 全オプションが説明されている
- [ ] 日本語・英語両対応

---

### Test 5.2: バージョン情報の確認

**目的**: バージョン情報が正しく表示されることを確認

**手順**:
```bash
./target/release/backup-suite --version
```

**期待結果**:
- ✅ `backup-suite X.Y.Z` の形式で表示

**確認項目**:
- [ ] バージョン番号が正しい
- [ ] フォーマットが統一されている

---

### Test 5.3: 進捗表示の確認

**目的**: 進捗表示が正しく動作することを確認

**手順**:
```bash
# 進捗表示ありでバックアップ
./target/release/backup-suite backup source/ backup/ --progress

# 大容量ファイルで進捗を視覚的に確認
dd if=/dev/urandom of=source/large.bin bs=1024 count=51200
./target/release/backup-suite backup source/ backup/ --progress
```

**期待結果**:
- ✅ プログレスバーが表示される
- ✅ パーセンテージが更新される
- ✅ 処理ファイル名が表示される

**確認項目**:
- [ ] プログレスバー表示
- [ ] リアルタイム更新
- [ ] 完了時に100%

---

### Test 5.4: エラーメッセージの分かりやすさ

**目的**: エラーメッセージがユーザーにとって分かりやすいことを確認

**手順**:
```bash
# 引数不足
./target/release/backup-suite backup

# 不正なオプション
./target/release/backup-suite backup source/ backup/ --invalid-option

# 不正な圧縮形式
./target/release/backup-suite backup source/ backup/ --compression invalid
```

**期待結果**:
- ✅ 各エラーで適切なメッセージが表示される
- ✅ 解決方法のヒントが含まれる

**確認項目**:
- [ ] エラーメッセージが明確
- [ ] ヒント・解決策が提示される
- [ ] 日本語で分かりやすい

---

## 統合シナリオテスト

### Scenario 1: 実務的なバックアップフロー

**シナリオ**: 開発者が日次バックアップを実行

**手順**:
```bash
# Day 1: 初回バックアップ（暗号化+圧縮）
./target/release/backup-suite backup ~/projects/ ~/backups/daily-backup/ \
    --password "MySecurePassword2024" \
    --compression zstd \
    --progress

# Day 2: ファイル変更後の2回目バックアップ
# (ファイルを一部変更)
echo "New content" >> ~/projects/file.txt

./target/release/backup-suite backup ~/projects/ ~/backups/daily-backup-2/ \
    --password "MySecurePassword2024" \
    --compression zstd

# 復元が必要になった場合
./target/release/backup-suite restore ~/backups/daily-backup-2/ ~/restored-projects/ \
    --password "MySecurePassword2024"

# 復元確認
diff -r ~/projects/ ~/restored-projects/
```

**期待結果**:
- ✅ 全操作が正常完了
- ✅ 復元されたファイルが元と一致

**確認項目**:
- [ ] 複数回のバックアップ成功
- [ ] 復元が正確
- [ ] パフォーマンスが許容範囲

---

### Scenario 2: 災害復旧シナリオ

**シナリオ**: システムクラッシュ後の完全復旧

**手順**:
```bash
# 1. 重要データのバックアップ
./target/release/backup-suite backup ~/important-data/ ~/disaster-recovery/ \
    --password "EmergencyPassword" \
    --compression zstd \
    --verify

# 2. システムクラッシュをシミュレート（重要データ削除）
# rm -rf ~/important-data/*  # 注意: テスト環境のみで実行

# 3. バックアップから復旧
./target/release/backup-suite restore ~/disaster-recovery/ ~/important-data/ \
    --password "EmergencyPassword" \
    --verify

# 4. データ整合性確認
# SHA256ハッシュやdiffで確認
```

**期待結果**:
- ✅ データが完全に復旧される
- ✅ 整合性検証が成功

**確認項目**:
- [ ] 完全復旧成功
- [ ] データロスなし
- [ ] 検証機能が動作

---

### Scenario 3: カテゴリフィルタリングシナリオ

**シナリオ**: ドキュメントファイルのみをバックアップ

**手順**:
```bash
# 混在ファイルを作成
mkdir -p source-mixed
echo "Document" > source-mixed/report.pdf
echo "Code" > source-mixed/main.rs
echo "Image" > source-mixed/photo.png

# ドキュメントのみバックアップ
./target/release/backup-suite backup source-mixed/ backup-docs/ \
    --category documents

# バックアップ内容確認
ls backup-docs/
```

**期待結果**:
- ✅ ドキュメントファイルのみがバックアップされる
- ✅ 他のファイルタイプは除外される

**確認項目**:
- [ ] フィルタリング動作
- [ ] 正しいファイルのみバックアップ

---

## テスト結果記録テンプレート

### テスト実施記録

| Test ID | テスト名 | 実施日 | 結果 | 備考 |
|---------|---------|--------|------|------|
| 1.1 | 基本バックアップ | YYYY-MM-DD | ✅/❌ |  |
| 1.2 | 基本復元 | YYYY-MM-DD | ✅/❌ |  |
| 1.3 | ファイル完全性 | YYYY-MM-DD | ✅/❌ |  |
| 2.1 | 暗号化バックアップ | YYYY-MM-DD | ✅/❌ |  |
| 2.2 | 暗号化復元（正） | YYYY-MM-DD | ✅/❌ |  |
| 2.3 | 暗号化復元（誤） | YYYY-MM-DD | ✅/❌ |  |
| 3.1 | Zstd圧縮 | YYYY-MM-DD | ✅/❌ |  |
| 3.2 | 大容量ファイル | YYYY-MM-DD | ✅/❌ |  |
| 4.1 | 存在しないディレクトリ | YYYY-MM-DD | ✅/❌ |  |
| 5.1 | ヘルプメッセージ | YYYY-MM-DD | ✅/❌ |  |
| S1 | 実務フロー | YYYY-MM-DD | ✅/❌ |  |
| S2 | 災害復旧 | YYYY-MM-DD | ✅/❌ |  |

---

## 不具合報告テンプレート

### 不具合発見時の記録フォーマット

```markdown
## 不具合タイトル

**発見日時**: YYYY-MM-DD HH:MM
**テストID**: X.X
**重大度**: Critical / High / Medium / Low

### 再現手順
1.
2.
3.

### 期待結果


### 実際の結果


### 環境情報
- OS:
- バージョン:
- その他:

### 追加情報

```

---

## チェックリスト総括

### 全テスト完了確認

- [ ] 基本機能テスト（1.1-1.3）全て完了
- [ ] セキュリティ機能テスト（2.1-2.4）全て完了
- [ ] パフォーマンステスト（3.1-3.3）全て完了
- [ ] エラーハンドリングテスト（4.1-4.3）全て完了
- [ ] ユーザビリティテスト（5.1-5.4）全て完了
- [ ] 統合シナリオテスト（S1-S3）全て完了

### 品質基準

- [ ] クリティカルバグ: 0件
- [ ] ハイ優先度バグ: 0件
- [ ] ミディアム優先度バグ: 許容範囲内
- [ ] ユーザビリティ問題: 許容範囲内

---

**最終確認者**: __________________
**確認日**: YYYY-MM-DD
**総合評価**: ✅ 合格 / ❌ 不合格 / ⚠️ 条件付き合格
