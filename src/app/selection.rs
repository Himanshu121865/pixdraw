// ── app/selection.rs ───────────────────────────────────────────────
// Selection operations: copy, cut, paste, and nudge (arrow keys).
//
// The selection is a rectangle defined by `selection_start` and
// `selection_end`. `selection_buffer` stores the copied pixels as
// (dx, dy, colour) relative to the selection's top-left corner.
//
// How selection works:
//   1. Press `s` to enter selection mode (select_mode = true).
//   2. Click to set selection_start, drag to set selection_end.
//      A border of `·` dots is drawn around the selected area.
//   3. Ctrl+C copies pixels into selection_buffer (relative coords).
//   4. Ctrl+V pastes at the current mouse position.
//   5. Arrow keys nudge the selection contents by 1 cell.
//
// Relativity: pixels are stored as (x - x1, y - y1, colour) so they
// can be pasted at any position without recalculating offsets.

use ratatui::{
    layout::Position,
    style::Color,
};

use crate::app::DrawingApp;

impl DrawingApp {
    /// Copy all pixels inside the current selection rect to the buffer.
    /// Pixels are stored with coordinates relative to the selection origin.
    pub(crate) fn copy_selection(&mut self) {
        let Some(start) = self.selection_start else { return };
        let Some(end) = self.selection_end else { return };
        self.selection_buffer.clear();
        let x1 = start.x.min(end.x);
        let x2 = start.x.max(end.x);
        let y1 = start.y.min(end.y);
        let y2 = start.y.max(end.y);
        for x in x1..=x2 {
            for y in y1..=y2 {
                if let Some(&color) = self.points.get(&(x, y)) {
                    self.selection_buffer.push((x - x1, y - y1, color));
                }
            }
        }
    }

    /// Copy then delete all pixels in the selection rect.
    pub(crate) fn cut_selection(&mut self) {
        self.copy_selection();
        let Some(start) = self.selection_start else { return };
        let Some(end) = self.selection_end else { return };
        self.push_history();
        let x1 = start.x.min(end.x);
        let x2 = start.x.max(end.x);
        let y1 = start.y.min(end.y);
        let y2 = start.y.max(end.y);
        for x in x1..=x2 {
            for y in y1..=y2 {
                self.points.remove(&(x, y));
            }
        }
    }

    /// Paste the buffered selection at `at` (canvas-local position).
    /// The top-left of the pasted content goes to `at`.
    pub(crate) fn paste_selection(&mut self, at: Position) {
        self.push_history();
        let to_insert: Vec<(u16, u16, Color)> = self.selection_buffer.iter()
            .map(|(dx, dy, color)| (at.x + dx, at.y + dy, *color))
            .collect();
        for (x, y, color) in to_insert {
            self.points.insert((x, y), color);
        }
    }

    /// Move the contents of the selection rect by (dx, dy).
    /// Pixels that move out of bounds wrap via wrapping_add_signed.
    /// The selection rectangle is also moved so it stays aligned.
    pub(crate) fn nudge_selection(&mut self, dx: i16, dy: i16) {
        let Some(start) = self.selection_start else { return };
        let Some(end) = self.selection_end else { return };
        self.push_history();
        let x1 = start.x.min(end.x);
        let x2 = start.x.max(end.x);
        let y1 = start.y.min(end.y);
        let y2 = start.y.max(end.y);

        // Collect all pixels in the selection.
        let mut selected: Vec<(u16, u16, Color)> = Vec::new();
        for x in x1..=x2 {
            for y in y1..=y2 {
                if let Some(&c) = self.points.get(&(x, y)) {
                    selected.push((x, y, c));
                }
            }
        }
        // Remove old positions.
        for (x, y, _) in &selected {
            self.points.remove(&(*x, *y));
        }
        // Insert at new positions.
        for (x, y, c) in selected {
            let nx = x.wrapping_add_signed(dx);
            let ny = y.wrapping_add_signed(dy);
            self.points.insert((nx, ny), c);
        }
        // Update selection rect to match the new position.
        self.selection_start = Some(Position::new(
            start.x.wrapping_add_signed(dx),
            start.y.wrapping_add_signed(dy),
        ));
        self.selection_end = Some(Position::new(
            end.x.wrapping_add_signed(dx),
            end.y.wrapping_add_signed(dy),
        ));
    }
}
