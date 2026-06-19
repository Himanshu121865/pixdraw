// ── ui/status.rs ─────────────────────────────────────────────────────
// Single-line status bar at the bottom of the screen showing pixel count,
// mouse coordinates, colour history swatches, current colour name/hex,
// and brush size.
//
// Note the rendering pattern:
//   We manually fill the entire status row with background colour,
//   then use `Paragraph::new(...).render()` to write the text.
//   The manual fill ensures no stale characters from the previous frame
//   show through where the text doesn't reach.
//
//   `Paragraph::render(area, buffer)` is used instead of
//   `frame.render_widget()` because we want to render into a specific
//   buffer area without a full Frame widget lifecycle.

use std::collections::VecDeque;

use ratatui::{
    Frame,
    layout::{Position, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use super::col::*;

#[allow(clippy::too_many_arguments)]
pub fn render_status_bar(
    frame: &mut Frame<'_>,
    area: Rect,
    mouse_pos: Option<Position>,
    _canvas_area: Rect,
    point_count: usize,
    brush_size: u16,
    history: &VecDeque<Color>,
    current: Color,
    current_name: &str,
    hex: &str,
) {
    let cursor_str = match mouse_pos {
        Some(p) => format!("{:>3},{:<3}", p.x, p.y),
        None => " -, - ".to_string(),
    };

    let left = format!(" pts:{:<5} pos:{} ", point_count, cursor_str);
    let right = format!(" {} {} {}  brush[{}] ", current_name, hex, "■", brush_size);

    let status_style = Style::default().bg(status_bg());

    // Fill entire row with status bg to clear any stale cells.
    for x in area.x..area.x.saturating_add(area.width) {
        if let Some(cell) = frame.buffer_mut().cell_mut(Position::new(x, area.y)) {
            cell.set_char(' ').set_style(status_style);
        }
    }

    let mut spans: Vec<Span> = Vec::new();
    spans.push(Span::styled(&left, Style::default().fg(text()).bg(status_bg())));
    spans.push(Span::styled(" │ ", Style::default().fg(dim()).bg(status_bg())));

    // Colour history swatches — small ring of recently-used colours.
    for &hc in history {
        spans.push(Span::styled("■", Style::default().fg(hc).bg(status_bg())));
        spans.push(Span::raw(" "));
    }

    // Padding to right-align the current colour info.
    let left_w = left.len() + 3 + history.len() * 2;
    let right_w = right.len() + 2;
    let pad = area.width.saturating_sub(left_w as u16).saturating_sub(right_w as u16);
    if pad > 0 {
        spans.push(Span::raw(" ".repeat(pad as usize)));
    }

    let swatch = Span::styled("■", Style::default().fg(current).bg(status_bg()));
    let name_hex = Span::styled(
        format!(" {} {} ", current_name, hex),
        Style::default().fg(current).bg(status_bg()),
    );
    spans.push(swatch);
    spans.push(name_hex);

    spans.push(Span::styled(
        format!(" brush[{}] ", brush_size),
        Style::default().fg(accent()).bg(status_bg()),
    ));

    Paragraph::new(Line::from(spans)).render(area, frame.buffer_mut());
}
