// ── app/handlers.rs ──────────────────────────────────────────────────
// Popup/modal keyboard handlers, file-browser event processing, and
// palette-bar click handling.
//
// These were extracted from `event.rs` to keep the main dispatch short.
// Each handler is an `impl DrawingApp` method just like the dispatch code
// in event.rs — Rust allows impl blocks to be spread across files.
//
// All handlers take `&mut self` so they can mutate application state.
// They return `io::Result<()>` to allow ? propagation of IO errors.

use std::io;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::Position;
use ratatui::style::Color;

use crate::app::DrawingApp;
use crate::file_browser::FileBrowserMode;

impl DrawingApp {
    // ── File browser sub-dialog ────────────────────────────────
    // The file browser is special: it has its OWN nested event loop
    // inside `handle_file_browser_event`. It blocks on event::read()
    // directly rather than going through the main handle_event → on_key
    // path. This keeps the file browser logic self-contained.

    pub(crate) fn handle_file_browser_event(&mut self) -> io::Result<()> {
        use crossterm::event;
        match event::read()? {
            event::Event::Key(key) => {
                if key.kind != KeyEventKind::Press {
                    return Ok(());
                }
                let is_save_like = self.file_browser.mode == FileBrowserMode::Save
                    || self.file_browser.mode == FileBrowserMode::ExportPng;
                if is_save_like && self.file_browser.filename_input_active {
                    // Typing mode: all chars go to filename input.
                    match key.code {
                        KeyCode::Esc => self.file_browser.filename_input_active = false,
                        KeyCode::Enter => {
                            let name = self.file_browser.filename_input.trim().to_string();
                            let filename = if self.file_browser.mode == FileBrowserMode::Save {
                                if name.ends_with(".txt") { name } else { format!("{}.txt", name) }
                            } else {
                                if name.ends_with(".png") { name } else { format!("{}.png", name) }
                            };
                            let full = self.file_browser.current_path.join(&filename);
                            if self.file_browser.mode == FileBrowserMode::Save {
                                self.save_to(&full)?;
                            } else {
                                self.export_png_to(&full)?;
                            }
                            self.file_browser.active = false;
                        }
                        KeyCode::Char(c) => self.file_browser.filename_input.push(c),
                        KeyCode::Backspace => { self.file_browser.filename_input.pop(); }
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Esc => self.file_browser.active = false,
                        KeyCode::Up | KeyCode::Char('k') => self.file_browser.navigate_up(),
                        KeyCode::Down | KeyCode::Char('j') => self.file_browser.navigate_down(),
                        KeyCode::Char('i') if is_save_like => {
                            self.file_browser.filename_input_active = true;
                        }
                        KeyCode::Char('u') => {
                            self.generate_three_colors();
                            self.color_selector_idx = 0;
                        }
                        KeyCode::Enter => {
                            let path = self.file_browser.selected_path();
                            match self.file_browser.mode {
                                FileBrowserMode::Save => {
                                    if !self.file_browser.filename_input.is_empty() {
                                        let name = self.file_browser.filename_input.trim();
                                        let filename = if name.ends_with(".txt") {
                                            name.to_string()
                                        } else {
                                            format!("{}.txt", name)
                                        };
                                        let full = self.file_browser.current_path.join(&filename);
                                        self.save_to(&full)?;
                                        self.file_browser.active = false;
                                    } else if let Some(p) = path {
                                        if p.is_dir() {
                                            self.file_browser.enter_selected();
                                        } else {
                                            self.save_to(&p)?;
                                            self.file_browser.active = false;
                                        }
                                    }
                                }
                                FileBrowserMode::ExportPng => {
                                    if !self.file_browser.filename_input.is_empty() {
                                        let name = self.file_browser.filename_input.trim();
                                        let filename = if name.ends_with(".png") {
                                            name.to_string()
                                        } else {
                                            format!("{}.png", name)
                                        };
                                        let full = self.file_browser.current_path.join(&filename);
                                        self.export_png_to(&full)?;
                                        self.file_browser.active = false;
                                    } else if let Some(p) = path {
                                        if p.is_dir() {
                                            self.file_browser.enter_selected();
                                        } else {
                                            self.export_png_to(&p)?;
                                            self.file_browser.active = false;
                                        }
                                    }
                                }
                                FileBrowserMode::Load => {
                                    if let Some(p) = path {
                                        if p.is_dir() {
                                            self.file_browser.enter_selected();
                                        } else {
                                            self.load_from(&p)?;
                                            self.file_browser.active = false;
                                        }
                                    }
                                }
                            }
                        }
                        KeyCode::Char(c) => {
                            self.file_browser.filename_input.push(c);
                        }
                        KeyCode::Backspace => {
                            self.file_browser.filename_input.pop();
                        }
                        KeyCode::Tab => {
                            self.file_browser.go_up_dir();
                        }
                        _ => {}
                    }
                }
            }
            event::Event::Mouse(mouse) => self.file_browser_event_mouse(mouse),
            _ => {}
        }
        Ok(())
    }

    pub(crate) fn file_browser_event_mouse(&mut self, event: crossterm::event::MouseEvent) {
        use crossterm::event::MouseButton;
        use crossterm::event::MouseEventKind;
        if event.kind != MouseEventKind::Down(MouseButton::Left) {
            return;
        }
        let pos = Position::new(event.column, event.row);
        if !self.file_browser_area.contains(pos) {
            self.file_browser.active = false;
            return;
        }
        let inner_top = self.file_browser_area.y + 3;
        let rel_row = pos.y.saturating_sub(inner_top);
        let idx = self.file_browser.scroll_offset + rel_row as usize;
        if idx < self.file_browser.entries.len() {
            self.file_browser.selected = idx;
            let path = self.file_browser.selected_path();
            match self.file_browser.mode {
                FileBrowserMode::Save => {
                    if let Some(p) = path {
                        if p.is_dir() {
                            self.file_browser.enter_selected();
                        } else {
                            let _ = self.save_to(&p);
                            self.file_browser.active = false;
                        }
                    }
                }
                FileBrowserMode::ExportPng => {
                    if let Some(p) = path {
                        if p.is_dir() {
                            self.file_browser.enter_selected();
                        } else {
                            let _ = self.export_png_to(&p);
                            self.file_browser.active = false;
                        }
                    }
                }
                FileBrowserMode::Load => {
                    if let Some(p) = path {
                        if p.is_dir() {
                            self.file_browser.enter_selected();
                        } else {
                            let _ = self.load_from(&p);
                            self.file_browser.active = false;
                        }
                    }
                }
            }
        }
    }

    // ── Colour picker ───────────────────────────────────────────
    // The colour picker (^Tab) renders a vertical list of all palette
    // colours plus custom colours. Keys 1-9 select a palette colour,
    // 0 selects the most recent custom colour, Esc dismisses.

    pub(crate) fn on_key_color_picker(&mut self, key: KeyEvent) -> io::Result<()> {
        match key.code {
            KeyCode::Esc => self.show_color_picker = false,
            KeyCode::Char(d @ '1'..='9') => {
                let idx = (d as u8 - b'1') as usize;
                if idx < self.palette.colors.len() {
                    self.palette.select(idx);
                    self.custom_color_override = None;
                    self.push_color_history(self.palette.current());
                }
                self.show_color_picker = false;
            }
            KeyCode::Char('0') => {
                if let Some(&c) = self.custom_colors.last() {
                    self.custom_color_override = Some(c);
                    self.push_color_history(c);
                }
                self.show_color_picker = false;
            }
            _ => {}
        }
        Ok(())
    }

    // ── Colour selector ─────────────────────────────────────────
    // The colour selector popup (shown after pressing `u` to generate
    // custom colours) lets the user pick one of the 3 generated colours
    // or enter a custom RGB value.

    pub(crate) fn on_key_color_selector(&mut self, key: KeyEvent) -> io::Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.show_color_selector = false;
            }
            KeyCode::Up | KeyCode::Char('k')
                if self.color_selector_idx > 0 => {
                    self.color_selector_idx -= 1;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let max = self.custom_colors.len();
                if self.color_selector_idx < max {
                    self.color_selector_idx += 1;
                }
            }
            KeyCode::Char('u') => {
                self.generate_three_colors();
                self.color_selector_idx = 0;
            }
            KeyCode::Enter => {
                if self.color_selector_idx < self.custom_colors.len() {
                    if let Some(&c) = self.custom_colors.get(self.color_selector_idx) {
                        self.custom_color_override = Some(c);
                        self.push_color_history(c);
                        self.custom_cycle_idx = (self.color_selector_idx + 1) % self.custom_colors.len();
                    }
                    self.show_color_selector = false;
                } else {
                    // "Custom RGB..." option — open the input dialog.
                    self.show_color_selector = false;
                    self.show_color_input = true;
                    self.color_input_buffer.clear();
                }
            }
            _ => {}
        }
        Ok(())
    }

    // ── Inline text mode ────────────────────────────────────────
    // When `text_mode` is true, ALL key presses go here (via the
    // early return in `on_key`). Characters are pushed into the text
    // buffer and shown as a preview on the canvas. Enter commits the
    // text and exits to brush mode. Esc cancels.

    pub(crate) fn on_key_text_mode(&mut self, key: KeyEvent) -> io::Result<()> {
        match key.code {
            KeyCode::Esc => {
                // Reset everything to brush mode — discard any pending text.
                self.text_mode = false;
                self.line_mode = false;
                self.spray_mode = false;
                self.shape_mode = None;
                self.text_buffer.clear();
                self.text_cursor = None;
            }
            KeyCode::Enter => {
                // Commit the text at the cursor position.
                if let Some(pos) = self.text_cursor
                    && !self.text_buffer.is_empty() {
                        self.push_history();
                        let text = self.text_buffer.clone();
                        self.text_entries.push((pos, text));
                        self.text_buffer.clear();
                    }
                self.text_mode = false;
                self.text_cursor = None;
            }
            KeyCode::Backspace => {
                self.text_buffer.pop();
            }
            KeyCode::Char(c) => self.text_buffer.push(c),
            _ => {}
        }
        Ok(())
    }

    // ── Colour input ────────────────────────────────────────────
    // Dialog for typing RGB values manually (e.g. "255 128 0").
    // Accepts digits, spaces, commas, and tabs as separators.

    pub(crate) fn on_key_color_input(&mut self, key: KeyEvent) -> io::Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.show_color_input = false;
                self.color_input_buffer.clear();
            }
            KeyCode::Enter => {
                if let Some(color) = DrawingApp::parse_rgb_buffer(&self.color_input_buffer) {
                    self.add_custom_color(color);
                    self.push_color_history(color);
                }
                self.show_color_input = false;
                self.color_input_buffer.clear();
            }
            KeyCode::Backspace => {
                self.color_input_buffer.pop();
            }
            KeyCode::Char(c) if c.is_ascii_digit() || c == ' ' || c == ',' || c == '\t' => {
                self.color_input_buffer.push(c);
                // Live-update the override on every number, filling 0 for missing components.
                let parts: Vec<&str> = self.color_input_buffer
                    .split(&[',', ' ', '\t'][..])
                    .filter(|s| !s.is_empty())
                    .collect();
                if !parts.is_empty()
                    && let Ok(r) = parts[0].parse::<u8>()
                {
                    let g = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                    let b = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
                    self.custom_color_override = Some(Color::Rgb(r, g, b));
                }
            }
            _ => {}
        }
        Ok(())
    }

    // ── Startup dialog ──────────────────────────────────────────
    // When a prior session is found, this dialog offers three choices:
    // Restore, Save & New, or Discard & New.

    pub(crate) fn on_key_startup(&mut self, key: KeyEvent) -> io::Result<()> {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.startup_dialog_idx = self.startup_dialog_idx.wrapping_add(1) % 3;
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.startup_dialog_idx = self.startup_dialog_idx.wrapping_sub(1) % 3;
            }
            KeyCode::Enter => match self.startup_dialog_idx {
                0 => {
                    let _ = self.restore_session();
                    self.show_startup_dialog = false;
                }
                1 => {
                    self.startup_save_and_new = true;
                    self.file_browser.open(FileBrowserMode::Save);
                }
                _ => {
                    DrawingApp::delete_session();
                    self.show_startup_dialog = false;
                }
            },
            _ => {}
        }
        Ok(())
    }

    // ── Context menu ────────────────────────────────────────────
    // The right-click context menu supports: erase point, clear canvas,
    // copy, paste, and select all.

    pub(crate) fn on_key_context_menu(&mut self, key: KeyEvent) -> io::Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.show_context_menu = false;
            }
            KeyCode::Up | KeyCode::Char('k')
                if self.context_menu_idx > 0 => {
                    self.context_menu_idx -= 1;
            }
            KeyCode::Down | KeyCode::Char('j')
                if self.context_menu_idx < 5 => {
                    self.context_menu_idx += 1;
            }
            KeyCode::Enter => {
                let pos = self.context_menu_pos;
                self.show_context_menu = false;
                match self.context_menu_idx {
                    0 => {
                        // Erase at point
                        if let Some(local) = self.local_canvas_position(pos) {
                            self.push_history();
                            self.stamp_erase((local.x, local.y));
                        }
                    }
                    1 => {
                        // Clear canvas
                        self.push_history();
                        self.points.clear();
                        self.last_localition = None;
                    }
                    2 => {
                        // Copy
                        self.copy_selection();
                    }
                    3 => {
                        // Paste at cursor
                        if let Some(mpos) = self.mouse_position
                            && let Some(local) = self.local_canvas_position(mpos) {
                                self.paste_selection(local);
                            }
                    }
                    _ => {
                        // Select All — select entire canvas
                        let w = self.canvas_area.width.saturating_sub(1);
                        let h = self.canvas_area.height.saturating_sub(1);
                        self.select_mode = true;
                        self.selection_start = Some(Position::new(0, 0));
                        self.selection_end = Some(Position::new(w, h));
                        self.selecting = false;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    // ── Canvas resize ───────────────────────────────────────────
    // Dialog for entering new canvas dimensions (width height).

    pub(crate) fn on_key_canvas_resize(&mut self, key: KeyEvent) -> io::Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.show_canvas_resize = false;
                self.canvas_resize_buffer.clear();
            }
            KeyCode::Enter => {
                let parts: Vec<&str> = self.canvas_resize_buffer
                    .split(&[',', ' ', '\t'][..])
                    .filter(|s| !s.is_empty())
                    .collect();
                if parts.len() == 2
                    && let (Ok(w), Ok(h)) = (parts[0].parse::<u16>(), parts[1].parse::<u16>())
                {
                    self.canvas_width = w.clamp(8, 500);
                    self.canvas_height = h.clamp(8, 500);
                }
                self.show_canvas_resize = false;
                self.canvas_resize_buffer.clear();
            }
            KeyCode::Backspace => {
                self.canvas_resize_buffer.pop();
            }
            KeyCode::Char(c) if c.is_ascii_digit() || c == ' ' || c == ',' || c == '\t' => {
                self.canvas_resize_buffer.push(c);
            }
            _ => {}
        }
        Ok(())
    }

    // ── Tab rename ──────────────────────────────────────────────
    // Dialog for renaming the current tab.

    pub(crate) fn on_key_tab_rename(&mut self, key: KeyEvent) -> io::Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.show_tab_rename = false;
                self.tab_rename_buffer.clear();
            }
            KeyCode::Enter => {
                let name = self.tab_rename_buffer.trim().to_string();
                if !name.is_empty() {
                    self.tabs[self.current_tab].name = name;
                }
                self.show_tab_rename = false;
                self.tab_rename_buffer.clear();
            }
            KeyCode::Char(c)
                if !c.is_control() => {
                    self.tab_rename_buffer.push(c);
            }
            KeyCode::Backspace => {
                self.tab_rename_buffer.pop();
            }
            _ => {}
        }
        Ok(())
    }

    // ── Palette bar click ───────────────────────────────────────
    // The palette bar at the bottom shows palette colour swatches (○/●)
    // and custom colour slots (C1◆, C2◆, etc.). This handler figures out
    // which swatch was clicked based on the X offset.

    pub(crate) fn handle_palette_bar_click(&mut self, pos: Position) {
        let bar_x = self.palette_bar_area.x.saturating_add(1);
        let col = pos.x;
        if col < bar_x {
            return;
        }
        let label_len: u16 = "Palette:".len() as u16 + 1; // label + trailing space
        let click_pos = col.saturating_sub(bar_x + label_len);
        let pal_count = self.palette.colors.len() as u16;

        // Click on the label — ignore.
        if click_pos == 0 && col < bar_x + label_len {
            return;
        }
        let offset = click_pos as usize;

        // Click on a palette colour swatch.
        if offset < pal_count as usize {
            self.palette.select(offset);
            self.custom_color_override = None;
            self.push_color_history(self.palette.current());
            return;
        }

        // Skip the separator `┆` at offset == pal_count
        if offset == pal_count as usize {
            return;
        }

        // Custom colours: each slot takes 3 chars (e.g. "C1◆")
        let custom_offset = offset - pal_count as usize - 1;  // -1 for the separator
        let cust_idx = custom_offset / 3;
        if cust_idx < self.custom_colors.len() {
            let c = self.custom_colors[cust_idx];
            self.custom_color_override = Some(c);
            self.push_color_history(c);
        }
    }
}
