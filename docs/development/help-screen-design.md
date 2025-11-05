# Backup Suite ヘルプ画面デザインガイド

## デザイン仕様書

### 1. レイアウト構造

```
┌─────────────────────────────────────────────────────────┐
│ Backup Suite v1.0.0                           [GREEN]   │
│ 高速ローカルバックアップツール - Rust製・型安全・高性能   │
│                                                          │
│ 使用方法:                                     [MAGENTA]  │
│   backup-suite <コマンド> [オプション]                  │
│                                                          │
│ 📋 基本コマンド                               [MAGENTA]  │
│   add          対象追加（インタラクティブ選択対応）      │
│   list, ls     一覧表示                       [YELLOW]   │
│   remove       対象削除（インタラクティブ選択対応）      │
│   clear        一括削除                                 │
│                                                          │
│ 🚀 実行コマンド                               [MAGENTA]  │
│   run          バックアップ実行               [YELLOW]   │
│   restore      復元                                     │
│   cleanup      古いバックアップ削除                     │
│                                                          │
│ 📊 情報表示                                   [MAGENTA]  │
│   status       ステータス表示                 [YELLOW]   │
│   history      履歴表示                                 │
│   dashboard    ダッシュボード表示                       │
│                                                          │
│ ⚙️  設定管理                                  [MAGENTA]  │
│   enable       自動バックアップ有効化         [YELLOW]   │
│   disable      自動バックアップ無効化                   │
│   schedule     スケジュール管理                         │
│                                                          │
│ 🔧 ユーティリティ                             [MAGENTA]  │
│   open         バックアップディレクトリを開く [YELLOW]   │
│   version      バージョン表示                           │
│   completion   シェル補完スクリプト生成                 │
│                                                          │
│ オプション:                                   [MAGENTA]  │
│   -h, --help       このヘルプを表示                     │
│   -V, --version    バージョン情報を表示                 │
│                                                          │
│ 使用例:                                       [MAGENTA]  │
│   # インタラクティブでファイルを追加          [GRAY]     │
│   backup-suite add --interactive                        │
│                                                          │
│   # 高優先度のバックアップを実行              [GRAY]     │
│   backup-suite run --priority high                      │
│                                                          │
│   # 30日以上前のバックアップを削除（ドライラン）[GRAY]   │
│   backup-suite cleanup --days 30 --dry-run              │
│                                                          │
│   # スケジュールを設定して有効化              [GRAY]     │
│   backup-suite schedule setup --high daily --medium ... │
│   backup-suite schedule enable                          │
│                                                          │
│ 詳細情報:                                     [MAGENTA]  │
│   各コマンドの詳細: backup-suite <コマンド> --help      │
│   設定ファイル: ~/.config/backup-suite/config.toml      │
│   バックアップ先: ~/.local/share/backup-suite/backups/  │
└─────────────────────────────────────────────────────────┘
```

### 2. カラーパレット

```rust
// ANSIカラーコード
const GREEN:   &str = "\x1b[32m";  // #00FF00 - タイトル、成功
const YELLOW:  &str = "\x1b[33m";  // #FFFF00 - コマンド名
const MAGENTA: &str = "\x1b[35m";  // #FF00FF - セクション見出し
const GRAY:    &str = "\x1b[90m";  // #808080 - コメント
const RESET:   &str = "\x1b[0m";   // リセット
```

### 3. タイポグラフィ

```
フォント: モノスペース（ターミナルデフォルト）
行間: 1行（標準）
セクション間: 1空行（視覚的分離）
インデント: 2スペース（統一）

文字幅:
- コマンド名: 12文字固定（左寄せ）
- 説明: 残りの幅（可変）
- 使用例コメント: 40文字程度
```

### 4. 絵文字マッピング

| カテゴリ | 絵文字 | Unicode | 意味 |
|----------|--------|---------|------|
| 基本コマンド | 📋 | U+1F4CB | クリップボード・管理 |
| 実行コマンド | 🚀 | U+1F680 | ロケット・アクション |
| 情報表示 | 📊 | U+1F4CA | グラフ・データ可視化 |
| 設定管理 | ⚙️ | U+2699 | 歯車・設定 |
| ユーティリティ | 🔧 | U+1F527 | レンチ・ツール |

### 5. レスポンシブデザイン

```
最小幅: 80文字（標準ターミナル）
推奨幅: 100文字（快適な閲覧）
最大幅: 120文字（広い画面）

折り返し戦略:
- 80文字未満: 説明テキストを2行に分割
- 100文字: デフォルトレイアウト
- 120文字以上: 横並びオプション表示
```

## 実装ガイドライン

### コード構造

```rust
fn print_help() {
    // 1. カラー初期化
    let green = get_color("green");
    let yellow = get_color("yellow");
    let magenta = get_color("magenta");
    let gray = get_color("gray");
    let reset = get_color("reset");

    // 2. タイトルセクション
    println!("{}Backup Suite v1.0.0{}", green, reset);
    println!("高速ローカルバックアップツール - Rust製・型安全・高性能");
    println!();

    // 3. 使用方法セクション
    println!("{}使用方法:{}", magenta, reset);
    println!("  backup-suite <コマンド> [オプション]");
    println!();

    // 4. コマンドカテゴリ（5セクション）
    print_category("📋 基本コマンド", &[
        ("add", "対象追加（インタラクティブ選択対応）"),
        ("list, ls", "一覧表示"),
        // ...
    ], magenta, yellow, reset);

    // 5. オプションセクション
    // 6. 使用例セクション
    // 7. 詳細情報セクション
}

fn print_category(
    title: &str,
    commands: &[(&str, &str)],
    title_color: &str,
    cmd_color: &str,
    reset: &str
) {
    println!("{}{}{}", title_color, title, reset);
    for (cmd, desc) in commands {
        println!("  {}{:12}{} {}", cmd_color, cmd, reset, desc);
    }
    println!();
}
```

### テスト項目

```bash
# 1. 基本動作
backup-suite --help
backup-suite -h

# 2. カラー出力（ターミナル）
backup-suite --help | less -R

# 3. カラーなし環境
NO_COLOR=1 backup-suite --help

# 4. パイプ出力
backup-suite --help | cat

# 5. リダイレクト
backup-suite --help > help.txt

# 6. 幅制限
backup-suite --help | fold -w 80
```

## アクセシビリティ

### 1. カラー依存排除

```
✅ カラーなしでも情報は完全
✅ 絵文字はセマンティックな補助（必須でない）
✅ インデントで構造を視覚化
```

### 2. スクリーンリーダー対応

```
✅ 平文テキストで全情報提供
✅ 論理的な階層構造
✅ 明確な見出しテキスト
```

### 3. 視覚障害対応

```
✅ 高コントラスト（明度差200%以上）
✅ カラーブラインドフレンドリー
   - 赤/緑の二項対立を避ける
   - magenta/yellow/gray の区別可能な組み合わせ
```

## パフォーマンス

```rust
// ベンチマーク目標
Time to display:  < 5ms
Memory usage:     < 1KB
Binary size impact: < 2KB
```

## 国際化（将来対応）

```rust
// 言語別ヘルプ文字列
struct HelpText {
    ja: &'static str,  // 日本語（デフォルト）
    en: &'static str,  // 英語
}

fn print_help_i18n(lang: &str) {
    match lang {
        "ja" | "ja_JP" => print_help_ja(),
        "en" | "en_US" => print_help_en(),
        _ => print_help_ja(), // デフォルト
    }
}
```

## メンテナンス

### コマンド追加時のチェックリスト

- [ ] 適切なカテゴリに分類
- [ ] 12文字以内の短いコマンド名
- [ ] 簡潔な日本語説明（30文字以内）
- [ ] 使用例が必要な場合は追加
- [ ] アルファベット順（カテゴリ内）

### 定期レビュー

```
月次:
- カテゴリ分類の妥当性確認
- 使用例の実用性確認
- ユーザーフィードバックの反映

四半期:
- 国際化対応の検討
- アクセシビリティ監査
- デザイントレンドの調査
```

## ベストプラクティス

### ✅ DO（推奨）

1. **簡潔な説明**: 30文字以内で要点を伝える
2. **実用的な例**: コピペで動く実例を提供
3. **明確な階層**: セクション間に空行を入れる
4. **一貫性**: インデント・カラー・用語を統一

### ❌ DON'T（非推奨）

1. **冗長な説明**: 「このコマンドは...を行います」的な回りくどさ
2. **抽象的な例**: `<FILE>` や `[OPTIONS]` だけの例
3. **フラットな構造**: すべてを同じレベルに並べる
4. **カラー乱用**: 3色以上の装飾

## 参考実装

### 類似ツールとの比較

| ツール | カテゴリ分類 | 使用例 | 日本語 | 評価 |
|--------|------------|--------|--------|------|
| cargo | ✅ | ✅ | ✅ | 優秀 |
| git | ✅ | ❌ | ❌ | 良好 |
| docker | ✅ | ✅ | ❌ | 良好 |
| backup-suite | ✅ | ✅ | ✅ | 目標 |

## 結論

このヘルプ画面デザインは：

1. **統一性** - 引数なし実行時と完全に統一された視覚スタイル
2. **効率性** - 5秒以内に目的のコマンドを発見可能
3. **教育性** - 初心者が即座に真似できる実用例
4. **完全性** - 設定ファイル位置まで含む包括的情報

これにより、CLIツールとしての完成度を大幅に向上させました。
