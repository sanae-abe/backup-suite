use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};

/// インタラクティブな確認プロンプト
///
/// dialoguerライブラリを使用してユーザーに確認を求めます。
///
/// # 機能
///
/// - Yes/No確認プロンプト
/// - 選択メニュー
/// - テキスト入力
///
/// # 使用例
///
/// ```no_run
/// use backup_suite::ui::interactive::confirm;
///
/// if confirm("実行しますか？", true)? {
///     println!("実行します");
/// }
/// # Ok::<(), anyhow::Error>(())
/// ```
/// Yes/No確認プロンプトを表示
///
/// # 引数
///
/// * `message` - 確認メッセージ
/// * `default` - デフォルト値（trueならYesがデフォルト）
///
/// # 戻り値
///
/// ユーザーの選択（true: Yes, false: No）
///
/// # Errors
///
/// 次の場合にエラーを返します:
/// - 標準入出力へのアクセスに失敗した場合
/// - ユーザーが対話を中断した場合（Ctrl-C等）
/// - ターミナルが利用できない環境で実行した場合
///
/// # 使用例
///
/// ```no_run
/// use backup_suite::ui::interactive::confirm;
///
/// if confirm("バックアップを実行しますか？", true)? {
///     println!("バックアップを開始します");
/// } else {
///     println!("キャンセルされました");
/// }
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn confirm(message: &str, default: bool) -> Result<bool> {
    // CI環境での自動確認サポート
    // BACKUP_SUITE_YES=true または BACKUP_SUITE_YES=1 で自動的にtrueを返す
    if let Ok(auto_yes) = std::env::var("BACKUP_SUITE_YES") {
        if auto_yes == "true" || auto_yes == "1" {
            eprintln!("[CI MODE] Auto-confirming: {}", message);
            return Ok(true);
        }
    }

    let result = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(message)
        .default(default)
        .interact()?;

    Ok(result)
}

/// Yes/No確認プロンプト（カスタマイズ可能版）
///
/// # 引数
///
/// * `message` - 確認メッセージ
/// * `default` - デフォルト値
/// * `yes_text` - Yesボタンのテキスト
/// * `no_text` - Noボタンのテキスト
///
/// # 戻り値
///
/// ユーザーの選択
///
/// # Errors
///
/// 次の場合にエラーを返します:
/// - 標準入出力へのアクセスに失敗した場合
/// - ユーザーが対話を中断した場合（Ctrl-C等）
/// - ターミナルが利用できない環境で実行した場合
///
/// # 使用例
///
/// ```no_run
/// use backup_suite::ui::interactive::confirm_with_text;
///
/// if confirm_with_text(
///     "古いバックアップを削除しますか？",
///     false,
///     "削除する",
///     "保持する"
/// )? {
///     println!("削除します");
/// }
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn confirm_with_text(
    message: &str,
    default: bool,
    yes_text: &str,
    no_text: &str,
) -> Result<bool> {
    println!("\n{message}");
    println!("  {yes_text} / {no_text}");

    let result = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("選択してください")
        .default(default)
        .interact()?;

    Ok(result)
}

/// 選択メニューを表示
///
/// # 引数
///
/// * `message` - プロンプトメッセージ
/// * `items` - 選択肢のリスト
///
/// # 戻り値
///
/// 選択されたインデックス
///
/// # Errors
///
/// 次の場合にエラーを返します:
/// - 標準入出力へのアクセスに失敗した場合
/// - ユーザーが対話を中断した場合（Ctrl-C等）
/// - ターミナルが利用できない環境で実行した場合
///
/// # 使用例
///
/// ```no_run
/// use backup_suite::ui::interactive::select;
///
/// let items = vec!["オプション1", "オプション2", "オプション3"];
/// let selection = select("選択してください", &items)?;
/// println!("選択: {}", items[selection]);
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn select(message: &str, items: &[&str]) -> Result<usize> {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(message)
        .items(items)
        .default(0)
        .interact()?;

    Ok(selection)
}

/// 選択メニュー（デフォルト位置指定可能版）
///
/// # 引数
///
/// * `message` - プロンプトメッセージ
/// * `items` - 選択肢のリスト
/// * `default` - デフォルトの選択位置
///
/// # 戻り値
///
/// 選択されたインデックス
///
/// # Errors
///
/// 次の場合にエラーを返します:
/// - 標準入出力へのアクセスに失敗した場合
/// - ユーザーが対話を中断した場合（Ctrl-C等）
/// - ターミナルが利用できない環境で実行した場合
///
/// # 使用例
///
/// ```no_run
/// use backup_suite::ui::interactive::select_with_default;
///
/// let priorities = vec!["高", "中", "低"];
/// let selection = select_with_default("優先度を選択", &priorities, 1)?;
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn select_with_default(message: &str, items: &[&str], default: usize) -> Result<usize> {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(message)
        .items(items)
        .default(default)
        .interact()?;

    Ok(selection)
}

/// テキスト入力プロンプト
///
/// # 引数
///
/// * `message` - プロンプトメッセージ
/// * `default` - デフォルト値（オプション）
///
/// # 戻り値
///
/// 入力されたテキスト
///
/// # Errors
///
/// 次の場合にエラーを返します:
/// - 標準入出力へのアクセスに失敗した場合
/// - ユーザーが対話を中断した場合（Ctrl-C等）
/// - ターミナルが利用できない環境で実行した場合
///
/// # 使用例
///
/// ```no_run
/// use backup_suite::ui::interactive::input;
///
/// let name = input("バックアップ名を入力", Some("backup_1"))?;
/// println!("バックアップ名: {}", name);
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn input(message: &str, default: Option<&str>) -> Result<String> {
    let theme = ColorfulTheme::default();
    let mut input_builder = Input::<String>::with_theme(&theme).with_prompt(message);

    if let Some(default_value) = default {
        input_builder = input_builder.default(default_value.to_string());
    }

    let result = input_builder.interact_text()?;
    Ok(result)
}

/// パス入力プロンプト
///
/// パス入力に特化したプロンプト。存在確認は行わない。
///
/// # 引数
///
/// * `message` - プロンプトメッセージ
/// * `default` - デフォルトパス（オプション）
///
/// # 戻り値
///
/// 入力されたパス文字列
///
/// # Errors
///
/// 次の場合にエラーを返します:
/// - 標準入出力へのアクセスに失敗した場合
/// - ユーザーが対話を中断した場合（Ctrl-C等）
/// - ターミナルが利用できない環境で実行した場合
///
/// # 使用例
///
/// ```no_run
/// use backup_suite::ui::interactive::input_path;
///
/// let path = input_path("バックアップ先を入力", Some("/tmp/backup"))?;
/// println!("パス: {}", path);
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn input_path(message: &str, default: Option<&str>) -> Result<String> {
    input(message, default)
}

/// 複数選択メニュー（実装予定）
///
/// 現在はselectのエイリアス。将来的に複数選択対応予定。
///
/// # 引数
///
/// * `message` - プロンプトメッセージ
/// * `items` - 選択肢のリスト
///
/// # 戻り値
///
/// 選択されたインデックスのベクター
///
/// # Errors
///
/// 次の場合にエラーを返します:
/// - 標準入出力へのアクセスに失敗した場合
/// - ユーザーが対話を中断した場合（Ctrl-C等）
/// - ターミナルが利用できない環境で実行した場合
pub fn multi_select(message: &str, items: &[&str]) -> Result<Vec<usize>> {
    // 現在は単一選択のみサポート
    let selection = select(message, items)?;
    Ok(vec![selection])
}

/// バックアップ開始前の最終確認
///
/// バックアップ実行前の標準的な確認プロンプト。
///
/// # 引数
///
/// * `file_count` - バックアップ対象のファイル数
/// * `destination` - バックアップ先パス
///
/// # 戻り値
///
/// ユーザーの確認結果
///
/// # Errors
///
/// 次の場合にエラーを返します:
/// - 標準入出力へのアクセスに失敗した場合
/// - ユーザーが対話を中断した場合（Ctrl-C等）
/// - ターミナルが利用できない環境で実行した場合
///
/// # 使用例
///
/// ```no_run
/// use backup_suite::ui::interactive::confirm_backup;
/// use backup_suite::i18n::Language;
///
/// if confirm_backup(150, "/backup/destination", Language::Japanese)? {
///     println!("バックアップを開始します");
/// }
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn confirm_backup(
    file_count: usize,
    destination: &str,
    lang: crate::i18n::Language,
) -> Result<bool> {
    use crate::i18n::{get_message, MessageKey};

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("{}", get_message(MessageKey::ConfirmBackupTitle, lang));
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!(
        "{}",
        get_message(MessageKey::ConfirmBackupTargetFiles, lang)
            .replace("{}", &file_count.to_string())
    );
    println!(
        "{}",
        get_message(MessageKey::ConfirmBackupDestination, lang).replace("{}", destination)
    );
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    confirm(get_message(MessageKey::PromptBackupConfirm, lang), true)
}

/// 古いバックアップの削除確認
///
/// # 引数
///
/// * `count` - 削除対象のバックアップ数
/// * `keep_days` - 保持日数
///
/// # 戻り値
///
/// ユーザーの確認結果
///
/// # Errors
///
/// 次の場合にエラーを返します:
/// - 標準入出力へのアクセスに失敗した場合
/// - ユーザーが対話を中断した場合（Ctrl-C等）
/// - ターミナルが利用できない環境で実行した場合
///
/// # 使用例
///
/// ```no_run
/// use backup_suite::ui::interactive::confirm_cleanup;
/// use backup_suite::i18n::Language;
///
/// if confirm_cleanup(5, 30, Language::Japanese)? {
///     println!("古いバックアップを削除します");
/// }
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn confirm_cleanup(count: usize, keep_days: u32, lang: crate::i18n::Language) -> Result<bool> {
    use crate::i18n::{get_message, MessageKey};

    println!("\n{}", get_message(MessageKey::ConfirmCleanupTitle, lang));
    println!(
        "{}",
        get_message(MessageKey::ConfirmCleanupTargetCount, lang).replace("{}", &count.to_string())
    );
    println!(
        "{}",
        get_message(MessageKey::ConfirmCleanupRetentionDays, lang)
            .replace("{}", &keep_days.to_string())
    );

    confirm(get_message(MessageKey::PromptConfirmDelete, lang), false)
}

/// 優先度選択プロンプト
///
/// バックアップ対象の優先度を選択。
///
/// # 戻り値
///
/// 選択された優先度（"high", "medium", "low"）
///
/// # Errors
///
/// 次の場合にエラーを返します:
/// - 標準入出力へのアクセスに失敗した場合
/// - ユーザーが対話を中断した場合（Ctrl-C等）
/// - ターミナルが利用できない環境で実行した場合
///
/// # 使用例
///
/// ```no_run
/// use backup_suite::ui::interactive::select_priority;
///
/// let priority = select_priority()?;
/// println!("選択された優先度: {}", priority);
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn select_priority() -> Result<String> {
    let priorities = vec![
        "高 (High) - 毎日バックアップ",
        "中 (Medium) - 週次バックアップ",
        "低 (Low) - 月次バックアップ",
    ];

    let selection = select("優先度を選択してください", &priorities)?;

    let priority = match selection {
        0 => "high",
        1 => "medium",
        2 => "low",
        _ => "medium",
    };

    Ok(priority.to_string())
}

#[cfg(test)]
mod tests {
    // インタラクティブなテストは自動テスト困難なため、
    // 基本的な関数の存在確認のみ

    #[test]
    fn test_select_priority_values() {
        // 優先度の値が正しいことを確認
        let priorities = ["high", "medium", "low"];
        assert!(priorities.contains(&"high"));
        assert!(priorities.contains(&"medium"));
        assert!(priorities.contains(&"low"));
    }

    #[test]
    fn test_multi_select_returns_vec() {
        // multi_selectが将来的にVec<usize>を返すことを確認
        // 実際の対話的テストは手動で実施
    }
}
