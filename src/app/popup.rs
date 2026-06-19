
use std::io;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::Position;
use ratatui::style::Color;

use crate::app::DrawingApp;
use crate::file_browser::FileBrowserMode;

impl DrawingApp {
                    
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
                                        self.show_color_selector = false;
                    self.show_color_input = true;
                    self.color_input_buffer.clear();
                }
            }
            _ => {}
        }
        Ok(())
    }

                    
    pub(crate) fn on_key_text_mode(&mut self, key: KeyEvent) -> io::Result<()> {
        match key.code {
            KeyCode::Esc => {
                                self.text_mode = false;
                self.line_mode = false;
                self.spray_mode = false;
                self.shape_mode = None;
                self.text_buffer.clear();
                self.text_cursor = None;
            }
            KeyCode::Enter => {
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
                                                if let Some(local) = self.local_canvas_position(pos) {
                            self.push_history();
                            self.stamp_erase((local.x, local.y));
                        }
                    }
                    1 => {
                                                self.push_history();
                        self.points.clear();
                        self.last_localition = None;
                    }
                    2 => {
                                                self.copy_selection();
                    }
                    3 => {
                                                if let Some(mpos) = self.mouse_position
                            && let Some(local) = self.local_canvas_position(mpos) {
                                self.paste_selection(local);
                            }
                    }
                    _ => {
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

                
    pub(crate) fn handle_palette_bar_click(&mut self, pos: Position) {
        let bar_x = self.palette_bar_area.x.saturating_add(1);
        let col = pos.x;
        if col < bar_x {
            return;
        }
        let label_len: u16 = "Palette:".len() as u16 + 1;         let click_pos = col.saturating_sub(bar_x + label_len);
        let pal_count = self.palette.colors.len() as u16;

                if click_pos == 0 && col < bar_x + label_len {
            return;
        }
        let offset = click_pos as usize;

                if offset < pal_count as usize {
            self.palette.select(offset);
            self.custom_color_override = None;
            self.push_color_history(self.palette.current());
            return;
        }

                if offset == pal_count as usize {
            return;
        }

                let custom_offset = offset - pal_count as usize - 1;          let cust_idx = custom_offset / 3;
        if cust_idx < self.custom_colors.len() {
            let c = self.custom_colors[cust_idx];
            self.custom_color_override = Some(c);
            self.push_color_history(c);
        }
    }
}
