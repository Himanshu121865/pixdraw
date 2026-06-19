// ── app/event.rs ──────────────────────────────────────────────────────
// Main event loop dispatch: reads keyboard and mouse events from
// Crossterm and delegates to DrawingApp methods.
//
// Popup-specific handlers (colour picker, file browser, text mode,
// context menu, resize dialog, etc.) live in `popup.rs` — they
// are called from here via `self.on_key_*()`.
//
// Mouse handling (drawing, erasing, shaping, selecting, etc.) is
// in `mouse.rs` — extracted to keep this file focused on keyboard.
//
// Crossterm event model:
//   event::read() blocks until a Key or Mouse event arrives.
//   Key events have `kind`: Press, Repeat, or Release.
//     We only handle Press — repeats are ignored to avoid
//     double-processing from key repeat.
//   Mouse events have `kind`: Down, Up, Drag, ScrollUp/Down,
//     and Moved. The `kind` field tells us what the user did.
//
// Ratatui coordinate system:
//   Position(x, y) where x is column, y is row (0,0 = top-left).
//   This matches terminal cell coordinates from crossterm.

use std::io;

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
};

use crate::app::{DrawingApp, ShapeKind};
use crate::file_browser::FileBrowserMode;

impl DrawingApp {
    /// Process a single raw crossterm event (non-blocking — event is already read).
    pub fn handle_raw_event(&mut self, event: &Event) -> io::Result<()> {
        match event {
            Event::Key(key) => self.on_key(*key),
            Event::Mouse(mouse) => self.on_mouse(*mouse),
            _ => Ok(()),
        }
    }

    pub fn handle_event(&mut self) -> io::Result<()> {
        // Auto-backup every 60 seconds — silent save to session file.
        // The `let _ =` discards any IO error; a failed backup is non-fatal.
        if self.last_backup.elapsed().as_secs() >= 60 {
            let _ = self.save_session();
            self.last_backup = std::time::Instant::now();
        }

        // If the file browser is active, route all events through its
        // own handler (which reads one event internally).
        if self.file_browser.active {
            let result = self.handle_file_browser_event();
            // Special case: startup dialog "Save & New" path.
            if !self.file_browser.active && self.startup_save_and_new {
                DrawingApp::delete_session();
                self.show_startup_dialog = false;
                self.startup_save_and_new = false;
            }
            return result;
        }

        if self.life_mode {
            // Auto-advance GoL every 150ms while in life mode.
            // Polls for events with a timeout — if the user clicks or
            // presses a key, we handle it immediately. Otherwise the
            // simulation advances one generation.
            if event::poll(std::time::Duration::from_millis(150))? {
                match event::read()? {
                    event::Event::Key(key) => self.on_key(key)?,
                    event::Event::Mouse(mouse) => self.on_mouse(mouse)?,
                    _ => {}
                }
            } else {
                self.run_life_generation();
            }
        } else {
            match event::read()? {
                event::Event::Key(key) => self.on_key(key)?,
                event::Event::Mouse(mouse) => self.on_mouse(mouse)?,
                _ => {}
            }
        }
        Ok(())
    }

    // ── Keyboard dispatch ─────────────────────────────────────────

    fn on_key(&mut self, key: KeyEvent) -> io::Result<()> {
        // Only handle Press events — ignore auto-repeat and release.
        if key.kind != KeyEventKind::Press {
            return Ok(());
        }

        // ── Popup priority dispatch ──────────────────────────────
        // Popups that capture ALL input are checked first. The early
        // return means no other keybinds fire while these are open.
        //
        // Order matters: if two popups are somehow both open, the
        // first match wins. In practice only one should be active.

        if self.show_color_picker {
            return self.on_key_color_picker(key);
        }
        if self.show_color_selector {
            return self.on_key_color_selector(key);
        }
        // Text mode captures ALL keys when active — including letters
        // that would otherwise trigger tool modes.
        if self.text_mode {
            return self.on_key_text_mode(key);
        }
        if self.show_help {
            if self.help_search_active {
                match key.code {
                    // Esc: if buffer is non-empty, clear it; otherwise exit search mode.
                    KeyCode::Esc => {
                        if !self.help_search_buffer.is_empty() {
                            self.help_search_buffer.clear();
                        } else {
                            self.help_search_active = false;
                        }
                    }
                    KeyCode::Enter => {
                        // Commit search results, stay in filtered view.
                        self.help_search_active = false;
                    }
                    KeyCode::Backspace => {
                        self.help_search_buffer.pop();
                    }
                    KeyCode::Char(c) if !c.is_control() => {
                        self.help_search_buffer.push(c);
                    }
                    _ => {}
                }
            } else {
                match key.code {
                    // Esc in browse mode: if there's a filter, clear it and go back to search;
                    // otherwise close help.
                    KeyCode::Esc => {
                        if !self.help_search_buffer.is_empty() {
                            self.help_search_buffer.clear();
                            self.help_search_active = true;
                        } else {
                            self.show_help = false;
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        self.help_selected = if self.help_selected == 0 {
                            self.help_cat_expanded.len().saturating_sub(1)
                        } else {
                            self.help_selected - 1
                        };
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        self.help_selected = (self.help_selected + 1) % self.help_cat_expanded.len();
                    }
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        let idx = self.help_selected;
                        if idx < self.help_cat_expanded.len() {
                            self.help_cat_expanded[idx] = !self.help_cat_expanded[idx];
                        }
                    }
                    KeyCode::Char('9') => {
                        self.help_cat_expanded = [true; 9];
                    }
                    KeyCode::Char('0') => {
                        self.help_cat_expanded = [false; 9];
                    }
                    // i reactivates search from browse mode.
                    KeyCode::Char('i') => {
                        self.help_search_active = true;
                    }
                    _ => {}
                }
            }
            return Ok(());
        }
        if self.show_color_input {
            return self.on_key_color_input(key);
        }
        if self.show_startup_dialog {
            return self.on_key_startup(key);
        }
        if self.show_context_menu {
            return self.on_key_context_menu(key);
        }
        if self.show_canvas_resize {
            return self.on_key_canvas_resize(key);
        }
        if self.show_tab_rename {
            return self.on_key_tab_rename(key);
        }

        // ── Main keybinds (no modal popup active) ────────────────
        //
        // match on key.code only — most binds don't need modifiers.
        // When they do (Ctrl+letter), the guard clause checks modifiers.

        match key.code {
            KeyCode::Char('q') => {
                let _ = self.save_session();
                self.should_quit = true;
            }

            KeyCode::Char('?') | KeyCode::Char('/') => {
                self.show_help = !self.show_help;
                if self.show_help {
                    self.help_search_active = true;
                    self.help_search_buffer.clear();
                    self.help_scroll = 0;
                    self.help_cat_expanded = [false; 9];
                }
            }

            // ── Tab management ────────────────────────────────
            KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.new_tab();
            }
            KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.close_tab();
            }
            KeyCode::Tab => {
                self.current_tab = (self.current_tab + 1) % self.tabs.len();
            }
            KeyCode::BackTab => {
                if self.current_tab > 0 {
                    self.current_tab -= 1;
                } else {
                    self.current_tab = self.tabs.len() - 1;
                }
            }

            // ── Ctrl combos ────────────────────────────────────
            KeyCode::Char('z') if key.modifiers.contains(KeyModifiers::CONTROL) => self.undo(),
            KeyCode::Char('y') if key.modifiers.contains(KeyModifiers::CONTROL) => self.redo(),
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.file_browser.open(FileBrowserMode::Save);
            }
            KeyCode::Char('o') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.file_browser.open(FileBrowserMode::Load);
            }
            KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.file_browser.open(FileBrowserMode::ExportPng);
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.copy_selection();
            }
            KeyCode::Char('x') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.cut_selection();
            }
            KeyCode::Char('v') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Paste at the stored mouse position. If unavailable, do nothing.
                if let Some(mpos) = self.mouse_position
                    && let Some(local) = self.local_canvas_position(mpos) {
                        self.paste_selection(local);
                    }
            }
            KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.show_canvas_resize = true;
                if self.canvas_width > 0 && self.canvas_height > 0 {
                    self.canvas_resize_buffer = format!("{} {}", self.canvas_width, self.canvas_height);
                } else {
                    self.canvas_resize_buffer.clear();
                }
            }

            // ── Clear canvas ──────────────────────────────────────
            KeyCode::Char('c') => {
                self.push_history();
                self.points.clear();
                self.text_entries.clear();
                self.last_localition = None;
            }

            // ── Selection nudge (arrow keys) ──────────────────────
            KeyCode::Up
                if self.select_mode && self.selection_start.is_some() => {
                    self.nudge_selection(0, -1i16);
                }
            KeyCode::Down
                if self.select_mode && self.selection_start.is_some() => {
                    self.nudge_selection(0, 1);
                }
            KeyCode::Left
                if self.select_mode && self.selection_start.is_some() => {
                    self.nudge_selection(-1i16, 0);
                }
            KeyCode::Right
                if self.select_mode && self.selection_start.is_some() => {
                    self.nudge_selection(1, 0);
                }

            KeyCode::F(2) => {
                self.show_tab_rename = true;
                self.tab_rename_buffer = self.tab_name().to_string();
            }

            // ── Tool toggles ──────────────────────────────────────
            // Each tool key toggles the mode on/off. For exclusive
            // tool groups (spray, eraser, eyedropper, fill), activating
            // one deactivates the others.

            KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.file_browser.open(FileBrowserMode::ExportPng);
            }
            KeyCode::Char('p') => {
                self.spray_mode = !self.spray_mode;
                if self.spray_mode {
                    self.line_mode = false;
                    self.text_mode = false;
                    self.shape_mode = None;
                }
            }
            KeyCode::Char('b') => {
                self.rainbow_mode = !self.rainbow_mode;
            }
            KeyCode::Char('i') => {
                self.eyedropper_mode = !self.eyedropper_mode;
                if self.eyedropper_mode {
                    self.eraser_mode = false;
                    self.fill_mode = false;
                    self.spray_mode = false;
                }
            }
            KeyCode::Char('e') => {
                self.eraser_mode = !self.eraser_mode;
                if self.eraser_mode {
                    self.eyedropper_mode = false;
                    self.fill_mode = false;
                    self.spray_mode = false;
                }
            }
            KeyCode::Char('f') => {
                self.fill_mode = !self.fill_mode;
                if self.fill_mode {
                    self.eyedropper_mode = false;
                    self.eraser_mode = false;
                    self.spray_mode = false;
                }
            }
            KeyCode::Char('m') => self.symmetry_mode = !self.symmetry_mode,
            KeyCode::Char('g') => self.show_grid = !self.show_grid,
            KeyCode::Char('L') => {
                if self.life_mode {
                    self.push_history();
                    self.run_life_generation();
                } else {
                    self.life_mode = true;
                }
            }
            KeyCode::Char('P')
                if !self.points.is_empty() => {
                    self.push_history();
                    self.posterize(8);
            }

            // ── Colour / palette ───────────────────────────────
            KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.show_color_picker = true;
            }
            KeyCode::Char(' ') => {
                self.custom_color_override = None;
                self.palette.next();
                self.push_color_history(self.palette.current());
            }
            KeyCode::Char('t') => {
                self.text_mode = true;
                self.text_buffer.clear();
                self.text_cursor = None;
            }
            KeyCode::Char(d @ '1'..='9') => {
                self.custom_color_override = None;
                self.palette.select((d as u8 - b'1') as usize);
                self.push_color_history(self.palette.current());
            }

            // ── Shape tools ─────────────────────────────────────
            KeyCode::Char('r') => {
                self.shape_mode = Some(ShapeKind::Rect);
                self.shape_anchor = None;
            }
            KeyCode::Char('R') => {
                self.shape_mode = Some(ShapeKind::FilledRect);
                self.shape_anchor = None;
            }
            KeyCode::Char('o') => {
                self.shape_mode = Some(ShapeKind::Circle);
                self.shape_anchor = None;
            }
            KeyCode::Char('O') => {
                self.shape_mode = Some(ShapeKind::FilledCircle);
                self.shape_anchor = None;
            }
            KeyCode::Char('l') => {
                self.line_mode = true;
                self.line_anchor = None;
            }

            // ── Select / gradient ──────────────────────────────
            KeyCode::Char('s') => {
                self.select_mode = !self.select_mode;
                if !self.select_mode {
                    self.selection_start = None;
                    self.selection_end = None;
                    self.selecting = false;
                }
            }

            KeyCode::Char('G') => {
                self.gradient_mode = !self.gradient_mode;
                self.gradient_anchor = None;
            }

            // ── Custom colour generation ─────────────────────────
            KeyCode::Char('u') => {
                self.generate_three_colors();
                self.color_selector_idx = 0;
                self.show_color_selector = true;
            }
            KeyCode::Char('0')
                if !self.custom_colors.is_empty() => {
                    let idx = self.custom_cycle_idx % self.custom_colors.len();
                    let c = self.custom_colors[idx];
                    self.custom_color_override = Some(c);
                    self.push_color_history(c);
                    self.custom_cycle_idx = (self.custom_cycle_idx + 1) % self.custom_colors.len();
            }

            // ── Universal escape: reset everything to brush ─────
            KeyCode::Esc => {
                // Reset all tool modes back to plain brush.
                self.line_mode = false;
                self.spray_mode = false;
                self.text_mode = false;
                self.shape_mode = None;
                self.eyedropper_mode = false;
                self.eraser_mode = false;
                self.fill_mode = false;
                self.symmetry_mode = false;
                self.select_mode = false;
                self.gradient_mode = false;
                self.selection_start = None;
                self.selection_end = None;
                self.selecting = false;
                self.shape_anchor = None;
                self.line_anchor = None;
                self.gradient_anchor = None;
                self.shape_preview = None;
                self.life_mode = false;
                // Dismiss any open popup.
                self.show_context_menu = false;
                self.show_color_selector = false;
                self.show_color_input = false;
                self.show_canvas_resize = false;
                self.show_tab_rename = false;
            }

            // ── Brush size ─────────────────────────────────────
            KeyCode::Char('[') | KeyCode::Char('-')
                if self.brush_size > 1 => {
                    self.brush_size -= 1;
                }
            KeyCode::Char(']') | KeyCode::Char('=') | KeyCode::Char('+') => {
                self.brush_size = self.brush_size.saturating_add(1).min(5);
            }

            _ => {}
        }
        Ok(())
    }
}
