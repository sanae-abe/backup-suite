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
    pub fn new(scheme: ColorScheme) -> Self {
        Self { scheme }
    }

    pub fn auto() -> Self {
        Self::new(ColorScheme::detect())
    }

    /// 成功メッセージ用スタイル（緑）
    pub fn success(&self) -> Style {
        match self.scheme {
            ColorScheme::Dark => Style::new().green().bright(),
            ColorScheme::Light => Style::new().green(),
            ColorScheme::Auto => Style::new().green(),
            ColorScheme::HighContrast => Style::new().bold(),
        }
    }

    /// エラーメッセージ用スタイル（赤）
    pub fn error(&self) -> Style {
        match self.scheme {
            ColorScheme::Dark => Style::new().red().bright(),
            ColorScheme::Light => Style::new().red(),
            ColorScheme::Auto => Style::new().red().bold(),
            ColorScheme::HighContrast => Style::new().bold().underlined(),
        }
    }

    /// 警告メッセージ用スタイル（黄）
    pub fn warning(&self) -> Style {
        match self.scheme {
            ColorScheme::Dark => Style::new().yellow().bright(),
            ColorScheme::Light => Style::new().yellow(),
            ColorScheme::Auto => Style::new().yellow(),
            ColorScheme::HighContrast => Style::new().bold(),
        }
    }

    /// 情報メッセージ用スタイル（青）
    pub fn info(&self) -> Style {
        match self.scheme {
            ColorScheme::Dark => Style::new().cyan().bright(),
            ColorScheme::Light => Style::new().blue(),
            ColorScheme::Auto => Style::new().cyan(),
            ColorScheme::HighContrast => Style::new(),
        }
    }

    /// 見出し用スタイル
    pub fn header(&self) -> Style {
        match self.scheme {
            ColorScheme::Dark => Style::new().magenta().bright().bold(),
            ColorScheme::Light => Style::new().magenta().bold(),
            ColorScheme::Auto => Style::new().magenta().bold(),
            ColorScheme::HighContrast => Style::new().bold().underlined(),
        }
    }

    /// 強調表示用スタイル
    pub fn highlight(&self) -> Style {
        match self.scheme {
            ColorScheme::Dark => Style::new().white().bright().bold(),
            ColorScheme::Light => Style::new().black().bold(),
            ColorScheme::Auto => Style::new().white().bold(),
            ColorScheme::HighContrast => Style::new().bold(),
        }
    }

    /// 通常テキスト用スタイル
    pub fn normal(&self) -> Style {
        Style::new()
    }

    /// 薄い色のテキスト（補助情報用）
    pub fn dim(&self) -> Style {
        match self.scheme {
            ColorScheme::Dark => Style::new().dim(),
            ColorScheme::Light => Style::new().dim(),
            ColorScheme::Auto => Style::new().dim(),
            ColorScheme::HighContrast => Style::new(),
        }
    }

    /// 優先度別カラー
    pub fn priority_high(&self) -> Style {
        self.error() // 赤系
    }

    pub fn priority_medium(&self) -> Style {
        self.warning() // 黄系
    }

    pub fn priority_low(&self) -> Style {
        self.info() // 青系
    }
}

/// ヘルパー関数: 成功メッセージ
pub fn success(msg: &str) -> String {
    ColorTheme::auto().success().apply_to(format!("✓ {}", msg)).to_string()
}

/// ヘルパー関数: エラーメッセージ
pub fn error(msg: &str) -> String {
    ColorTheme::auto().error().apply_to(format!("✗ {}", msg)).to_string()
}

/// ヘルパー関数: 警告メッセージ
pub fn warning(msg: &str) -> String {
    ColorTheme::auto().warning().apply_to(format!("⚠ {}", msg)).to_string()
}

/// ヘルパー関数: 情報メッセージ
pub fn info(msg: &str) -> String {
    ColorTheme::auto().info().apply_to(format!("ℹ {}", msg)).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_scheme_detection() {
        let scheme = ColorScheme::detect();
        // 環境に依存するのでパニックしないことだけ確認
        assert!(matches!(scheme, ColorScheme::Auto | ColorScheme::HighContrast));
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
