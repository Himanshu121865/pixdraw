
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

        for x in area.x..area.x.saturating_add(area.width) {
        if let Some(cell) = frame.buffer_mut().cell_mut(Position::new(x, area.y)) {
            cell.set_char(' ').set_style(status_style);
        }
    }

    let mut spans: Vec<Span> = Vec::new();
    spans.push(Span::styled(&left, Style::default().fg(text()).bg(status_bg())));
    spans.push(Span::styled(" │ ", Style::default().fg(dim()).bg(status_bg())));

        for &hc in history {
        spans.push(Span::styled("■", Style::default().fg(hc).bg(status_bg())));
        spans.push(Span::raw(" "));
    }

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
