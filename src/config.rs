
use std::collections::HashMap;
use std::path::PathBuf;

use ratatui::style::Color;
use serde::Deserialize;

pub(crate) fn load() -> Config {
    let paths = [
        dirs_config_dir().map(|d| d.join("pixdraw").join("config.toml")),
        Some(PathBuf::from("config.toml")),
    ];

    for path in paths.into_iter().flatten() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(raw) = toml::from_str::<ConfigRaw>(&content) {
                return Config::from_raw(raw);
            }
        }
    }

            if let Some(config_dir) = dirs_config_dir().map(|d| d.join("pixdraw")) {
        std::fs::create_dir_all(&config_dir).ok();
        let path = config_dir.join("config.toml");
        if !path.exists() {
            let content = default_config_content();
            std::fs::write(&path, content).ok();
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(raw) = toml::from_str::<ConfigRaw>(&content) {
                    return Config::from_raw(raw);
                }
            }
        }
    }

    Config::default()
}

fn dirs_config_dir() -> Option<PathBuf> {
    std::env::var("XDG_CONFIG_HOME")
        .ok()
        .map(PathBuf::from)
        .or_else(|| std::env::var("HOME").ok().map(|h| PathBuf::from(h).join(".config")))
}

#[derive(Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "kebab-case")]
enum ThemePreset {
    TokyoNight,
    Catppuccin,
    Gruvbox,
    Nord,
    Dracula,
    OneDark,
    RosePine,
}

impl ThemePreset {
    fn theme(&self) -> Theme {
        match self {
            ThemePreset::TokyoNight => Theme {
                bg:             Color::Rgb(0x1A, 0x1B, 0x26),
                surface:        Color::Rgb(0x24, 0x28, 0x3B),
                text:           Color::Rgb(0xC0, 0xCA, 0xF5),
                subtle:         Color::Rgb(0x56, 0x5F, 0x89),
                accent:         Color::Rgb(0x00, 0xD4, 0xFF),
                highlight:      Color::Rgb(0xE0, 0xAF, 0x68),
                border:         Color::Rgb(0x3B, 0x42, 0x61),
                tab_active_bg:  Color::Rgb(0x33, 0x46, 0x7C),
                tab_inactive_bg:Color::Rgb(0x2E, 0x34, 0x4E),
                status_bg:      Color::Rgb(0x16, 0x17, 0x22),
                dim:            Color::Rgb(0x56, 0x5F, 0x89),
            },
            ThemePreset::Catppuccin => Theme {
                bg:             Color::Rgb(0x1E, 0x1E, 0x2E),
                surface:        Color::Rgb(0x31, 0x32, 0x44),
                text:           Color::Rgb(0xCD, 0xD6, 0xF4),
                subtle:         Color::Rgb(0x58, 0x5B, 0x70),
                accent:         Color::Rgb(0xCB, 0xA6, 0xF7),
                highlight:      Color::Rgb(0xF5, 0xE0, 0xDC),
                border:         Color::Rgb(0x45, 0x47, 0x5A),
                tab_active_bg:  Color::Rgb(0x45, 0x47, 0x5A),
                tab_inactive_bg:Color::Rgb(0x31, 0x32, 0x44),
                status_bg:      Color::Rgb(0x18, 0x18, 0x25),
                dim:            Color::Rgb(0x58, 0x5B, 0x70),
            },
            ThemePreset::Gruvbox => Theme {
                bg:             Color::Rgb(0x28, 0x28, 0x28),
                surface:        Color::Rgb(0x3C, 0x38, 0x36),
                text:           Color::Rgb(0xEB, 0xDB, 0xB2),
                subtle:         Color::Rgb(0x92, 0x83, 0x74),
                accent:         Color::Rgb(0x83, 0xA5, 0x98),
                highlight:      Color::Rgb(0xFA, 0xBD, 0x2F),
                border:         Color::Rgb(0x50, 0x49, 0x45),
                tab_active_bg:  Color::Rgb(0x50, 0x49, 0x45),
                tab_inactive_bg:Color::Rgb(0x3C, 0x38, 0x36),
                status_bg:      Color::Rgb(0x1D, 0x20, 0x21),
                dim:            Color::Rgb(0x92, 0x83, 0x74),
            },
            ThemePreset::Nord => Theme {
                bg:             Color::Rgb(0x2E, 0x34, 0x40),
                surface:        Color::Rgb(0x3B, 0x42, 0x52),
                text:           Color::Rgb(0xEC, 0xEF, 0xF4),
                subtle:         Color::Rgb(0x61, 0x6E, 0x88),
                accent:         Color::Rgb(0x88, 0xC0, 0xD0),
                highlight:      Color::Rgb(0xEB, 0xCB, 0x8B),
                border:         Color::Rgb(0x4C, 0x56, 0x6A),
                tab_active_bg:  Color::Rgb(0x4C, 0x56, 0x6A),
                tab_inactive_bg:Color::Rgb(0x3B, 0x42, 0x52),
                status_bg:      Color::Rgb(0x24, 0x29, 0x33),
                dim:            Color::Rgb(0x61, 0x6E, 0x88),
            },
            ThemePreset::Dracula => Theme {
                bg:             Color::Rgb(0x28, 0x2A, 0x36),
                surface:        Color::Rgb(0x44, 0x47, 0x5A),
                text:           Color::Rgb(0xF8, 0xF8, 0xF2),
                subtle:         Color::Rgb(0x62, 0x72, 0xA4),
                accent:         Color::Rgb(0xBD, 0x93, 0xF9),
                highlight:      Color::Rgb(0xF1, 0xFA, 0x8C),
                border:         Color::Rgb(0x55, 0x57, 0x70),
                tab_active_bg:  Color::Rgb(0x44, 0x47, 0x5A),
                tab_inactive_bg:Color::Rgb(0x34, 0x37, 0x46),
                status_bg:      Color::Rgb(0x21, 0x22, 0x2C),
                dim:            Color::Rgb(0x62, 0x72, 0xA4),
            },
            ThemePreset::OneDark => Theme {
                bg:             Color::Rgb(0x28, 0x2C, 0x34),
                surface:        Color::Rgb(0x35, 0x3B, 0x45),
                text:           Color::Rgb(0xAB, 0xB2, 0xBF),
                subtle:         Color::Rgb(0x5C, 0x63, 0x70),
                accent:         Color::Rgb(0x61, 0xAF, 0xEF),
                highlight:      Color::Rgb(0xE5, 0xC0, 0x7B),
                border:         Color::Rgb(0x3E, 0x44, 0x52),
                tab_active_bg:  Color::Rgb(0x3E, 0x44, 0x52),
                tab_inactive_bg:Color::Rgb(0x35, 0x3B, 0x45),
                status_bg:      Color::Rgb(0x21, 0x25, 0x2B),
                dim:            Color::Rgb(0x5C, 0x63, 0x70),
            },
            ThemePreset::RosePine => Theme {
                bg:             Color::Rgb(0x19, 0x19, 0x24),
                surface:        Color::Rgb(0x2A, 0x27, 0x3F),
                text:           Color::Rgb(0xE0, 0xDE, 0xF4),
                subtle:         Color::Rgb(0x6E, 0x6A, 0x86),
                accent:         Color::Rgb(0xEB, 0x6F, 0x92),
                highlight:      Color::Rgb(0xF6, 0xC1, 0x77),
                border:         Color::Rgb(0x3E, 0x3A, 0x52),
                tab_active_bg:  Color::Rgb(0x3E, 0x3A, 0x52),
                tab_inactive_bg:Color::Rgb(0x2A, 0x27, 0x3F),
                status_bg:      Color::Rgb(0x13, 0x13, 0x1D),
                dim:            Color::Rgb(0x6E, 0x6A, 0x86),
            },
        }
    }
}

#[derive(Deserialize)]
struct ConfigRaw {
    #[serde(default)]
    theme_preset: Option<ThemePreset>,
    #[serde(default)]
    theme: ThemeRaw,
    #[serde(default)]
    palette: PaletteRaw,
    #[serde(default)]
    keybinds: HashMap<String, String>,
}

#[derive(Deserialize, Default)]
struct ThemeRaw {
    bg: Option<String>,
    surface: Option<String>,
    text: Option<String>,
    subtle: Option<String>,
    accent: Option<String>,
    highlight: Option<String>,
    border: Option<String>,
    tab_active_bg: Option<String>,
    tab_inactive_bg: Option<String>,
    status_bg: Option<String>,
}

#[derive(Deserialize, Default)]
struct PaletteRaw {
    colors: Option<Vec<String>>,
}

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub(crate) struct Config {
    pub theme: Theme,
    pub palette: Palette,
            keybinds: HashMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            palette: Palette::default(),
            keybinds: HashMap::new(),
        }
    }
}

impl Config {
    fn from_raw(raw: ConfigRaw) -> Self {
        let base = raw.theme_preset.map(|p| p.theme()).unwrap_or_default();
        Self {
            theme: Theme::from_raw_with_base(raw.theme, base),
            palette: Palette::from_raw(raw.palette),
            keybinds: raw.keybinds,
        }
    }

            pub(crate) fn key_is(&self, key: &KeyEvent, action_name: &str, default_spec: &str) -> bool {
        let spec = self.keybinds.get(action_name).map(|s| s.as_str()).unwrap_or(default_spec);
        key_spec_matches(key, spec)
    }
}

pub(crate) struct Theme {
    pub bg: Color,
    pub surface: Color,
    pub text: Color,
    pub subtle: Color,
    pub accent: Color,
    pub highlight: Color,
    pub border: Color,
    pub tab_active_bg: Color,
    pub tab_inactive_bg: Color,
    pub status_bg: Color,
        pub dim: Color,
}

fn hex_to_color(hex: &str, fallback: Color) -> Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 { return fallback; }
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    Color::Rgb(r, g, b)
}

impl Default for Theme {
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

impl Theme {
    fn from_raw_with_base(raw: ThemeRaw, base: Theme) -> Self {
        let d = base;
        Self {
            bg: raw.bg.as_deref().map(|h| hex_to_color(h, d.bg)).unwrap_or(d.bg),
            surface: raw.surface.as_deref().map(|h| hex_to_color(h, d.surface)).unwrap_or(d.surface),
            text: raw.text.as_deref().map(|h| hex_to_color(h, d.text)).unwrap_or(d.text),
            subtle: raw.subtle.as_deref().map(|h| hex_to_color(h, d.subtle)).unwrap_or(d.subtle),
            accent: raw.accent.as_deref().map(|h| hex_to_color(h, d.accent)).unwrap_or(d.accent),
            highlight: raw.highlight.as_deref().map(|h| hex_to_color(h, d.highlight)).unwrap_or(d.highlight),
            border: raw.border.as_deref().map(|h| hex_to_color(h, d.border)).unwrap_or(d.border),
            tab_active_bg: raw.tab_active_bg.as_deref().map(|h| hex_to_color(h, d.tab_active_bg)).unwrap_or(d.tab_active_bg),
            tab_inactive_bg: raw.tab_inactive_bg.as_deref().map(|h| hex_to_color(h, d.tab_inactive_bg)).unwrap_or(d.tab_inactive_bg),
            status_bg: raw.status_bg.as_deref().map(|h| hex_to_color(h, d.status_bg)).unwrap_or(d.status_bg),
            dim: raw.subtle.as_deref().map(|h| hex_to_color(h, d.subtle)).unwrap_or(d.dim),
        }
    }
}

pub(crate) struct Palette {
    pub colors: Vec<Color>,
    pub names: Vec<String>,
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            colors: vec![
                Color::Rgb(255, 255, 255),
                Color::Rgb(194, 195, 199),
                Color::Rgb(95, 87, 79),
                Color::Rgb(0, 0, 0),
                Color::Rgb(255, 0, 77),
                Color::Rgb(255, 163, 0),
                Color::Rgb(255, 236, 39),
                Color::Rgb(0, 228, 54),
                Color::Rgb(0, 90, 74),
                Color::Rgb(41, 173, 255),
                Color::Rgb(43, 62, 91),
                Color::Rgb(255, 119, 168),
                Color::Rgb(171, 82, 54),
                Color::Rgb(255, 204, 170),
            ],
            names: vec![
                "White", "Light Gray", "Gray", "Black",
                "Red", "Orange", "Yellow", "Green",
                "Dark Green", "Blue", "Dark Blue",
                "Pink", "Brown", "Peach",
            ].into_iter().map(String::from).collect(),
        }
    }
}

impl Palette {
    fn from_raw(raw: PaletteRaw) -> Self {
        let Some(hex_colors) = raw.colors else { return Self::default() };
        if hex_colors.len() != 14 { return Self::default(); }
        let defaults = Self::default();
        let colors: Vec<Color> = hex_colors.iter()
            .zip(defaults.colors.iter())
            .map(|(h, fallback)| hex_to_color(h, *fallback))
            .collect();
        Self { colors, ..Self::default() }
    }
}


fn default_config_content() -> String {
    r##"# ── Opendraw Configuration ─────────────────────────────────────────
# This file was auto-generated on first run.
# Uncomment and edit any line to override the default.

# ── Theme Preset ─────────────────────────────────────────────────
# Available presets: tokyo-night, catppuccin, gruvbox, nord, dracula,
#                    one-dark, rose-pine
#
# theme_preset = "tokyo-night"

# ── Theme Overrides ──────────────────────────────────────────────
# Individual colours override the preset (or define a full custom theme).
# Format: "#RRGGBB" (hex with or without the # prefix).
#
# [theme]
# bg = "#1A1B26"
# surface = "#24283B"
# text = "#C0CAF5"
# subtle = "#565F89"
# accent = "#00D4FF"
# highlight = "#E0AF68"
# border = "#3B4261"
# tab_active_bg = "#33467C"
# tab_inactive_bg = "#2E344E"
# status_bg = "#161722"

# ── Palette ──────────────────────────────────────────────────────
# 14 colours used by the drawing tools. Order matches keys 1-9,0
# in the palette bar. Each is a hex string.
#
# [palette]
# colors = [
#     "#FFFFFF", "#C2C3C7", "#5F574F", "#000000",
#     "#FF004D", "#FFA300", "#FFEC27", "#00E436",
#     "#005A4A", "#29ADFF", "#2B3E5B", "#FF77A8",
#     "#AB5236", "#FFCCAA",
# ]

# ── Keybind Overrides ────────────────────────────────────────────
# Each action name maps to a key spec string. Uncomment and change
# the value to rebind. Supported spec formats:
#   q                single letter (case-insensitive)
#   Ctrl+S           control+letter
#   Alt+Enter        alt+key
#   Shift+Tab        shift+tab (also "BackTab")
#   F1-F12           function keys
#   Enter, Esc, Tab, Space, Backspace, Delete
#   Up, Down, Left, Right, Home, End
#
# [keybinds]
# ── Drawing tools ──
# eyedropper  = "i"
# eraser      = "e"
# flood_fill  = "f"
# symmetry    = "m"
# spray       = "p"
# rainbow     = "b"
# toggle_grid = "g"
#
# ── Shapes ──
# rectangle      = "r"
# filled_rect    = "R"
# circle         = "o"
# filled_circle  = "O"
# line_tool      = "l"
#
# ── Selection ──
# select_mode = "s"
# copy        = "Ctrl+C"
# cut         = "Ctrl+X"
# paste       = "Ctrl+V"
# nudge_up    = "Up"
# nudge_down  = "Down"
# nudge_left  = "Left"
# nudge_right = "Right"
#
# ── Colours ──
# next_colour     = "Space"
# generate_custom = "u"
# cycle_custom    = "0"
# gradient        = "G"
# color_picker    = "Ctrl+N"
#
# ── Tabs ──
# new_tab    = "Ctrl+T"
# close_tab  = "Ctrl+W"
# next_tab   = "Tab"
# prev_tab   = "Shift+Tab"
# rename_tab = "F2"
#
# ── File ──
# save       = "Ctrl+S"
# load       = "Ctrl+O"
# export_png = "Ctrl+E"
# resize     = "Ctrl+R"
# undo       = "Ctrl+Z"
# redo       = "Ctrl+Y"
#
# ── Effects ──
# life_toggle = "L"
# posterize   = "P"
#
# ── Other ──
# text_tool    = "t"
# clear        = "c"
# quit         = "q"
# help         = "?"
# esc_reset    = "Esc"
# shrink_brush = "["
# grow_brush   = "]"
"##
    .to_string()
}

fn key_spec_matches(key: &KeyEvent, spec: &str) -> bool {
    let (modifiers, code) = parse_key_spec(spec);
    key.modifiers == modifiers && key.code == code
}

fn parse_key_spec(spec: &str) -> (KeyModifiers, KeyCode) {
    let spec = spec.trim();

        if spec.eq_ignore_ascii_case("Shift+Tab") || spec.eq_ignore_ascii_case("BackTab") {
        return (KeyModifiers::NONE, KeyCode::BackTab);
    }

    let mut mods = KeyModifiers::NONE;
    let rest = if let Some(r) = spec.strip_prefix("Ctrl+") {
        mods.insert(KeyModifiers::CONTROL);
        r
    } else if let Some(r) = spec.strip_prefix("Alt+") {
        mods.insert(KeyModifiers::ALT);
        r
    } else if let Some(r) = spec.strip_prefix("Shift+") {
        mods.insert(KeyModifiers::SHIFT);
        r
    } else {
        spec
    };

    let code = match rest {
        "Enter" => KeyCode::Enter,
        "Esc" => KeyCode::Esc,
        "Tab" => KeyCode::Tab,
        "Space" => KeyCode::Char(' '),
        "Backspace" => KeyCode::Backspace,
        "Delete" => KeyCode::Delete,
        "Up" => KeyCode::Up,
        "Down" => KeyCode::Down,
        "Left" => KeyCode::Left,
        "Right" => KeyCode::Right,
        "Home" => KeyCode::Home,
        "End" => KeyCode::End,
        s if s.starts_with('F') && s.len() > 1 => {
            let n: u8 = s[1..].parse().unwrap_or(1);
            KeyCode::F(n)
        }
                        s if s.len() == 1 => KeyCode::Char(s.to_ascii_lowercase().chars().next().unwrap()),
        _ => KeyCode::Char('?'),     };

    (mods, code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_char() {
        let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        assert!(key_spec_matches(&key, "q"));
        assert!(key_spec_matches(&key, "Q"));
    }

    #[test]
    fn ctrl_combination() {
        let key = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL);
        assert!(key_spec_matches(&key, "Ctrl+S"));
    }

    #[test]
    fn function_key() {
        let key = KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE);
        assert!(key_spec_matches(&key, "F1"));
    }

    #[test]
    fn named_key() {
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert!(key_spec_matches(&key, "Enter"));
    }

    #[test]
    fn shift_tab() {
        let key = KeyEvent::new(KeyCode::BackTab, KeyModifiers::NONE);
        assert!(key_spec_matches(&key, "Shift+Tab"));
    }
}
