// ── ui/canvas.rs ─────────────────────────────────────────────────────
// The canvas is the heart of opendraw. This module handles everything
// that appears inside the bordered canvas area: pixels, grid dots, the
// brush cursor preview, and text overlays.
//
// Two rendering strategies (why direct buffer?):
//   Ratatui's `Canvas` widget is a high-level painter that maps floating-
//   point coordinates to cells. For pixel art, this adds overhead —
//   every point goes through a HashMap → f64 conversion → half-block
//   mapping pipeline, and the coordinate system doesn't align with
//   integer pixel positions.
//
//   Direct buffer access (`frame.buffer_mut().cell_mut()`) sidesteps
//   all that: each pixel is a full-block character (█) at its exact cell
//   position. No HashMap, no floats, no half-block alignment issues.
//   Just (x, y) → cell. Simple and fast.

use ratatui::{
    Frame,
    layout::{Margin, Position, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Paragraph},
};

use crate::app::{DrawingApp, ShapeKind};

use super::col::*;

/// Render the drawing canvas: a bordered block with pixel data painted
/// via direct buffer cell manipulation.
///
/// Every pixel occupies exactly one terminal cell — no half-block
/// mapping, no HashMap allocation per frame. The trade-off is that
/// each frame must redraw every pixel (the buffer is reset by Ratatui),
/// but iterating a few thousand entries is fast enough in practice.
pub fn render_canvas(
    app: &mut DrawingApp,
    frame: &mut Frame<'_>,
    area: Rect,
    canvas_width: u16,
    canvas_height: u16,
    shape_preview: Option<(Position, Position, ShapeKind)>,
) {
    // ── Virtual canvas ───────────────────────────────────────────
    // "Virtual canvas" means the user has set a fixed canvas size
    // (via Ctrl+R or by loading an image). When set, the visible
    // block is constrained to that size + 2 (for the border) and
    // centered on screen. When unset (0,0), the canvas fills the
    // entire available body area.
    let has_virtual_size = canvas_width > 0 && canvas_height > 0;

    let canvas_block_width = if has_virtual_size {
        canvas_width.saturating_add(2).min(area.width)
    } else {
        area.width
    };
    let canvas_block_height = if has_virtual_size {
        canvas_height.saturating_add(2).min(area.height)
    } else {
        area.height
    };

    // Centre the block within the available area.
    let block_x = area.x + (area.width - canvas_block_width) / 2;
    let block_y = area.y + (area.height - canvas_block_height) / 2;
    let block_area = Rect::new(block_x, block_y, canvas_block_width, canvas_block_height);

    // ── Title bar ────────────────────────────────────────────────
    // Shows current tool mode, brush colour swatch, and brush size.
    let mode_indicator = app.mode_string();
    let title_suffix = if !mode_indicator.is_empty() {
        format!("  [{}]", mode_indicator)
    } else {
        String::new()
    };

    let title = Line::from(vec![
        Span::styled(" Canvas ", Style::default().fg(highlight()).bold()),
        Span::styled("■", Style::default().fg(app.palette.current())),
        Span::styled(format!(" {} [{}]{}", app.palette.name(), app.brush_size, title_suffix),
            Style::default().fg(subtle())),
    ]);

    // Draw just the border frame — the interior pixels are drawn
    // cell-by-cell below.
    frame.render_widget(
        Block::bordered()
            .border_style(Style::default().fg(border()))
            .title(title),
        block_area,
    );

    // `inner` is the drawable area INSIDE the border (1 cell inset).
    let inner = block_area.inner(Margin { horizontal: 1, vertical: 1 });
    app.canvas_area = inner;

    // ── Clipping ─────────────────────────────────────────────────
    // clip_w/clip_h define the virtual canvas bounds (from which
    // pixels are loaded). y_max is the visible height — pixels with
    // y > y_max are beyond the terminal's displayable rows and must
    // be skipped (otherwise they'd wrap around or glitch).
    let clip_w = if has_virtual_size { canvas_width } else { u16::MAX };
    let clip_h = if has_virtual_size { canvas_height } else { u16::MAX };
    let y_max = inner.height.saturating_sub(1);

    // ── Pixel drawing ────────────────────────────────────────────
    // We bypass Ratatui's Canvas widget entirely. Instead, we reach
    // into the frame's cell buffer directly and set each pixel as a
    // full-block character (█) with the pixel's colour as foreground.
    //
    // Why not use Cell's background colour?
    //   Using bg() would fill the entire cell background, including
    //   the gutter around the character — but then we'd lose the
    //   half-block resolution. For pixel art, fg() with █ is crisp
    //   and unambiguous.
    //
    // The guard `y > y_max` is critical: without it, `y_max - y`
    // would underflow (via saturating_sub) to 0, making all off-
    // screen pixel rows pile up on the bottom line and flicker.
    for (&(x, y), &c) in &app.points {
        if x >= clip_w || y >= clip_h || y > y_max { continue; }
        let cx = inner.x.saturating_add(x);
        let cy = inner.y.saturating_add(y);
        if cx >= inner.x + inner.width || cy >= inner.y + inner.height { continue; }
        if let Some(cell) = frame.buffer_mut().cell_mut(Position::new(cx, cy)) {
            cell.set_char('█').set_style(Style::default().fg(c));
        }
    }

    // ── Shape preview ────────────────────────────────────────────
    // When the user drags to draw a shape, this generates the preview
    // points and renders them in the same direct-buffer style.
    // Shape preview is purely visual — the actual shape is committed
    // to `points` only when the mouse button is released. This lets
    // the user see what they're drawing before committing.
    if let Some((anchor, end, kind)) = shape_preview {
        let color = app.draw_color();
        let preview_pts = match kind {
            ShapeKind::Rect => {
                let x1 = anchor.x.min(end.x);
                let x2 = anchor.x.max(end.x);
                let y1 = anchor.y.min(end.y);
                let y2 = anchor.y.max(end.y);
                let mut pts = Vec::new();
                for x in x1..=x2 { pts.push((x, y1)); pts.push((x, y2)); }
                for y in y1..=y2 { pts.push((x1, y)); pts.push((x2, y)); }
                pts
            }
            ShapeKind::FilledRect => {
                let x1 = anchor.x.min(end.x);
                let x2 = anchor.x.max(end.x);
                let y1 = anchor.y.min(end.y);
                let y2 = anchor.y.max(end.y);
                let mut pts = Vec::new();
                for x in x1..=x2 {
                    for y in y1..=y2 { pts.push((x, y)); }
                }
                pts
            }
            ShapeKind::Circle | ShapeKind::FilledCircle => {
                let dx = anchor.x.abs_diff(end.x);
                let dy = anchor.y.abs_diff(end.y);
                let r = dx.max(dy) as i16;
                let cx = anchor.x as i16;
                let cy = anchor.y as i16;
                let mut pts = Vec::new();
                if kind == ShapeKind::FilledCircle {
                    for dy2 in -r..=r {
                        let dx2 = ((r * r - dy2 * dy2) as f64).sqrt().round() as i16;
                        for x2 in (cx - dx2).max(0)..=(cx + dx2).max(0) {
                            let py = cy + dy2;
                            if py >= 0 { pts.push((x2 as u16, py as u16)); }
                        }
                    }
                } else {
                    let mut x = r;
                    let mut y = 0i16;
                    let mut err = 1 - r;
                    while x >= y {
                        for (px, py) in &[
                            (cx + x, cy + y), (cx - x, cy + y),
                            (cx + x, cy - y), (cx - x, cy - y),
                            (cx + y, cy + x), (cx - y, cy + x),
                            (cx + y, cy - x), (cx - y, cy - x),
                        ] {
                            if *px >= 0 && *py >= 0 { pts.push((*px as u16, *py as u16)); }
                        }
                        y += 1;
                        if err <= 0 { err += 2 * y + 1; } else { x -= 1; err += 2 * (y - x) + 1; }
                    }
                }
                pts
            }
        };
        for (px, py) in preview_pts {
            if px >= clip_w || py >= clip_h || py > y_max { continue; }
            let cx = inner.x.saturating_add(px);
            let cy = inner.y.saturating_add(py);
            if cx >= inner.x + inner.width || cy >= inner.y + inner.height { continue; }
            if let Some(cell) = frame.buffer_mut().cell_mut(Position::new(cx, cy)) {
                cell.set_char('█').set_style(Style::default().fg(color));
            }
        }
    }

    if app.points.is_empty() && app.text_entries.is_empty() && !app.text_mode {
        let placeholder = Rect::new(
            inner.x,
            inner.y.saturating_add(inner.height.saturating_sub(1) / 2),
            inner.width,
            1,
        );
        frame.render_widget(
            Paragraph::new("Click or drag to start drawing!")
                .fg(dim())
                .centered(),
            placeholder,
        );
    }
}

/// Overlay a grid of dots every 4 cells across the canvas.
/// Only draws on empty cells (`cell.symbol() == " "`) to avoid
/// overwriting existing pixels.
///
/// Why every 4 cells?
///   A 4-cell grid is fine enough for alignment guides but sparse
///   enough to stay subtle. It's a standard choice in pixel editors.
pub fn render_grid(frame: &mut Frame<'_>, canvas: Rect) {
    let step = 4u16;
    // `inner` is the area inside the border (1 cell inset on each side).
    let inner = Rect::new(canvas.x + 1, canvas.y + 1, canvas.width - 2, canvas.height - 2);
    for y in (inner.y..inner.y.saturating_add(inner.height)).step_by(step as usize) {
        for x in (inner.x..inner.x.saturating_add(inner.width)).step_by(step as usize) {
            // Only draw on empty cells so grid dots don't stomp on pixels.
            if let Some(cell) = frame.buffer_mut().cell_mut(Position::new(x, y))
                && cell.symbol() == " " {
                    cell.set_char('·').set_style(Style::default().fg(dim()));
                }
        }
    }
}

/// Show a brush-outline cursor at the current mouse position.
/// For `brush_size == 1`, shows a single dot (`·`). For larger brushes,
/// shows a circle of dots matching the brush radius.
///
/// Cursor colour indicates the active tool:
///   - White  = brush (normal draw)
///   - Red    = eraser mode
///   - Green  = fill mode
///   - Yellow = eyedropper mode
///
/// Why draw on the buffer instead of using Ratatui's cursor?
///   The terminal cursor can only show position, not shape or colour.
///   Drawing a custom cursor lets us show the brush radius and tool
///   colour at a glance — essential for pixel art.
///
/// Only draws on cells where `cell.symbol() == " "` so the cursor
/// preview never overwrites existing pixels. This means when the
/// cursor hovers over a pixel, the preview dot simply disappears
/// instead of corrupting the pixel — a deliberate UX choice.
pub fn render_cursor_preview(app: &DrawingApp, frame: &mut Frame<'_>, canvas: Rect) {
    let Some(mpos) = app.mouse_position else { return };
    let Some(local) = app.local_canvas_position(mpos) else { return };

    let r = app.brush_size.saturating_sub(1) as i16;
    let inner_top = canvas.y + 1;
    let inner_left = canvas.x + 1;

    let color = if app.eraser_mode {
        Color::Red
    } else if app.fill_mode {
        Color::Green
    } else if app.eyedropper_mode {
        Color::Yellow
    } else {
        Color::White
    };

    if r == 0 {
        let cx = inner_left.wrapping_add_signed(local.x as i16);
        let cy = inner_top.wrapping_add_signed(local.y as i16);
        if let Some(cell) = frame.buffer_mut().cell_mut(Position::new(cx, cy)) {
            cell.set_char('·').set_style(Style::default().fg(color));
        }
        return;
    }

    for dy in -r..=r {
        for dx in -r..=r {
            if dx * dx + dy * dy <= r * r {
                let cx = inner_left.wrapping_add_signed(local.x as i16 + dx);
                let cy = inner_top.wrapping_add_signed(local.y as i16 + dy);
                if let Some(cell) = frame.buffer_mut().cell_mut(Position::new(cx, cy))
                    && cell.symbol() == " " {
                        cell.set_char('·').set_style(Style::default().fg(color));
                    }
            }
        }
    }
}

/// Render placed and in-progress text on the canvas.
///
/// Placed text (from `text_entries`) uses the current palette colour.
/// In-progress text (while typing, stored in `text_buffer`) previews
/// in white with a `▌` block cursor at the insertion point.
///
/// Text colour is NOT stored per-entry — it always uses the palette's
/// current colour when drawn. This is a deliberate simplification:
/// text inherits the active colour, just like brush strokes.
///
/// Text never overwrites pixels: the `occupied` set tracks all pixel
/// positions, and text characters skip occupied cells.
pub fn render_text_overlay(app: &DrawingApp, frame: &mut Frame<'_>, canvas: Rect) {
    // Fast path: when there's no text at all, skip the entire function
    // (including the expensive occupied-set build from all points).
    if app.text_entries.is_empty() && !app.text_mode { return; }

    let inner_x0 = canvas.x + 1;
    let inner_y0 = canvas.y + 1;

    // Build a set of all pixel positions so we don't paint text on top of them.
    // This is O(n) in the number of pixels — acceptable since text rendering
    // is rare compared to plain drawing.
    let mut occupied: std::collections::HashSet<(u16, u16)> = std::collections::HashSet::new();
    for (x, y) in app.points.keys() {
        occupied.insert((*x, *y));
    }

    let color = app.palette.current();
    for (pos, text) in &app.text_entries {
        for (ci, ch) in text.chars().enumerate() {
            let cx = inner_x0 + pos.x + ci as u16;
            let cy = inner_y0 + pos.y;
            let local_c = (pos.x + ci as u16, pos.y);
            if occupied.contains(&local_c) { continue; }
            if let Some(cell) = frame.buffer_mut().cell_mut(Position::new(cx, cy)) {
                cell.set_char(ch).set_style(Style::default().fg(color));
            }
            occupied.insert(local_c);
        }
    }

    // ── In-progress text preview ─────────────────────────────────
    // While typing (text_mode), the buffer is shown in white as a
    // live preview. Press Enter to commit, Esc to cancel.
    if app.text_mode && !app.text_buffer.is_empty()
        && let Some(tpos) = app.text_cursor {
            for (ci, ch) in app.text_buffer.chars().enumerate() {
                let cx = inner_x0 + tpos.x + ci as u16;
                let cy = inner_y0 + tpos.y;
                if let Some(cell) = frame.buffer_mut().cell_mut(Position::new(cx, cy)) {
                    cell.set_char(ch).set_style(Style::default().fg(Color::White));
                }
            }
            // Cursor: a right-half block character (▌) at the end of the buffer.
            // White on white creates a solid block that visually marks the
            // insertion point — more visible than a simple underscore.
            let cursor_cx = inner_x0 + tpos.x + app.text_buffer.len() as u16;
            let cursor_cy = inner_y0 + tpos.y;
            if let Some(cell) = frame.buffer_mut().cell_mut(Position::new(cursor_cx, cursor_cy)) {
                cell.set_char('▌').set_style(Style::default().fg(Color::White).bg(Color::White));
            }
        }
}
