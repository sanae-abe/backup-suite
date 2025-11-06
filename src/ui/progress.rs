use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::Arc;
use std::time::Duration;

/// バックアップ進捗表示機能
///
/// indicatifライブラリを使用してリアルタイムの進捗状況を表示します。
///
/// # 機能
///
/// - メインプログレスバー: 全体の進捗を表示
/// - 詳細プログレスバー: 現在処理中のファイル情報を表示
/// - 経過時間と推定残り時間を表示
///
/// # 使用例
///
/// ```no_run
/// use backup_suite::ui::progress::BackupProgress;
///
/// let progress = BackupProgress::new(100);
/// progress.set_message("処理中: /path/to/file.txt");
/// progress.inc(1);
/// progress.finish("バックアップ完了");
/// ```
#[derive(Clone)]
pub struct BackupProgress {
    #[allow(dead_code)]
    multi: Arc<MultiProgress>,
    main_bar: ProgressBar,
    detail_bar: ProgressBar,
}

impl BackupProgress {
    /// 新しいBackupProgressインスタンスを作成
    ///
    /// # 引数
    ///
    /// * `total_files` - バックアップ対象の総ファイル数
    ///
    /// # 戻り値
    ///
    /// BackupProgressインスタンス
    ///
    /// # 使用例
    ///
    /// ```no_run
    /// use backup_suite::ui::progress::BackupProgress;
    ///
    /// let progress = BackupProgress::new(100);
    /// ```
    pub fn new(total_files: u64) -> Self {
        let multi = Arc::new(MultiProgress::new());

        // メインプログレスバー
        let main_bar = multi.add(ProgressBar::new(total_files));
        main_bar.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ファイル {msg}")
                .unwrap()
                .progress_chars("█▉▊▋▌▍▎▏  "),
        );

        // 詳細プログレスバー
        let detail_bar = multi.add(ProgressBar::new(0));
        detail_bar.set_style(ProgressStyle::default_bar().template("{wide_msg}").unwrap());

        Self {
            multi,
            main_bar,
            detail_bar,
        }
    }

    /// プログレスバーを指定量進める
    ///
    /// # 引数
    ///
    /// * `delta` - 進める量（通常は1）
    ///
    /// # 使用例
    ///
    /// ```no_run
    /// use backup_suite::ui::progress::BackupProgress;
    ///
    /// let progress = BackupProgress::new(100);
    /// progress.inc(1); // 1つ進める
    /// ```
    pub fn inc(&self, delta: u64) {
        self.main_bar.inc(delta);
    }

    /// 詳細メッセージを設定
    ///
    /// 現在処理中のファイルや操作の詳細を表示します。
    ///
    /// # 引数
    ///
    /// * `msg` - 表示するメッセージ
    ///
    /// # 使用例
    ///
    /// ```no_run
    /// use backup_suite::ui::progress::BackupProgress;
    ///
    /// let progress = BackupProgress::new(100);
    /// progress.set_message("処理中: /path/to/file.txt");
    /// ```
    pub fn set_message(&self, msg: &str) {
        self.detail_bar.set_message(msg.to_string());
    }

    /// メインプログレスバーのメッセージを設定
    ///
    /// # 引数
    ///
    /// * `msg` - 表示するメッセージ
    ///
    /// # 使用例
    ///
    /// ```no_run
    /// use backup_suite::ui::progress::BackupProgress;
    ///
    /// let progress = BackupProgress::new(100);
    /// progress.set_main_message("高優先度ファイル処理中");
    /// ```
    pub fn set_main_message(&self, msg: &str) {
        self.main_bar.set_message(msg.to_string());
    }

    /// プログレスバーを完了させる
    ///
    /// 最終メッセージを表示してプログレスバーを終了します。
    ///
    /// # 引数
    ///
    /// * `msg` - 完了メッセージ
    ///
    /// # 使用例
    ///
    /// ```no_run
    /// use backup_suite::ui::progress::BackupProgress;
    ///
    /// let progress = BackupProgress::new(100);
    /// // ... 処理 ...
    /// progress.finish("バックアップ完了！");
    /// ```
    pub fn finish(&self, msg: &str) {
        self.main_bar.finish_with_message(msg.to_string());
        self.detail_bar.finish_and_clear();
    }

    /// 現在の位置を設定
    ///
    /// # 引数
    ///
    /// * `pos` - 新しい位置
    ///
    /// # 使用例
    ///
    /// ```no_run
    /// use backup_suite::ui::progress::BackupProgress;
    ///
    /// let progress = BackupProgress::new(100);
    /// progress.set_position(50); // 50%に設定
    /// ```
    pub fn set_position(&self, pos: u64) {
        self.main_bar.set_position(pos);
    }

    /// 総数を設定
    ///
    /// 処理中に総ファイル数が判明した場合に使用します。
    ///
    /// # 引数
    ///
    /// * `len` - 新しい総数
    ///
    /// # 使用例
    ///
    /// ```no_run
    /// use backup_suite::ui::progress::BackupProgress;
    ///
    /// let progress = BackupProgress::new(0);
    /// progress.set_length(150); // 実際の総数が判明
    /// ```
    pub fn set_length(&self, len: u64) {
        self.main_bar.set_length(len);
    }

    /// プログレスバーを非表示にして完了
    ///
    /// メッセージを表示せずに終了します。
    ///
    /// # 使用例
    ///
    /// ```no_run
    /// use backup_suite::ui::progress::BackupProgress;
    ///
    /// let progress = BackupProgress::new(100);
    /// progress.finish_and_clear();
    /// ```
    pub fn finish_and_clear(&self) {
        self.main_bar.finish_and_clear();
        self.detail_bar.finish_and_clear();
    }

    /// スピナーモードのプログレスバーを作成
    ///
    /// ファイル数が不明な場合に使用するスピナー表示。
    ///
    /// # 戻り値
    ///
    /// スピナーモードのBackupProgress
    ///
    /// # 使用例
    ///
    /// ```no_run
    /// use backup_suite::ui::progress::BackupProgress;
    ///
    /// let progress = BackupProgress::new_spinner();
    /// progress.set_message("ファイル検索中...");
    /// // ... 処理 ...
    /// progress.finish("検索完了");
    /// ```
    pub fn new_spinner() -> Self {
        let multi = Arc::new(MultiProgress::new());

        let main_bar = multi.add(ProgressBar::new_spinner());
        main_bar.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .unwrap()
                .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ "),
        );
        main_bar.enable_steady_tick(Duration::from_millis(120));

        let detail_bar = multi.add(ProgressBar::new(0));
        detail_bar.set_style(ProgressStyle::default_bar().template("{wide_msg}").unwrap());

        Self {
            multi,
            main_bar,
            detail_bar,
        }
    }
}

/// シンプルなプログレスバーを作成
///
/// 単純な進捗表示が必要な場合の便利関数。
///
/// # 引数
///
/// * `total` - 総数
/// * `message` - 表示メッセージ
///
/// # 戻り値
///
/// ProgressBarインスタンス
///
/// # 使用例
///
/// ```no_run
/// use backup_suite::ui::progress::create_progress_bar;
///
/// let pb = create_progress_bar(100, "処理中");
/// for _ in 0..100 {
///     pb.inc(1);
/// }
/// pb.finish_with_message("完了");
/// ```
pub fn create_progress_bar(total: u64, message: &str) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(&format!("{{spinner:.green}} {} [{{elapsed_precise}}] {{bar:40.cyan/blue}} {{pos}}/{{len}} {{msg}}", message))
            .unwrap()
            .progress_chars("█▉▊▋▌▍▎▏  "),
    );
    pb
}

/// スピナーを作成
///
/// 不定期間の処理を表示する場合に使用。
///
/// # 引数
///
/// * `message` - 表示メッセージ
///
/// # 戻り値
///
/// スピナーのProgressBarインスタンス
///
/// # 使用例
///
/// ```no_run
/// use backup_suite::ui::progress::create_spinner;
///
/// let spinner = create_spinner("接続中...");
/// // ... 処理 ...
/// spinner.finish_with_message("接続完了");
/// ```
pub fn create_spinner(message: &str) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template(&format!("{{spinner:.cyan}} {}", message))
            .unwrap()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ "),
    );
    spinner.enable_steady_tick(Duration::from_millis(120));
    spinner
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_progress() {
        let progress = BackupProgress::new(100);
        // 基本的な作成テスト
        progress.set_message("テスト");
        progress.inc(1);
        progress.finish_and_clear();
    }

    #[test]
    fn test_new_spinner() {
        let progress = BackupProgress::new_spinner();
        progress.set_message("スピナーテスト");
        progress.finish_and_clear();
    }

    #[test]
    fn test_set_position() {
        let progress = BackupProgress::new(100);
        progress.set_position(50);
        progress.finish_and_clear();
    }

    #[test]
    fn test_set_length() {
        let progress = BackupProgress::new(0);
        progress.set_length(100);
        progress.inc(10);
        progress.finish_and_clear();
    }

    #[test]
    fn test_create_progress_bar() {
        let pb = create_progress_bar(100, "テスト");
        pb.inc(10);
        pb.finish_and_clear();
    }

    #[test]
    fn test_create_spinner() {
        let spinner = create_spinner("読み込み中");
        spinner.finish_and_clear();
    }

    #[test]
    fn test_progress_messages() {
        let progress = BackupProgress::new(100);
        progress.set_main_message("メイン");
        progress.set_message("詳細");
        progress.inc(1);
        progress.finish("完了");
    }
}
