// ── ui/col.rs ────────────────────────────────────────────────────────
// Theme colour accessors. At startup `init()` is called with the
// config theme; after that these functions return the configured values.
// If not initialised, they fall back to the default Tokyo Night theme.

use std::sync::OnceLock;

use ratatui::style::Color;

struct ThemeColors {
    bg: Color,
    surface: Color,
    text: Color,
    subtle: Color,
    accent: Color,
    highlight: Color,
    border: Color,
    tab_active_bg: Color,
    tab_inactive_bg: Color,
    status_bg: Color,
    dim: Color,
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self {
            bg: Color::Rgb(26, 27, 38),
            surface: Color::Rgb(36, 40, 59),
            text: Color::Rgb(192, 202, 245),
            subtle: Color::Rgb(86, 95, 137),
            accent: Color::Rgb(0, 212, 255),
            highlight: Color::Rgb(255, 199, 119),
            border: Color::Rgb(59, 66, 97),
            tab_active_bg: Color::Rgb(51, 70, 124),
            tab_inactive_bg: Color::Rgb(46, 52, 78),
            status_bg: Color::Rgb(22, 23, 34),
            dim: Color::Rgb(86, 95, 137),
        }
    }
}

static C: OnceLock<ThemeColors> = OnceLock::new();

pub(crate) fn init_from_config(t: &crate::config::Theme) {
    let _ = C.set(ThemeColors {
        bg: t.bg,
        surface: t.surface,
        text: t.text,
        subtle: t.subtle,
        accent: t.accent,
        highlight: t.highlight,
        border: t.border,
        tab_active_bg: t.tab_active_bg,
        tab_inactive_bg: t.tab_inactive_bg,
        status_bg: t.status_bg,
        dim: t.dim,
    });
}

fn t() -> &'static ThemeColors {
    C.get_or_init(ThemeColors::default)
}

pub fn bg() -> Color { t().bg }
pub fn surface() -> Color { t().surface }
pub fn text() -> Color { t().text }
pub fn subtle() -> Color { t().subtle }
pub fn accent() -> Color { t().accent }
pub fn highlight() -> Color { t().highlight }
pub fn border() -> Color { t().border }
pub fn tab_active_bg() -> Color { t().tab_active_bg }
pub fn tab_inactive_bg() -> Color { t().tab_inactive_bg }
pub fn status_bg() -> Color { t().status_bg }
pub fn dim() -> Color { t().dim }
