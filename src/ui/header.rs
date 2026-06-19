
use ratatui::{
    Frame,
    layout::{Position, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use super::col::*;

#[allow(clippy::too_many_arguments)]
pub fn render_header(
    frame: &mut Frame<'_>,
    area: Rect,
    text_mode: bool,
    brush_size: u16,
    current_color: Color,
    current_name: &str,
    mode_str: &str,
    tab_count: usize,
    current_tab: usize,
    tab_name: &str,
) {
                let tab_y = area.y;
    let mut x = area.x;
    let end = area.x.saturating_add(area.width);

    for ti in 0..tab_count {
        let display = if ti == current_tab {
            format!(" {} ", tab_name)
        } else {
            format!(" {} ", ti + 1)
        };
        let display = if display.len() > 14 { format!("{}..", &display[..12]) } else { display };

        let is_active = ti == current_tab;
        let fg = if is_active { text() } else { subtle() };
        let bg = if is_active { tab_active_bg() } else { tab_inactive_bg() };
        for (ci, ch) in display.chars().enumerate() {
            let px = x + ci as u16;
            if px >= end { break; }
            if let Some(cell) = frame.buffer_mut().cell_mut(Position::new(px, tab_y)) {
                cell.set_char(ch).set_style(Style::default().fg(fg).bg(bg));
            }
        }
        x += display.len() as u16;

                if ti < tab_count - 1 && x < end {
            if let Some(cell) = frame.buffer_mut().cell_mut(Position::new(x, tab_y)) {
                cell.set_char('│').set_style(Style::default().fg(dim()));
            }
            x += 1;
        }
    }

        while x < end {
        if let Some(cell) = frame.buffer_mut().cell_mut(Position::new(x, tab_y)) {
            cell.set_char(' ').set_style(Style::default().bg(surface()));
        }
        x += 1;
    }

        let info_y = area.y + 1;

        let title_style = if text_mode {
        Style::default().fg(Color::Green).bold()
    } else {
        Style::default().fg(text()).bold()
    };

        let mode_tag: String = if text_mode {
        " text() ".to_string()
    } else if !mode_str.is_empty() {
        format!(" {} ", mode_str)
    } else {
        String::new()
    };

    let mode_style = if text_mode {
        Style::default().fg(Color::Black).bg(Color::Green)
    } else if !mode_str.is_empty() {
        Style::default().fg(bg()).bg(highlight())
    } else {
        Style::default()
    };

        let left_spans = vec![
        Span::styled(" ◆ ", Style::default().fg(accent())),
        Span::styled("Opendraw", title_style),
        Span::styled("  ", Style::default()),
    ];

    let brush_span = Span::styled(
        format!("brush[{}]", brush_size),
        Style::default().fg(accent()),
    );
    let swatch = Span::styled(" ■ ", Style::default().fg(current_color));
    let color_span = Span::styled(current_name, Style::default().fg(subtle()));

    let center_spans = vec![
        brush_span,
        swatch,
        color_span,
    ];

    let right_spans = vec![
        Span::styled(mode_tag, mode_style),
        Span::styled("  ", Style::default()),
        Span::styled("^T^W", Style::default().fg(subtle())),
        Span::styled("  ?", Style::default().fg(subtle())),
    ];

            let left: Line = left_spans.into();
    let center: Line = center_spans.into();
    let right: Line = right_spans.into();
    let left_w = left.width() as u16;
    let right_w = right.width() as u16;
    let padding = 2u16;
    let avail = area.width.saturating_sub(left_w).saturating_sub(right_w).saturating_sub(padding * 2);
    let center_w = center.width() as u16;
    let gap = avail.saturating_sub(center_w);

    let mut full_spans: Vec<Span> = Vec::new();
    full_spans.extend(left.spans);
    full_spans.push(Span::raw(" ".repeat(padding as usize)));
    full_spans.push(Span::raw(" ".repeat(gap as usize / 2)));
    full_spans.extend(center.spans);
    full_spans.push(Span::raw(" ".repeat(gap.saturating_sub(gap / 2) as usize)));
    full_spans.push(Span::raw(" ".repeat(padding as usize)));
    full_spans.extend(right.spans);

            let info_area = Rect::new(area.x, info_y, area.width, 1);
    frame.render_widget(Paragraph::new(Line::from(full_spans)).style(Style::default().bg(surface())), info_area);

        let sep_y = area.y + 2;
    for sx in area.x..end {
        if let Some(cell) = frame.buffer_mut().cell_mut(Position::new(sx, sep_y)) {
            cell.set_char('─').set_style(Style::default().fg(dim()));
        }
    }
}
