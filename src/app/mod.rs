
mod draw;
mod event;
mod history;
mod mouse;
mod popup;
mod selection;
mod session;
pub mod tab;

use std::collections::VecDeque;
use std::ops::{Deref, DerefMut};

use ratatui::{
    Frame,
    layout::{Position, Rect},
    style::{Color, Style},
};

use crate::file_browser::FileBrowser;
use crate::palette::Palette;
use crate::ui;

pub use tab::TabData;


fn distance(a: Position, b: Position) -> u16 {
    let dx = a.x.abs_diff(b.x);
    let dy = a.y.abs_diff(b.y);
    dx.max(dy)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ShapeKind {
    Rect,             FilledRect,       Circle,           FilledCircle, }


pub struct DrawingApp {
    pub should_quit: bool,
    pub config: crate::config::Config,

        pub tabs: Vec<TabData>,
        pub current_tab: usize,

                    pub canvas_area: Rect,
    pub color_picker_area: Rect,
    pub palette_bar_area: Rect,

        pub mouse_position: Option<Position>,
        pub last_cursor_position: Option<Position>,

        pub palette: Palette,
    pub brush_size: u16,
    pub file_browser: FileBrowser,
    pub file_browser_area: Rect,
        pub color_history: VecDeque<Color>,
        pub custom_colors: Vec<Color>,
        pub custom_color_override: Option<Color>,
        pub custom_cycle_idx: usize,
        pub color_gen_seed: u64,

            pub eyedropper_mode: bool,
    pub eraser_mode: bool,
    pub fill_mode: bool,
    pub symmetry_mode: bool,
    pub show_grid: bool,
    pub show_help: bool,

                pub show_color_picker: bool,
    pub show_color_selector: bool,
    pub color_selector_idx: usize,
    pub show_color_input: bool,
    pub color_input_buffer: String,
    pub show_startup_dialog: bool,
    pub startup_dialog_idx: usize,
    pub startup_save_and_new: bool,
    pub show_context_menu: bool,
    pub context_menu_idx: usize,
    pub context_menu_pos: Position,

        pub canvas_width: u16,
    pub canvas_height: u16,
    pub show_canvas_resize: bool,
    pub canvas_resize_buffer: String,

                pub shape_preview: Option<(Position, Position, ShapeKind)>,

        pub rainbow_mode: bool,
        pub rainbow_idx: usize,
    pub spray_mode: bool,

        pub life_mode: bool,

            pub last_backup: std::time::Instant,

        pub show_tab_rename: bool,
    pub tab_rename_buffer: String,

            pub help_cat_expanded: [bool; 9],
        pub help_selected: usize,
        pub help_scroll: u16,
        pub help_search_buffer: String,
        pub help_search_active: bool,
}


impl Deref for DrawingApp {
    type Target = TabData;
    fn deref(&self) -> &TabData {
        &self.tabs[self.current_tab]
    }
}

impl DerefMut for DrawingApp {
    fn deref_mut(&mut self) -> &mut TabData {
        &mut self.tabs[self.current_tab]
    }
}


impl DrawingApp {
    pub fn new() -> Self {
                        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(42);
        let cfg = crate::config::load();
        let palette = Palette::from_config(&cfg.palette);
        crate::ui::init_from_config(&cfg.theme);
        Self {
            should_quit: false,
            config: cfg,
            tabs: vec![TabData::new("Untitled-1".to_string())],
            current_tab: 0,
            canvas_area: Rect::default(),
            color_picker_area: Rect::default(),
            palette_bar_area: Rect::default(),
            mouse_position: None,
            last_cursor_position: None,
            palette,
            brush_size: 1,
            file_browser: FileBrowser::new(),
            file_browser_area: Rect::default(),
            color_history: VecDeque::new(),
            custom_colors: Vec::new(),
            custom_color_override: None,
            custom_cycle_idx: 0,
            color_gen_seed: seed,
            eyedropper_mode: false,
            eraser_mode: false,
            fill_mode: false,
            symmetry_mode: false,
            show_grid: false,
            show_help: false,
            show_color_picker: false,
            show_color_selector: false,
            color_selector_idx: 0,
            show_color_input: false,
            color_input_buffer: String::new(),
            show_startup_dialog: false,
            startup_dialog_idx: 0,
            startup_save_and_new: false,
            show_context_menu: false,
            context_menu_idx: 0,
            context_menu_pos: Position::new(0, 0),
            canvas_width: 0,
            canvas_height: 0,
            show_canvas_resize: false,
            canvas_resize_buffer: String::new(),
            shape_preview: None,
            rainbow_mode: false,
            rainbow_idx: 0,
            spray_mode: false,
            life_mode: false,
            last_backup: std::time::Instant::now(),
            show_tab_rename: false,
            tab_rename_buffer: String::new(),
            help_cat_expanded: [false; 9],
            help_selected: 0,
            help_scroll: 0,
            help_search_buffer: String::new(),
            help_search_active: false,
        }
    }

    
    pub fn new_tab(&mut self) {
        let n = self.tabs.len() + 1;
        self.tabs.push(TabData::new(format!("Untitled-{}", n)));
        self.current_tab = self.tabs.len() - 1;
    }

    pub fn close_tab(&mut self) {
                if self.tabs.len() <= 1 {
            return;
        }
        self.tabs.remove(self.current_tab);
                if self.current_tab >= self.tabs.len() {
            self.current_tab = self.tabs.len() - 1;
        }
    }

    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    pub fn tab_name(&self) -> &str {
        &self.tabs[self.current_tab].name
    }

    
            fn push_color_history(&mut self, c: Color) {
        if self.color_history.back() == Some(&c) {
            return;
        }
        self.color_history.push_back(c);
        if self.color_history.len() > 5 {
            self.color_history.pop_front();
        }
    }

                pub(crate) fn parse_rgb_buffer(buffer: &str) -> Option<Color> {
        let parts: Vec<&str> = buffer
            .split(&[',', ' ', '\t'][..])
            .filter(|s| !s.is_empty())
            .collect();
        if parts.len() == 3
            && let (Ok(r), Ok(g), Ok(b)) = (
                parts[0].parse::<u8>(),
                parts[1].parse::<u8>(),
                parts[2].parse::<u8>(),
            )
        {
            return Some(Color::Rgb(r, g, b));
        }
        None
    }

            fn hsl_to_rgb(h: f64, s: f64, l: f64) -> Color {
        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = l - c / 2.0;
        let (r, g, b) = if h < 60.0 {
            (c, x, 0.0)
        } else if h < 120.0 {
            (x, c, 0.0)
        } else if h < 180.0 {
            (0.0, c, x)
        } else if h < 240.0 {
            (0.0, x, c)
        } else if h < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };
        Color::Rgb(
            ((r + m) * 255.0).round() as u8,
            ((g + m) * 255.0).round() as u8,
            ((b + m) * 255.0).round() as u8,
        )
    }

                fn lcg_next(seed: &mut u64) -> u64 {
        *seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        *seed
    }

                pub(crate) fn generate_three_colors(&mut self) {
        let golden = 0.618033988749895;         let offset = (Self::lcg_next(&mut self.color_gen_seed) as f64 / u64::MAX as f64) * 360.0;
        self.custom_colors = (0..3)
            .map(|i| {
                let hue = (offset + i as f64 * golden).fract() * 360.0;
                let sat =
                    0.45 + (Self::lcg_next(&mut self.color_gen_seed) as f64 % 1000.0) / 2000.0;
                let lig =
                    0.30 + (Self::lcg_next(&mut self.color_gen_seed) as f64 % 1000.0) / 2000.0;
                Self::hsl_to_rgb(hue, sat.min(1.0), lig.min(1.0))
            })
            .collect();
        self.custom_cycle_idx = 0;
        self.color_selector_idx = 0;
    }

            fn add_custom_color(&mut self, c: Color) {
        if self.custom_colors.contains(&c) {
            return;
        }
        if self.custom_colors.len() >= 3 {
            self.custom_colors.remove(0);
        }
        self.custom_colors.push(c);
        self.custom_cycle_idx = self.custom_colors.len() - 1;
        self.custom_color_override = Some(c);
    }

                pub fn draw_color(&mut self) -> Color {
        if self.rainbow_mode {
            self.rainbow_idx = (self.rainbow_idx + 1) % self.palette.colors.len();
            return self.palette.colors[self.rainbow_idx].0;
        }
                self.custom_color_override
            .unwrap_or_else(|| self.palette.current())
    }

        fn current_display_name(&self) -> String {
        if let Some(c) = &self.custom_color_override {
            if let Some(i) = self.custom_colors.iter().position(|c2| c2 == c) {
                return format!("C{}", i + 1);
            }
            return "custom".to_string();
        }
        self.palette.name().to_string()
    }

            pub fn mode_string(&self) -> String {
        let mut parts: Vec<&str> = Vec::new();
        if self.eyedropper_mode {
            parts.push("EYE");
        }
        if self.eraser_mode {
            parts.push("ERA");
        }
        if self.fill_mode {
            parts.push("FILL");
        }
        if self.symmetry_mode {
            parts.push("MIR");
        }
        if self.show_grid {
            parts.push("GRD");
        }
        if self.text_mode {
            parts.push("TEXT");
        }
        if let Some(k) = &self.shape_mode {
            match k {
                ShapeKind::Rect => parts.push("RECT"),
                ShapeKind::FilledRect => parts.push("FRECT"),
                ShapeKind::Circle => parts.push("CIRC"),
                ShapeKind::FilledCircle => parts.push("FCIRC"),
            }
        }
        if self.line_mode {
            parts.push("LINE");
        }
        if self.select_mode {
            parts.push("SEL");
        }
        if self.gradient_mode {
            parts.push("GRAD");
        }
        if self.life_mode {
            parts.push("LIFE");
        }
        parts.join(" ")
    }

    
                        pub fn local_canvas_position(&self, position: Position) -> Option<Position> {
        let within_x = position.x >= self.canvas_area.x
            && position.x < self.canvas_area.x.saturating_add(self.canvas_area.width);
        let within_y = position.y >= self.canvas_area.y
            && position.y < self.canvas_area.y.saturating_add(self.canvas_area.height);
        if !within_x || !within_y {
            return None;
        }
        Some(Position::new(
            position.x.saturating_sub(self.canvas_area.x),
            position.y.saturating_sub(self.canvas_area.y),
        ))
    }
}


impl DrawingApp {
                pub(crate) fn constrain_angle(start: Position, end: Position) -> Position {
        let dx = end.x as i32 - start.x as i32;
        let dy = end.y as i32 - start.y as i32;
        let adx = dx.unsigned_abs();
        let ady = dy.unsigned_abs();
                if adx > ady * 2 {
            return Position::new(end.x, start.y);
        }
                if ady > adx * 2 {
            return Position::new(start.x, end.y);
        }
                if adx == 0 && ady == 0 {
            return end;
        }
        let len = adx.max(ady) as i16;
        Position::new(
            start
                .x
                .wrapping_add_signed(if dx >= 0 { len } else { -len }),
            start
                .y
                .wrapping_add_signed(if dy >= 0 { len } else { -len }),
        )
    }

            pub(crate) fn constrain_square(start: Position, end: Position) -> Position {
        let dx = end.x as i32 - start.x as i32;
        let dy = end.y as i32 - start.y as i32;
        let len = dx.abs().max(dy.abs()) as i16;
        Position::new(
            start
                .x
                .wrapping_add_signed(if dx >= 0 { len } else { -len }),
            start
                .y
                .wrapping_add_signed(if dy >= 0 { len } else { -len }),
        )
    }

        pub fn color_to_hex(c: &Color) -> String {
        let (r, g, b) = crate::app::draw::color_to_rgb(*c);
        format!("#{:02X}{:02X}{:02X}", r, g, b)
    }
}


impl DrawingApp {
                                                pub fn render(&mut self, frame: &mut Frame<'_>) {
        let area = frame.area();
        let (header, _body, footer, status_area, canvas_area) = ui::layout(area);
        self.palette_bar_area = footer;

        let current = self.draw_color();
        let pal_slice: Vec<(Color, &str)> = self.palette.colors.iter()
            .map(|(c, n)| (*c, n.as_str()))
            .collect();
        ui::render_palette_bar(
            frame,
            footer,
            current,
            &pal_slice,
            &self.custom_colors,
            self.custom_color_override,
        );

        ui::render_header(
            frame,
            header,
            self.text_mode,
            self.brush_size,
            current,
            &self.current_display_name(),
            &self.mode_string(),
            self.tabs.len(),
            self.current_tab,
            &self.tabs[self.current_tab].name,
        );

        ui::render_canvas(
            self,
            frame,
            canvas_area,
            self.canvas_width,
            self.canvas_height,
            self.shape_preview,
        );

        if self.show_grid {
            ui::render_grid(frame, canvas_area);
        }

        ui::render_text_overlay(self, frame, canvas_area);

                                if let Some(anchor) = self
            .shape_anchor
            .or(self.line_anchor)
            .or(self.gradient_anchor)
        {
            let (ix, iy) = (canvas_area.x + 1 + anchor.x, canvas_area.y + 1 + anchor.y);
            if let Some(cell) = frame.buffer_mut().cell_mut(Position::new(ix, iy)) {
                cell.set_char('+')
                    .set_style(Style::default().fg(Color::Cyan));
            }
        }

                                if self.select_mode
            && let (Some(start), Some(end)) = (self.selection_start, self.selection_end)
        {
            let (x1, x2) = (start.x.min(end.x), start.x.max(end.x));
            let (y1, y2) = (start.y.min(end.y), start.y.max(end.y));
            let ix0 = canvas_area.x + 1;
            let iy0 = canvas_area.y + 1;
            for x in x1..=x2 {
                for (y, _is_y1) in [(y1, true), (y2, false)] {
                    if let Some(cell) = frame.buffer_mut().cell_mut(Position::new(ix0 + x, iy0 + y))
                        && cell.symbol() == " "
                    {
                        cell.set_char('·')
                            .set_style(Style::default().fg(Color::Cyan));
                    }
                }
            }
            for y in y1..=y2 {
                for (x, _is_x1) in [(x1, true), (x2, false)] {
                    if let Some(cell) = frame.buffer_mut().cell_mut(Position::new(ix0 + x, iy0 + y))
                        && cell.symbol() == " "
                    {
                        cell.set_char('·')
                            .set_style(Style::default().fg(Color::Cyan));
                    }
                }
            }
        }

        ui::render_cursor_preview(self, frame, canvas_area);

        let current_name = self.palette.name();
        let hex = Self::color_to_hex(&current);
        ui::render_status_bar(
            frame,
            status_area,
            self.mouse_position,
            self.canvas_area,
            self.points.len(),
            self.brush_size,
            &self.color_history,
            current,
            current_name,
            &hex,
        );

                                
        if self.show_startup_dialog {
            ui::render_startup_dialog(frame, area, self.startup_dialog_idx);
            return;
        }
        if self.show_context_menu {
            ui::render_context_menu(frame, area, &self.context_menu_pos, self.context_menu_idx);
            return;
        }
        if self.show_canvas_resize {
            ui::render_canvas_resize_dialog(frame, area, &self.canvas_resize_buffer);
            return;
        }
        if self.show_tab_rename {
            ui::render_tab_rename_dialog(frame, area, &self.tab_rename_buffer);
            return;
        }

                        if self.show_color_picker {
            ui::render_color_picker(self, frame, canvas_area);
        }
        if self.show_color_selector {
            ui::render_color_selector(frame, area, &self.custom_colors, self.color_selector_idx);
        }
        if self.show_color_input {
            ui::render_color_input(frame, area, &self.color_input_buffer);
        }
        if self.file_browser.active {
            ui::render_file_browser(self, frame, area);
        }
        if self.show_help {
            ui::render_help_popup(
                frame,
                area,
                &self.help_cat_expanded,
                self.help_selected,
                self.help_scroll,
                &self.help_search_buffer,
                self.help_search_active,
            );
        }
    }
}
