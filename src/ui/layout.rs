// ── ui/layout.rs ─────────────────────────────────────────────────────
// Compute the screen layout: header (3 rows), canvas body (remaining),
// palette bar footer (1 row), and status bar (1 row).
//
// Returns `(header, body, footer, status, canvas)` where body == canvas.
//
// Ratatui Rects use saturating arithmetic (no negative values), so we
// always use `saturating_add` / `saturating_sub` to avoid panics when
// the terminal is resized very small.

use ratatui::layout::Rect;

pub fn layout(area: Rect) -> (Rect, Rect, Rect, Rect, Rect) {
    // Row layout of the full screen:
    //   row 0-2: header  (tab bar, info bar, separator)
    //   rows 3..(h-2): canvas body
    //   row h-2: status bar (pixel count, coords, colour history)
    //   row h-1: footer (palette bar)
    let header = Rect::new(area.x, area.y, area.width, 3);
    let body = Rect::new(
        area.x,
        area.y.saturating_add(3),
        area.width,
        area.height.saturating_sub(5),
    );
    let status_y = body.y.saturating_add(body.height);
    let status = Rect::new(area.x, status_y, area.width, 1);
    let footer = Rect::new(area.x, status_y.saturating_add(1), area.width, 1);
    let canvas = Rect::new(body.x, body.y, body.width, body.height);
    (header, body, footer, status, canvas)
}
