// ── app/tab.rs ──────────────────────────────────────────────────────
// Per-tab document state. Each tab holds its own canvas data (sparse
// point map), undo/redo stacks, tool anchors, selection state, and
// inline text buffer.
//
// DrawingApp uses Deref/DerefMut to auto-delegate to the current tab,
// so most code can write `self.points` instead of `self.tabs[idx].points`.
//
// This is the ONLY place TabData is defined. Adding a new per-tab field
// means:
//   1. Add the field to the struct
//   2. Initialise it in `new()`
//   3. If it needs session persistence, add save/restore lines in session.rs
//
// Why BTreeMap for pixels instead of Vec<Vec<Option<Color>>>?
//   Sparse pixel maps (BTreeMap) only allocate for occupied cells. For a
//   typical drawing with scattered marks this uses far less memory than a
//   dense [u16; 2] grid. BTreeMap also gives sorted iteration (useful for
//   PNG export and deterministic save output).

use std::collections::BTreeMap;
use ratatui::{layout::Position, style::Color};
use crate::app::ShapeKind;

/// All per-document state that belongs to one tab.
pub struct TabData {
    // ── Canvas data ──────────────────────────────────────────────
    /// The canvas as a sparse BTreeMap from (x, y) → colour.
    /// Empty cells are simply absent from the map.
    pub points: BTreeMap<(u16, u16), Color>,

    /// Placed text entries: (top-left position, content string).
    /// Note: text colour is NOT stored per-entry — it always uses the
    /// current palette colour at render time (see canvas.rs render_text_overlay).
    pub text_entries: Vec<(Position, String)>,

    // ── Undo / Redo ──────────────────────────────────────────────
    /// Undo stack — full BTreeMap snapshots.
    /// Each push saves a clone of `points` before mutation.
    /// This is simple and correct, but memory-hungry for large drawings.
    /// Note: text_entries are NOT part of undo snapshots — undoing a
    /// clear will restore pixels but NOT text. This is a known limitation.
    pub history: Vec<BTreeMap<(u16, u16), Color>>,

    /// Redo stack — states that were undone. Cleared on new mutations.
    pub redo_stack: Vec<BTreeMap<(u16, u16), Color>>,

    // ── Tool modes ─────────────────────────────────────────────
    pub text_mode: bool,
    /// Cursor position for the next text placement (set by mouse click).
    pub text_cursor: Option<Position>,
    /// Characters typed so far while in text mode.
    pub text_buffer: String,

    pub select_mode: bool,
    /// True while the user is dragging to define a selection rect.
    pub selecting: bool,
    pub selection_start: Option<Position>,
    pub selection_end: Option<Position>,
    /// Copied pixel data, stored as (dx, dy, colour) relative to selection origin.
    pub selection_buffer: Vec<(u16, u16, Color)>,

    /// Active shape tool (Rect, FilledRect, Circle, FilledCircle).
    /// None means no shape mode is active.
    pub shape_mode: Option<ShapeKind>,
    /// Anchor point for shape/line/gradient — set on first click.
    pub shape_anchor: Option<Position>,

    pub line_mode: bool,
    pub line_anchor: Option<Position>,

    pub gradient_mode: bool,
    pub gradient_anchor: Option<Position>,

    /// Last mouse position during drag (for Bresenham interpolation).
    /// When the mouse moves, we draw a line from this position to the
    /// current position to fill gaps (no teleporting).
    pub last_localition: Option<Position>,

    /// Display name shown in the tab bar.
    pub name: String,
}

impl TabData {
    pub fn new(name: String) -> Self {
        Self {
            points: BTreeMap::new(),
            text_entries: Vec::new(),
            history: Vec::new(),
            redo_stack: Vec::new(),
            text_mode: false,
            text_cursor: None,
            text_buffer: String::new(),
            select_mode: false,
            selecting: false,
            selection_start: None,
            selection_end: None,
            selection_buffer: Vec::new(),
            shape_mode: None,
            shape_anchor: None,
            line_mode: false,
            line_anchor: None,
            gradient_mode: false,
            gradient_anchor: None,
            last_localition: None,
            name,
        }
    }
}
