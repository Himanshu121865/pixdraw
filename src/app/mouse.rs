// ── app/mouse.rs ──────────────────────────────────────────────────────
// Mouse event dispatch: drawing, erasing, shaping, selecting, and all
// other canvas interactions. Extracted from event.rs to keep keyboard
// and mouse handling separate.
//
// Crossterm mouse events carry:
//   - column, row (screen coordinates)
//   - kind: Down, Up, Drag, Moved, ScrollUp, ScrollDown
//   - button: Left, Right, Middle, None
//   - modifiers: Shift, Ctrl, Alt, etc.
//
// We don't handle Moved events (they would flood the render loop).
// Only Down, Up, Drag, and Scroll events are processed.

use std::io;

use crossterm::{
    cursor,
    event::{MouseButton, MouseEvent, MouseEventKind, KeyModifiers},
    execute,
};
use ratatui::layout::{Position, Rect};

use crate::app::{distance, DrawingApp, ShapeKind};

impl DrawingApp {
    pub(crate) fn on_mouse(&mut self, event: MouseEvent) -> io::Result<()> {
        let position = Position::new(event.column, event.row);
        self.mouse_position = Some(position);

        // ── Cursor teleport ────────────────────────────────────
        // crossterm's mouse tracking doesn't move the terminal cursor
        // automatically. We manually `MoveTo` the cursor whenever it
        // jumps by ≥4 cells (Chebyshev distance). This keeps the
        // terminal cursor near the mouse for widgets that use it.
        let should_teleport = self
            .last_cursor_position
            .is_none_or(|last| distance(last, position) >= 4);
        if should_teleport {
            execute!(io::stdout(), cursor::MoveTo(position.x, position.y))?;
            self.last_cursor_position = Some(position);
        }

        // ── Palette bar click ────────────────────────────────
        // Handle BEFORE any mode-specific logic.
        if let MouseEventKind::Down(MouseButton::Left) = event.kind
            && self.palette_bar_area.contains(position) {
                self.handle_palette_bar_click(position);
                return Ok(());
        }

        // ── Context menu click ──────────────────────────────
        if self.show_context_menu
            && let MouseEventKind::Down(MouseButton::Left) = event.kind {
                let menu_width = 22;
                let menu_height = 9;
                let mx = self.context_menu_pos.x.min(self.canvas_area.x.saturating_add(
                    self.canvas_area.width.saturating_sub(menu_width)));
                let my = self.context_menu_pos.y;
                let menu_rect = Rect::new(mx, my, menu_width, menu_height);
                if menu_rect.contains(position) {
                    let rel_row = position.y.saturating_sub(my + 1);
                    if rel_row <= 5 {
                        self.context_menu_idx = rel_row as usize;
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
                            2 => self.copy_selection(),
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
                } else {
                    self.show_context_menu = false;
                }
                return Ok(());
        }

        // ── Canvas resize click dismiss ─────────────────────
        if self.show_canvas_resize
            && let MouseEventKind::Down(MouseButton::Left) = event.kind {
                self.show_canvas_resize = false;
                self.canvas_resize_buffer.clear();
                return Ok(());
        }

        // ── Colour picker popup click ────────────────────────
        if self.show_color_picker {
            if let MouseEventKind::Down(MouseButton::Left) = event.kind {
                if self.color_picker_area.contains(position) {
                    let row = position.y.saturating_sub(self.color_picker_area.y + 1);
                    if row < self.palette.colors.len() as u16 {
                        self.palette.select(row as usize);
                        self.custom_color_override = None;
                        self.push_color_history(self.palette.current());
                        self.show_color_picker = false;
                    } else if row > self.palette.colors.len() as u16 {
                        let custom_idx = row as usize - self.palette.colors.len() - 1;
                        if custom_idx < self.custom_colors.len() {
                            let c = self.custom_colors[custom_idx];
                            self.custom_color_override = Some(c);
                            self.push_color_history(c);
                            self.show_color_picker = false;
                        }
                    }
                } else {
                    self.show_color_picker = false;
                }
            }
            return Ok(());
        }

        // ── Text mode click ──────────────────────────────────
        // When in text mode, clicking on the canvas sets the text
        // cursor position and commits any pending text buffer.
        if self.text_mode {
            let Some(local) = self.local_canvas_position(position) else {
                return Ok(());
            };
            if let MouseEventKind::Down(MouseButton::Left) = event.kind {
                // If there's existing text in the buffer, commit it first.
                if !self.text_buffer.is_empty()
                    && let Some(cpos) = self.text_cursor {
                        self.push_history();
                        let text = self.text_buffer.clone();
                        self.text_entries.push((cpos, text));
                        self.text_buffer.clear();
                    }
                self.text_cursor = Some(local);
            }
            return Ok(());
        }

        // ── General canvas interactions ────────────────────────
        // Everything below requires a valid canvas-local position.
        let Some(local) = self.local_canvas_position(position) else {
            return Ok(());
        };

        let shift_held = event.modifiers.contains(KeyModifiers::SHIFT);

        match event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if self.eyedropper_mode {
                    self.pick_color_at(local);
                    self.eyedropper_mode = false;
                } else if self.fill_mode {
                    self.push_history();
                    self.flood_fill((local.x, local.y));
                } else if self.eraser_mode {
                    self.push_history();
                    self.stamp_erase((local.x, local.y));
                    self.last_localition = Some(local);
                } else if let Some(kind) = self.shape_mode {
                    if let Some(anchor) = self.shape_anchor {
                        // Second click: set up preview for Up handler.
                        let end = if shift_held {
                            Self::constrain_square(anchor, local)
                        } else {
                            local
                        };
                        self.shape_preview = Some((anchor, end, kind));
                    } else {
                        self.shape_anchor = Some(local);
                    }
                } else if self.spray_mode {
                    self.push_history();
                } else if self.line_mode {
                    if self.line_anchor.is_none() {
                        self.line_anchor = Some(local);
                    } else {
                        let anchor = self.line_anchor.unwrap();
                        let end = if shift_held {
                            Self::constrain_angle(anchor, local)
                        } else {
                            local
                        };
                        self.push_history();
                        let color = self.draw_color();
                        for p in self.bresenham_points((anchor.x, anchor.y), (end.x, end.y)) {
                            self.points.insert(p, color);
                        }
                        self.line_anchor = None;
                        // Continuous line: line_anchor stays as None,
                        // user can click again to draw next segment.
                    }
                } else if self.gradient_mode {
                    if self.gradient_anchor.is_none() {
                        self.gradient_anchor = Some(local);
                    } else {
                        let anchor = self.gradient_anchor.unwrap();
                        let end = if shift_held {
                            Self::constrain_square(anchor, local)
                        } else {
                            local
                        };
                        self.push_history();
                        let ca = self.draw_color();
                        let cb = self.custom_color_override.unwrap_or_else(|| {
                            let idx = (self.palette.index + 1) % self.palette.colors.len();
                            self.palette.colors[idx].0
                        });
                        self.gradient_fill(anchor, end, ca, cb);
                        self.gradient_anchor = None;
                        self.gradient_mode = false;
                    }
                } else if self.life_mode {
                    self.push_history();
                    self.seed_life_cells(local);
                } else if self.select_mode {
                    if self.selecting {
                        self.selection_end = Some(local);
                        self.selecting = false;
                    } else {
                        self.selection_start = Some(local);
                        self.selection_end = Some(local);
                        self.selecting = true;
                    }
                } else {
                    // Default: freehand brush stroke.
                    self.push_history();
                    let color = self.draw_color();
                    self.stamp_brush((local.x, local.y), color);
                    self.place_symmetry((local.x, local.y), color);
                    self.last_localition = Some(local);
                }
            }

            MouseEventKind::Down(MouseButton::Right) => {
                if self.life_mode {
                    self.push_history();
                    self.run_life_generation();
                } else {
                    // Show context menu at the mouse position.
                    self.show_context_menu = true;
                    self.context_menu_idx = 0;
                    self.context_menu_pos = position;
                }
            }

            MouseEventKind::Down(MouseButton::Middle) => {
                self.push_history();
                self.points.clear();
                self.last_localition = None;
            }

            MouseEventKind::Drag(MouseButton::Left) => {
                let constrained = if shift_held {
                    if let Some(anchor) = self.shape_anchor {
                        Self::constrain_square(anchor, local)
                    } else if let Some(anchor) = self.line_anchor {
                        Self::constrain_angle(anchor, local)
                    } else {
                        local
                    }
                } else {
                    local
                };

                if self.select_mode && self.selecting {
                    self.selection_end = Some(constrained);
                } else if self.eraser_mode {
                    self.erase_line(constrained);
                } else if let Some(kind) = self.shape_mode
                    && self.shape_anchor.is_some()
                {
                    let anchor = self.shape_anchor.unwrap();
                    let end = if shift_held {
                        Self::constrain_square(anchor, constrained)
                    } else {
                        constrained
                    };
                    self.shape_preview = Some((anchor, end, kind));
                } else if self.spray_mode {
                    // Spray can: stamp random points within brush radius using LCG.
                    let radius = self.brush_size.saturating_sub(1) as i16;
                    let cx = constrained.x as i16;
                    let cy = constrained.y as i16;
                    let mut seed = self.color_gen_seed;
                    for _ in 0..15 {
                        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                        let dx = (seed >> 33) as i16 % (radius * 2 + 1).max(1) - radius;
                        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                        let dy = (seed >> 33) as i16 % (radius * 2 + 1).max(1) - radius;
                        if dx * dx + dy * dy > radius * radius { continue; }
                        let px = (cx + dx) as u16;
                        let py = (cy + dy) as u16;
                        let color = if self.rainbow_mode {
                            self.rainbow_idx = (self.rainbow_idx + 1) % self.palette.colors.len();
                            self.palette.colors[self.rainbow_idx].0
                        } else {
                            self.draw_color()
                        };
                        self.points.insert((px, py), color);
                    }
                    self.color_gen_seed = seed;
                } else if self.line_anchor.is_some() || self.gradient_mode {
                    // No-op during drag for line/gradient.
                } else if shift_held && self.last_localition.is_some() {
                    // Shift-drag: constrained line drawing
                    let start = self.last_localition.unwrap();
                    let end = Self::constrain_angle(start, constrained);
                    self.draw_line_to(end);
                } else {
                    self.draw_line(constrained);
                }
            }

            MouseEventKind::Drag(MouseButton::Right) => self.erase_line(local),

            MouseEventKind::Up(_) => {
                self.last_localition = None;
                if let Some(kind) = self.shape_mode
                    && let Some(anchor) = self.shape_anchor
                    && let Some(preview) = self.shape_preview
                {
                    let (_, end, _) = preview;
                    self.push_history();
                    let color = self.draw_color();
                    match kind {
                        ShapeKind::Rect => {
                            self.stamp_rect((anchor.x, anchor.y), (end.x, end.y), color);
                        }
                        ShapeKind::FilledRect => {
                            self.stamp_filled_rect((anchor.x, anchor.y), (end.x, end.y), color);
                        }
                        ShapeKind::Circle => {
                            let dx = anchor.x.abs_diff(end.x);
                            let dy = anchor.y.abs_diff(end.y);
                            let r = dx.max(dy);
                            self.stamp_circle((anchor.x, anchor.y), r, color);
                        }
                        ShapeKind::FilledCircle => {
                            let dx = anchor.x.abs_diff(end.x);
                            let dy = anchor.y.abs_diff(end.y);
                            let r = dx.max(dy);
                            self.stamp_filled_circle((anchor.x, anchor.y), r, color);
                        }
                    }
                    self.shape_anchor = None;
                    self.shape_mode = None;
                    self.shape_preview = None;
                }
                self.shape_preview = None;
            }

            MouseEventKind::ScrollUp => {
                self.custom_color_override = None;
                self.palette.prev();
            }
            MouseEventKind::ScrollDown => {
                self.custom_color_override = None;
                self.palette.next();
            }
            _ => {}
        }
        Ok(())
    }

    /// Like draw_line but constrained to a single Bresenham segment
    /// (no history push). Used for Shift-drag constrained line drawing.
    fn draw_line_to(&mut self, end: Position) {
        let color = self.draw_color();
        let Some(start) = self.last_localition else {
            self.stamp_brush((end.x, end.y), color);
            self.place_symmetry((end.x, end.y), color);
            return;
        };
        for p in self.bresenham_points((start.x, start.y), (end.x, end.y)) {
            self.points.insert(p, color);
            self.place_symmetry(p, color);
        }
    }
}
