
use ratatui::{
    Frame,
    layout::{Alignment, Position, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Clear, Paragraph, Widget},
};

use super::col::*;


pub fn render_tab_rename_dialog(frame: &mut Frame<'_>, screen: Rect, buffer: &str) {
    let width = 42u16.min(screen.width.saturating_sub(4));
    let height = 5;
    let x = screen.x + (screen.width - width) / 2;
    let y = screen.y + (screen.height - height) / 2;
    let area = Rect::new(x, y, width, height);

    let block = Block::bordered()
        .title(" Rename Tab ")
        .border_style(Style::default().fg(border()));
    let inner = block.inner(area);
    frame.render_widget(Clear, area);
    block.render(area, frame.buffer_mut());

    let display = if buffer.is_empty() {
        Line::from(Span::styled("  type name and press Enter", Style::default().fg(dim())))
    } else {
        Line::from(vec![
            Span::styled(" ", Style::default()),
            Span::styled(buffer, Style::default().fg(Color::Green)),
            Span::styled("▌", Style::default().fg(Color::Green)),
        ])
    };
    frame.render_widget(Paragraph::new(display), inner);
}


pub fn render_startup_dialog(
    frame: &mut Frame<'_>,
    screen: Rect,
    selected: usize,
) {
    let options = ["  Restore  ", "  Save & New  ", "  Discard & New  "];
    let title = " Previous session found ";

    let width = 46u16.min(screen.width.saturating_sub(4));
    let height = 8u16.min(screen.height.saturating_sub(2));
    let x = screen.x + (screen.width - width) / 2;
    let y = screen.y + (screen.height - height) / 2;
    let area = Rect::new(x, y, width, height);

    let block = Block::bordered()
        .title(format!(" {} ", title))
        .border_style(Style::default().fg(border()));
    let inner = block.inner(area);
    frame.render_widget(Clear, area);
    block.render(area, frame.buffer_mut());

    let mut lines = Vec::new();
    lines.push(Line::from(Span::raw("")));
    let mut option_line = Vec::new();
    for (i, opt) in options.iter().enumerate() {
        if i > 0 {
            option_line.push(Span::raw("  "));
        }
        if i == selected {
            option_line.push(Span::styled(*opt, Style::default().fg(Color::Black).bg(Color::White)));
        } else {
            option_line.push(Span::styled(*opt, Style::default().fg(Color::White)));
        }
    }
    lines.push(Line::from(option_line));
    lines.push(Line::from(Span::raw("")));
    lines.push(Line::from(Span::styled(
        " j/k navigate  Enter select  q quit",
        Style::default().fg(dim()),
    )));
    Paragraph::new(lines).alignment(Alignment::Center).render(inner, frame.buffer_mut());
}


pub fn render_context_menu(
    frame: &mut Frame<'_>,
    screen: Rect,
    click_pos: &Position,
    selected: usize,
) {
    let items = ["  Erase point  ", "  Clear canvas  ", "  Copy  ", "  Paste  ", "  Select all  "];
    let width = 20u16;
    let height = 9u16;
    let mx = click_pos.x.saturating_add(2)
        .min(screen.x.saturating_add(screen.width.saturating_sub(width)));
    let my = click_pos.y.saturating_add(1);
    let area = Rect::new(mx, my, width, height);

    let block = Block::bordered()
        .title(" Menu ")
        .border_style(Style::default().fg(border()));
    let inner = block.inner(area);
    frame.render_widget(Clear, area);
    block.render(area, frame.buffer_mut());

    let mut lines = Vec::new();
    for (i, item) in items.iter().enumerate() {
        let style = if i == selected {
            Style::default().fg(Color::Black).bg(Color::White)
        } else {
            Style::default().fg(Color::White)
        };
        lines.push(Line::from(Span::styled(*item, style)));
    }
    frame.render_widget(Paragraph::new(lines), inner);
}


pub fn render_canvas_resize_dialog(
    frame: &mut Frame<'_>,
    screen: Rect,
    buffer: &str,
) {
    let width = 36u16.min(screen.width.saturating_sub(4));
    let height = 6;
    let x = screen.x + (screen.width - width) / 2;
    let y = screen.y + (screen.height - height) / 2;
    let area = Rect::new(x, y, width, height);

    let block = Block::bordered()
        .title(" Canvas Size ")
        .border_style(Style::default().fg(border()));
    let inner = block.inner(area);
    frame.render_widget(Clear, area);
    block.render(area, frame.buffer_mut());

    let line1 = Line::from(Span::styled(
        "  Enter width and height (e.g. 80 40):",
        Style::default().fg(dim()),
    ));

    let line2 = Line::from(vec![
        Span::styled(" ", Style::default()),
        Span::styled(buffer, Style::default().fg(Color::Green)),
        Span::styled("▌", Style::default().fg(Color::Green)),
    ]);

    frame.render_widget(Paragraph::new(vec![line1, line2]), inner);
}
