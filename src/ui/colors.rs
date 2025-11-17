/// カラースキーム管理モジュール
///
/// ダーク/ライトモード対応、ステータス別カラーコード、アクセシビリティ対応
///
use console::Style;

/// カラースキームモード
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorScheme {
    Dark,
    Light,
    Auto,
    HighContrast,
}

impl ColorScheme {
    /// 環境変数から自動検出
    #[must_use]
    pub fn detect() -> Self {
        if let Ok(term) = std::env::var("TERM") {
            if term.contains("256color") {
                return Self::Auto;
            }
        }

        // NO_COLORが設定されている場合はハイコントラスト
        if std::env::var("NO_COLOR").is_ok() {
            return Self::HighContrast;
        }

        Self::Auto
    }
}

/// カラーテーマ
pub struct ColorTheme {
    scheme: ColorScheme,
}

impl ColorTheme {
    #[must_use]
    pub fn new(scheme: ColorScheme) -> Self {
        Self { scheme }
    }

    #[must_use]
    pub fn auto() -> Self {
        Self::new(ColorScheme::detect())
    }

    /// --no-color フラグに基づいてテーマを作成
    #[must_use]
    pub fn from_no_color(no_color: bool) -> Self {
        if no_color {
            Self::new(ColorScheme::HighContrast)
        } else {
            Self::auto()
        }
    }

    /// 成功メッセージ用スタイル（緑 - 落ち着いた色調）
    #[must_use]
    pub fn success(&self) -> Style {
        match self.scheme {
            ColorScheme::Dark | ColorScheme::Auto => Style::new().green(),
            ColorScheme::Light => Style::new().green(),
            ColorScheme::HighContrast => Style::new().bold(),
        }
    }

    /// エラーメッセージ用スタイル（赤 - 落ち着いた色調）
    #[must_use]
    pub fn error(&self) -> Style {
        match self.scheme {
            ColorScheme::Dark | ColorScheme::Auto => Style::new().red(),
            ColorScheme::Light => Style::new().red(),
            ColorScheme::HighContrast => Style::new().bold(),
        }
    }

    /// 警告メッセージ用スタイル（黄 - 落ち着いた色調）
    #[must_use]
    pub fn warning(&self) -> Style {
        match self.scheme {
            ColorScheme::Dark | ColorScheme::Auto => Style::new().yellow(),
            ColorScheme::Light => Style::new().yellow(),
            ColorScheme::HighContrast => Style::new().bold(),
        }
    }

    /// 情報メッセージ用スタイル（青 - 落ち着いた色調）
    #[must_use]
    pub fn info(&self) -> Style {
        match self.scheme {
            ColorScheme::Dark | ColorScheme::Auto => Style::new().blue(),
            ColorScheme::Light => Style::new().blue(),
            ColorScheme::HighContrast => Style::new(),
        }
    }

    /// 見出し用スタイル（シンプル）
    #[must_use]
    pub fn header(&self) -> Style {
        match self.scheme {
            ColorScheme::Dark | ColorScheme::Auto | ColorScheme::Light => Style::new().bold(),
            ColorScheme::HighContrast => Style::new().bold(),
        }
    }

    /// 強調表示用スタイル（シンプル）
    #[must_use]
    pub fn highlight(&self) -> Style {
        Style::new().bold()
    }

    /// 通常テキスト用スタイル
    #[must_use]
    pub fn normal(&self) -> Style {
        Style::new()
    }

    /// 薄い色のテキスト（補助情報用）
    #[must_use]
    pub fn dim(&self) -> Style {
        match self.scheme {
            ColorScheme::Dark => Style::new().dim(),
            ColorScheme::Light => Style::new().dim(),
            ColorScheme::Auto => Style::new().dim(),
            ColorScheme::HighContrast => Style::new(),
        }
    }

    /// 優先度別カラー
    #[must_use]
    pub fn priority_high(&self) -> Style {
        self.error() // 赤系
    }

    #[must_use]
    pub fn priority_medium(&self) -> Style {
        self.warning() // 黄系
    }

    #[must_use]
    pub fn priority_low(&self) -> Style {
        self.info() // 青系
    }
}

/// ヘルパー関数: 成功メッセージ（絵文字なし・シンプル）
#[must_use]
pub fn success(msg: &str) -> String {
    ColorTheme::auto()
        .success()
        .apply_to(format!("[OK] {msg}"))
        .to_string()
}

/// ヘルパー関数: エラーメッセージ（絵文字なし・シンプル）
#[must_use]
pub fn error(msg: &str) -> String {
    ColorTheme::auto()
        .error()
        .apply_to(format!("[ERROR] {msg}"))
        .to_string()
}

/// ヘルパー関数: 警告メッセージ（絵文字なし・シンプル）
#[must_use]
pub fn warning(msg: &str) -> String {
    ColorTheme::auto()
        .warning()
        .apply_to(format!("[WARN] {msg}"))
        .to_string()
}

/// ヘルパー関数: 情報メッセージ（絵文字なし・シンプル）
#[must_use]
pub fn info(msg: &str) -> String {
    ColorTheme::auto()
        .info()
        .apply_to(format!("[INFO] {msg}"))
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_scheme_detection() {
        let scheme = ColorScheme::detect();
        // 環境に依存するのでパニックしないことだけ確認
        assert!(matches!(
            scheme,
            ColorScheme::Auto | ColorScheme::HighContrast
        ));
    }

    #[test]
    fn test_theme_styles() {
        let theme = ColorTheme::new(ColorScheme::Dark);

        // スタイルが適用可能であることを確認
        let _ = theme.success().apply_to("test");
        let _ = theme.error().apply_to("test");
        let _ = theme.warning().apply_to("test");
        let _ = theme.info().apply_to("test");
    }

    #[test]
    fn test_helper_functions() {
        // ヘルパー関数がパニックしないことを確認
        let _ = success("Test success");
        let _ = error("Test error");
        let _ = warning("Test warning");
        let _ = info("Test info");
    }
}
