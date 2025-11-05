/// UI/UX 改善モジュール
///
/// Phase 2: プログレスバー、インタラクティブプロンプト
/// Phase 3: ダッシュボード、テーブル表示、カラースキーム
pub mod colors;
pub mod dashboard;
pub mod interactive;
pub mod progress;
pub mod table;

// Phase 2: 基本UI機能
pub use interactive::{
    confirm, confirm_backup, confirm_cleanup, confirm_with_text, input, input_path,
    multi_select, select, select_priority, select_with_default,
};
pub use progress::{create_progress_bar, create_spinner, BackupProgress};

// Phase 3: 高度なUI機能
pub use colors::{ColorScheme, ColorTheme};
pub use dashboard::display_dashboard;
pub use table::{display_backup_result, display_history, display_targets};
